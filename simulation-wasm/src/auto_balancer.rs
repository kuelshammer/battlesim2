use crate::model::{Creature, TimelineStep, MonsterRole};
use crate::creature_adjustment::{detect_role, adjust_hp, adjust_damage, adjust_dc};
use crate::decile_analysis::{EncounterArchetype, IntensityTier, AggregateOutput};
use crate::run_event_driven_simulation_rust;


pub struct AutoBalancer {
    pub max_iterations: usize,
    pub target_simulations: usize,
}

impl AutoBalancer {
    pub fn new() -> Self {
        Self {
            max_iterations: 30,
            target_simulations: 503,
        }
    }

    pub fn balance_encounter(
        &self, 
        players: Vec<Creature>, 
        mut monsters: Vec<Creature>,
        full_day_timeline: Vec<TimelineStep>,
        encounter_index: usize,
    ) -> (Vec<Creature>, AggregateOutput) {
        // 1. Initial Simulation with full context
        let mut analysis = self.run_analysis(&players, &monsters, &full_day_timeline, encounter_index);
        let initial_archetype = analysis.vitals.as_ref().map(|v| v.archetype.clone()).unwrap_or(EncounterArchetype::Standard);

        // 2. Role Detection for each monster
        let total_hp: f64 = monsters.iter().map(|m| m.hp as f64 * m.count).sum();
        
        // Estimate party DPR
        let mut party_dpr = 0.0;
        for p in &players {
            let stats = crate::combat_stats::CombatantStats::calculate(p);
            party_dpr += stats.total_dpr * p.count;
        }
        
        let roles: Vec<MonsterRole> = monsters.iter().map(|m| detect_role(m, total_hp, party_dpr)).collect();

        // 3. Optimization Loop
        for _ in 0..self.max_iterations {
            if self.is_balanced(&analysis, &initial_archetype) {
                break;
            }

            let Some(typical) = analysis.global_median.as_ref() else {
                break;
            };
            let vitality = typical.vitality_timeline.last().cloned().unwrap_or(100.0);
            let power = typical.power_timeline.last().cloned().unwrap_or(100.0);

            if self.is_too_deadly(&analysis) {
                // Safety Clamp (Nerf)
                let step = if analysis.vitals.as_ref().map(|v| v.archetype == EncounterArchetype::Broken).unwrap_or(false) { -0.15 } else { -0.05 };
                self.apply_adjustment(&mut monsters, &roles, step, true, vitality, power);
            } else if self.is_too_easy(&analysis) {
                // Intensity Pump (Buff)
                // Slog Filter: Stop buffing HP if rounds > 8
                if analysis.battle_duration_rounds > 8 {
                    // Switch to damage buff instead of HP buff to avoid slog
                    self.apply_adjustment(&mut monsters, &roles, 0.05, false, vitality, power);
                } else {
                    self.apply_adjustment(&mut monsters, &roles, 0.10, false, vitality, power);
                }
            } else {
                break;
            }

            analysis = self.run_analysis(&players, &monsters, &full_day_timeline, encounter_index);
        }

        // 4. Finalize dice notation
        for m in &mut monsters {
            crate::creature_adjustment::finalize_adjustments(m);
        }

        (monsters, analysis)
    }

    fn run_analysis(&self, players: &[Creature], monsters: &[Creature], full_day_timeline: &[TimelineStep], encounter_index: usize) -> AggregateOutput {
        // Construct a timeline that is identical to the full day, 
        // EXCEPT the target encounter uses the currently-being-optimized monsters.
        let mut modified_timeline = full_day_timeline.to_vec();
        if let Some(TimelineStep::Combat(enc)) = modified_timeline.get_mut(encounter_index) {
            enc.monsters = monsters.to_vec();
        }
        
        let runs = run_event_driven_simulation_rust(
            players.to_vec(),
            modified_timeline,
            self.target_simulations,
            false,
            None,
        );
        let raw_results: Vec<_> = runs.into_iter().map(|r| r.result).collect();
        
        let sr_count = full_day_timeline.iter().filter(|s| matches!(s, TimelineStep::ShortRest(_))).count();

        // We analyze the SPECIFIC encounter at encounter_index
        crate::decile_analysis::run_encounter_analysis(&raw_results, encounter_index, "Auto-Balance", players.len(), sr_count)
    }

