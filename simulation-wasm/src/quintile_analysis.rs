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
    pub median_run_data: Option<EncounterResult>, // Full round-by-round data
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

    if result.is_empty() {
        return (combatants, battle_duration);
    }
    
    for encounter in result {
        battle_duration += encounter.rounds.len();
    }

    if let Some(final_encounter) = result.last() {
        if let Some(last_round) = final_encounter.rounds.last() {
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

/// Assess risk factor based on quintile analysis - implements Safety Grade system (A-F)
fn assess_risk_factor(quintiles: &[QuintileStats]) -> RiskFactor {
    if quintiles.len() < 3 { return RiskFactor::Safe; }
    let disaster = &quintiles[0];  // Disaster run (#125) - 5th Percentile
    let struggle = &quintiles[1]; // Struggle run (#627) - 25th Percentile
    let typical = &quintiles[2];  // Typical run (#1255) - 50th Percentile

    if typical.median_survivors == 0 {
        return RiskFactor::Suicidal; // Safety Grade F
    }

    if struggle.median_survivors == 0 {
        return RiskFactor::TPKRisk; // Safety Grade D
    }

    if disaster.median_survivors == 0 && struggle.median_survivors > 0 {
        return RiskFactor::Lethal; // Safety Grade C
    }

    let disaster_lowest_hp = disaster.median_run_visualization
        .iter()
        .filter(|c| c.is_player && !c.is_dead)
        .map(|c| c.hp_percentage)
        .fold(100.0_f64, |acc, hp| acc.min(hp));

    if disaster.median_survivors > 0 && disaster_lowest_hp < 5.0 {
        return RiskFactor::Volatile; // Safety Grade B
    }

    RiskFactor::Safe // Safety Grade A
}

/// Assess difficulty based on typical quintile HP remaining - implements Intensity Tier system (1-5)
fn assess_difficulty(quintiles: &[QuintileStats]) -> Difficulty {
    if quintiles.len() < 3 { return Difficulty::Medium; }
    let typical = &quintiles[2];

    let surviving_players: Vec<_> = typical.median_run_visualization
        .iter()
        .filter(|c| c.is_player && !c.is_dead)
        .collect();

    if surviving_players.is_empty() {
        return Difficulty::Grueling;
    }

    let avg_hp_percent = surviving_players
        .iter()
        .map(|c| c.hp_percentage)
        .sum::<f64>()
        / surviving_players.len() as f64;

    if avg_hp_percent > 90.0 {
        Difficulty::Easy
    } else if avg_hp_percent >= 70.0 {
        Difficulty::Medium
    } else if avg_hp_percent >= 40.0 {
        Difficulty::Hard
    } else {
        Difficulty::Grueling
    }
}

/// Get combined encounter label based on risk and difficulty
fn get_encounter_label(risk: &RiskFactor, difficulty: &Difficulty) -> EncounterLabel {
    match (risk, difficulty) {
        (RiskFactor::Volatile, Difficulty::Grueling) => EncounterLabel::Deadly,
        (RiskFactor::Safe, Difficulty::Hard) => EncounterLabel::Grinder,
        (RiskFactor::Volatile, Difficulty::Medium) => EncounterLabel::Spicy,
        (RiskFactor::Lethal, Difficulty::Medium) => EncounterLabel::Catastrophic,
        (RiskFactor::Safe, Difficulty::Grueling) => EncounterLabel::Grinder,
        (RiskFactor::Safe, Difficulty::Easy) => EncounterLabel::Trivial,
        (RiskFactor::Safe, Difficulty::Medium) => EncounterLabel::Standard,
        (RiskFactor::Volatile, Difficulty::Easy) => EncounterLabel::GlassCannon,
        (RiskFactor::Lethal, Difficulty::Hard) => EncounterLabel::Deadly,
        (RiskFactor::TPKRisk, _) => EncounterLabel::Catastrophic,
        (RiskFactor::Suicidal, _) => EncounterLabel::Catastrophic,
        _ => EncounterLabel::Standard,
    }
}

/// Generate analysis summary based on ratings
fn generate_analysis_summary(_risk: &RiskFactor, _difficulty: &Difficulty, quintiles: &[QuintileStats]) -> String {
    if quintiles.len() < 3 { return "Insufficient data".to_string(); }
    let disaster = &quintiles[0];
    let struggle = &quintiles[1];
    let typical = &quintiles[2];

    let safety_grade = if disaster.median_survivors == 0 && struggle.median_survivors == 0 { "F (Broken)" }
    else if struggle.median_survivors == 0 { "D (Unstable)" }
    else if disaster.median_survivors == 0 { "C (Risky)" }
    else if disaster.median_run_visualization.iter().filter(|c| c.is_player && !c.is_dead).map(|c| c.hp_percentage).fold(100.0_f64, |acc, hp| acc.min(hp)) < 5.0 { "B (Fair)" }
    else { "A (Secure)" };

    let intensity_tier = {
        let avg = typical.median_run_visualization.iter().filter(|c| c.is_player && !c.is_dead).map(|c| c.hp_percentage).sum::<f64>() / typical.median_run_visualization.len().max(1) as f64;
        if avg > 90.0 { "1 (Trivial)" }
        else if avg >= 70.0 { "2 (Light)" }
        else if avg >= 40.0 { "3 (Moderate)" }
        else if avg >= 10.0 { "4 (Heavy)" }
        else { "5 (Extreme)" }
    };

    format!("Safety: {} | Intensity: {} | Disaster: {}/{} survivors | Typical: {}/{} survivors",
        safety_grade, intensity_tier, disaster.median_survivors, disaster.party_size, typical.median_survivors, typical.party_size)
}

/// Generate tuning suggestions based on encounter analysis
fn generate_tuning_suggestions(risk: &RiskFactor, difficulty: &Difficulty, _quintiles: &[QuintileStats]) -> Vec<String> {
    let mut suggestions = Vec::new();
    match risk {
        RiskFactor::Volatile => suggestions.push("High variance. Reduce monster burst damage.".to_string()),
        RiskFactor::Lethal => suggestions.push("Dangerous. Reduce monster damage or increase party survivability.".to_string()),
        RiskFactor::TPKRisk => suggestions.push("TPK Risk. Significant nerfing required.".to_string()),
        RiskFactor::Suicidal => suggestions.push("Impossible. Major rebalance needed.".to_string()),
        _ => {}
    }
    match difficulty {
        Difficulty::Easy => suggestions.push("May be too easy for the party.".to_string()),
        Difficulty::Grueling => suggestions.push("Resource-intensive. Consider a short rest opportunity.".to_string()),
        _ => {}
    }
    suggestions
}

/// Run analysis on a set of results (helper function)
fn analyze_results(results: &[SimulationResult], scenario_name: &str, party_size: usize) -> AggregateOutput {
    if results.is_empty() {
        return AggregateOutput {
            scenario_name: scenario_name.to_string(), total_runs: 0, quintiles: Vec::new(),
            risk_factor: RiskFactor::Safe, difficulty: Difficulty::Medium, encounter_label: EncounterLabel::Standard,
            analysis_summary: "No data.".to_string(), tuning_suggestions: Vec::new(),
        };
    }

    let total_runs = results.len();
    if total_runs == 0 {
        return AggregateOutput {
            scenario_name: scenario_name.to_string(), total_runs: 0, quintiles: Vec::new(),
            risk_factor: RiskFactor::Safe, difficulty: Difficulty::Medium, encounter_label: EncounterLabel::Standard,
            analysis_summary: "No data.".to_string(), tuning_suggestions: Vec::new(),
        };
    }

    // Calculate dynamic indices based on current total_runs
    // We want the medians of deciles 1, 3, 5 (global median), 8, and 10.
    let decile_size = total_runs as f64 / 10.0;
    let disaster_run_idx = (decile_size * 0.5) as usize;
    let struggle_run_idx = (decile_size * 2.5) as usize;
    let typical_run_idx = (total_runs / 2) as usize;
    let heroic_run_idx = (decile_size * 7.5) as usize;
    let legend_run_idx = (decile_size * 9.5) as usize;

    let battle_card_indices = [
        (1, "Disaster", disaster_run_idx), (2, "Struggle", struggle_run_idx),
        (3, "Typical", typical_run_idx), (4, "Heroic", heroic_run_idx), (5, "Legend", legend_run_idx),
    ];

    let mut quintiles = Vec::new();
    for (num, label, run_idx) in battle_card_indices {
        if run_idx >= results.len() {
            quintiles.push(QuintileStats {
                quintile: num, label: label.to_string(), median_survivors: 0, party_size, total_hp_lost: 0.0,
                hp_lost_percent: 0.0, win_rate: 0.0, median_run_visualization: Vec::new(), median_run_data: None, battle_duration_rounds: 0,
            });
            continue;
        }

        let single_run = &results[run_idx];
        let score = crate::aggregation::calculate_score(single_run);
        let survivors = ((score / 10000.0).floor() as usize).min(party_size);
        let (visualization_data, battle_duration) = extract_combatant_visualization(single_run);

        let mut party_max_hp = 0.0;
        if let Some(enc) = single_run.first() {
            if let Some(round) = enc.rounds.first() {
                for c in &round.team1 { party_max_hp += c.creature.hp; }
            }
        }

        let hp_lost = (party_max_hp - (score - (survivors as f64 * 10000.0))).max(0.0);
        let hp_lost_percent = if party_max_hp > 0.0 { (hp_lost / party_max_hp) * 100.0 } else { 0.0 };

        // For encounter analysis, we want to capture the specific encounter's round data
        let median_run_data = if !single_run.is_empty() && results[0].len() == 1 {
            Some(single_run[0].clone())
        } else {
            None
        };

        quintiles.push(QuintileStats {
            quintile: num, label: label.to_string(), median_survivors: survivors, party_size, total_hp_lost: hp_lost,
            hp_lost_percent, win_rate: if survivors > 0 { 100.0 } else { 0.0 },
            median_run_visualization: visualization_data, median_run_data, battle_duration_rounds: battle_duration,
        });
    }

    let risk_factor = assess_risk_factor(&quintiles);
    let difficulty = assess_difficulty(&quintiles);
    let encounter_label = get_encounter_label(&risk_factor, &difficulty);
    let analysis_summary = generate_analysis_summary(&risk_factor, &difficulty, &quintiles);
    let tuning_suggestions = generate_tuning_suggestions(&risk_factor, &difficulty, &quintiles);

    AggregateOutput {
        scenario_name: scenario_name.to_string(), total_runs: results.len(), quintiles,
        risk_factor, difficulty, encounter_label, analysis_summary, tuning_suggestions,
    }
}

pub fn run_quintile_analysis(results: &[SimulationResult], scenario_name: &str, party_size: usize) -> AggregateOutput {
    analyze_results(results, scenario_name, party_size)
}

pub fn run_day_analysis(results: &[SimulationResult], scenario_name: &str, party_size: usize) -> AggregateOutput {
    analyze_results(results, scenario_name, party_size)
}

pub fn run_encounter_analysis(results: &[SimulationResult], encounter_idx: usize, scenario_name: &str, party_size: usize) -> AggregateOutput {
    let mut encounter_results: Vec<SimulationResult> = results.iter()
        .filter_map(|run| {
            if encounter_idx < run.len() { Some(vec![run[encounter_idx].clone()]) } else { None }
        })
        .collect();

    encounter_results.sort_by(|a, b| {
        let score_a = crate::aggregation::calculate_score(a);
        let score_b = crate::aggregation::calculate_score(b);
        score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    analyze_results(&encounter_results, scenario_name, party_size)
}