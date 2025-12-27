use clap::{Parser, Subcommand};
use simulation_wasm::aggregation::calculate_score;
use simulation_wasm::dice;
use simulation_wasm::events::Event;
use simulation_wasm::model::{Action, Creature, DiceFormula, Encounter, SimulationResult, SimulationRun, TimelineStep};
use simulation_wasm::decile_analysis::run_decile_analysis;
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
    /// Generate aggregated statistics for each decile from 2511 simulation runs
    Aggregate {
        /// Path to the scenario JSON file
        scenario: PathBuf,
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
    /// Find the simulation run closest to the median decile
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
    /// Generate detailed event logs for multiple random simulation runs
    BatchLog {
        /// Path to the scenario JSON file
        scenario: PathBuf,
        /// Number of runs to generate (default 10)
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
}

// --- Main Entry Point ---

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Aggregate { scenario } => {
            run_aggregate(&scenario);
        }
        Commands::BatchLog { scenario, count } => {
            run_batch_log(&scenario, count);
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

fn run_aggregate(scenario_path: &PathBuf) {
    let (players, timeline, scenario_name) = load_scenario(scenario_path);

    // Get party size
    let party_size = players.len();

    // Run 2511 iterations to match frontend and methodology (10 slices of 251 + 1 median)
    let iterations = 2511;
    println!("Running {} iterations for Adventuring Day: {}...", iterations, scenario_name);
    let mut results = run_event_driven_simulation_rust(players, timeline, iterations, false);

    // Sort results by score from worst to best performance
    results.sort_by(|a, b| calculate_score(&a.result).partial_cmp(&calculate_score(&b.result)).unwrap_or(std::cmp::Ordering::Equal));

    let raw_results: Vec<_> = results.iter().map(|r| r.result.clone()).collect();

    // 1. Run Per-Encounter Analysis
    let num_encounters = raw_results.first().map(|r| r.encounters.len()).unwrap_or(0);
    
    if num_encounters > 1 {
        println!("\n--- Individual Encounter Breakdown ---");
        for i in 0..num_encounters {
            let enc_analysis = simulation_wasm::decile_analysis::run_encounter_analysis(&raw_results, i, &format!("Encounter {}", i + 1), party_size);
            println!("Encounter {}: {:<20} | Grade: {:<10} | Tier: {:<15} | {}", 
                i + 1, 
                format!("{}", enc_analysis.encounter_label),
                format!("{}", enc_analysis.safety_grade),
                format!("{}", enc_analysis.intensity_tier),
                if enc_analysis.is_good_design { "✅ Good" } else { "⚠️ Review" }
            );
        }
        println!("---------------------------------------\n");
    }

    // 2. Run Overall Analysis
    let output = run_decile_analysis(&raw_results, &scenario_name, party_size);

    // Output summary and rating
    println!("OVERALL ADVENTURING DAY RATING: {}", output.scenario_name);
    println!("=====================================");
    println!("Combined Label: {} ({})", output.encounter_label, output.safety_grade);
    println!("Intensity:      {}", output.intensity_tier);
    println!("Description:    {}", output.analysis_summary);
    println!("Result:         {}", if output.is_good_design { "🏆 PERFECT DAY (B/Tier 5 or A/Tier 3-4)" } else if output.safety_grade == simulation_wasm::decile_analysis::SafetyGrade::B && output.intensity_tier == simulation_wasm::decile_analysis::IntensityTier::Tier5 { "🏆 PERFECT DAY (B/Tier 5)" } else { "⚠️ Imbalanced" });
    println!("=====================================\n");

    // Output table format
    println!("{:>15} | {:>12} | {:>12} | {:>12} | {:>10}", 
              "Decile / %ile", "Survivors", "HP Lost", "HP Lost %", "Win Rate");
    println!("----------------|--------------|--------------|------------|----------");
    for (i, decile) in output.deciles.iter().enumerate() {
        let percentile = match i {
            0 => "5th %ile",
            4 => "50th %ile",
            9 => "95th %ile",
            _ => "",
        };
        
        println!(
            "{:>15} | {:>13} | {:>12.1} | {:>10.1}% | {:>8.1}%",
            format!("{} ({})", decile.label, percentile),
            format!("{}/{}", decile.median_survivors, decile.party_size),
            decile.total_hp_lost, 
            decile.hp_lost_percent, 
            decile.win_rate
        );
    }
}

// --- Log Subcommand ---

fn run_log(scenario_path: &PathBuf, format: &str, run_index: Option<usize>) {
    let (players, timeline, _) = load_scenario(scenario_path);

    // If run_index is provided, run that many + 1 and pick the specific one
    // Otherwise, run a single simulation with logging enabled
    let runs = if let Some(idx) = run_index {
        run_event_driven_simulation_rust(players, timeline, idx + 1, true)
    } else {
        run_event_driven_simulation_rust(players, timeline, 1, true)
    };

    if runs.is_empty() {
        println!("No simulation results!");
        return;
    }

    let run = &runs[run_index.unwrap_or(0).min(runs.len() - 1)];
    let result = &run.result;

    // We need to rebuild the name map for formatting logs
    let mut combatant_names = HashMap::new();
    if let Some(encounter) = result.encounters.first() {
        if let Some(round) = encounter.rounds.first() {
            for c in round.team1.iter().chain(round.team2.iter()) {
                combatant_names.insert(c.id.clone(), c.creature.name.clone());
            }
        }
    }

    match format {
        "json" => {
            let events = &run.events;
            let output = serde_json::json!({
                "result": result,
                "events": events,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        "markdown" | _ => {
            // We need formatted events for markdown log
            // Since we now store events in SimulationRun, get events from the selected run
            let events = &run.events;
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

    for (enc_idx, encounter) in result.encounters.iter().enumerate() {
        println!("## Encounter {}\n", enc_idx + 1);
        for (round_idx, round) in encounter.rounds.iter().enumerate() {
            println!("### Round {}\n", round_idx + 1);

            // Show all combatants sorted by initiative
            let mut all: Vec<_> = round.team1.iter().chain(round.team2.iter()).collect();
            all.sort_by(|a, b| {
                b.initiative
                    .partial_cmp(&a.initiative)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            for c in all {
                if c.initial_state.current_hp == 0 && c.final_state.current_hp == 0 {
                    continue; // Skip dead
                }

                println!("#### {} (Initiative: {:.1})", c.creature.name, c.initiative);
                println!(
                    "- **HP**: {} → {}",
                    c.initial_state.current_hp, c.final_state.current_hp
                );

                if c.actions.is_empty() && c.final_state.current_hp > 0 {
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
    let (players, timeline, _) = load_scenario(scenario_path);

    // Run 2511 iterations
    let iterations = 2511;
    println!("Running {} iterations...", iterations);
    let runs = run_event_driven_simulation_rust(players, timeline, iterations, false);

    // Results are sorted by score. Middle decile is around indices 1004-1506.
    // We want to find the run closest to the median of the MIDDLE decile.

    let middle_start = iterations * 4 / 10; // 1004
    let middle_end = iterations * 6 / 10; // 1506
    let middle_slice = &runs[middle_start..middle_end];

    // Calculate median score of middle decile
    let median_idx = middle_slice.len() / 2;
    let median_score = calculate_score(&middle_slice[median_idx].result);

    // Also calculate average HP spread (max_hp - min_hp) for winning team and avg rounds
    let (avg_hp_spread, avg_rounds) = calculate_averages(middle_slice);

    // Now find the run within middle decile closest to these averages
    let mut best_idx = middle_start;
    let mut best_distance = f64::MAX;

    for (i, run) in runs[middle_start..middle_end].iter().enumerate() {
        let score = calculate_score(&run.result);
        let (hp_spread, rounds) = get_run_metrics(&run.result);

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
    println!("  Score: {:.2}", calculate_score(&runs[best_idx].result));
    println!("  Distance from median: {:.2}", best_distance);
}

fn calculate_averages(runs: &[SimulationRun]) -> (f64, f64) {
    let mut total_hp_spread = 0.0;
    let mut total_rounds = 0.0;
    let count = runs.len() as f64;

    for run in runs {
        let (hp_spread, rounds) = get_run_metrics(&run.result);
        total_hp_spread += hp_spread;
        total_rounds += rounds as f64;
    }

    (total_hp_spread / count, total_rounds / count)
}

fn get_run_metrics(result: &SimulationResult) -> (f64, usize) {
    if let Some(encounter) = result.encounters.last() {
        if let Some(last_round) = encounter.rounds.last() {
            let team1_alive = last_round
                .team1
                .iter()
                .any(|c| c.final_state.current_hp > 0);
            let team2_alive = last_round
                .team2
                .iter()
                .any(|c| c.final_state.current_hp > 0);

            let winning_team = if team1_alive && !team2_alive {
                &last_round.team1
            } else if team2_alive && !team1_alive {
                &last_round.team2
            } else {
                return (0.0, encounter.rounds.len());
            };

            let hps: Vec<f64> = winning_team
                .iter()
                .map(|c| c.final_state.current_hp as f64)
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
    let (players, timeline, _) = load_scenario(scenario_path);

    // Run simulation
    let _idx = run_index.unwrap_or(0);
    let runs = if let Some(idx) = run_index {
        run_event_driven_simulation_rust(players.clone(), timeline.clone(), idx + 1, true)
    } else {
        run_event_driven_simulation_rust(players.clone(), timeline.clone(), 1, true)
    };

    // Extract results and events from the runs
    let results: Vec<SimulationResult> = runs.iter().map(|run| run.result.clone()).collect();
    let run_idx = run_index.unwrap_or(0).min(runs.len().saturating_sub(1));
    let events = &runs[run_idx].events;

    if events.is_empty() {
        println!("No events generated.");
        return;
    }

    // Build name map
    let mut combatant_names = HashMap::new();
    let mut combatant_actions = HashMap::new(); // ID -> Vec<Action>

    if let Some(result) = results.first() {
        if let Some(encounter) = result.encounters.first() {
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

    for event in events {
        match event {
            Event::ActionStarted {
                actor_id,
                action_id,
                ..
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
                if let Some(action_id) = current_action.get(attacker_id.as_str()) {
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
                if let Some(action_id) = current_action.get(attacker_id.as_str()) {
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
    let (players, timeline, _) = load_scenario(scenario_path);

    // Find Attacker and Defender
    // We search in players and all combat encounters
    let mut all_creatures: Vec<Creature> = players.clone();
    for step in &timeline {
        if let TimelineStep::Combat(encounter) = step {
            all_creatures.extend(encounter.monsters.clone());
        }
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
    println!("Defender AC: {}\n", defender.ac);

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
            let needed_roll = defender.ac as f64 - to_hit_bonus;
            // Clamp needed roll to 1-20 range logic (nat 1 is miss, nat 20 is hit)
            // Chance = (21 - needed) / 20

            let mut hit_chance: f64 = if needed_roll > 20.0 {
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

fn load_scenario(path: &PathBuf) -> (Vec<Creature>, Vec<TimelineStep>, String) {
    let content = fs::read_to_string(path).expect("Failed to read scenario file");
    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    let name = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let players: Vec<Creature> =
        serde_json::from_value(data["players"].clone()).expect("Failed to parse players");
    
    let timeline: Vec<TimelineStep> = if let Some(t) = data.get("timeline") {
        serde_json::from_value(t.clone()).expect("Failed to parse timeline")
    } else {
        // Fallback to legacy encounters field
        let encounters: Vec<Encounter> = serde_json::from_value(data["encounters"].clone()).expect("Failed to parse encounters");
        encounters.into_iter().map(TimelineStep::Combat).collect()
    };

    (players, timeline, name)
}

// --- Sweep Subcommand ---

fn run_sweep(scenario_path: &PathBuf, target_name: &str, stat: &str, range_str: &str) {
    let (players, timeline, _) = load_scenario(scenario_path);

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
        let mut modified_timeline = timeline.clone();

        // Find and modify target in players
        for player in &mut modified_players {
            if player.name == target_name || player.id == target_name {
                modify_stat(player, stat, value as f64);
            }
        }

        // Find and modify target in monsters
        for step in &mut modified_timeline {
            if let TimelineStep::Combat(encounter) = step {
                for monster in &mut encounter.monsters {
                    if monster.name == target_name || monster.id == target_name {
                        modify_stat(monster, stat, value as f64);
                    }
                }
            }
        }

        // Run simulation (fewer runs for speed)
        let iterations = 201;
        let runs = run_event_driven_simulation_rust(
            modified_players,
            modified_timeline,
            iterations,
            false,
        );

        // Extract results from runs
        let results: Vec<SimulationResult> = runs.into_iter().map(|run| run.result).collect();

        // Calculate win rate for Team 1
        let mut wins = 0;
        let mut total_rounds = 0;
        for result in &results {
            if let Some(encounter) = result.encounters.last() {
                if let Some(last_round) = encounter.rounds.last() {
                    let team1_alive = last_round
                        .team1
                        .iter()
                        .any(|c| c.final_state.current_hp > 0);
                    let team2_alive = last_round
                        .team2
                        .iter()
                        .any(|c| c.final_state.current_hp > 0);
                    if team1_alive && !team2_alive {
                        wins += 1;
                    }
                    total_rounds += result.encounters.iter().map(|e| e.rounds.len()).sum::<usize>();
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
        "ac" => creature.ac = value as u32,
        "hp" => creature.hp = value as u32,
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

    let (players_a, timeline_a, name_a) = load_scenario(scenario_a_path);
    let (players_b, timeline_b, name_b) = load_scenario(scenario_b_path);

    // Run both
    let iterations = 2511;
    println!("Running {} iterations for each scenario...\n", iterations);

    let runs_a = run_event_driven_simulation_rust(players_a, timeline_a, iterations, false);
    let runs_b = run_event_driven_simulation_rust(players_b, timeline_b, iterations, false);

    // Extract results from runs
    let results_a: Vec<SimulationResult> = runs_a.into_iter().map(|run| run.result).collect();
    let results_b: Vec<SimulationResult> = runs_b.into_iter().map(|run| run.result).collect();

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
        if let Some(encounter) = result.encounters.last() {
            if let Some(last_round) = encounter.rounds.last() {
                total_rounds += result.encounters.iter().map(|e| e.rounds.len() as f64).sum::<f64>();

                let team1_alive = last_round
                    .team1
                    .iter()
                    .any(|c| c.final_state.current_hp > 0);
                let team2_alive = last_round
                    .team2
                    .iter()
                    .any(|c| c.final_state.current_hp > 0);

                if team1_alive && !team2_alive {
                    wins += 1;
                    total_hp += last_round
                        .team1
                        .iter()
                        .map(|c| c.final_state.current_hp as f64)
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



fn run_batch_log(scenario_path: &PathBuf, count: usize) {
    let (players, timeline, _) = load_scenario(scenario_path);
    let runs = run_event_driven_simulation_rust(players, timeline, count, true);
    let output = serde_json::json!({
        "runs": runs.iter().enumerate().map(|(i, run)| {
            serde_json::json!({
                "index": i,
                "result": run.result,
                "events": run.events,
            })
        }).collect::<Vec<_>>()
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
