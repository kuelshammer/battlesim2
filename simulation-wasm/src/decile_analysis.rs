use crate::model::*;
use crate::intensity_calculation::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EncounterArchetype {
    Trivial,
    Skirmish,
    Standard,
    TheGrind,
    EliteChallenge,
    BossFight,
    MeatGrinder,
    NovaTrap,
    Broken,
}

impl std::fmt::Display for EncounterArchetype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncounterArchetype::Trivial => write!(f, "Trivial"),
            EncounterArchetype::Skirmish => write!(f, "Skirmish"),
            EncounterArchetype::Standard => write!(f, "Standard"),
            EncounterArchetype::TheGrind => write!(f, "The Grind"),
            EncounterArchetype::EliteChallenge => write!(f, "Elite Challenge"),
            EncounterArchetype::BossFight => write!(f, "Boss Fight"),
            EncounterArchetype::MeatGrinder => write!(f, "Meat Grinder"),
            EncounterArchetype::NovaTrap => write!(f, "Nova Trap"),
            EncounterArchetype::Broken => write!(f, "Broken"),
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
    EpicChallenge,    
    TacticalGrinder,  
    ActionMovie,      
    TheTrap,          
    TheSlog,          
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
    pub resource_timeline: Vec<f64>, // Array of EHP % after each step
    pub vitality_timeline: Vec<f64>, // Array of Vitality % after each step
    pub power_timeline: Vec<f64>, // Array of Power % after each step
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimelineRange {
    pub p25: Vec<f64>,
    pub p75: Vec<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Vitals {
    pub lethality_index: f64, // Probability of 1+ death/KO (0.0 - 1.0)
    pub tpk_risk: f64,        // Probability of TPK (0.0 - 1.0)
    pub attrition_score: f64, // % of daily budget burned (0.0 - 1.0)
    pub volatility_index: f64, // Difference between P10 and P50 cost
    pub doom_horizon: f64,    // Projected encounters until failure
    pub deaths_door_index: f64, // Average rounds spent at <25% HP (Thrilling metric)
    pub archetype: EncounterArchetype,
    pub is_volatile: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DayPacing {
    pub archetype: String,
    pub director_score: f64,
    pub rhythm_score: f64,
    pub attrition_score: f64,
    pub recovery_score: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AggregateOutput {
    pub scenario_name: String,
    pub total_runs: usize,
    pub deciles: Vec<DecileStats>,
    pub global_median: Option<DecileStats>,
    pub vitality_range: Option<TimelineRange>,
    pub power_range: Option<TimelineRange>,
    pub decile_logs: Vec<Vec<crate::events::Event>>, // 11 logs: [P5, P15, ..., P50, ..., P95]
    pub battle_duration_rounds: usize,
    pub intensity_tier: IntensityTier,
    pub encounter_label: EncounterLabel,
    pub analysis_summary: String,
    pub tuning_suggestions: Vec<String>,
    pub is_good_design: bool,
    pub stars: usize,
    pub tdnw: f64, // Total Daily Net Worth
    pub num_encounters: usize,
    pub skyline: Option<crate::percentile_analysis::SkylineAnalysis>,
    pub vitals: Option<Vitals>,
    pub pacing: Option<DayPacing>,
}

fn extract_combatant_visualization_partial(result: &SimulationResult, encounter_idx: Option<usize>) -> (Vec<CombatantVisualization>, usize) {
    let mut combatants = Vec::new();
    let mut battle_duration = 0;

    let slice = if let Some(idx) = encounter_idx {
        if idx < result.encounters.len() {
            &result.encounters[idx..=idx]
        } else {
            &[]
        }
    } else {
        &result.encounters[..]
    };

    for encounter in slice {
        battle_duration += encounter.rounds.len();
    }

    if let Some(final_encounter) = slice.last() {
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

fn assess_archetype(vitals: &Vitals) -> EncounterArchetype {
    if vitals.tpk_risk > 0.5 { return EncounterArchetype::Broken; }
    if vitals.tpk_risk > 0.1 { return EncounterArchetype::MeatGrinder; }
    
    if vitals.lethality_index > 0.5 { return EncounterArchetype::MeatGrinder; }
    
    if vitals.lethality_index > 0.3 {
        if vitals.attrition_score < 0.2 { return EncounterArchetype::NovaTrap; }
        return EncounterArchetype::BossFight;
    }

    if vitals.lethality_index > 0.15 {
        if vitals.attrition_score > 0.4 { return EncounterArchetype::TheGrind; }
        return EncounterArchetype::EliteChallenge;
    }

    if vitals.lethality_index > 0.05 {
        if vitals.attrition_score > 0.3 { return EncounterArchetype::TheGrind; }
        return EncounterArchetype::Standard;
    }

    if vitals.attrition_score > 0.1 {
        return EncounterArchetype::Skirmish;
    }

    EncounterArchetype::Trivial
}

fn get_encounter_label(archetype: &EncounterArchetype) -> EncounterLabel {
    match archetype {
        EncounterArchetype::Broken => EncounterLabel::Broken,
        EncounterArchetype::MeatGrinder => EncounterLabel::TPKRisk,
        EncounterArchetype::BossFight => EncounterLabel::EpicChallenge,
        EncounterArchetype::EliteChallenge => EncounterLabel::TacticalGrinder,
        EncounterArchetype::TheGrind => EncounterLabel::TheSlog,
        EncounterArchetype::NovaTrap => EncounterLabel::TheTrap,
        EncounterArchetype::Skirmish => EncounterLabel::ActionMovie,
        EncounterArchetype::Trivial => EncounterLabel::TrivialMinions,
        EncounterArchetype::Standard => EncounterLabel::Standard,
    }
}

fn generate_analysis_summary(archetype: &EncounterArchetype, vitals: &Vitals, typical: &DecileStats) -> String {
    let archetype_desc = match archetype {
        EncounterArchetype::Trivial => "Negligible challenge.",
        EncounterArchetype::Skirmish => "A light warm-up.",
        EncounterArchetype::Standard => "Balanced and fair.",
        EncounterArchetype::TheGrind => "High resource drain, low risk.",
        EncounterArchetype::EliteChallenge => "Tactical and demanding.",
        EncounterArchetype::BossFight => "Significant risk of casualty.",
        EncounterArchetype::MeatGrinder => "High TPK potential.",
        EncounterArchetype::NovaTrap => "Burst damage threat.",
        EncounterArchetype::Broken => "Mathematically impossible.",
    };

    format!("{}: {} | Attrition: {}% | Typical Survivors: {}/{}",
        archetype, archetype_desc, (vitals.attrition_score * 100.0).round(), typical.median_survivors, typical.party_size)
}

fn calculate_vitals(
    results: &[&SimulationResult],
    encounter_idx: Option<usize>,
    party_size: usize,
    tdnw: f64,
) -> Vitals {
    let total_runs = results.len();
    if total_runs == 0 {
        return Vitals {
            lethality_index: 0.0,
            tpk_risk: 0.0,
            attrition_score: 0.0,
            volatility_index: 0.0,
            doom_horizon: 0.0,
            deaths_door_index: 0.0,
            archetype: EncounterArchetype::Trivial,
            is_volatile: false,
        };
    }

    // 1. Calculate Lethality and TPK Risk (Probabilities)
    let mut ko_count = 0;
    let mut tpk_count = 0;
    let mut crisis_count = 0;
    let mut total_deaths_door_rounds = 0;

    for &run in results {
        let encounters = if let Some(idx) = encounter_idx {
            if idx < run.encounters.len() { &run.encounters[idx..=idx] } else { &[] }
        } else {
            &run.encounters[..]
        };

        let mut run_has_ko = false;
        let mut run_is_tpk = false;
        let mut run_has_crisis = false;

        for enc in encounters {
            // Check Death's Door (rounds with survivor < 25% HP)
            for round in &enc.rounds {
                let any_at_deaths_door = round.team1.iter().any(|c| {
                    let hp_pct = if c.creature.hp > 0 { c.final_state.current_hp as f64 / c.creature.hp as f64 } else { 0.0 };
                    c.final_state.current_hp > 0 && hp_pct < 0.25
                });
                if any_at_deaths_door {
                    total_deaths_door_rounds += 1;
                }
            }

            if let Some(last_round) = enc.rounds.last() {
                let survivors = last_round.team1.iter().filter(|c| c.final_state.current_hp > 0).count();
                if survivors < party_size { run_has_ko = true; }
                if survivors == 0 { run_is_tpk = true; }
                
                if last_round.team1.iter().any(|c| (c.final_state.current_hp as f64 / c.creature.hp as f64) < 0.1) {
                    run_has_crisis = true;
                }
            }
        }

        if run_has_ko { ko_count += 1; }
        if run_is_tpk { tpk_count += 1; }
        if run_has_crisis { crisis_count += 1; }
    }

    let lethality_index = ko_count as f64 / total_runs as f64;
    let tpk_risk = tpk_count as f64 / total_runs as f64;
    let _crisis_risk = crisis_count as f64 / total_runs as f64;
    let deaths_door_index = total_deaths_door_rounds as f64 / total_runs as f64;

    // 2. Attrition and Volatility
    // Sort results by score to find P10 and P50
    // (results are already sorted worst-to-best by the caller analyze_results_internal)
    
    let p10_idx = (total_runs as f64 * 0.1) as usize;
    let p50_idx = total_runs / 2;

    let get_cost = |idx: usize| -> f64 {
        if idx >= total_runs || tdnw <= 0.0 { return 0.0; }
        let (burned, _, _, _, _, _, _) = calculate_run_stats_partial(results[idx], encounter_idx, party_size, tdnw, 0);
        burned / tdnw
    };

    let p10_cost = get_cost(p10_idx);
    let p50_cost = get_cost(p50_idx);
    
    let attrition_score = p50_cost;
    let volatility_index = (p10_cost - p50_cost).max(0.0);
    let is_volatile = volatility_index > 0.20;

    // 3. Archetype Determination
    let mut temp_vitals = Vitals {
        lethality_index,
        tpk_risk,
        attrition_score,
        volatility_index,
        doom_horizon: 0.0,
        deaths_door_index,
        archetype: EncounterArchetype::Standard,
        is_volatile,
    };
    temp_vitals.archetype = assess_archetype(&temp_vitals);

    // 4. Doom Horizon
    temp_vitals.doom_horizon = if attrition_score > 0.01 {
        1.0 / attrition_score
    } else {
        10.0 // Practically infinite
    };

    temp_vitals
}

fn calculate_run_stats_partial(run: &SimulationResult, encounter_idx: Option<usize>, party_size: usize, tdnw: f64, sr_count: usize) -> (f64, f64, usize, usize, Vec<f64>, Vec<f64>, Vec<f64>) {
    let score = crate::aggregation::calculate_score(run);
    
    // 1. Count survivors
    // ... (rest of survivor logic remains same)
    let survivors = if let Some(idx) = encounter_idx {
        if let Some(enc) = run.encounters.get(idx) {
            if let Some(last_round) = enc.rounds.last() {
                last_round.team1.iter().filter(|c| c.final_state.current_hp > 0).count()
            } else { 0 }
        } else { 0 }
    } else if let Some(last_enc) = run.encounters.last() {
        if let Some(last_round) = last_enc.rounds.last() {
            last_round.team1.iter().filter(|c| c.final_state.current_hp > 0).count()
        } else if score < 0.0 {
            0
        } else {
            ((score / 1_000_000.0).floor() as usize).min(party_size)
        }
    } else if score < 0.0 {
        0
    } else {
        ((score / 1_000_000.0).floor() as usize).min(party_size)
    };
    
    // 2. Calculate Timelines
    let mut timeline = Vec::new();
    let mut vitality_timeline = Vec::new();
    let mut power_timeline = Vec::new();
    let mut run_party_max_hp = 0.0;

    // Determine slice of encounters to analyze
    let encounters_slice = if let Some(idx) = encounter_idx {
        if idx < run.encounters.len() {
            &run.encounters[idx..=idx]
        } else {
            &[]
        }
    } else {
        &run.encounters[..]
    };

    // Pre-calculate daily budgets for all players in this run
    let mut player_budgets = HashMap::new();

    // Start with Initial State
    if let Some(first_enc) = encounters_slice.first() {
        if let Some(first_round) = first_enc.rounds.first() {
            let mut start_ehp = 0.0;
            let mut start_vit_sum = 0.0;
            let mut start_pow_sum = 0.0;
            let mut p_count = 0.0;

            for c in &first_round.team1 {
                p_count += 1.0;
                run_party_max_hp += c.creature.hp as f64;
                let ledger = c.creature.initialize_ledger();
                
                let budget = calculate_daily_budget(&c.creature, sr_count);
                player_budgets.insert(c.id.clone(), budget);

                start_ehp += calculate_serializable_ehp(
                    c.initial_state.current_hp, 
                    c.initial_state.temp_hp.unwrap_or(0),
                    &c.initial_state.resources, 
                    &ledger.reset_rules
                );

                start_vit_sum += calculate_vitality(
                    c.initial_state.current_hp, 
                    &c.initial_state.resources.current, 
                    c.creature.hp, 
                    &c.initial_state.resources.max, 
                    c.creature.con_modifier.unwrap_or(0.0)
                );

                start_pow_sum += calculate_strategic_power(
                    c.initial_state.cumulative_spent,
                    budget
                );
            }
            timeline.push(if tdnw > 0.0 { (start_ehp / tdnw) * 100.0 } else { 100.0 });
            vitality_timeline.push(if p_count > 0.0 { start_vit_sum / p_count } else { 100.0 });
            power_timeline.push(if p_count > 0.0 { start_pow_sum / p_count } else { 100.0 });
        }
    }

    // Per step
    for encounter in encounters_slice {
        if let Some(last_round) = encounter.rounds.last() {
            let mut step_ehp = 0.0;
            let mut step_vit_sum = 0.0;
            let mut step_pow_sum = 0.0;
            let mut p_count = 0.0;

            for c in &last_round.team1 {
                p_count += 1.0;
                let ledger = c.creature.initialize_ledger();
                
                let budget = *player_budgets.get(&c.id).unwrap_or(&0.0);

                step_ehp += calculate_serializable_ehp(
                    c.final_state.current_hp,
                    c.final_state.temp_hp.unwrap_or(0),
                    &c.final_state.resources, 
                    &ledger.reset_rules
                );

                step_vit_sum += calculate_vitality(
                    c.final_state.current_hp, 
                    &c.final_state.resources.current, 
                    c.creature.hp, 
                    &c.initial_state.resources.max, 
                    c.creature.con_modifier.unwrap_or(0.0)
                );

                step_pow_sum += calculate_strategic_power(
                    c.final_state.cumulative_spent,
                    budget
                );
            }
            timeline.push(if tdnw > 0.0 { (step_ehp / tdnw) * 100.0 } else { 0.0 });
            vitality_timeline.push(if p_count > 0.0 { step_vit_sum / p_count } else { 0.0 });
            power_timeline.push(if p_count > 0.0 { step_pow_sum / p_count } else { 0.0 });
        } else {
            let prev = timeline.last().cloned().unwrap_or(100.0);
            timeline.push(prev);
            let prev_vit = vitality_timeline.last().cloned().unwrap_or(100.0);
            vitality_timeline.push(prev_vit);
            let prev_pow = power_timeline.last().cloned().unwrap_or(100.0);
            power_timeline.push(prev_pow);
        }
    }

    let start_val = timeline.first().cloned().unwrap_or(100.0) * tdnw / 100.0;
    let end_val = timeline.last().cloned().unwrap_or(0.0) * tdnw / 100.0;
    let burned_resources = (start_val - end_val).max(-1000.0);
    
    let duration = encounters_slice.iter().map(|e| e.rounds.len()).sum::<usize>();
    
    (burned_resources, run_party_max_hp, survivors, duration, timeline, vitality_timeline, power_timeline)
}

pub fn calculate_tdnw(run: &SimulationResult, sr_count: usize) -> f64 {
    let mut total = 0.0;
    // Find the first encounter that has at least one round
    for encounter in &run.encounters {
        if let Some(first_round) = encounter.rounds.first() {
            for c in &first_round.team1 {
                total += crate::intensity_calculation::calculate_daily_budget(&c.creature, sr_count);
            }
            if total > 0.0 {
                return total;
            }
        }
    }
    total
}

pub fn calculate_tdnw_lightweight(players: &[Creature], sr_count: usize) -> f64 {
    let mut total = 0.0;
    for p in players {
        total += crate::intensity_calculation::calculate_daily_budget(p, sr_count) * (p.count as f64);
    }
    total
}

pub fn run_decile_analysis(results: &[SimulationResult], scenario_name: &str, party_size: usize, sr_count: usize) -> AggregateOutput {
    let refs: Vec<&SimulationResult> = results.iter().collect();
    analyze_results_internal(&refs, None, scenario_name, party_size, None, sr_count)
}

pub fn run_decile_analysis_with_logs(runs: &mut [crate::model::SimulationRun], scenario_name: &str, party_size: usize, sr_count: usize) -> AggregateOutput {
    let results: Vec<&SimulationResult> = runs.iter().map(|r| &r.result).collect();
    analyze_results_internal(&results, None, scenario_name, party_size, Some(runs), sr_count)
}

pub fn run_day_analysis(results: &[SimulationResult], scenario_name: &str, party_size: usize, sr_count: usize) -> AggregateOutput {
    let refs: Vec<&SimulationResult> = results.iter().collect();
    analyze_results_internal(&refs, None, scenario_name, party_size, None, sr_count)
}

pub fn run_encounter_analysis(results: &[SimulationResult], encounter_idx: usize, scenario_name: &str, party_size: usize, sr_count: usize) -> AggregateOutput {
    let refs: Vec<&SimulationResult> = results.iter().collect();
    analyze_results_internal(&refs, Some(encounter_idx), scenario_name, party_size, None, sr_count)
}

pub fn run_encounter_analysis_with_logs(runs: &mut [crate::model::SimulationRun], encounter_idx: usize, scenario_name: &str, party_size: usize, sr_count: usize) -> AggregateOutput {
    // 1. Sort the runs based on cumulative score up to encounter_idx
    // Use seed as tie-breaker for deterministic results
    runs.sort_by(|a, b| {
        let score_a = crate::aggregation::calculate_cumulative_score(&a.result, encounter_idx);
        let score_b = crate::aggregation::calculate_cumulative_score(&b.result, encounter_idx);
        score_a.partial_cmp(&score_b)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.result.seed.cmp(&b.result.seed))
    });

    // 2. Perform analysis using refs
    let refs: Vec<&SimulationResult> = runs.iter().map(|r| &r.result).collect();
    let mut output = analyze_results_internal(&refs, Some(encounter_idx), scenario_name, party_size, Some(runs), sr_count);
    
    // 3. Slice the logs to only include events for this specific encounter
    for log in &mut output.decile_logs {
        *log = slice_events_for_encounter(log, encounter_idx);
    }
    
    output
}

fn slice_events_for_encounter(events: &[crate::events::Event], encounter_idx: usize) -> Vec<crate::events::Event> {
    let mut sliced = Vec::new();
    let mut current_encounter = 0;
    let mut recording = false;

    for event in events {
        if let crate::events::Event::EncounterStarted { .. } = event {
            if recording {
                // We reached a new encounter without seeing EncounterEnded for the previous one
                break;
            }
            if current_encounter == encounter_idx {
                recording = true;
            }
            current_encounter += 1;
        }
        
        if recording {
            sliced.push(event.clone());
        }

        if let crate::events::Event::EncounterEnded { .. } = event {
            if recording {
                break;
            }
        }
    }
    sliced
}

fn calculate_day_pacing(
    results: &[&SimulationResult],
    encounter_idx: Option<usize>,
    tdnw: f64,
    sr_count: usize,
) -> Option<DayPacing> {
    if encounter_idx.is_some() || results.is_empty() {
        return None; // Only for overall day analysis
    }

    let total_runs = results.len();
    let median_idx = total_runs / 2;
    let median_run = results[median_idx];

    // 1. Attrition Score (Efficiency)
    // Ideal end state is 10-30% resources.
    let (burned, _, _, _, _, _, _) = calculate_run_stats_partial(median_run, None, 0, tdnw, sr_count);
    let end_res_pct = if tdnw > 0.0 {
        ((tdnw - burned) / tdnw) * 100.0
    } else {
        100.0
    };

    let attrition_score = if end_res_pct < 0.0 {
        20.0 // TPK/Total Exhaustion
    } else if end_res_pct < 10.0 {
        70.0 // Tense, maybe too much
    } else if end_res_pct < 35.0 {
        100.0 // Sweet spot
    } else if end_res_pct < 60.0 {
        60.0 // A bit easy
    } else {
        30.0 // Boring
    };

    // 2. Rhythm Score (Difficulty Escalation)
    let mut rhythm_score = 100.0;
    let mut max_weight = 0.0;
    let mut reversals = 0;
    
    for enc in &median_run.encounters {
        let w = enc.target_role.weight();
        if w < max_weight {
            reversals += 1;
        }
        max_weight = max_weight.max(w);
    }
    
    if median_run.encounters.len() > 1 {
        rhythm_score = (100.0 - (reversals as f64 * 30.0)).max(0.0);
    }

    // 3. Recovery Score (Placeholder for now)
    let recovery_score = 100.0;

    // 4. Archetype Determination
    let archetype = if rhythm_score >= 80.0 && attrition_score >= 80.0 {
        "The Hero's Journey".to_string()
    } else if end_res_pct > 60.0 {
        "The Slow Burn".to_string()
    } else if reversals > 1 {
        "The Nova Trap".to_string()
    } else if end_res_pct < 10.0 {
        "The Meat Grinder".to_string()
    } else {
        "The Gritty Adventure".to_string()
    };

    let director_score = rhythm_score * 0.4 + attrition_score * 0.4 + recovery_score * 0.2;

    Some(DayPacing {
        archetype,
        director_score,
        rhythm_score,
        attrition_score,
        recovery_score,
    })
}

fn analyze_results_internal(results: &[&SimulationResult], encounter_idx: Option<usize>, scenario_name: &str, party_size: usize, runs: Option<&[crate::model::SimulationRun]>, sr_count: usize) -> AggregateOutput {
    if results.is_empty() {
                return AggregateOutput {
                    scenario_name: scenario_name.to_string(),
                    total_runs: 0,
                    deciles: Vec::new(),
                    global_median: None,
                    vitality_range: None,
                    power_range: None,
                    decile_logs: Vec::new(),
                    battle_duration_rounds: 0,
                    intensity_tier: IntensityTier::Tier1, encounter_label: EncounterLabel::Standard,
            analysis_summary: "No data.".to_string(), tuning_suggestions: Vec::new(), is_good_design: false, stars: 0,
            tdnw: 0.0,
            num_encounters: 0,
            skyline: None,
            vitals: None,
            pacing: None,
        };
    }

    let tdnw = calculate_tdnw(results[0], sr_count);
    let num_encounters = results[0].encounters.len();
    
    // Compute vitals
    let vitals = Some(calculate_vitals(results, encounter_idx, party_size, tdnw));
    let pacing = calculate_day_pacing(results, encounter_idx, tdnw, sr_count);

    // Compute skyline analysis (100 buckets)
    // results are already sorted by overall score
    let owned_results: Vec<SimulationResult> = results.iter().map(|&r| r.clone()).collect();
    let skyline = Some(crate::percentile_analysis::run_skyline_analysis(&owned_results, party_size, encounter_idx));

    // Collect all timelines for independent percentile calculation
    let mut all_vits = Vec::with_capacity(results.len());
    let mut all_pows = Vec::with_capacity(results.len());
    for &run in results {
        let (_, _, _, _, _, vit, pow) = calculate_run_stats_partial(run, encounter_idx, party_size, tdnw, sr_count);
        all_vits.push(vit);
        all_pows.push(pow);
    }

    let num_steps = if !all_vits.is_empty() { all_vits[0].len() } else { 0 };
    let mut vit_p25 = Vec::with_capacity(num_steps);
    let mut vit_p75 = Vec::with_capacity(num_steps);
    let mut pow_p25 = Vec::with_capacity(num_steps);
    let mut pow_p75 = Vec::with_capacity(num_steps);

    for j in 0..num_steps {
        let mut step_vits: Vec<f64> = all_vits.iter().map(|t| t[j]).collect();
        let mut step_pows: Vec<f64> = all_pows.iter().map(|t| t[j]).collect();
        
        step_vits.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        step_pows.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let p25_idx = (step_vits.len() as f64 * 0.25) as usize;
        let p75_idx = (step_vits.len() as f64 * 0.75) as usize;

        if !step_vits.is_empty() {
            vit_p25.push(step_vits[p25_idx.min(step_vits.len() - 1)]);
            vit_p75.push(step_vits[p75_idx.min(step_vits.len() - 1)]);
            pow_p25.push(step_pows[p25_idx.min(step_pows.len() - 1)]);
            pow_p75.push(step_pows[p75_idx.min(step_pows.len() - 1)]);
        }
    }

    let vitality_range = Some(TimelineRange { p25: vit_p25, p75: vit_p75 });
    let power_range = Some(TimelineRange { p25: pow_p25, p75: pow_p75 });

    // Weighted Resource Pie Logic
    let total_day_weight: f64 = results[0].encounters.iter().map(|e| e.target_role.weight()).sum();
    let current_encounter_weight = if let Some(idx) = encounter_idx {
        results[0].encounters.get(idx).map(|e| e.target_role.weight()).unwrap_or(2.0)
    } else {
        if results[0].encounters.len() == 1 {
            results[0].encounters[0].target_role.weight()
        } else {
            results[0].encounters.get(0).map(|e| e.target_role.weight()).unwrap_or(2.0)
        }
    };

    let total_runs = results.len();
    let mut global_median = None;
    let mut decile_logs = Vec::new();
    let mut deciles = Vec::with_capacity(10);

    // Extract 11 logs if runs are provided
    if let Some(all_runs) = runs {
        let log_indices = if total_runs >= 11 && (total_runs - 1).is_multiple_of(10) {
            // Perfect 10n + 1 system
            let n = (total_runs - 1) / 10;
            vec![
                n / 2,           // 5%
                n + n / 2,       // 15%
                2 * n + n / 2,   // 25%
                3 * n + n / 2,   // 35%
                4 * n + n / 2,   // 45%
                5 * n,           // 50% (True Median)
                5 * n + n / 2 + 1, // 55%
                6 * n + n / 2 + 1, // 65%
                7 * n + n / 2 + 1, // 75%
                8 * n + n / 2 + 1, // 85%
                9 * n + n / 2 + 1, // 95%
            ]
        } else {
            // Fallback for non-perfect counts
            vec![
                (total_runs as f64 * 0.05) as usize,
                (total_runs as f64 * 0.15) as usize,
                (total_runs as f64 * 0.25) as usize,
                (total_runs as f64 * 0.35) as usize,
                (total_runs as f64 * 0.45) as usize,
                (total_runs as f64 * 0.50) as usize,
                (total_runs as f64 * 0.55) as usize,
                (total_runs as f64 * 0.65) as usize,
                (total_runs as f64 * 0.75) as usize,
                (total_runs as f64 * 0.85) as usize,
                (total_runs as f64 * 0.95) as usize,
            ]
        };
        
        for idx in log_indices {
            let safe_idx = idx.min(total_runs - 1);
            decile_logs.push(all_runs[safe_idx].events.clone());
        }
    }

    // Always calculate 10 deciles for backward compatibility and granular UI data
    let decile_size = total_runs as f64 / 10.0;
    for i in 0..10 {
        let start_idx = (i as f64 * decile_size).floor() as usize;
        let end_idx = ((i + 1) as f64 * decile_size).floor() as usize;
        let slice = &results[start_idx..end_idx.min(total_runs)];
        if !slice.is_empty() {
            deciles.push(calculate_decile_stats_internal(slice, encounter_idx, i + 1, party_size, tdnw, sr_count));
        }
    }

    let median_idx = total_runs / 2;
    if let Some(&median_run) = results.get(median_idx) {
        let (hp_lost, _max_hp, survivors, duration, timeline, vit_timeline, pow_timeline) = calculate_run_stats_partial(median_run, encounter_idx, party_size, tdnw, sr_count);
        let (visualization_data, _) = extract_combatant_visualization_partial(median_run, encounter_idx);
        
        global_median = Some(DecileStats {
            decile: 0,
            label: "Global Median".to_string(),
                        median_survivors: survivors,
                        party_size,
                        total_hp_lost: hp_lost,
                        hp_lost_percent: if tdnw > 0.0 { (hp_lost / tdnw) * 100.0 } else { 0.0 },
                        win_rate: if survivors > 0 { 100.0 } else { 0.0 },                median_run_visualization: visualization_data,
            median_run_data: if let Some(idx) = encounter_idx { median_run.encounters.get(idx).cloned() } else { median_run.encounters.get(0).cloned() },
            battle_duration_rounds: duration,
            resource_timeline: timeline,
            vitality_timeline: vit_timeline,
            power_timeline: pow_timeline,
        });
    }

    let vitals_val = vitals.as_ref().unwrap();
    let encounter_label = get_encounter_label(&vitals_val.archetype);
    let analysis_summary = generate_analysis_summary(&vitals_val.archetype, vitals_val, global_median.as_ref().unwrap());
    let tuning_suggestions = generate_tuning_suggestions(&vitals_val.archetype);
    
    // Design quality based on balance between risk and reward
    let is_good_design = vitals_val.lethality_index < 0.4 && vitals_val.attrition_score > 0.1;
    let stars = if is_good_design { 3 } else if vitals_val.lethality_index < 0.6 { 2 } else { 1 };

    let battle_duration_rounds = global_median.as_ref().map(|m| m.battle_duration_rounds).unwrap_or(0);

    AggregateOutput {
        scenario_name: scenario_name.to_string(), total_runs, deciles, global_median, 
        vitality_range, power_range,
        decile_logs,
        battle_duration_rounds,
        intensity_tier: assess_intensity_tier_dynamic(results, tdnw, total_day_weight, current_encounter_weight),
        encounter_label, analysis_summary, tuning_suggestions, is_good_design, stars,
        tdnw,
        num_encounters,
        skyline,
        vitals,
        pacing,
    }
}

fn calculate_decile_stats_internal(slice: &[&SimulationResult], encounter_idx: Option<usize>, decile_num: usize, party_size: usize, tdnw: f64, sr_count: usize) -> DecileStats {
    let mut total_wins = 0.0;
    let mut total_hp_lost = 0.0;
    let mut total_survivors = 0;
    let mut total_duration = 0;
    let mut timelines = Vec::new();
    let mut vitality_timelines = Vec::new();
    let mut power_timelines = Vec::new();

    for &run in slice {
        let (hp_lost, _max_hp, survivors, duration, timeline, vit, pow) = calculate_run_stats_partial(run, encounter_idx, party_size, tdnw, sr_count);
        if survivors > 0 { total_wins += 1.0; }
        total_survivors += survivors;
        total_hp_lost += hp_lost;
        total_duration += duration;
        timelines.push(timeline);
        vitality_timelines.push(vit);
        power_timelines.push(pow);
    }

    let count = slice.len() as f64;
    let avg_hp_lost = if count > 0.0 { total_hp_lost / count } else { 0.0 };

    // Average the timelines
    let mut avg_timeline = Vec::new();
    let mut avg_vitality_timeline = Vec::new();
    let mut avg_power_timeline = Vec::new();

    if !timelines.is_empty() {
        let steps = timelines[0].len();
        for s in 0..steps {
            let mut step_sum = 0.0;
            let mut vit_step_sum = 0.0;
            let mut pow_step_sum = 0.0;
            for i in 0..timelines.len() {
                step_sum += timelines[i].get(s).cloned().unwrap_or(0.0);
                vit_step_sum += vitality_timelines[i].get(s).cloned().unwrap_or(0.0);
                pow_step_sum += power_timelines[i].get(s).cloned().unwrap_or(0.0);
            }
            avg_timeline.push(step_sum / count);
            avg_vitality_timeline.push(vit_step_sum / count);
            avg_power_timeline.push(pow_step_sum / count);
        }
    }

    let median_in_slice_idx = slice.len() / 2;
    let median_run = slice[median_in_slice_idx];
    let (visualization_data, _) = extract_combatant_visualization_partial(median_run, encounter_idx);

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
        hp_lost_percent: if tdnw > 0.0 { (avg_hp_lost / tdnw) * 100.0 } else { 0.0 },
        win_rate: if count > 0.0 { (total_wins / count) * 100.0 } else { 0.0 },
        median_run_visualization: visualization_data,
        median_run_data: if let Some(idx) = encounter_idx { median_run.encounters.get(idx).cloned() } else { median_run.encounters.get(0).cloned() },
        battle_duration_rounds: if count > 0.0 { (total_duration as f64 / count).round() as usize } else { 0 },
        resource_timeline: avg_timeline,
        vitality_timeline: avg_vitality_timeline,
        power_timeline: avg_power_timeline,
    }
}

fn generate_tuning_suggestions(archetype: &EncounterArchetype) -> Vec<String> {
    let mut suggestions = Vec::new();
    match archetype {
        EncounterArchetype::Broken => suggestions.push("Mathematically impossible. Reduce monster damage or count.".to_string()),
        EncounterArchetype::MeatGrinder => suggestions.push("Extremely lethal. High chance of TPK.".to_string()),
        EncounterArchetype::NovaTrap => suggestions.push("Burst damage threat. Consider smoothing out damage across rounds.".to_string()),
        EncounterArchetype::Trivial => suggestions.push("Under-tuned. Increase monster stats for more impact.".to_string()),
        _ => {}
    }
    suggestions
}

fn assess_intensity_tier_dynamic(results: &[&SimulationResult], tdnw: f64, total_weight: f64, encounter_weight: f64) -> IntensityTier {
    if results.is_empty() || tdnw <= 0.0 { return IntensityTier::Tier1; }
    
    let total_runs = results.len();
    let typical = results[total_runs / 2];
    let (hp_lost, _, _, _, _, _, _) = calculate_run_stats_partial(typical, None, 0, tdnw, 0);
    
    // Cost % relative to TDNW
    let cost_percent = hp_lost / tdnw; 
    
    // Target Drain = Weight / Total Weight
    let total_w = if total_weight <= 0.0 { 1.0 } else { total_weight };
    let target = encounter_weight / total_w;

    if cost_percent < (0.2 * target) { IntensityTier::Tier1 }
    else if cost_percent < (0.6 * target) { IntensityTier::Tier2 }
    else if cost_percent < (1.3 * target) { IntensityTier::Tier3 }
    else if cost_percent < (2.0 * target) { IntensityTier::Tier4 }
    else { IntensityTier::Tier5 }
}
