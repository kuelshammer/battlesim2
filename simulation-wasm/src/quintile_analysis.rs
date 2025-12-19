use crate::model::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RiskFactor {
    Safe,           // Even worst case, party survives
    Volatile,       // Worst case = near death, typical case = fine
    Lethal,         // Worst case = guaranteed death
    TPKRisk,        // Worst case = total party kill
    Suicidal,       // Typical case = party loses >50%
}

impl std::fmt::Display for RiskFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskFactor::Safe => write!(f, "Safe"),
            RiskFactor::Volatile => write!(f, "Volatile"),
            RiskFactor::Lethal => write!(f, "Lethal"),
            RiskFactor::TPKRisk => write!(f, "TPKRisk"),
            RiskFactor::Suicidal => write!(f, "Suicidal"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Difficulty {
    Easy,           // >85% HP remaining
    Medium,         // 60-85% HP remaining  
    Hard,           // 30-60% HP remaining
    Grueling,       // <30% HP remaining
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EncounterLabel {
    Trivial,        // Safe + Easy
    Standard,       // Safe + Medium
    Grinder,        // Safe + Hard
    GlassCannon,    // Volatile + Easy
    Spicy,          // Volatile + Medium
    Deadly,         // Lethal + Hard
    Catastrophic,   // TPK Risk + Any
}

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
    // NEW: Encounter rating fields
    pub risk_factor: RiskFactor,
    pub difficulty: Difficulty,
    pub encounter_label: EncounterLabel,
    pub analysis_summary: String,
    pub tuning_suggestions: Vec<String>,
}

/// Extract combatant visualization data from a single simulation result
fn extract_combatant_visualization(result: &SimulationResult) -> (Vec<CombatantVisualization>, usize) {
    let mut combatants = Vec::new();
    let mut battle_duration = 0;

    // Validate result is not empty
    if result.is_empty() {
        return (combatants, battle_duration);
    }
    
    // Calculate total duration across all encounters
    for encounter in result {
        battle_duration += encounter.rounds.len();
    }

    // Only visualize the final state of the last encounter
    // This prevents cluttering the view with dead monsters from previous encounters
    if let Some(final_encounter) = result.last() {
        if let Some(last_round) = final_encounter.rounds.last() {
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
    _start_idx: usize, // Unused parameter, kept for API compatibility
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
    // Use dynamic middle index instead of fixed 100 to handle variable slice sizes
    let middle_idx_in_slice = if results.is_empty() { 
        0 
    } else { 
        results.len() / 2 
    };
    
    // Bounds checking to prevent panic
    if results.is_empty() || middle_idx_in_slice >= results.len() {
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

/// Assess risk factor based on quintile analysis - implements Safety Grade system (A-F)
fn assess_risk_factor(quintiles: &[QuintileStats]) -> RiskFactor {
    let disaster = &quintiles[0];  // Disaster run (#125) - 5th Percentile
    let struggle = &quintiles[1]; // Struggle run (#627) - 25th Percentile
    let typical = &quintiles[2];  // Typical run (#1255) - 50th Percentile

    // Check for Safety Grade F (Broken) - Typical run is TPK
    if typical.median_survivors == 0 {
        return RiskFactor::Suicidal; // Equivalent to Safety Grade F
    }

    // Check for Safety Grade D (Unstable) - Struggle run is TPK
    if struggle.median_survivors == 0 {
        return RiskFactor::TPKRisk; // Equivalent to Safety Grade D
    }

    // Check for Safety Grade C (Risky) - Disaster run is TPK but Struggle run is alive
    if disaster.median_survivors == 0 && struggle.median_survivors > 0 {
        return RiskFactor::Lethal; // Equivalent to Safety Grade C
    }

    // Check for Safety Grade B (Fair) - Disaster run is alive but has <5% HP remaining
    let disaster_lowest_hp = disaster.median_run_visualization
        .iter()
        .filter(|c| c.is_player && !c.is_dead)
        .map(|c| c.hp_percentage)
        .fold(100.0_f64, |acc, hp| acc.min(hp));

    if disaster.median_survivors > 0 && disaster_lowest_hp < 5.0 {
        return RiskFactor::Volatile; // Equivalent to Safety Grade B
    }

    // Safety Grade A (Secure) - Disaster run has >10% HP remaining
    RiskFactor::Safe
}

/// Assess difficulty based on typical quintile HP remaining - implements Intensity Tier system (1-5)
fn assess_difficulty(quintiles: &[QuintileStats]) -> Difficulty {
    let typical = &quintiles[2]; // Typical run (#1255) - 50th Percentile

    // Calculate total resources remaining (HP + Spell Slots proxy)
    // For simplicity, we'll use HP percentage as the primary resource indicator
    let surviving_players: Vec<_> = typical.median_run_visualization
        .iter()
        .filter(|c| c.is_player && !c.is_dead)
        .collect();

    if surviving_players.is_empty() {
        return Difficulty::Grueling; // All dead = hardest difficulty
    }

    let avg_hp_percent = surviving_players
        .iter()
        .map(|c| c.hp_percentage)
        .sum::<f64>()
        / surviving_players.len() as f64;

    // Intensity Tier 1: Trivial (> 90% Resources Left)
    if avg_hp_percent > 90.0 {
        Difficulty::Easy
    }
    // Intensity Tier 2: Light (70% – 90% Resources Left)
    else if avg_hp_percent >= 70.0 {
        Difficulty::Medium
    }
    // Intensity Tier 3: Moderate (40% – 70% Resources Left)
    else if avg_hp_percent >= 40.0 {
        Difficulty::Hard
    }
    // Intensity Tier 4: Heavy (10% – 40% Resources Left)
    else if avg_hp_percent >= 10.0 {
        Difficulty::Grueling
    }
    // Intensity Tier 5: Extreme (< 10% Resources Left)
    else {
        Difficulty::Grueling
    }
}

/// Get combined encounter label based on risk and difficulty - implements Sweet Spot labels
fn get_encounter_label(risk: &RiskFactor, difficulty: &Difficulty) -> EncounterLabel {
    match (risk, difficulty) {
        // The Epic Challenge: B + 4 (Fair + Heavy)
        (RiskFactor::Volatile, Difficulty::Grueling) => EncounterLabel::Deadly, // B + 4

        // The Tactical Grinder: A + 3 (Secure + Moderate)
        (RiskFactor::Safe, Difficulty::Hard) => EncounterLabel::Grinder, // A + 3

        // The Action Movie: B + 2 (Fair + Light)
        (RiskFactor::Volatile, Difficulty::Medium) => EncounterLabel::Spicy, // B + 2

        // The Trap: C + 2 (Risky + Light)
        (RiskFactor::Lethal, Difficulty::Medium) => EncounterLabel::Catastrophic, // C + 2

        // The Slog: A + 5 (Secure + Extreme)
        (RiskFactor::Safe, Difficulty::Grueling) => EncounterLabel::Grinder, // A + 5

        // Default mappings for other combinations
        (RiskFactor::Safe, Difficulty::Easy) => EncounterLabel::Trivial,
        (RiskFactor::Safe, Difficulty::Medium) => EncounterLabel::Standard,
        (RiskFactor::Volatile, Difficulty::Easy) => EncounterLabel::GlassCannon,
        (RiskFactor::Lethal, Difficulty::Hard) => EncounterLabel::Deadly,
        (RiskFactor::TPKRisk, _) => EncounterLabel::Catastrophic,
        (RiskFactor::Suicidal, _) => EncounterLabel::Catastrophic,
        _ => EncounterLabel::Standard, // Default fallback
    }
}

/// Generate analysis summary based on ratings - includes Safety Grade and Intensity Tier
fn generate_analysis_summary(_risk: &RiskFactor, _difficulty: &Difficulty, quintiles: &[QuintileStats]) -> String {
    let disaster = &quintiles[0];
    let struggle = &quintiles[1];
    let typical = &quintiles[2];

    // Determine Safety Grade
    let safety_grade = if disaster.median_survivors == 0 && struggle.median_survivors == 0 {
        "F (Broken)"
    } else if struggle.median_survivors == 0 {
        "D (Unstable)"
    } else if disaster.median_survivors == 0 {
        "C (Risky)"
    } else if disaster.median_run_visualization.iter()
        .filter(|c| c.is_player && !c.is_dead)
        .map(|c| c.hp_percentage)
        .fold(100.0_f64, |acc, hp| acc.min(hp)) < 5.0 {
        "B (Fair)"
    } else {
        "A (Secure)"
    };

    // Determine Intensity Tier
    let intensity_tier = if typical.median_run_visualization.iter()
        .filter(|c| c.is_player && !c.is_dead)
        .map(|c| c.hp_percentage)
        .sum::<f64>() / typical.median_run_visualization.len() as f64 > 90.0 {
        "1 (Trivial)"
    } else if typical.median_run_visualization.iter()
        .filter(|c| c.is_player && !c.is_dead)
        .map(|c| c.hp_percentage)
        .sum::<f64>() / typical.median_run_visualization.len() as f64 >= 70.0 {
        "2 (Light)"
    } else if typical.median_run_visualization.iter()
        .filter(|c| c.is_player && !c.is_dead)
        .map(|c| c.hp_percentage)
        .sum::<f64>() / typical.median_run_visualization.len() as f64 >= 40.0 {
        "3 (Moderate)"
    } else if typical.median_run_visualization.iter()
        .filter(|c| c.is_player && !c.is_dead)
        .map(|c| c.hp_percentage)
        .sum::<f64>() / typical.median_run_visualization.len() as f64 >= 10.0 {
        "4 (Heavy)"
    } else {
        "5 (Extreme)"
    };

    format!(
        "Safety Grade: {} | Intensity Tier: {} | Disaster: {} survivors | Typical: {} survivors",
        safety_grade, intensity_tier, disaster.median_survivors, typical.median_survivors
    )
}

/// Generate tuning suggestions based on encounter analysis - matches methodology
fn generate_tuning_suggestions(risk: &RiskFactor, difficulty: &Difficulty, _quintiles: &[QuintileStats]) -> Vec<String> {
    let mut suggestions = Vec::new();

    match risk {
        RiskFactor::Volatile => {
            suggestions.push("High variance encounter. Consider reducing monster burst damage to make outcomes more consistent.".to_string());
        }
        RiskFactor::Lethal => {
            suggestions.push("Dangerous encounter. Consider reducing monster damage or increasing player survivability to prevent TPKs.".to_string());
        }
        RiskFactor::TPKRisk => {
            suggestions.push("Total Party Kill risk. Significant nerfing required. Consider reducing monster damage or increasing party power.".to_string());
        }
        RiskFactor::Suicidal => {
            suggestions.push("Encounter is mathematically impossible. Consider reducing monster stats or increasing player resources.".to_string());
        }
        _ => {}
    }

    match difficulty {
        Difficulty::Easy => {
            suggestions.push("Encounter may be too easy for the party's power level.".to_string());
        }
        Difficulty::Grueling => {
            suggestions.push("Encounter is very resource-intensive. Consider adding a short rest opportunity.".to_string());
        }
        _ => {}
    }

    suggestions
}

/// Run analysis on a set of results (helper function)
fn analyze_results(results: &[SimulationResult], scenario_name: &str, party_size: usize) -> AggregateOutput {
    // Validate input
    if results.is_empty() {
        return AggregateOutput {
            scenario_name: scenario_name.to_string(),
            total_runs: 0,
            quintiles: Vec::new(),
            risk_factor: RiskFactor::Safe,
            difficulty: Difficulty::Medium,
            encounter_label: EncounterLabel::Standard,
            analysis_summary: "No simulation data available.".to_string(),
            tuning_suggestions: Vec::new(),
        };
    }

    // The methodology requires exactly 2,510 runs for proper quintile mapping
    let total_runs = results.len();
    let required_runs = 2510;

    // If we have fewer than required runs, use what we have but warn
    if total_runs < required_runs {
        eprintln!("WARNING: Only {} runs available, but methodology requires 2,510 for precise quintile mapping", total_runs);
    }

    let mut quintiles = Vec::new();

    // Define the specific run indices for the 5 Battle Cards according to methodology
    // UI Row Label | Statistical Meaning | Which Slice? | Run Index (0-2509)
    // 1. Disaster   | 5th Percentile      | Slice 1      | #125
    // 2. Struggle   | 25th Percentile     | Slice 3      | #627
    // 3. Typical    | 50th Percentile     | (Global)     | #1,255
    // 4. Heroic     | 75th Percentile     | Slice 8      | #1,882
    // 5. Legend     | 95th Percentile     | Slice 10     | #2,384

    // Calculate the specific run indices for each Battle Card
    let disaster_run_idx = 125;
    let struggle_run_idx = 627;
    let typical_run_idx = 1255;
    let heroic_run_idx = 1882;
    let legend_run_idx = 2384;

    // Process each of the 5 Battle Cards using the specific run indices
    let battle_card_indices = [
        (1, "Disaster", disaster_run_idx),
        (2, "Struggle", struggle_run_idx),
        (3, "Typical", typical_run_idx),
        (4, "Heroic", heroic_run_idx),
        (5, "Legend", legend_run_idx),
    ];

    for (quintile_num, label, run_idx) in battle_card_indices {
        // Bounds checking for the specific run index
        if run_idx >= results.len() {
            // Add empty quintile stats for invalid index
            quintiles.push(QuintileStats {
                quintile: quintile_num,
                label: label.to_string(),
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

        let single_run = &results[run_idx];
        let score = crate::aggregation::calculate_score(single_run);
        let survivors = ((score / 10000.0).floor() as usize).min(party_size);

        // Extract visualization data for this specific run
        let (visualization_data, battle_duration) = extract_combatant_visualization(single_run);

        // Calculate HP lost for this specific run
        let mut total_hp_lost = 0.0;
        let mut party_max_hp = 0.0;

        if let Some(encounter) = single_run.first() {
            if let Some(first_round) = encounter.rounds.first() {
                for c in first_round.team1.iter() {
                    if c.creature.hp > 0.0 {
                        party_max_hp += c.creature.hp;
                    }
                }
            }
        }

        let remaining_party_hp = score - (survivors as f64 * 10000.0);
        let hp_lost = party_max_hp - remaining_party_hp;
        if hp_lost >= 0.0 {
            total_hp_lost = hp_lost;
        }

        let hp_lost_percent = if party_max_hp > 0.0 {
            (total_hp_lost / party_max_hp) * 100.0
        } else {
            0.0
        };

        quintiles.push(QuintileStats {
            quintile: quintile_num,
            label: label.to_string(),
            median_survivors: survivors,
            party_size,
            total_hp_lost,
            hp_lost_percent,
            win_rate: if survivors > 0 { 100.0 } else { 0.0 }, // Single run has 100% win or 0% win
            median_run_visualization: visualization_data,
            battle_duration_rounds: battle_duration,
        });
    }

    // Calculate encounter ratings using the specific Battle Card data
    let risk_factor = assess_risk_factor(&quintiles);
    let difficulty = assess_difficulty(&quintiles);
    let encounter_label = get_encounter_label(&risk_factor, &difficulty);
    let analysis_summary = generate_analysis_summary(&risk_factor, &difficulty, &quintiles);
    let tuning_suggestions = generate_tuning_suggestions(&risk_factor, &difficulty, &quintiles);

    AggregateOutput {
        scenario_name: scenario_name.to_string(),
        total_runs: results.len(),
        quintiles,
        risk_factor,
        difficulty,
        encounter_label,
        analysis_summary,
        tuning_suggestions,
    }
}

/// Run quintile analysis on simulation results (Overall Adventure)
pub fn run_quintile_analysis(results: &[SimulationResult], scenario_name: &str, party_size: usize) -> AggregateOutput {
    analyze_results(results, scenario_name, party_size)
}

/// Run day analysis on simulation results (Adventuring Day)
pub fn run_day_analysis(results: &[SimulationResult], scenario_name: &str, party_size: usize) -> AggregateOutput {
    // The methodology requires exactly 2,510 runs for proper quintile mapping
    let total_runs = results.len();
    let required_runs = 2510;

    // If we have fewer than required runs, use what we have but warn
    if total_runs < required_runs {
        eprintln!("WARNING: Only {} runs available, but methodology requires 2,510 for precise day rating", total_runs);
    }

    let mut quintiles = Vec::new();

    // Define the specific run indices for the 5 Battle Cards according to methodology
    // UI Row Label | Statistical Meaning | Which Slice? | Run Index (0-2509)
    // 1. Disaster   | 5th Percentile      | Slice 1      | #125
    // 2. Struggle   | 25th Percentile     | Slice 3      | #627
    // 3. Typical    | 50th Percentile     | (Global)     | #1,255
    // 4. Heroic     | 75th Percentile     | Slice 8      | #1,882
    // 5. Legend     | 95th Percentile     | Slice 10     | #2,384

    // Calculate the specific run indices for each Battle Card
    let disaster_run_idx = 125;
    let struggle_run_idx = 627;
    let typical_run_idx = 1255;
    let heroic_run_idx = 1882;
    let legend_run_idx = 2384;

    // Process each of the 5 Battle Cards using the specific run indices
    let battle_card_indices = [
        (1, "Disaster", disaster_run_idx),
        (2, "Struggle", struggle_run_idx),
        (3, "Typical", typical_run_idx),
        (4, "Heroic", heroic_run_idx),
        (5, "Legend", legend_run_idx),
    ];

    for (quintile_num, label, run_idx) in battle_card_indices {
        // Bounds checking for the specific run index
        if run_idx >= results.len() {
            // Add empty quintile stats for invalid index
            quintiles.push(QuintileStats {
                quintile: quintile_num,
                label: label.to_string(),
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

        let single_run = &results[run_idx];
        let score = crate::aggregation::calculate_score(single_run);
        let survivors = ((score / 10000.0).floor() as usize).min(party_size);

        // Extract visualization data for this specific run
        let (visualization_data, battle_duration) = extract_combatant_visualization(single_run);

        // Calculate HP lost for this specific run
        let mut total_hp_lost = 0.0;
        let mut party_max_hp = 0.0;

        if let Some(encounter) = single_run.first() {
            if let Some(first_round) = encounter.rounds.first() {
                for c in first_round.team1.iter() {
                    if c.creature.hp > 0.0 {
                        party_max_hp += c.creature.hp;
                    }
                }
            }
        }

        let remaining_party_hp = score - (survivors as f64 * 10000.0);
        let hp_lost = party_max_hp - remaining_party_hp;
        if hp_lost >= 0.0 {
            total_hp_lost = hp_lost;
        }

        let hp_lost_percent = if party_max_hp > 0.0 {
            (total_hp_lost / party_max_hp) * 100.0
        } else {
            0.0
        };

        quintiles.push(QuintileStats {
            quintile: quintile_num,
            label: label.to_string(),
            median_survivors: survivors,
            party_size,
            total_hp_lost,
            hp_lost_percent,
            win_rate: if survivors > 0 { 100.0 } else { 0.0 }, // Single run has 100% win or 0% win
            median_run_visualization: visualization_data,
            battle_duration_rounds: battle_duration,
        });
    }

    // Calculate day ratings using the specific Battle Card data
    let risk_factor = assess_risk_factor(&quintiles);
    let difficulty = assess_difficulty(&quintiles);
    let encounter_label = get_encounter_label(&risk_factor, &difficulty);

    // Calculate day rating
    let day_rating = calculate_day_rating(&quintiles);

    let analysis_summary = generate_day_analysis_summary(&risk_factor, &difficulty, &quintiles, &day_rating);
    let tuning_suggestions = generate_day_tuning_suggestions(&risk_factor, &difficulty, &day_rating);

    AggregateOutput {
        scenario_name: scenario_name.to_string(),
        total_runs: results.len(),
        quintiles,
        risk_factor,
        difficulty,
        encounter_label,
        analysis_summary,
        tuning_suggestions,
    }
}

/// Calculate day rating based on methodology
fn calculate_day_rating(quintiles: &[QuintileStats]) -> String {
    let disaster = &quintiles[0];  // Disaster run (#125) - 5th Percentile
    let typical = &quintiles[2];  // Typical run (#1255) - 50th Percentile

    // Check for Safety: If Run #125 is TPK -> Grade C/D
    if disaster.median_survivors == 0 {
        return "Grade C/D".to_string();
    }

    // If Run #125 is Alive but <5% HP -> Grade B
    let disaster_lowest_hp = disaster.median_run_visualization
        .iter()
        .filter(|c| c.is_player && !c.is_dead)
        .map(|c| c.hp_percentage)
        .fold(100.0_f64, |acc, hp| acc.min(hp));

    if disaster_lowest_hp < 5.0 {
        return "Grade B".to_string();
    }

    // Otherwise -> Grade A
    "Grade A".to_string()
}

/// Generate day analysis summary based on ratings
fn generate_day_analysis_summary(_risk: &RiskFactor, _difficulty: &Difficulty, quintiles: &[QuintileStats], day_rating: &str) -> String {
    let disaster = &quintiles[0];
    let typical = &quintiles[2];

    // Determine Intensity based on Typical run resources
    let intensity = if typical.median_run_visualization.iter()
        .filter(|c| c.is_player && !c.is_dead)
        .map(|c| c.hp_percentage)
        .sum::<f64>() / typical.median_run_visualization.len() as f64 > 60.0 {
        "Under-tuned"
    } else if typical.median_run_visualization.iter()
        .filter(|c| c.is_player && !c.is_dead)
        .map(|c| c.hp_percentage)
        .sum::<f64>() / typical.median_run_visualization.len() as f64 >= 30.0 {
        "Standard Day"
    } else if typical.median_run_visualization.iter()
        .filter(|c| c.is_player && !c.is_dead)
        .map(|c| c.hp_percentage)
        .sum::<f64>() / typical.median_run_visualization.len() as f64 >= 5.0 {
        "Perfect Challenge"
    } else {
        "Overwhelming"
    };

    format!(
        "Day Rating: {} | Safety: {} | Intensity: {} | Disaster: {} survivors | Typical: {} survivors",
        day_rating, disaster.median_survivors, intensity, disaster.median_survivors, typical.median_survivors
    )
}

/// Generate day tuning suggestions based on analysis
fn generate_day_tuning_suggestions(_risk: &RiskFactor, difficulty: &Difficulty, day_rating: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    match day_rating {
        "Grade C/D" => {
            suggestions.push("Day is too dangerous. Consider reducing encounter difficulty or adding more short rests.".to_string());
        }
        "Grade B" => {
            suggestions.push("Day has good balance. Party survives worst case but may need resources for next encounters.".to_string());
        }
        "Grade A" => {
            suggestions.push("Day is very safe. Consider increasing encounter difficulty to provide better challenge.".to_string());
        }
        _ => {}
    }

    match difficulty {
        Difficulty::Easy => {
            suggestions.push("Day may be too easy for the party's power level.".to_string());
        }
        Difficulty::Grueling => {
            suggestions.push("Day is very resource-intensive. Consider adding a short rest opportunity.".to_string());
        }
        _ => {}
    }

    suggestions
}

/// Run quintile analysis for a specific encounter
pub fn run_encounter_analysis(results: &[SimulationResult], encounter_idx: usize, scenario_name: &str, party_size: usize) -> AggregateOutput {
    // Filter results to only include the specific encounter
    let mut encounter_results: Vec<SimulationResult> = results.iter()
        .filter_map(|run| {
            if encounter_idx < run.len() {
                Some(vec![run[encounter_idx].clone()])
            } else {
                None
            }
        })
        .collect();

    // Sort results by score from worst to best performance for this specific encounter
    encounter_results.sort_by(|a, b| {
        let score_a = crate::aggregation::calculate_score(a);
        let score_b = crate::aggregation::calculate_score(b);
        score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    analyze_results(&encounter_results, scenario_name, party_size)
}