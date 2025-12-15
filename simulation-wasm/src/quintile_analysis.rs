use crate::model::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CombatantVisualization {
    pub name: String,           // "Aragorn", "Gandalf", "Adult Red Dragon"
    pub max_hp: f64,           // 65, 35, 300
    pub current_hp: f64,        // 0, 13, 0
    pub is_dead: bool,         // true, false, true
    pub is_player: bool,       // true, true, false
    pub hp_percentage: f64,    // 0.0, 20.0, 100.0
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuintileStats {
    pub quintile: usize, // 1-5
    pub label: String,   // "Worst 20%", "Below Average", "Median", "Above Average", "Best 20%"
    pub median_survivors: usize, // Median number of survivors
    pub party_size: usize,       // Total party size
    pub total_hp_lost: f64,     // Average HP lost by party per run
    pub hp_lost_percent: f64,   // HP lost as percentage of total possible HP
    pub win_rate: f64,
    // New fields for 5-Timeline Dashboard
    pub median_run_visualization: Vec<CombatantVisualization>,
    pub battle_duration_rounds: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AggregateOutput {
    pub scenario_name: String,
    pub total_runs: usize,
    pub quintiles: Vec<QuintileStats>,
}

/// Extract combatant visualization data from a single simulation result
fn extract_combatant_visualization(result: &SimulationResult) -> (Vec<CombatantVisualization>, usize) {
    let mut combatants = Vec::new();
    let mut battle_duration = 0;

    // Validate result is not empty
    if result.is_empty() {
        return (combatants, battle_duration);
    }

    for encounter in result {
        battle_duration += encounter.rounds.len();
        
        // Get final states from last round
        if let Some(last_round) = encounter.rounds.last() {
            // Process team 1 (players)
            for combatant in &last_round.team1 {
                let hp_percentage = if combatant.creature.hp > 0.0 {
                    (combatant.final_state.current_hp / combatant.creature.hp) * 100.0
                } else {
                    0.0
                };

                combatants.push(CombatantVisualization {
                    name: combatant.creature.name.clone(),
                    max_hp: combatant.creature.hp,
                    current_hp: combatant.final_state.current_hp,
                    is_dead: combatant.final_state.current_hp <= 0.0,
                    is_player: true,
                    hp_percentage,
                });
            }

            // Process team 2 (monsters)
            for combatant in &last_round.team2 {
                let hp_percentage = if combatant.creature.hp > 0.0 {
                    (combatant.final_state.current_hp / combatant.creature.hp) * 100.0
                } else {
                    0.0
                };

                combatants.push(CombatantVisualization {
                    name: combatant.creature.name.clone(),
                    max_hp: combatant.creature.hp,
                    current_hp: combatant.final_state.current_hp,
                    is_dead: combatant.final_state.current_hp <= 0.0,
                    is_player: false,
                    hp_percentage,
                });
            }
        }
    }

    (combatants, battle_duration)
}

/// Calculate statistics for a quintile of simulation results
fn calculate_quintile_stats(
    results: &[SimulationResult],
    quintile_num: usize,
    label: &str,
    median_survivors: usize,
    party_size: usize,
    start_idx: usize,
) -> QuintileStats {
    // Validate input
    if results.is_empty() {
        return QuintileStats {
            quintile: quintile_num,
            label: label.to_string(),
            median_survivors,
            party_size,
            total_hp_lost: 0.0,
            hp_lost_percent: 0.0,
            win_rate: 0.0,
            median_run_visualization: Vec::new(),
            battle_duration_rounds: 0,
        };
    }

    let count = results.len() as f64;
    let mut wins = 0;
    let mut survivor_counts = Vec::new();
    let mut total_hp_lost = 0.0;

    // Calculate max HP once (assuming all runs have same creatures)
    let mut party_max_hp = 0.0;
    if let Some(result) = results.first() {
        if let Some(encounter) = result.last() {
            if let Some(first_round) = encounter.rounds.first() {
                for c in first_round.team1.iter() {
                    if c.creature.hp > 0.0 {
                        party_max_hp += c.creature.hp;
                    }
                }
            }
        }
    }

    // Calculate survivors and wins from score, and HP lost
    for result in results {
        // Validate result structure
        if result.is_empty() {
            continue;
        }
        
        let score = crate::aggregation::calculate_score(result);

        // Determine if win and survivors
        let survivors = ((score / 10000.0).floor() as usize).min(party_size);
        let is_win = survivors > 0;

        if is_win {
            wins += 1;
            survivor_counts.push(survivors as f64);

            // For wins: score = (survivors × 10,000) + remaining_party_hp - 0
            let remaining_party_hp = score - (survivors as f64 * 10000.0);
            let hp_lost = party_max_hp - remaining_party_hp;
            if hp_lost >= 0.0 {
                total_hp_lost += hp_lost;
            }
        } else {
            survivor_counts.push(0.0);

            // For losses: all party HP is lost
            if party_max_hp > 0.0 {
                total_hp_lost += party_max_hp;
            }
        }
    }

    // Calculate average survivors (for reference, though we use median)
    let _avg_survivors = survivor_counts.iter().sum::<f64>() / survivor_counts.len() as f64;

    // Calculate HP lost percentage
    let total_party_hp = party_max_hp * count;
    let hp_lost_percent = if total_party_hp > 0.0 {
        (total_hp_lost / total_party_hp) * 100.0
    } else {
        0.0
    };

    // Extract median run visualization data
    let middle_idx_in_slice = 100; // 101st run in a 201-run slice
    let absolute_median_run_idx = start_idx + middle_idx_in_slice;
    
    // Bounds checking to prevent panic
    if middle_idx_in_slice >= results.len() {
        return QuintileStats {
            quintile: quintile_num,
            label: label.to_string(),
            median_survivors,
            party_size,
            total_hp_lost: total_hp_lost / count, // Average HP lost per run
            hp_lost_percent,
            win_rate: (wins as f64 / count) * 100.0,
            median_run_visualization: Vec::new(),
            battle_duration_rounds: 0,
        };
    }
    
    let median_run = &results[middle_idx_in_slice];
    let (visualization_data, battle_duration) = extract_combatant_visualization(median_run);

    QuintileStats {
        quintile: quintile_num,
        label: label.to_string(),
        median_survivors,
        party_size,
        total_hp_lost: total_hp_lost / count, // Average HP lost per run
        hp_lost_percent,
        win_rate: (wins as f64 / count) * 100.0,
        median_run_visualization: visualization_data,
        battle_duration_rounds: battle_duration,
    }
}

/// Run quintile analysis on simulation results
pub fn run_quintile_analysis(results: &[SimulationResult], scenario_name: &str, party_size: usize) -> AggregateOutput {
    // Validate input
    if results.is_empty() {
        return AggregateOutput {
            scenario_name: scenario_name.to_string(),
            total_runs: 0,
            quintiles: Vec::new(),
        };
    }

    let quintile_labels = [
        "Worst 20%",
        "Below Average", 
        "Median",
        "Above Average",
        "Best 20%",
    ];
    let mut quintiles = Vec::new();

    // Calculate median survivors for each quintile (middle run of each quintile)
    let mut median_survivors = Vec::new();
    for q in 0..5 {
        let start = q * 201;
        let middle_idx = start + 100; // 101st run in quintile (0-indexed: 100)
        
        // Bounds checking
        if middle_idx >= results.len() {
            median_survivors.push(0);
            continue;
        }
        
        let score = crate::aggregation::calculate_score(&results[middle_idx]);
        let survivors = ((score / 10000.0).floor() as usize).min(party_size);
        median_survivors.push(survivors);
    }

    for q in 0..5 {
        let start = q * 201;
        let end = (q + 1) * 201;
        
        // Bounds checking for slice
        if start >= results.len() || end > results.len() {
            // Add empty quintile stats for invalid range
            quintiles.push(QuintileStats {
                quintile: q + 1,
                label: quintile_labels[q].to_string(),
                median_survivors: 0,
                party_size,
                total_hp_lost: 0.0,
                hp_lost_percent: 0.0,
                win_rate: 0.0,
                median_run_visualization: Vec::new(),
                battle_duration_rounds: 0,
            });
            continue;
        }
        
        let slice = &results[start..end];

        let stats = calculate_quintile_stats(slice, q + 1, quintile_labels[q], median_survivors[q], party_size, start);
        quintiles.push(stats);
    }

    AggregateOutput {
        scenario_name: scenario_name.to_string(),
        total_runs: results.len(),
        quintiles,
    }
}