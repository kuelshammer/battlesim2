use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use simulation_wasm::aggregation::calculate_score;
use simulation_wasm::dice;
use simulation_wasm::events::Event;
use simulation_wasm::model::{Action, Creature, DiceFormula, Encounter, SimulationResult};
use simulation_wasm::run_event_driven_simulation_rust;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sim_cli")]
#[command(about = "CLI tools for D&D combat simulation analysis")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate aggregated statistics for each quintile from 1005 simulation runs
    Aggregate {
        /// Path to the scenario JSON file
        scenario: PathBuf,
        /// Output file path (optional, defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate a detailed event log for a simulation run
    Log {
        /// Path to the scenario JSON file
        scenario: PathBuf,
        /// Output format: 'markdown' or 'json'
        #[arg(short, long, default_value = "markdown")]
        format: String,
        /// Run index (if running from a batch, 0-indexed)
        #[arg(short, long)]
        run_index: Option<usize>,
    },
    /// Find the simulation run closest to the median quintile
    FindMedian {
        /// Path to the scenario JSON file
        scenario: PathBuf,
    },
    /// Analyze action efficiency and damage sources from a single simulation run
    Breakdown {
        /// Path to the scenario JSON file
        scenario: PathBuf,
        /// Run index (optional, defaults to 0)
        #[arg(short, long)]
        run_index: Option<usize>,
    },
    /// Calculate theoretical probabilities and average damage
    Math {
        /// Path to the scenario JSON file
        scenario: PathBuf,
        /// Attacker name or ID
        #[arg(short, long)]
        attacker: String,
        /// Defender name or ID
        #[arg(short, long)]
        defender: String,
    },
    /// Sensitivity analysis: vary a stat across a range and plot win rates
    Sweep {
        /// Path to the scenario JSON file
        scenario: PathBuf,
        /// Target combatant name
        #[arg(short, long)]
        target: String,
        /// Stat to vary: "AC", "HP", "toHit", "damage"
        #[arg(short, long)]
        stat: String,
        /// Range in "start..end" format (e.g., "10..20")
        #[arg(short, long)]
        range: String,
    },
    /// Compare two scenarios side-by-side
    Compare {
        /// First scenario file (baseline)
        scenario_a: PathBuf,
        /// Second scenario file (variant)
        scenario_b: PathBuf,
    },
    /// Validate a scenario JSON for common errors
    Validate {
        /// Path to the scenario JSON file
        scenario: PathBuf,
    },
}

// --- Data Structures for Output ---

#[derive(Serialize, Deserialize, Debug)]
struct CombatantStats {
    id: String,
    name: String,
    avg_end_hp: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct QuintileStats {
    quintile: usize, // 1-5
    label: String,   // "Worst 20%", "Below Average", "Median", "Above Average", "Best 20%"
    win_rate: f64,
    avg_rounds: f64,
    combatants: Vec<CombatantStats>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AggregateOutput {
    scenario_name: String,
    total_runs: usize,
    quintiles: Vec<QuintileStats>,
}

// --- Main Entry Point ---

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Aggregate { scenario, output } => {
            run_aggregate(&scenario, output.as_deref());
        }
        Commands::Log {
            scenario,
            format,
            run_index,
        } => {
            run_log(&scenario, &format, run_index);
        }
        Commands::FindMedian { scenario } => {
            run_find_median(&scenario);
        }
        Commands::Breakdown {
            scenario,
            run_index,
        } => {
            run_breakdown(&scenario, run_index);
        }
        Commands::Math {
            scenario,
            attacker,
            defender,
        } => {
            run_math(&scenario, &attacker, &defender);
        }
        Commands::Sweep {
            scenario,
            target,
            stat,
            range,
        } => {
            run_sweep(&scenario, &target, &stat, &range);
        }
        Commands::Compare {
            scenario_a,
            scenario_b,
        } => {
            run_compare(&scenario_a, &scenario_b);
        }
        Commands::Validate { scenario } => {
            run_validate(&scenario);
        }
    }
}

// --- Aggregate Subcommand ---

