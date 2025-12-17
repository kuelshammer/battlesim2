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

/// Assess risk factor based on quintile analysis
fn assess_risk_factor(quintiles: &[QuintileStats]) -> RiskFactor {
    let disaster = &quintiles[0];  // Worst 20%
    let typical = &quintiles[2];   // Median (40-60%)
    
    // Check Suicidal first
    if typical.median_survivors < typical.party_size {
        return RiskFactor::Suicidal;
    }
    
    // Check TPK Risk
    if disaster.win_rate == 0.0 {
        return RiskFactor::TPKRisk;
    }
    
    // Check Lethal
    if disaster.median_survivors < disaster.party_size {
        return RiskFactor::Lethal;
    }
    
    // Check Volatile (near death but no deaths)
    let lowest_hp = disaster.median_run_visualization
        .iter()
        .filter(|c| c.is_player && !c.is_dead)
        .map(|c| c.hp_percentage)
        .fold(100.0_f64, |acc, hp| acc.min(hp));
    
    if lowest_hp < 10.0 {
        return RiskFactor::Volatile;
    }
    
    // Otherwise Safe
    RiskFactor::Safe
}

/// Assess difficulty based on typical quintile HP remaining
fn assess_difficulty(quintiles: &[QuintileStats]) -> Difficulty {
    let typical = &quintiles[2]; // Median quintile
    
    // Calculate average HP percentage for surviving players
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
    
    if avg_hp_percent > 85.0 {
        Difficulty::Easy
    } else if avg_hp_percent >= 60.0 {
        Difficulty::Medium
    } else if avg_hp_percent >= 30.0 {
        Difficulty::Hard
    } else {
        Difficulty::Grueling
    }
}

/// Get combined encounter label based on risk and difficulty
fn get_encounter_label(risk: &RiskFactor, difficulty: &Difficulty) -> EncounterLabel {
    match (risk, difficulty) {
        (RiskFactor::Safe, Difficulty::Easy) => EncounterLabel::Trivial,
        (RiskFactor::Safe, Difficulty::Medium) => EncounterLabel::Standard,
        (RiskFactor::Safe, Difficulty::Hard) => EncounterLabel::Grinder,
        (RiskFactor::Volatile, Difficulty::Easy) => EncounterLabel::GlassCannon,
        (RiskFactor::Volatile, Difficulty::Medium) => EncounterLabel::Spicy,
        (RiskFactor::Lethal, Difficulty::Hard) => EncounterLabel::Deadly,
        (RiskFactor::TPKRisk, _) => EncounterLabel::Catastrophic,
        // Fallback combinations
        (RiskFactor::Volatile, Difficulty::Hard) => EncounterLabel::Deadly,
        (RiskFactor::Volatile, Difficulty::Grueling) => EncounterLabel::Deadly,
        (RiskFactor::Lethal, Difficulty::Grueling) => EncounterLabel::Catastrophic,
        (RiskFactor::Suicidal, _) => EncounterLabel::Catastrophic,
        _ => EncounterLabel::Standard, // Default fallback
    }
}

/// Generate analysis summary based on ratings
fn generate_analysis_summary(risk: &RiskFactor, _difficulty: &Difficulty, quintiles: &[QuintileStats]) -> String {
    let disaster = &quintiles[0];
    let typical = &quintiles[2];
    
    match risk {
        RiskFactor::Safe => format!(
            "Stable encounter. Party survives even in worst cases ({} survivors in disaster timeline).",
            disaster.median_survivors
        ),
        RiskFactor::Volatile => format!(
            "High variance encounter. Worst case: {} survivors, Typical case: {} survivors.",
            disaster.median_survivors, typical.median_survivors
        ),
        RiskFactor::Lethal => format!(
            "Dangerous encounter. Worst case guarantees deaths ({} survivors), typical case: {} survivors.",
            disaster.median_survivors, typical.median_survivors
        ),
        RiskFactor::TPKRisk => "Total Party Kill possible in worst case scenarios.".to_string(),
        RiskFactor::Suicidal => "Party loses more than 50% of the time even in typical scenarios.".to_string(),
    }
}

/// Generate tuning suggestions based on encounter analysis
fn generate_tuning_suggestions(risk: &RiskFactor, difficulty: &Difficulty, _quintiles: &[QuintileStats]) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    match risk {
        RiskFactor::Volatile => {
            suggestions.push("Consider increasing monster HP and decreasing damage to reduce swinginess.".to_string());
        }
        RiskFactor::Lethal => {
            suggestions.push("Reduce monster burst damage or increase player survivability.".to_string());
        }
        RiskFactor::TPKRisk => {
            suggestions.push("Significant nerfing required. Consider reducing monster damage or increasing party power.".to_string());
        }
        RiskFactor::Suicidal => {
            suggestions.push("Encounter is too difficult. Consider reducing monster stats or increasing player resources.".to_string());
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

/// Run quintile analysis on simulation results
pub fn run_quintile_analysis(results: &[SimulationResult], scenario_name: &str, party_size: usize) -> AggregateOutput {
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

    let quintile_labels = [
        "Worst 20%",
        "Below Average", 
        "Median",
        "Above Average",
        "Best 20%",
    ];
    let mut quintiles = Vec::new();

    // Calculate dynamic quintile sizes based on actual number of results
    let total_runs = results.len();
    let quintile_size = total_runs / 5;
    let remainder = total_runs % 5;
    
    // Calculate median survivors for each quintile (middle run of each quintile)
    let mut median_survivors = Vec::new();
    for q in 0..5 {
        // Calculate start and end indices for this quintile
        let start = q * quintile_size + std::cmp::min(q, remainder);
        let end = start + quintile_size + if q < remainder { 1 } else { 0 };
        
        // Calculate middle index within this quintile
        let middle_idx = if end > start {
            start + (end - start) / 2
        } else {
            start
        };
        
        // Bounds checking
        if middle_idx >= results.len() {
            median_survivors.push(0);
            continue;
        }
        
        let score = crate::aggregation::calculate_score(&results[middle_idx]);
        let survivors = ((score / 10000.0).floor() as usize).min(party_size);
        median_survivors.push(survivors);
    }

    // Process each quintile using dynamic sizing
    for q in 0..5 {
        // Calculate start and end indices for this quintile
        let start = q * quintile_size + std::cmp::min(q, remainder);
        let end = start + quintile_size + if q < remainder { 1 } else { 0 };
        
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

    // Calculate encounter ratings
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