    fn is_balanced(&self, analysis: &AggregateOutput, initial_archetype: &EncounterArchetype) -> bool {
        let vitals = match &analysis.vitals {
            Some(v) => v,
            None => return true,
        };

        let safety_ok = if *initial_archetype == EncounterArchetype::Broken {
            vitals.archetype != EncounterArchetype::Broken && vitals.tpk_risk < 0.2
        } else {
            vitals.tpk_risk < 0.1
        };

        safety_ok && matches!(analysis.intensity_tier, IntensityTier::Tier3)
    }

    fn is_too_deadly(&self, analysis: &AggregateOutput) -> bool {
        let vitals = match &analysis.vitals {
            Some(v) => v,
            None => return false,
        };

        // Too deadly if archetype is Broken/MeatGrinder OR if it's Tier 4/5 (should be Tier 3)
        matches!(vitals.archetype, EncounterArchetype::Broken | EncounterArchetype::MeatGrinder) ||
        matches!(analysis.intensity_tier, IntensityTier::Tier4 | IntensityTier::Tier5)
    }

    fn is_too_easy(&self, analysis: &AggregateOutput) -> bool {
        matches!(analysis.intensity_tier, IntensityTier::Tier1 | IntensityTier::Tier2)
    }

    fn apply_adjustment(
        &self, 
        monsters: &mut [Creature], 
        roles: &[MonsterRole], 
        step: f64, 
        is_nerf: bool,
        vitality: f64,
        power: f64
    ) {
        let is_burst_risk = vitality < power - 15.0; // Significant gap
        let is_slog_risk = power < vitality - 15.0;

        for (m, role) in monsters.iter_mut().zip(roles.iter()) {
            match (role, is_nerf) {
                (MonsterRole::Boss, true) => {
                    if is_burst_risk { adjust_damage(m, step); }
                    else if is_slog_risk { adjust_hp(m, step); }
                    else { adjust_damage(m, step); adjust_hp(m, step); }
                },
                (MonsterRole::Boss, false) => {
                    if is_burst_risk { adjust_hp(m, step); }
                    else if is_slog_risk { adjust_damage(m, step); }
                    else { adjust_hp(m, step); }
                },
                
                (MonsterRole::Brute, true) => {
                    if is_burst_risk { adjust_damage(m, step); }
                    else { adjust_hp(m, step); }
                },
                (MonsterRole::Brute, false) => {
                    if is_slog_risk { adjust_damage(m, step); }
                    else { adjust_hp(m, step); }
                },
                
                (MonsterRole::Striker, true) => adjust_damage(m, step),
                (MonsterRole::Striker, false) => adjust_damage(m, step),
                
                (MonsterRole::Controller, true) => {
                    if is_burst_risk { adjust_dc(m, -0.5); }
                    else { adjust_hp(m, step); }
                },
                (MonsterRole::Controller, false) => adjust_hp(m, step),
                
                (MonsterRole::Minion, true) => { if m.count > 1.0 { m.count -= 1.0; } },
                (MonsterRole::Minion, false) => { m.count += 1.0; },
                
                (MonsterRole::Unknown, true) => {
                    if is_burst_risk { adjust_damage(m, step); }
                    else { adjust_hp(m, step); }
                },
                (MonsterRole::Unknown, false) => {
                    if is_slog_risk { adjust_damage(m, step); }
                    else { adjust_hp(m, step); }
                },
            }
        }
    }
}

impl Default for AutoBalancer {
    fn default() -> Self {
        Self::new()
    }
}