fn run_aggregate(scenario_path: &PathBuf, output_path: Option<&std::path::Path>) {
    let (players, encounters, scenario_name) = load_scenario(scenario_path);

    // Run 1005 iterations
    let iterations = 1005;
    println!("Running {} iterations...", iterations);
    let (results, _) = run_event_driven_simulation_rust(players, encounters, iterations, false);

    // Results are already sorted by score (ascending: worst to best)
    let quintile_labels = [
        "Worst 20%",
        "Below Average",
        "Median",
        "Above Average",
        "Best 20%",
    ];
    let mut quintiles = Vec::new();

    for q in 0..5 {
        let start = q * 201;
        let end = (q + 1) * 201;
        let slice = &results[start..end];

        let stats = calculate_quintile_stats(slice, q + 1, quintile_labels[q]);
        quintiles.push(stats);
    }

    let output = AggregateOutput {
        scenario_name,
        total_runs: iterations,
        quintiles,
    };

    let json = serde_json::to_string_pretty(&output).expect("Failed to serialize output");

    if let Some(path) = output_path {
        fs::write(path, &json).expect("Failed to write output file");
        println!("Wrote output to {:?}", path);
    } else {
        println!("{}", json);
    }
}

fn calculate_quintile_stats(
    results: &[SimulationResult],
    quintile_num: usize,
    label: &str,
) -> QuintileStats {
    let count = results.len() as f64;
    let mut wins = 0;
    let mut total_rounds = 0;

    // Track HP per combatant (id -> (name, total_hp, count))
    let mut combatant_hp: std::collections::HashMap<String, (String, f64, usize)> =
        std::collections::HashMap::new();

    for result in results {
        if let Some(encounter) = result.last() {
            if let Some(last_round) = encounter.rounds.last() {
                total_rounds += encounter.rounds.len();

                // Check win condition
                let team1_alive = last_round
                    .team1
                    .iter()
                    .any(|c| c.final_state.current_hp > 0.0);
                let team2_alive = last_round
                    .team2
                    .iter()
                    .any(|c| c.final_state.current_hp > 0.0);
                if team1_alive && !team2_alive {
                    wins += 1;
                }

                // Accumulate HP stats for all combatants
                for c in last_round.team1.iter().chain(last_round.team2.iter()) {
                    let entry = combatant_hp.entry(c.id.clone()).or_insert((
                        c.creature.name.clone(),
                        0.0,
                        0,
                    ));
                    entry.1 += c.final_state.current_hp;
                    entry.2 += 1;
                }
            }
        }
    }

    let combatants: Vec<CombatantStats> = combatant_hp
        .into_iter()
        .map(|(id, (name, total_hp, cnt))| CombatantStats {
            id,
            name,
            avg_end_hp: total_hp / cnt as f64,
        })
        .collect();

    QuintileStats {
        quintile: quintile_num,
        label: label.to_string(),
        win_rate: (wins as f64 / count) * 100.0,
        avg_rounds: total_rounds as f64 / count,
        combatants,
    }
}

// --- Log Subcommand ---

