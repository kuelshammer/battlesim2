use super::types::*;

/// Assess encounter archetype based on vitals
pub fn assess_archetype(vitals: &Vitals) -> EncounterArchetype {
    if vitals.tpk_risk > 0.5 { return EncounterArchetype::Broken; }

    // Check for High Volatility (Coin Flip)
    // High chance of death/failure, but not necessarily a guaranteed grind.
    // Volatility index > 0.15 means P10 and P50 are very different.
    if vitals.volatility_index > 0.15 && vitals.lethality_index > 0.05 {
        return EncounterArchetype::CoinFlip;
    }

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

/// Get encounter label from archetype
pub fn get_encounter_label(archetype: &EncounterArchetype) -> EncounterLabel {
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
        EncounterArchetype::CoinFlip => EncounterLabel::TPKRisk,
    }
}

/// Generate analysis summary text
pub fn generate_analysis_summary(archetype: &EncounterArchetype, vitals: &Vitals, typical: &DecileStats) -> String {
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
        EncounterArchetype::CoinFlip => "High volatility. Swingy.",
    };

    format!("{}: {} | Attrition: {}% | Typical Survivors: {}/{}",
        archetype, archetype_desc, (vitals.attrition_score * 100.0).round(), typical.median_survivors, typical.party_size)
}

/// Generate tuning suggestions based on archetype
pub fn generate_tuning_suggestions(archetype: &EncounterArchetype) -> Vec<String> {
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

/// Calculate day pacing metrics (Director's Score, Rhythm, etc.)
pub fn calculate_day_pacing<F>(
    results: &[&crate::model::SimulationResult],
    encounter_idx: Option<usize>,
    tdnw: f64,
    sr_count: usize,
    calc_stats_fn: F,
) -> Option<DayPacing>
where
    F: FnOnce(&crate::model::SimulationResult, Option<usize>, usize, f64, usize) -> (f64, f64, usize, usize, Vec<f64>, Vec<f64>, Vec<f64>),
{
    if encounter_idx.is_some() || results.is_empty() {
        return None; // Only for overall day analysis
    }

    let total_runs = results.len();
    let median_idx = total_runs / 2;
    let median_run = results[median_idx];

    // 1. Attrition Score (Efficiency)
    // Ideal end state is 10-30% resources.
    let (burned, _, _, _, _, _, _) = calc_stats_fn(median_run, None, 0, tdnw, sr_count);
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
    // Logic: Allow 1 "Breather" (Dip in difficulty). Penalize 2+ dips.
    // "Dip" is defined as weight < 0.9 * max_weight_so_far.
    let mut rhythm_score = 100.0;
    let mut max_weight_so_far = 0.0;
    let mut dips = 0;

    for enc in &median_run.encounters {
        let w = enc.target_role.weight();

        // Check for dip with 10% tolerance (wiggle room)
        if w < max_weight_so_far * 0.9 {
            dips += 1;
        }

        max_weight_so_far = max_weight_so_far.max(w);
    }

    // Penalize only if we have more than 1 dip (allow 1 breather)
    let penalty_dips = if dips > 1 { dips - 1 } else { 0 };

    if median_run.encounters.len() > 1 {
        rhythm_score = (100.0 - (penalty_dips as f64 * 30.0)).max(0.0);
    }

    // 3. Recovery Score (Placeholder for now)
    let recovery_score = 100.0;

    // 4. Archetype Determination
    let archetype = if rhythm_score >= 80.0 && attrition_score >= 80.0 {
        "The Hero's Journey".to_string()
    } else if end_res_pct > 60.0 {
        "The Slow Burn".to_string()
    } else if penalty_dips > 0 {
        "The Rollercoaster".to_string()
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

/// Assess intensity tier dynamically based on resource cost vs target
pub fn assess_intensity_tier_dynamic<F>(
    results: &[&crate::model::SimulationResult],
    tdnw: f64,
    total_weight: f64,
    encounter_weight: f64,
    calc_stats_fn: F,
) -> IntensityTier
where
    F: FnOnce(&crate::model::SimulationResult, Option<usize>, usize, f64, usize) -> (f64, f64, usize, usize, Vec<f64>, Vec<f64>, Vec<f64>),
{
    if results.is_empty() || tdnw <= 0.0 { return IntensityTier::Tier1; }

    let total_runs = results.len();
    let typical = results[total_runs / 2];
    let (hp_lost, _, _, _, _, _, _) = calc_stats_fn(typical, None, 0, tdnw, 0);

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
