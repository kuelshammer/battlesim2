use super::types::*;
use crate::intensity_calculation::*;
use crate::model::*;
use std::collections::HashMap;

/// Calculate Total Daily Net Worth (sum of all player budgets)
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

/// Calculate TDNW from lightweight player data (before simulation)
pub fn calculate_tdnw_lightweight(players: &[Creature], sr_count: usize) -> f64 {
    let mut total = 0.0;
    for p in players {
        total += crate::intensity_calculation::calculate_daily_budget(p, sr_count) * (p.count as f64);
    }
    total
}

/// Calculate run statistics (resources, survivors, timelines)
pub fn calculate_run_stats_partial(
    run: &SimulationResult,
    encounter_idx: Option<usize>,
    party_size: usize,
    tdnw: f64,
    sr_count: usize,
) -> RunMetrics {
    let score = crate::aggregation::calculate_score(run);

    // 1. Count survivors
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

    RunMetrics {
        burned: burned_resources,
        survivors,
        duration,
        ehp_timeline: timeline,
        vitality_timeline,
        power_timeline,
    }
}

/// Calculate vitals (lethality, TPK risk, attrition, etc.) with custom config
pub fn calculate_vitals_with_config(
    results: &[&SimulationResult],
    encounter_idx: Option<usize>,
    party_size: usize,
    tdnw: f64,
    config: &GameBalance,
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
            near_death_survivors: 0.0,
            crisis_participation_rate: 0.0,
            min_hp_threshold: 1.0,
            avg_unconscious_rounds: 0.0,
            archetype: EncounterArchetype::Trivial,
            is_volatile: false,
            difficulty_grade: "S".to_string(),
            safety_grade: "A".to_string(),
            pacing_label: "Breezy".to_string(),
        };
    }

    // VALIDATION: Verify party_size matches actual team size
    let actual_party_size = results.iter().find_map(|&run| {
        let encounters = if let Some(idx) = encounter_idx {
            if idx < run.encounters.len() { &run.encounters[idx..=idx] } else { &[] }
        } else {
            &run.encounters[..]
        };

        encounters.iter()
            .filter_map(|enc| enc.rounds.first())
            .map(|round| round.team1.len())
            .next()
    });

    let validated_party_size = actual_party_size.unwrap_or(party_size);

    // 1. Calculate Lethality and TPK Risk
    let mut ko_count = 0;
    let mut tpk_count = 0;
    let mut _crisis_count = 0;
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
                if survivors < validated_party_size { run_has_ko = true; }
                if survivors == 0 { run_is_tpk = true; }

                if last_round.team1.iter().any(|c| (c.final_state.current_hp as f64 / c.creature.hp as f64) < 0.1) {
                    run_has_crisis = true;
                }
            }
        }

        if run_has_ko { ko_count += 1; }
        if run_is_tpk { tpk_count += 1; }
        if run_has_crisis { _crisis_count += 1; }
    }

    let lethality_index = ko_count as f64 / total_runs as f64;
    let tpk_risk = tpk_count as f64 / total_runs as f64;
    let deaths_door_index = total_deaths_door_rounds as f64 / total_runs as f64;

    // 2. Attrition and Volatility
    let p10_idx = (total_runs as f64 * 0.1) as usize;
    let p50_idx = total_runs / 2;

    let get_cost = |idx: usize| -> f64 {
        if idx >= total_runs || tdnw <= 0.0 { return 0.0; }
        let metrics = calculate_run_stats_partial(results[idx], encounter_idx, party_size, tdnw, 0);
        metrics.burned / tdnw
    };

    let p10_cost = get_cost(p10_idx);
    let p50_cost = get_cost(p50_idx);

    let attrition_score = p50_cost;
    let volatility_index = (p10_cost - p50_cost).max(0.0);
    let is_volatile = volatility_index > config.is_volatile_threshold;

    // 2.5. Experience-based metrics
    let mut total_near_death_survivors = 0.0;
    let mut total_crisis_participants = 0.0;
    let mut total_min_hp_threshold: f64 = 1.0;
    let mut total_unconscious_rounds = 0.0;

    for &run in results {
        let encounters = if let Some(idx) = encounter_idx {
            if idx < run.encounters.len() { &run.encounters[idx..=idx] } else { &[] }
        } else {
            &run.encounters[..]
        };

        let mut run_near_death_count = 0;
        let mut run_crisis_participants = 0;
        let mut run_min_hp: f64 = 1.0;
        let mut run_unconscious_rounds = 0;

        for enc in encounters {
            let mut char_min_hp: HashMap<String, f64> = HashMap::new();
            let mut char_crisis_hit: HashMap<String, bool> = HashMap::new();
            let mut char_unconscious_rounds: HashMap<String, usize> = HashMap::new();

            for round in &enc.rounds {
                for c in &round.team1 {
                    let char_id = &c.id;
                    let max_hp = c.creature.hp as f64;
                    let current_hp = c.final_state.current_hp as f64;
                    let hp_pct = if max_hp > 0.0 { current_hp / max_hp } else { 0.0 };

                    let entry = char_min_hp.entry(char_id.clone()).or_insert(1.0);
                    *entry = (*entry).min(hp_pct);

                    if hp_pct < 0.25 && hp_pct > 0.0 {
                        char_crisis_hit.insert(char_id.clone(), true);
                    }

                    if current_hp == 0.0 {
                        *char_unconscious_rounds.entry(char_id.clone()).or_insert(0) += 1;
                    }
                }
            }

            for (_id, min_hp) in &char_min_hp {
                run_min_hp = run_min_hp.min(*min_hp);
            }
            run_crisis_participants += char_crisis_hit.len();
            run_unconscious_rounds += char_unconscious_rounds.values().sum::<usize>();

            if let Some(last_round) = enc.rounds.last() {
                for c in &last_round.team1 {
                    let max_hp = c.creature.hp as f64;
                    let current_hp = c.final_state.current_hp as f64;
                    let hp_pct = if max_hp > 0.0 { current_hp / max_hp } else { 0.0 };

                    if hp_pct >= 0.01 && hp_pct <= 0.10 {
                        run_near_death_count += 1;
                    }
                }
            }
        }

        total_near_death_survivors += run_near_death_count as f64;
        total_crisis_participants += run_crisis_participants as f64;
        total_min_hp_threshold = total_min_hp_threshold.min(run_min_hp);
        total_unconscious_rounds += run_unconscious_rounds as f64;
    }

    let near_death_survivors = total_near_death_survivors / total_runs as f64;
    let crisis_participation_rate = if validated_party_size > 0 {
        total_crisis_participants / (total_runs as f64 * validated_party_size as f64)
    } else {
        0.0
    };
    let avg_unconscious_rounds = if validated_party_size > 0 {
        total_unconscious_rounds / (total_runs as f64 * validated_party_size as f64)
    } else {
        0.0
    };

    // 3. Archetype Determination
    let mut temp_vitals = Vitals {
        lethality_index,
        tpk_risk,
        attrition_score,
        volatility_index,
        doom_horizon: 0.0,
        deaths_door_index,
        near_death_survivors,
        crisis_participation_rate,
        min_hp_threshold: total_min_hp_threshold,
        avg_unconscious_rounds,
        archetype: EncounterArchetype::Standard,
        is_volatile,
        difficulty_grade: "".to_string(),
        safety_grade: "".to_string(),
        pacing_label: "".to_string(),
    };
    temp_vitals.archetype = super::narrative::assess_archetype_with_config(&temp_vitals, config);
    temp_vitals.difficulty_grade = super::narrative::calculate_difficulty_grade(temp_vitals.lethality_index);
    temp_vitals.safety_grade = super::narrative::calculate_safety_grade(&temp_vitals);
    temp_vitals.pacing_label = super::narrative::generate_pacing_label(&temp_vitals);

    // 4. Doom Horizon
    temp_vitals.doom_horizon = if attrition_score > 0.01 {
        1.0 / attrition_score
    } else {
        10.0 // Practically infinite
    };

    temp_vitals
}