fn run_log(scenario_path: &PathBuf, format: &str, run_index: Option<usize>) {
    let (players, encounters, _) = load_scenario(scenario_path);

    // If run_index is provided, run that many + 1 and pick the specific one
    // Otherwise, run a single simulation with logging enabled
    let (results, events) = if let Some(idx) = run_index {
        let (res, _) = run_event_driven_simulation_rust(players, encounters, idx + 1, true);
        // Pick the specific run's events (we'd need per-run events, but for now just use last)
        (res, Vec::new()) // TODO: per-run event capture
    } else {
        run_event_driven_simulation_rust(players, encounters, 1, true)
    };

    if results.is_empty() {
        println!("No simulation results!");
        return;
    }

    let result = &results[run_index.unwrap_or(0).min(results.len() - 1)];

    // We need to rebuild the name map for formatting logs
    let mut combatant_names = HashMap::new();
    if let Some(encounter) = result.first() {
        if let Some(round) = encounter.rounds.first() {
            for c in round.team1.iter().chain(round.team2.iter()) {
                combatant_names.insert(c.id.clone(), c.creature.name.clone());
            }
        }
    }

    match format {
        "json" => {
            // TODO: Serialize raw events or formatted events?
            // Let's serialize formatted for now to match previous behavior roughly
            let formatted_events: Vec<String> = events
                .iter()
                .filter_map(|e| e.format_for_log(&combatant_names))
                .collect();

            let output = serde_json::json!({
                "result": result,
                "events": formatted_events,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        "markdown" | _ => {
            // We need formatted events for markdown log
            let formatted_events: Vec<String> = events
                .iter()
                .filter_map(|e| e.format_for_log(&combatant_names))
                .collect();
            print_markdown_log(result, &formatted_events);
        }
    }
}

fn print_markdown_log(result: &SimulationResult, events: &[String]) {
    println!("# Combat Log\n");

    if let Some(encounter) = result.first() {
        for (round_idx, round) in encounter.rounds.iter().enumerate() {
            println!("## Round {}\n", round_idx + 1);

            // Show all combatants sorted by initiative
            let mut all: Vec<_> = round.team1.iter().chain(round.team2.iter()).collect();
            all.sort_by(|a, b| {
                b.initiative
                    .partial_cmp(&a.initiative)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            for c in all {
                if c.initial_state.current_hp <= 0.0 && c.final_state.current_hp <= 0.0 {
                    continue; // Skip dead
                }

                println!("### {} (Initiative: {:.1})", c.creature.name, c.initiative);
                println!(
                    "- **HP**: {:.1} → {:.1}",
                    c.initial_state.current_hp, c.final_state.current_hp
                );

                if c.actions.is_empty() && c.final_state.current_hp > 0.0 {
                    println!("- *No actions taken*");
                }

                for action in &c.actions {
                    let targets: Vec<String> = action
                        .targets
                        .iter()
                        .map(|(id, count)| {
                            if *count > 1 {
                                format!("{} (x{})", id, count)
                            } else {
                                id.clone()
                            }
                        })
                        .collect();
                    println!("- **{}**: → {:?}", action.action.base().name, targets);
                }
                println!();
            }
        }
    }

    // Print raw events if available
    if !events.is_empty() {
        println!("## Raw Event Log\n");
        for event in events {
            println!("- {}", event);
        }
    }
}

// --- Find Median Subcommand ---

fn run_find_median(scenario_path: &PathBuf) {
    let (players, encounters, _) = load_scenario(scenario_path);

    // Run 1005 iterations
    let iterations = 1005;
    println!("Running {} iterations...", iterations);
    let (results, _) = run_event_driven_simulation_rust(players, encounters, iterations, false);

    // Results are sorted by score. Middle quintile is indices 402-602.
    // We want to find the run closest to the median of the MIDDLE quintile.

    let middle_start = 402;
    let middle_end = 603;
    let middle_slice = &results[middle_start..middle_end];

    // Calculate median score of middle quintile
    let median_idx = middle_slice.len() / 2;
    let median_score = calculate_score(&middle_slice[median_idx]);

    // Also calculate average HP spread (max_hp - min_hp) for winning team and avg rounds
    let (avg_hp_spread, avg_rounds) = calculate_averages(middle_slice);

    // Now find the run within middle quintile closest to these averages
    let mut best_idx = middle_start;
    let mut best_distance = f64::MAX;

    for (i, result) in results[middle_start..middle_end].iter().enumerate() {
        let score = calculate_score(result);
        let (hp_spread, rounds) = get_run_metrics(result);

        // Weighted distance: score is primary
        let score_diff = (score - median_score).abs();
        let hp_diff = (hp_spread - avg_hp_spread).abs();
        let rounds_diff = (rounds as f64 - avg_rounds).abs();

        let distance = score_diff * 10.0 + hp_diff + rounds_diff;

        if distance < best_distance {
            best_distance = distance;
            best_idx = middle_start + i;
        }
    }

    println!("Best match run index: {}", best_idx);
    println!("  Score: {:.2}", calculate_score(&results[best_idx]));
    println!("  Distance from median: {:.2}", best_distance);
}

fn calculate_averages(results: &[SimulationResult]) -> (f64, f64) {
    let mut total_hp_spread = 0.0;
    let mut total_rounds = 0.0;
    let count = results.len() as f64;

    for result in results {
        let (hp_spread, rounds) = get_run_metrics(result);
        total_hp_spread += hp_spread;
        total_rounds += rounds as f64;
    }

    (total_hp_spread / count, total_rounds / count)
}

fn get_run_metrics(result: &SimulationResult) -> (f64, usize) {
    if let Some(encounter) = result.last() {
        if let Some(last_round) = encounter.rounds.last() {
            let team1_alive = last_round
                .team1
                .iter()
                .any(|c| c.final_state.current_hp > 0.0);
            let team2_alive = last_round
                .team2
                .iter()
                .any(|c| c.final_state.current_hp > 0.0);

            let winning_team = if team1_alive && !team2_alive {
                &last_round.team1
            } else if team2_alive && !team1_alive {
                &last_round.team2
            } else {
                return (0.0, encounter.rounds.len());
            };

            let hps: Vec<f64> = winning_team
                .iter()
                .map(|c| c.final_state.current_hp)
                .collect();
            let max_hp = hps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let min_hp = hps.iter().cloned().fold(f64::INFINITY, f64::min);

            return (max_hp - min_hp, encounter.rounds.len());
        }
    }
    (0.0, 0)
}

// --- Breakdown Subcommand ---

#[derive(Debug, Default)]
struct ActionStats {
    uses: u32,
    hits: u32,
    misses: u32,
    total_damage: f64,
}

fn run_breakdown(scenario_path: &PathBuf, run_index: Option<usize>) {
    let (players, encounters, _) = load_scenario(scenario_path);

    // Run simulation
    let _idx = run_index.unwrap_or(0);
    let (results, events) = if let Some(idx) = run_index {
        let (res, events) =
            run_event_driven_simulation_rust(players.clone(), encounters.clone(), idx + 1, true);
        // If we run multiple times, we might want events from the specific run?
        // run_event_driven_simulation_rust returns events from the FIRST run.
        // If we want events from run #5, we need to run it 6 times and discard the first 5?
        // The current API only returns events for run 0.
        // So providing run_index > 0 is misleading with current lib API.
        // For now, we just run 1 simulation and analyze it.
        (res, events)
    } else {
        run_event_driven_simulation_rust(players.clone(), encounters.clone(), 1, true)
    };

    if events.is_empty() {
        println!("No events generated.");
        return;
    }

    // Build name map
    let mut combatant_names = HashMap::new();
    let mut combatant_actions = HashMap::new(); // ID -> Vec<Action>

    if let Some(result) = results.first() {
        if let Some(encounter) = result.first() {
            // We can get combatants from rounds or final state
            if let Some(round) = encounter.rounds.first() {
                for c in round.team1.iter().chain(round.team2.iter()) {
                    combatant_names.insert(c.id.clone(), c.creature.name.clone());
                    combatant_actions.insert(c.id.clone(), c.creature.actions.clone());
                }
            }
        }
    }

    // Breakdown stats
    // Map: ActorID -> ActionID -> Stats
    let mut breakdown: HashMap<String, HashMap<String, ActionStats>> = HashMap::new();
    let mut current_action: HashMap<String, String> = HashMap::new(); // Actor -> ActionID

    for event in &events {
        match event {
            Event::ActionStarted {
                actor_id,
                action_id,
            } => {
                current_action.insert(actor_id.clone(), action_id.clone());
                breakdown
                    .entry(actor_id.clone())
                    .or_default()
                    .entry(action_id.clone())
                    .or_default()
                    .uses += 1;
            }
            Event::AttackHit {
                attacker_id,
                damage,
                ..
            } => {
                if let Some(action_id) = current_action.get(attacker_id) {
                    let stats = breakdown
                        .entry(attacker_id.clone())
                        .or_default()
                        .entry(action_id.clone())
                        .or_default();
                    stats.hits += 1;
                    stats.total_damage += damage;
                }
            }
            Event::AttackMissed { attacker_id, .. } => {
                if let Some(action_id) = current_action.get(attacker_id) {
                    let stats = breakdown
                        .entry(attacker_id.clone())
                        .or_default()
                        .entry(action_id.clone())
                        .or_default();
                    stats.misses += 1;
                }
            }
            // Could handle healing, etc.
            _ => {}
        }
    }

    // Print Report
    println!("=== Breakdown Report ===\n");

    for (actor_id, actions) in breakdown {
        let name = combatant_names.get(&actor_id).unwrap_or(&actor_id);
        println!("Combatant: {}", name);

        for (action_id, stats) in actions {
            // Find action name
            let action_name = if let Some(creature_actions) = combatant_actions.get(&actor_id) {
                creature_actions
                    .iter()
                    .find(|a| a.base().id == action_id)
                    .map(|a| a.base().name.clone())
                    .unwrap_or(action_id.clone())
            } else {
                action_id.clone()
            };

            let action_name_display = if action_name.is_empty() {
                format!("(ID: {})", action_id)
            } else {
                action_name
            };

            println!("  Action: {}", action_name_display);
            println!("    - Uses: {}", stats.uses);
            if stats.hits + stats.misses > 0 {
                let total_attacks = stats.hits + stats.misses;
                let accuracy = (stats.hits as f64 / total_attacks as f64) * 100.0;
                println!(
                    "    - Attacks: {} ({} Hits, {} Misses)",
                    total_attacks, stats.hits, stats.misses
                );
                println!("    - Accuracy: {:.1}%", accuracy);
            }
            if stats.total_damage > 0.0 {
                println!("    - Total Damage: {:.1}", stats.total_damage);
                println!(
                    "    - Avg Dmg/Hit: {:.1}",
                    stats.total_damage / stats.hits as f64
                );
                println!(
                    "    - Avg Dmg/Use: {:.1}",
                    stats.total_damage / stats.uses as f64
                );
            }
            println!();
        }
        println!("--------------------------------------------------");
    }
}

// --- Math Subcommand ---

fn run_math(scenario_path: &PathBuf, attacker_name: &str, defender_name: &str) {
    let (players, encounters, _) = load_scenario(scenario_path);

    // Find Attacker and Defender
    // We search in players and first encounter monsters
    let mut all_creatures: Vec<Creature> = players.clone();
    if let Some(encounter) = encounters.first() {
        all_creatures.extend(encounter.monsters.clone());
    }

    let attacker = all_creatures
        .iter()
        .find(|c| c.name == attacker_name || c.id == attacker_name);
    let defender = all_creatures
        .iter()
        .find(|c| c.name == defender_name || c.id == defender_name);

    if attacker.is_none() || defender.is_none() {
        println!(
            "Error: Could not find attacker '{}' or defender '{}'",
            attacker_name, defender_name
        );
        println!("Available combatants:");
        for c in &all_creatures {
            println!("- {} (ID: {})", c.name, c.id);
        }
        return;
    }

    let attacker = attacker.unwrap();
    let defender = defender.unwrap();

    println!("=== Theoretical Math Report ===");
    println!("Matchup: {} vs {}", attacker.name, defender.name);
    println!("Defender AC: {:.0}\n", defender.ac);

    for action in &attacker.actions {
        if let Action::Atk(atk) = action {
            println!(
                "Action: {}",
                if atk.name.is_empty() {
                    "(Unnamed Attack)"
                } else {
                    &atk.name
                }
            );

            // Calculate To Hit
            let to_hit_bonus = dice::average(&atk.to_hit);
            println!("  - To Hit Bonus: {:.1}", to_hit_bonus);

            // Calculate Hit Chance
            // Roll + Bonus >= AC  =>  Roll >= AC - Bonus
            let needed_roll = defender.ac - to_hit_bonus;
            // Clamp needed roll to 1-20 range logic (nat 1 is miss, nat 20 is hit)
            // Chance = (21 - needed) / 20

            let mut hit_chance = if needed_roll > 20.0 {
                0.05 // Only nat 20 hits (and crits)
            } else if needed_roll <= 1.0 {
                0.95 // Only nat 1 misses
            } else {
                (21.0 - needed_roll) / 20.0
            };
            // Ensure bounds
            hit_chance = hit_chance.max(0.05).min(0.95);

            println!(
                "  - Hit Chance: {:.1}% (Needs roll {:.0}+)",
                hit_chance * 100.0,
                needed_roll
            );

            // Calculate Damage
            let avg_dice_damage = dice::average(&atk.dpr);

            // Crit damage: Usually double the DICE part.
            // dice::average returns total average.
            // We need to separate dice from static modifier to calculate crit correctly.
            // But `dice::average` flattens it.
            // Approximation: Most damage is dice. Or we can parse it?
            // Let's assume the dice formula is parsed correctly by dice::average.
            // To get crit damage, we need to know how much of that is dice.
            // `dice.rs` has `parse_average`.
            // If we can't easily separate, we can estimate or improve `dice.rs` later.
            // For D&D 5e, crit is: Roll damage dice twice.
            // If `dpr` is "1d10+5", avg is 5.5 + 5 = 10.5.
            // Crit adds another 1d10 (5.5). Total 16.0.
            // We can calculate "Avg Dice Only" by calculating `average(formula) - average(formula_with_0_dice)`.
            // `dice.rs` `parse_term` handles "1d1".
            // Let's just use total avg for now and note that crit math might be approximate if we don't split.

            // Actually, `dice::average` parses terms.
            // We can implement a "dice only average" helper?
            // Or just parse "1d1" as 1.
            // Let's try to rely on `dice::average` for the base hit damage.
            // And for crit... we add 5% of the *dice component*.
            // This is hard without parsing.
            // Simplified Math:
            // Avg Dmg = Hit% * BaseDmg + Crit% * ExtraCritDmg
            // where ExtraCritDmg ~ BaseDmg - Modifiers.
            // Let's just output "Avg Damage on Hit" for now.

            println!("  - Avg Damage (On Hit): {:.1}", avg_dice_damage);

            // DPR = HitChance * AvgDmg + CritChance * ExtraCritDmg
            // Assuming ExtraCritDmg = AvgDmg (Optimistic / All Dice) vs (Conservative / No Dice)
            // Let's just show DPR = HitChance * AvgDmg (ignoring crits extra damage for simplicity unless we improve parsing)
            // Actually, crit is 5%.
            // DPR ~= HitChance * BaseDamage
            let dpr = hit_chance * avg_dice_damage;
            println!("  - Estimated DPR (ignoring crits): {:.2}", dpr);

            println!();
        }
    }
}

// --- Helper Functions ---

fn load_scenario(path: &PathBuf) -> (Vec<Creature>, Vec<Encounter>, String) {
    let content = fs::read_to_string(path).expect("Failed to read scenario file");
    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    let name = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let players: Vec<Creature> =
        serde_json::from_value(data["players"].clone()).expect("Failed to parse players");
    let encounters: Vec<Encounter> =
        serde_json::from_value(data["encounters"].clone()).expect("Failed to parse encounters");

    (players, encounters, name)
}

// --- Sweep Subcommand ---

fn run_sweep(scenario_path: &PathBuf, target_name: &str, stat: &str, range_str: &str) {
    let (players, encounters, _) = load_scenario(scenario_path);

    // Parse range (e.g., "10..20")
    let parts: Vec<&str> = range_str.split("..").collect();
    if parts.len() != 2 {
        println!("Error: Invalid range format. Use 'start..end' (e.g., '10..20')");
        return;
    }
    let start: i32 = parts[0].parse().expect("Invalid start value");
    let end: i32 = parts[1].parse().expect("Invalid end value");

    println!("=== Sensitivity Sweep ===");
    println!(
        "Target: {}, Stat: {}, Range: {}..{}\n",
        target_name, stat, start, end
    );
    println!("{:>6} | {:>10} | {:>10}", stat, "Win Rate", "Avg Rounds");
    println!("-------|------------|----------");

    for value in start..=end {
        // Clone and modify scenario
        let mut modified_players = players.clone();
        let mut modified_encounters = encounters.clone();

        // Find and modify target in players
        for player in &mut modified_players {
            if player.name == target_name || player.id == target_name {
                modify_stat(player, stat, value as f64);
            }
        }

        // Find and modify target in monsters
        for encounter in &mut modified_encounters {
            for monster in &mut encounter.monsters {
                if monster.name == target_name || monster.id == target_name {
                    modify_stat(monster, stat, value as f64);
                }
            }
        }

        // Run simulation (fewer runs for speed)
        let iterations = 201;
        let (results, _) = run_event_driven_simulation_rust(
            modified_players,
            modified_encounters,
            iterations,
            false,
        );

        // Calculate win rate for Team 1
        let mut wins = 0;
        let mut total_rounds = 0;
        for result in &results {
            if let Some(encounter) = result.last() {
                if let Some(last_round) = encounter.rounds.last() {
                    let team1_alive = last_round
                        .team1
                        .iter()
                        .any(|c| c.final_state.current_hp > 0.0);
                    let team2_alive = last_round
                        .team2
                        .iter()
                        .any(|c| c.final_state.current_hp > 0.0);
                    if team1_alive && !team2_alive {
                        wins += 1;
                    }
                    total_rounds += encounter.rounds.len();
                }
            }
        }
        let win_rate = (wins as f64 / iterations as f64) * 100.0;
        let avg_rounds = total_rounds as f64 / iterations as f64;

        // ASCII bar
        let bar_len = (win_rate / 5.0) as usize;
        let bar: String = "█".repeat(bar_len);

        println!(
            "{:>6} | {:>9.1}% | {:>10.1} {}",
            value, win_rate, avg_rounds, bar
        );
    }
}

fn modify_stat(creature: &mut Creature, stat: &str, value: f64) {
    match stat.to_lowercase().as_str() {
        "ac" => creature.ac = value,
        "hp" => creature.hp = value,
        "tohit" => {
            // Modify toHit on all attack actions
            for action in &mut creature.actions {
                if let Action::Atk(atk) = action {
                    atk.to_hit = DiceFormula::Value(value);
                }
            }
        }
        "damage" | "dpr" => {
            // Modify dpr on all attack actions (set to flat value for simplicity)
            for action in &mut creature.actions {
                if let Action::Atk(atk) = action {
                    atk.dpr = DiceFormula::Expr(format!("{}d1", value as i32));
                }
            }
        }
        _ => println!("[WARN] Unknown stat: {}", stat),
    }
}

// --- Compare Subcommand ---

fn run_compare(scenario_a_path: &PathBuf, scenario_b_path: &PathBuf) {
    println!("=== Scenario Comparison ===\n");

    let (players_a, encounters_a, name_a) = load_scenario(scenario_a_path);
    let (players_b, encounters_b, name_b) = load_scenario(scenario_b_path);

    // Run both
    let iterations = 1005;
    println!("Running {} iterations for each scenario...\n", iterations);

    let (results_a, _) =
        run_event_driven_simulation_rust(players_a, encounters_a, iterations, false);
    let (results_b, _) =
        run_event_driven_simulation_rust(players_b, encounters_b, iterations, false);

    // Calculate stats for each
    let stats_a = calculate_scenario_stats(&results_a);
    let stats_b = calculate_scenario_stats(&results_b);

    // Print comparison table
    println!(
        "{:<15} | {:>15} | {:>15} | {:>10}",
        "Metric", &name_a, &name_b, "Diff"
    );
    println!("----------------|-----------------|-----------------|----------");

    let win_diff = stats_b.win_rate - stats_a.win_rate;
    let hp_diff = stats_b.avg_hp_left - stats_a.avg_hp_left;
    let rounds_diff = stats_b.avg_rounds - stats_a.avg_rounds;

    println!(
        "{:<15} | {:>14.1}% | {:>14.1}% | {:>+9.1}%",
        "Win Rate", stats_a.win_rate, stats_b.win_rate, win_diff
    );
    println!(
        "{:<15} | {:>15.1} | {:>15.1} | {:>+10.1}",
        "Avg HP Left", stats_a.avg_hp_left, stats_b.avg_hp_left, hp_diff
    );
    println!(
        "{:<15} | {:>15.1} | {:>15.1} | {:>+10.1}",
        "Avg Rounds", stats_a.avg_rounds, stats_b.avg_rounds, rounds_diff
    );

    println!();
    if win_diff > 5.0 {
        println!(
            "✅ {} is significantly better (Win Rate +{:.1}%)",
            name_b, win_diff
        );
    } else if win_diff < -5.0 {
        println!(
            "❌ {} is significantly worse (Win Rate {:.1}%)",
            name_b, win_diff
        );
    } else {
        println!("≈ Both scenarios perform similarly.");
    }
}

struct ScenarioStats {
    win_rate: f64,
    avg_hp_left: f64,
    avg_rounds: f64,
}

fn calculate_scenario_stats(results: &[SimulationResult]) -> ScenarioStats {
    let count = results.len() as f64;
    let mut wins = 0;
    let mut total_hp = 0.0;
    let mut total_rounds = 0.0;

    for result in results {
        if let Some(encounter) = result.last() {
            if let Some(last_round) = encounter.rounds.last() {
                total_rounds += encounter.rounds.len() as f64;

                let team1_alive = last_round
                    .team1
                    .iter()
                    .any(|c| c.final_state.current_hp > 0.0);
                let team2_alive = last_round
                    .team2
                    .iter()
                    .any(|c| c.final_state.current_hp > 0.0);

                if team1_alive && !team2_alive {
                    wins += 1;
                    total_hp += last_round
                        .team1
                        .iter()
                        .map(|c| c.final_state.current_hp)
                        .sum::<f64>();
                }
            }
        }
    }

    ScenarioStats {
        win_rate: (wins as f64 / count) * 100.0,
        avg_hp_left: if wins > 0 {
            total_hp / wins as f64
        } else {
            0.0
        },
        avg_rounds: total_rounds / count,
    }
}

// --- Validate Subcommand ---

fn run_validate(scenario_path: &PathBuf) {
    println!("=== Validating Scenario ===\n");

    let content = match fs::read_to_string(scenario_path) {
        Ok(c) => c,
        Err(e) => {
            println!("[ERROR] Failed to read file: {}", e);
            return;
        }
    };

    let data: serde_json::Value = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            println!("[ERROR] Invalid JSON: {}", e);
            return;
        }
    };

    let mut errors = 0;
    let mut warnings = 0;

    // Check for required fields
    if data.get("players").is_none() {
        println!("[ERROR] Missing 'players' array");
        errors += 1;
    }
    if data.get("encounters").is_none() {
        println!("[ERROR] Missing 'encounters' array");
        errors += 1;
    }

    // Validate players
    if let Some(players) = data.get("players").and_then(|p| p.as_array()) {
        for (i, player) in players.iter().enumerate() {
            validate_creature(
                player,
                &format!("players[{}]", i),
                &mut errors,
                &mut warnings,
            );
        }
    }

    // Validate encounters
    if let Some(encounters) = data.get("encounters").and_then(|e| e.as_array()) {
        for (i, encounter) in encounters.iter().enumerate() {
            if let Some(monsters) = encounter.get("monsters").and_then(|m| m.as_array()) {
                for (j, monster) in monsters.iter().enumerate() {
                    validate_creature(
                        monster,
                        &format!("encounters[{}].monsters[{}]", i, j),
                        &mut errors,
                        &mut warnings,
                    );
                }
            } else {
                println!("[ERROR] encounters[{}] missing 'monsters' array", i);
                errors += 1;
            }
        }
    }

    println!();
    if errors == 0 && warnings == 0 {
        println!("[INFO] ✅ Scenario is valid. Ready to run.");
    } else {
        println!(
            "[INFO] Validation complete: {} errors, {} warnings",
            errors, warnings
        );
    }
}

fn validate_creature(
    creature: &serde_json::Value,
    path: &str,
    errors: &mut u32,
    warnings: &mut u32,
) {
    // Check name
    if let Some(name) = creature.get("name").and_then(|n| n.as_str()) {
        if name.is_empty() {
            println!("[WARN] {} has empty name", path);
            *warnings += 1;
        }
    } else {
        println!("[ERROR] {} missing 'name' field", path);
        *errors += 1;
    }

    // Check HP
    if let Some(hp) = creature.get("hp").and_then(|h| h.as_f64()) {
        if hp <= 0.0 {
            println!("[ERROR] {} has HP <= 0 ({})", path, hp);
            *errors += 1;
        }
    } else {
        println!("[ERROR] {} missing 'hp' field", path);
        *errors += 1;
    }

    // Check AC (allow negative for edge cases, but warn)
    if let Some(ac) = creature.get("AC").and_then(|a| a.as_f64()) {
        if ac < 0.0 {
            println!("[WARN] {} has negative AC ({})", path, ac);
            *warnings += 1;
        }
    }

    // Check actions
    if let Some(actions) = creature.get("actions").and_then(|a| a.as_array()) {
        for (i, action) in actions.iter().enumerate() {
            let action_path = format!("{}.actions[{}]", path, i);

            // Check for empty action name
            if let Some(name) = action.get("name").and_then(|n| n.as_str()) {
                if name.is_empty() {
                    println!("[WARN] {} has empty name", action_path);
                    *warnings += 1;
                }
            }

            // Check toHit is not a dice formula (common mistake)
            if let Some(to_hit) = action.get("toHit").and_then(|t| t.as_str()) {
                if to_hit.contains("d20") {
                    println!("[ERROR] {} toHit contains 'd20'. toHit should be a modifier, not a full roll formula.", action_path);
                    *errors += 1;
                }
            }
        }
    } else {
        println!("[WARN] {} has no 'actions' array", path);
        *warnings += 1;
    }
}
