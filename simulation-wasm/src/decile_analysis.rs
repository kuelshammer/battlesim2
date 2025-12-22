use crate::model::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SafetyGrade {
    A, // Secure
    B, // Fair
    C, // Risky
    D, // Unstable
    F, // Broken
}

impl std::fmt::Display for SafetyGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SafetyGrade::A => write!(f, "A (Secure)"),
            SafetyGrade::B => write!(f, "B (Fair)"),
            SafetyGrade::C => write!(f, "C (Risky)"),
            SafetyGrade::D => write!(f, "D (Unstable)"),
            SafetyGrade::F => write!(f, "F (Broken)"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum IntensityTier {
    Tier1, // Trivial
    Tier2, // Light
    Tier3, // Moderate
    Tier4, // Heavy
    Tier5, // Extreme
}

impl std::fmt::Display for IntensityTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntensityTier::Tier1 => write!(f, "Tier 1 (Trivial)"),
            IntensityTier::Tier2 => write!(f, "Tier 2 (Light)"),
            IntensityTier::Tier3 => write!(f, "Tier 3 (Moderate)"),
            IntensityTier::Tier4 => write!(f, "Tier 4 (Heavy)"),
            IntensityTier::Tier5 => write!(f, "Tier 5 (Extreme)"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EncounterLabel {
    EpicChallenge,    // Grade B + Tier 4
    TacticalGrinder,  // Grade A + Tier 3
    ActionMovie,      // Grade B + Tier 2
    TheTrap,          // Grade C + Tier 2
    TheSlog,          // Grade A + Tier 5
    Standard,
    TrivialMinions,
    TPKRisk,
    Broken,
}

impl std::fmt::Display for EncounterLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncounterLabel::EpicChallenge => write!(f, "The Epic Challenge"),
            EncounterLabel::TacticalGrinder => write!(f, "The Tactical Grinder"),
            EncounterLabel::ActionMovie => write!(f, "The Action Movie"),
            EncounterLabel::TheTrap => write!(f, "The Trap"),
            EncounterLabel::TheSlog => write!(f, "The Slog"),
            EncounterLabel::Standard => write!(f, "Standard Encounter"),
            EncounterLabel::TrivialMinions => write!(f, "Trivial / Minions"),
            EncounterLabel::TPKRisk => write!(f, "TPK Risk"),
            EncounterLabel::Broken => write!(f, "Broken / Impossible"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CombatantVisualization {
    pub name: String,
    pub max_hp: u32,
    pub start_hp: u32,
    pub current_hp: u32,
    pub is_dead: bool,
    pub is_player: bool,
    pub hp_percentage: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DecileStats {
    pub decile: usize,
    pub label: String,
    pub median_survivors: usize,
    pub party_size: usize,
    pub total_hp_lost: f64,
    pub hp_lost_percent: f64,
    pub win_rate: f64,
    pub median_run_visualization: Vec<CombatantVisualization>,
    pub median_run_data: Option<EncounterResult>,
    pub battle_duration_rounds: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AggregateOutput {
    pub scenario_name: String,
    pub total_runs: usize,
    pub deciles: Vec<DecileStats>,
    pub global_median: Option<DecileStats>,
    pub safety_grade: SafetyGrade,
    pub intensity_tier: IntensityTier,
    pub encounter_label: EncounterLabel,
    pub analysis_summary: String,
    pub tuning_suggestions: Vec<String>,
    pub is_good_design: bool,
}

fn extract_combatant_visualization(result: &SimulationResult) -> (Vec<CombatantVisualization>, usize) {
    let mut combatants = Vec::new();
    let mut battle_duration = 0;

    for encounter in &result.encounters {
        battle_duration += encounter.rounds.len();
    }

    if let Some(final_encounter) = result.encounters.last() {
        if let (Some(first_round), Some(last_round)) = (final_encounter.rounds.first(), final_encounter.rounds.last()) {
            let start_hps: std::collections::HashMap<String, u32> = first_round.team1.iter().chain(first_round.team2.iter())
                .map(|c| (c.id.clone(), c.initial_state.current_hp))
                .collect();

            for combatant in &last_round.team1 {
                let hp_percentage = if combatant.creature.hp > 0 {
                    (combatant.final_state.current_hp as f64 / combatant.creature.hp as f64) * 100.0
                } else {
                    0.0
                };

                combatants.push(CombatantVisualization {
                    name: combatant.creature.name.clone(),
                    max_hp: combatant.creature.hp,
                    start_hp: *start_hps.get(&combatant.id).unwrap_or(&combatant.creature.hp),
                    current_hp: combatant.final_state.current_hp,
                    is_dead: combatant.final_state.current_hp == 0,
                    is_player: true,
                    hp_percentage,
                });
            }

            for combatant in &last_round.team2 {
                let hp_percentage = if combatant.creature.hp > 0 {
                    (combatant.final_state.current_hp as f64 / combatant.creature.hp as f64) * 100.0
                } else {
                    0.0
                };

                combatants.push(CombatantVisualization {
                    name: combatant.creature.name.clone(),
                    max_hp: combatant.creature.hp,
                    start_hp: *start_hps.get(&combatant.id).unwrap_or(&combatant.creature.hp),
                    current_hp: combatant.final_state.current_hp,
                    is_dead: combatant.final_state.current_hp == 0,
                    is_player: false,
                    hp_percentage,
                });
            }
        }
    }

    (combatants, battle_duration)
}

fn assess_safety_grade(deciles: &[DecileStats], global_median: &Option<DecileStats>) -> SafetyGrade {
    if deciles.is_empty() { return SafetyGrade::A; }
    
    let typical = global_median.as_ref().or_else(|| deciles.get(deciles.len() / 2)).unwrap_or(&deciles[0]);
    let disaster = &deciles[0];
    let struggle = deciles.get(deciles.len() / 4).unwrap_or(&deciles[0]);

    if typical.median_survivors == 0 { return SafetyGrade::F; }
    if struggle.median_survivors == 0 { return SafetyGrade::D; }
    if disaster.median_survivors == 0 { return SafetyGrade::C; }

    let disaster_lowest_hp = disaster.median_run_visualization
        .iter()
        .filter(|c| c.is_player && !c.is_dead)
        .map(|c| c.hp_percentage)
        .fold(100.0_f64, |acc, hp| acc.min(hp));

    if disaster_lowest_hp > 10.0 {
        SafetyGrade::A
    } else {
        SafetyGrade::B
    }
}

fn assess_intensity_tier(deciles: &[DecileStats], global_median: &Option<DecileStats>) -> IntensityTier {
    if deciles.is_empty() { return IntensityTier::Tier1; }
    
    let typical = global_median.as_ref().or_else(|| deciles.get(deciles.len() / 2)).unwrap_or(&deciles[0]);
    let resources_left = 100.0 - typical.hp_lost_percent;

    if resources_left > 90.0 { IntensityTier::Tier1 }
    else if resources_left >= 70.0 { IntensityTier::Tier2 }
    else if resources_left >= 40.0 { IntensityTier::Tier3 }
    else if resources_left >= 10.0 { IntensityTier::Tier4 }
    else { IntensityTier::Tier5 }
}

fn get_encounter_label(grade: &SafetyGrade, tier: &IntensityTier) -> EncounterLabel {
    match (grade, tier) {
        (SafetyGrade::B, IntensityTier::Tier4) => EncounterLabel::EpicChallenge,
        (SafetyGrade::A, IntensityTier::Tier3) => EncounterLabel::TacticalGrinder,
        (SafetyGrade::B, IntensityTier::Tier2) => EncounterLabel::ActionMovie,
        (SafetyGrade::C, IntensityTier::Tier2) => EncounterLabel::TheTrap,
        (SafetyGrade::A, IntensityTier::Tier5) => EncounterLabel::TheSlog,
        (SafetyGrade::F, _) => EncounterLabel::Broken,
        (SafetyGrade::D, _) => EncounterLabel::TPKRisk,
        (SafetyGrade::A, IntensityTier::Tier1) => EncounterLabel::TrivialMinions,
        (SafetyGrade::A, IntensityTier::Tier2) => EncounterLabel::Standard,
        _ => EncounterLabel::Standard,
    }
}

fn generate_analysis_summary(grade: &SafetyGrade, tier: &IntensityTier, deciles: &[DecileStats], global_median: &Option<DecileStats>) -> String {
    if deciles.is_empty() { return "Insufficient data".to_string(); }
    
    let typical = global_median.as_ref().or_else(|| deciles.get(deciles.len() / 2)).unwrap_or(&deciles[0]);

    let safety_desc = match grade {
        SafetyGrade::A => "Party is secure even with terrible luck.",
        SafetyGrade::B => "Bad luck hurts, but the party typically survives.",
        SafetyGrade::C => "Bottom 10% of scenarios result in a TPK.",
        SafetyGrade::D => "High risk of failure. Bottom 25% are TPKs.",
        SafetyGrade::F => "Mathematically impossible for the party to win consistently.",
    };

    let intensity_desc = match tier {
        IntensityTier::Tier1 => "Negligible resource drain.",
        IntensityTier::Tier2 => "A light warm-up fight.",
        IntensityTier::Tier3 => "A solid, balanced challenge.",
        IntensityTier::Tier4 => "Resource intensive and tense.",
        IntensityTier::Tier5 => "Players will end with empty tanks.",
    };

    format!("Grade {}: {} | {}: {} | Typical Survivors: {}/{}",
        grade, safety_desc, tier, intensity_desc, typical.median_survivors, typical.party_size)
}

fn generate_tuning_suggestions(grade: &SafetyGrade, tier: &IntensityTier, _deciles: &[DecileStats]) -> Vec<String> {
    let mut suggestions = Vec::new();
    match grade {
        SafetyGrade::C => suggestions.push("Risky floor. Consider lowering monster burst damage.".to_string()),
        SafetyGrade::D => suggestions.push("Unstable. Reduce number of monsters or lower their damage stats.".to_string()),
        SafetyGrade::F => suggestions.push("Impossible. Major rebalance needed - monsters are too strong.".to_string()),
        _ => {}
    }
    match tier {
        IntensityTier::Tier1 => suggestions.push("Under-tuned. Increase monster HP or count for more impact.".to_string()),
        IntensityTier::Tier5 => suggestions.push("Resource slog. Ensure players have a rest opportunity after this.".to_string()),
        _ => {}
    }
    suggestions
}

fn calculate_run_stats(run: &SimulationResult, party_size: usize) -> (f64, f64, usize, usize) {
    let score = crate::aggregation::calculate_score(run);
    let survivors = ((score / 1_000_000.0).floor() as usize).min(party_size);
    
    let mut run_party_max_hp = 0.0;
    if let Some(enc) = run.encounters.first() {
        if let Some(round) = enc.rounds.first() {
            for c in &round.team1 { run_party_max_hp += c.creature.hp as f64; }
        }
    }
    let hp_lost = (run_party_max_hp - (score - (survivors as f64 * 1_000_000.0))).max(0.0);
    let duration = run.encounters.iter().map(|e| e.rounds.len()).sum::<usize>();
    
    (hp_lost, run_party_max_hp, survivors, duration)
}

fn analyze_results(results: &[SimulationResult], scenario_name: &str, party_size: usize) -> AggregateOutput {
    if results.is_empty() {
        return AggregateOutput {
            scenario_name: scenario_name.to_string(), total_runs: 0, deciles: Vec::new(), global_median: None,
            safety_grade: SafetyGrade::A, intensity_tier: IntensityTier::Tier1, encounter_label: EncounterLabel::Standard,
            analysis_summary: "No data.".to_string(), tuning_suggestions: Vec::new(), is_good_design: false,
        };
    }

    let total_runs = results.len();
    let is_perfect = total_runs > 0 && (total_runs - 1) % 10 == 0;
    let slice_size = if is_perfect { (total_runs - 1) / 10 } else { (total_runs / 10).max(1) };
    
    let mut deciles = Vec::with_capacity(10);
    let mut global_median = None;

    if is_perfect && total_runs >= 11 {
        let median_idx = total_runs / 2;
        let median_run = &results[median_idx];
        let (hp_lost, max_hp, survivors, duration) = calculate_run_stats(median_run, party_size);
        let (visualization_data, _) = extract_combatant_visualization(median_run);
        
        global_median = Some(DecileStats {
            decile: 0,
            label: "Global Median".to_string(),
            median_survivors: survivors,
            party_size,
            total_hp_lost: hp_lost,
            hp_lost_percent: if max_hp > 0.0 { (hp_lost / max_hp) * 100.0 } else { 0.0 },
            win_rate: if survivors > 0 { 100.0 } else { 0.0 },
            median_run_visualization: visualization_data,
            median_run_data: if !median_run.encounters.is_empty() { Some(median_run.encounters[0].clone()) } else { None },
            battle_duration_rounds: duration,
        });

        for i in 0..10 {
            let start_idx = if i < 5 { i * slice_size } else { i * slice_size + 1 };
            let end_idx = start_idx + slice_size;
            if start_idx < total_runs && end_idx <= total_runs {
                let slice = &results[start_idx..end_idx];
                deciles.push(calculate_decile_stats(slice, i + 1, party_size));
            }
        }
    } else {
        let decile_size = total_runs as f64 / 10.0;
        for i in 0..10 {
            let start_idx = (i as f64 * decile_size).floor() as usize;
            let end_idx = ((i + 1) as f64 * decile_size).floor() as usize;
            let slice = &results[start_idx..end_idx.min(total_runs)];
            if !slice.is_empty() {
                deciles.push(calculate_decile_stats(slice, i + 1, party_size));
            }
        }
        
        let median_idx = total_runs / 2;
        if let Some(median_run) = results.get(median_idx) {
            let (hp_lost, max_hp, survivors, duration) = calculate_run_stats(median_run, party_size);
            let (visualization_data, _) = extract_combatant_visualization(median_run);
            
            global_median = Some(DecileStats {
                decile: 0,
                label: "Global Median".to_string(),
                median_survivors: survivors,
                party_size,
                total_hp_lost: hp_lost,
                hp_lost_percent: if max_hp > 0.0 { (hp_lost / max_hp) * 100.0 } else { 0.0 },
                win_rate: if survivors > 0 { 100.0 } else { 0.0 },
                median_run_visualization: visualization_data,
                median_run_data: if !median_run.encounters.is_empty() { Some(median_run.encounters[0].clone()) } else { None },
                battle_duration_rounds: duration,
            });
        }
    }

    let safety_grade = assess_safety_grade(&deciles, &global_median);
    let intensity_tier = assess_intensity_tier(&deciles, &global_median);
    let encounter_label = get_encounter_label(&safety_grade, &intensity_tier);
    let analysis_summary = generate_analysis_summary(&safety_grade, &intensity_tier, &deciles, &global_median);
    let tuning_suggestions = generate_tuning_suggestions(&safety_grade, &intensity_tier, &deciles);
    
    let is_good_design = matches!(safety_grade, SafetyGrade::A | SafetyGrade::B) && 
                         matches!(intensity_tier, IntensityTier::Tier3 | IntensityTier::Tier4);

    AggregateOutput {
        scenario_name: scenario_name.to_string(), total_runs, deciles, global_median,
        safety_grade, intensity_tier, encounter_label, analysis_summary, tuning_suggestions, is_good_design,
    }
}

fn calculate_decile_stats(slice: &[SimulationResult], decile_num: usize, party_size: usize) -> DecileStats {
    let mut total_wins = 0.0;
    let mut total_hp_lost = 0.0;
    let mut total_survivors = 0;
    let mut total_duration = 0;
    let mut total_party_max_hp = 0.0;

    for run in slice {
        let (hp_lost, max_hp, survivors, duration) = calculate_run_stats(run, party_size);
        if survivors > 0 { total_wins += 1.0; }
        total_survivors += survivors;
        total_hp_lost += hp_lost;
        total_party_max_hp += max_hp;
        total_duration += duration;
    }

    let count = slice.len() as f64;
    let avg_hp_lost = if count > 0.0 { total_hp_lost / count } else { 0.0 };
    let avg_party_max_hp = if count > 0.0 { total_party_max_hp / count } else { 0.0 };

    let median_in_slice_idx = slice.len() / 2;
    let median_run = &slice[median_in_slice_idx];
    let (visualization_data, _) = extract_combatant_visualization(median_run);

    let label = match decile_num {
        1 => "Decile 1 (Worst)",
        10 => "Decile 10 (Best)",
        _ => "Decile",
    };

    DecileStats {
        decile: decile_num,
        label: format!("{} {}", label, decile_num),
        median_survivors: if count > 0.0 { (total_survivors as f64 / count).round() as usize } else { 0 },
        party_size,
        total_hp_lost: avg_hp_lost,
        hp_lost_percent: if avg_party_max_hp > 0.0 { (avg_hp_lost / avg_party_max_hp) * 100.0 } else { 0.0 },
        win_rate: if count > 0.0 { (total_wins / count) * 100.0 } else { 0.0 },
        median_run_visualization: visualization_data,
        median_run_data: if !median_run.encounters.is_empty() { Some(median_run.encounters[0].clone()) } else { None },
        battle_duration_rounds: if count > 0.0 { (total_duration as f64 / count).round() as usize } else { 0 },
    }
}

pub fn run_decile_analysis(results: &[SimulationResult], scenario_name: &str, party_size: usize) -> AggregateOutput {
    analyze_results(results, scenario_name, party_size)
}

pub fn run_day_analysis(results: &[SimulationResult], scenario_name: &str, party_size: usize) -> AggregateOutput {
    analyze_results(results, scenario_name, party_size)
}

pub fn run_encounter_analysis(results: &[SimulationResult], encounter_idx: usize, scenario_name: &str, party_size: usize) -> AggregateOutput {
    let mut encounter_results: Vec<SimulationResult> = results.iter()
        .filter_map(|run| {
            if encounter_idx < run.encounters.len() { 
                Some(SimulationResult { 
                    encounters: vec![run.encounters[encounter_idx].clone()],
                    score: run.score 
                }) 
            } else { 
                None 
            }
        })
        .collect();

    encounter_results.sort_by(|a, b| {
        let score_a = crate::aggregation::calculate_score(a);
        let score_b = crate::aggregation::calculate_score(b);
        score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    analyze_results(&encounter_results, scenario_name, party_size)
}