/// Calculate vitals (lethality, TPK risk, attrition, etc.)
pub fn calculate_vitals(
    results: &[&SimulationResult],
    encounter_idx: Option<usize>,
    party_size: usize,
    tdnw: f64,
) -> Vitals {
    calculate_vitals_with_config(results, encounter_idx, party_size, tdnw, &GameBalance::default())
}

/// Calculate statistics for a decile slice
pub fn calculate_decile_stats_internal(
    slice: &[&SimulationResult],
    encounter_idx: Option<usize>,
    decile_num: usize,
    party_size: usize,
    tdnw: f64,
    sr_count: usize,
    extract_vis_fn: &dyn Fn(&SimulationResult, Option<usize>) -> (Vec<super::types::CombatantVisualization>, usize),
) -> DecileStats {
    let mut total_wins = 0.0;
    let mut total_hp_lost = 0.0;
    let mut total_survivors = 0;
    let mut total_duration = 0;
    let mut timelines = Vec::new();
    let mut vitality_timelines = Vec::new();
    let mut power_timelines = Vec::new();

    for &run in slice {
        let metrics = calculate_run_stats_partial(run, encounter_idx, party_size, tdnw, sr_count);
        if metrics.survivors > 0 { total_wins += 1.0; }
        total_survivors += metrics.survivors;
        total_hp_lost += metrics.burned;
        total_duration += metrics.duration;
        timelines.push(metrics.ehp_timeline);
        vitality_timelines.push(metrics.vitality_timeline);
        power_timelines.push(metrics.power_timeline);
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
    let (visualization_data, _) = extract_vis_fn(median_run, encounter_idx);

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

/// Internal analysis results processing
pub fn analyze_results_internal(
    results: &[&SimulationResult],
    encounter_idx: Option<usize>,
    scenario_name: &str,
    party_size: usize,
    runs: Option<&[crate::model::SimulationRun]>,
    sr_count: usize,
    extract_vis_fn: &dyn Fn(&SimulationResult, Option<usize>) -> (Vec<super::types::CombatantVisualization>, usize),
) -> AggregateOutput {
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

    // Compute pacing using pre-computed metrics for median run
    let pacing = if encounter_idx.is_none() && !results.is_empty() {
        let median_idx = results.len() / 2;
        let median_run = results[median_idx];
        let median_metrics = calculate_run_stats_partial(median_run, None, party_size, tdnw, sr_count);
        super::narrative::calculate_day_pacing(median_run, &median_metrics, tdnw)
    } else {
        None
    };

    // Compute skyline analysis (100 buckets)
    let skyline = Some(crate::percentile_analysis::run_skyline_analysis(results, party_size, encounter_idx));

    // Collect all timelines for independent percentile calculation
    let mut all_vits = Vec::with_capacity(results.len());
    let mut all_pows = Vec::with_capacity(results.len());
    for &run in results {
        let metrics = calculate_run_stats_partial(run, encounter_idx, party_size, tdnw, sr_count);
        all_vits.push(metrics.vitality_timeline);
        all_pows.push(metrics.power_timeline);
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
        let log_indices = vec![
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
        ];

        for idx in log_indices {
            let safe_idx = idx.min(total_runs - 1);
            decile_logs.push(all_runs[safe_idx].events.clone());
        }
    }

    // Always calculate 10 deciles
    let decile_size = total_runs as f64 / 10.0;
    for i in 0..10 {
        let start_idx = (i as f64 * decile_size).floor() as usize;
        let end_idx = ((i + 1) as f64 * decile_size).floor() as usize;
        let slice = &results[start_idx..end_idx.min(total_runs)];
        if !slice.is_empty() {
            deciles.push(calculate_decile_stats_internal(slice, encounter_idx, i + 1, party_size, tdnw, sr_count, extract_vis_fn));
        }
    }

    let median_idx = total_runs / 2;
    if let Some(&median_run) = results.get(median_idx) {
        let metrics = calculate_run_stats_partial(median_run, encounter_idx, party_size, tdnw, sr_count);
        let (visualization_data, _) = extract_vis_fn(median_run, encounter_idx);

        global_median = Some(DecileStats {
            decile: 0,
            label: "Global Median".to_string(),
            median_survivors: metrics.survivors,
            party_size,
            total_hp_lost: metrics.burned,
            hp_lost_percent: if tdnw > 0.0 { (metrics.burned / tdnw) * 100.0 } else { 0.0 },
            win_rate: if metrics.survivors > 0 { 100.0 } else { 0.0 },
            median_run_visualization: visualization_data,
            median_run_data: if let Some(idx) = encounter_idx { median_run.encounters.get(idx).cloned() } else { median_run.encounters.get(0).cloned() },
            battle_duration_rounds: metrics.duration,
            resource_timeline: metrics.ehp_timeline,
            vitality_timeline: metrics.vitality_timeline,
            power_timeline: metrics.power_timeline,
        });
    }

    let vitals_val = vitals.as_ref().unwrap();
    let encounter_label = super::narrative::get_encounter_label(&vitals_val.archetype);
    let analysis_summary = super::narrative::generate_analysis_summary(&vitals_val.archetype, vitals_val, global_median.as_ref().unwrap());
    let tuning_suggestions = super::narrative::generate_tuning_suggestions(&vitals_val.archetype);

    let is_good_design = vitals_val.lethality_index < 0.4 && vitals_val.attrition_score > 0.1;
    let stars = if is_good_design { 3 } else if vitals_val.lethality_index < 0.6 { 2 } else { 1 };

    let battle_duration_rounds = global_median.as_ref().map(|m| m.battle_duration_rounds).unwrap_or(0);

    AggregateOutput {
        scenario_name: scenario_name.to_string(), total_runs, deciles, global_median,
        vitality_range, power_range,
        decile_logs,
        battle_duration_rounds,
        intensity_tier: {
            let typical_metrics = if !results.is_empty() {
                let typical_idx = results.len() / 2;
                Some(calculate_run_stats_partial(results[typical_idx], encounter_idx, party_size, tdnw, sr_count))
            } else {
                None
            };
            if let Some(ref metrics) = typical_metrics {
                super::narrative::assess_intensity_tier_dynamic(metrics, tdnw, total_day_weight, current_encounter_weight)
            } else {
                IntensityTier::Tier1
            }
        },
        encounter_label, analysis_summary, tuning_suggestions, is_good_design, stars,
        tdnw,
        num_encounters,
        skyline,
        vitals,
        pacing,
    }
}
