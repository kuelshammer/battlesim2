use super::types::*;

/// Assess encounter archetype based on vitals
///
/// Uses game balance configuration for thresholds, enabling tuning without code changes.
pub fn assess_archetype(vitals: &Vitals) -> EncounterArchetype {
    assess_archetype_with_config(vitals, &GameBalance::default())
}

/// Assess encounter archetype using custom game balance configuration
pub fn assess_archetype_with_config(vitals: &Vitals, config: &GameBalance) -> EncounterArchetype {
    if vitals.tpk_risk > config.tpk_broken_threshold {
        return EncounterArchetype::Broken;
    }

    // Check for High Volatility (Coin Flip)
    // High chance of death/failure, but not necessarily a guaranteed grind.
    // Volatility index > threshold means P10 and P50 are very different.
    if vitals.volatility_index > config.volatility_high_threshold
        && vitals.lethality_index > config.coin_flip_lethality_threshold
    {
        return EncounterArchetype::CoinFlip;
    }

    if vitals.tpk_risk > config.tpk_meat_grinder_threshold {
        return EncounterArchetype::MeatGrinder;
    }

    if vitals.lethality_index > config.lethality_boss_threshold {
        return EncounterArchetype::MeatGrinder;
    }

    if vitals.lethality_index > config.lethality_elite_threshold {
        if vitals.attrition_score < config.attrition_nova_trap_threshold {
            return EncounterArchetype::NovaTrap;
        }
        return EncounterArchetype::BossFight;
    }

    if vitals.lethality_index > config.lethality_standard_threshold {
        if vitals.attrition_score > config.attrition_grind_high_threshold {
            return EncounterArchetype::TheGrind;
        }
        return EncounterArchetype::EliteChallenge;
    }

    if vitals.lethality_index > config.lethality_skirmish_threshold {
        if vitals.attrition_score > config.attrition_grind_low_threshold {
            return EncounterArchetype::TheGrind;
        }
        return EncounterArchetype::Standard;
    }

    if vitals.attrition_score > config.attrition_skirmish_threshold {
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
pub fn generate_analysis_summary(
    archetype: &EncounterArchetype,
    vitals: &Vitals,
    typical: &DecileStats,
) -> String {
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

    format!(
        "{}: {} | Attrition: {}% | Typical Survivors: {}/{}",
        archetype,
        archetype_desc,
        (vitals.attrition_score * 100.0).round(),
        typical.median_survivors,
        typical.party_size
    )
}

/// Generate tuning suggestions based on archetype
pub fn generate_tuning_suggestions(archetype: &EncounterArchetype) -> Vec<String> {
    let mut suggestions = Vec::new();
    match archetype {
        EncounterArchetype::Broken => suggestions
            .push("Mathematically impossible. Reduce monster damage or count.".to_string()),
        EncounterArchetype::MeatGrinder => {
            suggestions.push("Extremely lethal. High chance of TPK.".to_string())
        }
        EncounterArchetype::NovaTrap => suggestions
            .push("Burst damage threat. Consider smoothing out damage across rounds.".to_string()),
        EncounterArchetype::Trivial => {
            suggestions.push("Under-tuned. Increase monster stats for more impact.".to_string())
        }
        _ => {}
    }
    suggestions
}

/// Calculate day pacing metrics (Director's Score, Rhythm, etc.)
///
/// This version accepts pre-computed metrics for the median run and optional game balance config.
pub fn calculate_day_pacing_with_config(
    median_run: &crate::model::SimulationResult,
    median_metrics: &super::types::RunMetrics,
    tdnw: f64,
    config: &GameBalance,
) -> Option<DayPacing> {
    // 1. Attrition Score (Efficiency)
    // Ideal end state is sweet spot range (from config).
    let burned = median_metrics.burned;
    let end_res_pct = if tdnw > 0.0 {
        ((tdnw - burned) / tdnw) * 100.0
    } else {
        100.0
    };

    let attrition_score = if end_res_pct < config.pacing_exhaustion_pct {
        20.0 // TPK/Total Exhaustion
    } else if end_res_pct < config.pacing_tense_pct {
        70.0 // Tense, maybe too much
    } else if end_res_pct < config.pacing_sweet_spot_high_pct
        && end_res_pct >= config.pacing_sweet_spot_low_pct
    {
        100.0 // Sweet spot
    } else if end_res_pct < config.pacing_easy_pct {
        60.0 // A bit easy
    } else {
        30.0 // Boring
    };

    // 2. Rhythm Score (Difficulty Escalation)
    // Logic: Allow 1 "Breather" (Dip in difficulty). Penalize 2+ dips.
    // "Dip" is defined as weight < dip_tolerance * max_weight_so_far.
    let mut rhythm_score = 100.0;
    let mut max_weight_so_far = 0.0;
    let mut dips = 0;

    for enc in &median_run.encounters {
        let w = enc.target_role.weight();

        // Check for dip with configured tolerance
        if w < max_weight_so_far * config.dip_tolerance {
            dips += 1;
        }

        max_weight_so_far = max_weight_so_far.max(w);
    }

    // Penalize only if we have more than 1 dip (allow 1 breather)
    let penalty_dips = if dips > 1 { dips - 1 } else { 0 };

    if median_run.encounters.len() > 1 {
        rhythm_score = (100.0 - (penalty_dips as f64 * config.dip_penalty)).max(0.0);
    }

    // 3. Recovery Score (Placeholder for now)
    let recovery_score = 100.0;

    // 4. Archetype Determination
    let archetype = if rhythm_score >= 80.0 && attrition_score >= 80.0 {
        "The Hero's Journey".to_string()
    } else if end_res_pct > config.pacing_easy_pct {
        "The Slow Burn".to_string()
    } else if penalty_dips > 0 {
        "The Rollercoaster".to_string()
    } else if end_res_pct < config.pacing_tense_pct {
        "The Meat Grinder".to_string()
    } else {
        "The Gritty Adventure".to_string()
    };

    let director_score = rhythm_score * config.director_rhythm_weight
        + attrition_score * config.director_attrition_weight
        + recovery_score * config.director_recovery_weight;

    Some(DayPacing {
        archetype,
        director_score,
        rhythm_score,
        attrition_score,
        recovery_score,
    })
}

/// Calculate day pacing metrics (Director's Score, Rhythm, etc.)
///
/// This version accepts pre-computed metrics for the median run, eliminating the callback pattern.
/// Uses default game balance configuration.
pub fn calculate_day_pacing(
    median_run: &crate::model::SimulationResult,
    median_metrics: &super::types::RunMetrics,
    tdnw: f64,
) -> Option<DayPacing> {
    calculate_day_pacing_with_config(median_run, median_metrics, tdnw, &GameBalance::default())
}

/// Assess intensity tier dynamically based on resource cost vs target
///
/// This version accepts pre-computed metrics for the typical (median) run and custom config.
pub fn assess_intensity_tier_dynamic_with_config(
    typical_metrics: &super::types::RunMetrics,
    tdnw: f64,
    total_weight: f64,
    encounter_weight: f64,
    config: &GameBalance,
) -> IntensityTier {
    if tdnw <= 0.0 {
        return IntensityTier::Tier1;
    }

    // Cost % relative to TDNW
    let cost_percent = typical_metrics.burned / tdnw;

    // Target Drain = Weight / Total Weight
    let total_w = if total_weight <= 0.0 {
        1.0
    } else {
        total_weight
    };
    let target = encounter_weight / total_w;

    if cost_percent < (config.intensity_tier1_multiplier * target) {
        IntensityTier::Tier1
    } else if cost_percent < (config.intensity_tier2_multiplier * target) {
        IntensityTier::Tier2
    } else if cost_percent < (config.intensity_tier3_multiplier * target) {
        IntensityTier::Tier3
    } else if cost_percent < (config.intensity_tier4_multiplier * target) {
        IntensityTier::Tier4
    } else {
        IntensityTier::Tier5
    }
}

/// Assess intensity tier dynamically based on resource cost vs target
///
/// This version accepts pre-computed metrics for the typical (median) run.
/// Uses default game balance configuration.
pub fn assess_intensity_tier_dynamic(
    typical_metrics: &super::types::RunMetrics,
    tdnw: f64,
    total_weight: f64,
    encounter_weight: f64,
) -> IntensityTier {
    assess_intensity_tier_dynamic_with_config(
        typical_metrics,
        tdnw,
        total_weight,
        encounter_weight,
        &GameBalance::default(),
    )
}

/// Calculate difficulty grade (S-F) for backward compatibility and coloring
pub fn calculate_difficulty_grade(lethality_index: f64) -> String {
    if lethality_index < 0.05 {
        "S".to_string()
    } else if lethality_index < 0.15 {
        "A".to_string()
    } else if lethality_index < 0.30 {
        "B".to_string()
    } else if lethality_index < 0.50 {
        "C".to_string()
    } else if lethality_index < 0.70 {
        "D".to_string()
    } else {
        "F".to_string()
    }
}

/// Calculate safety grade (A-D) for banner coloring
pub fn calculate_safety_grade(vitals: &Vitals) -> String {
    if vitals.tpk_risk > 0.1 {
        "D".to_string()
    } else if vitals.lethality_index > 0.3 {
        "C".to_string()
    } else if vitals.lethality_index > 0.1 {
        "B".to_string()
    } else {
        "A".to_string()
    }
}

/// Generate narrative pacing label
pub fn generate_pacing_label(vitals: &Vitals) -> String {
    if vitals.volatility_index > 0.2 {
        "Chaotic".to_string()
    } else if vitals.lethality_index > 0.4 && vitals.attrition_score < 0.2 {
        "Sudden Death".to_string()
    } else if vitals.lethality_index < 0.1 && vitals.attrition_score > 0.4 {
        "War of Attrition".to_string()
    } else if vitals.lethality_index > 0.3 && vitals.attrition_score > 0.3 {
        "Epic".to_string()
    } else if vitals.lethality_index < 0.05 && vitals.attrition_score < 0.1 {
        "Breezy".to_string()
    } else {
        "Steady".to_string()
    }
}
