use crate::model::{Creature, Encounter, TimelineStep, MonsterRole};
use crate::creature_adjustment::{detect_role, adjust_hp, adjust_damage, adjust_dc};
use crate::decile_analysis::{run_decile_analysis, SafetyGrade, IntensityTier, AggregateOutput};
use crate::lib::run_event_driven_simulation_rust;
use std::collections::HashMap;

pub struct AutoBalancer {
    pub max_iterations: usize,
    pub target_simulations: usize,
}

impl AutoBalancer {
    pub fn new() -> Self {
        Self {
            max_iterations: 15,
            target_simulations: 2510,
        }
    }

    pub fn balance_encounter(
        &self, 
        players: Vec<Creature>, 
        mut monsters: Vec<Creature>
    ) -> (Vec<Creature>, AggregateOutput) {
        // 1. Initial Simulation
        let mut analysis = self.run_analysis(&players, &monsters);

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
            if self.is_balanced(&analysis) {
                break;
            }

            if self.is_too_deadly(&analysis) {
                // Safety Clamp (Nerf)
                let step = if analysis.safety_grade == SafetyGrade::F { -0.15 } else { -0.05 };
                self.apply_adjustment(&mut monsters, &roles, step, true);
            } else if self.is_too_easy(&analysis) {
                // Intensity Pump (Buff)
                // Slog Filter: Stop buffing HP if rounds > 8
                if analysis.battle_duration_rounds > 8 {
                    // Switch to damage buff instead of HP buff to avoid slog
                    self.apply_adjustment(&mut monsters, &roles, 0.05, false);
                } else {
                    self.apply_adjustment(&mut monsters, &roles, 0.10, false);
                }
            } else {
                break;
            }

            analysis = self.run_analysis(&players, &monsters);
        }

        (monsters, analysis)
    }

    fn run_analysis(&self, players: &[Creature], monsters: &[Creature]) -> AggregateOutput {
        let timeline = vec![TimelineStep::Combat(Encounter {
            monsters: monsters.to_vec(),
            players_surprised: None,
            monsters_surprised: None,
            short_rest: None,
            players_precast: None,
            monsters_precast: None,
        })];
        
        let runs = run_event_driven_simulation_rust(
            players.to_vec(), 
            timeline, 
            self.target_simulations, 
            false
        );
        let raw_results: Vec<_> = runs.into_iter().map(|r| r.result).collect();
        run_decile_analysis(&raw_results, "Auto-Balance", players.len())
    }

    fn is_balanced(&self, analysis: &AggregateOutput) -> bool {
        matches!(analysis.safety_grade, SafetyGrade::A | SafetyGrade::B) &&
        matches!(analysis.intensity_tier, IntensityTier::Tier3 | IntensityTier::Tier4)
    }

    fn is_too_deadly(&self, analysis: &AggregateOutput) -> bool {
        matches!(analysis.safety_grade, SafetyGrade::C | SafetyGrade::D | SafetyGrade::F)
    }

    fn is_too_easy(&self, analysis: &AggregateOutput) -> bool {
        matches!(analysis.intensity_tier, IntensityTier::Tier1 | IntensityTier::Tier2)
    }

    fn apply_adjustment(&self, monsters: &mut Vec<Creature>, roles: &[MonsterRole], step: f64, is_nerf: bool) {
        for (m, role) in monsters.iter_mut().zip(roles.iter()) {
            match (role, is_nerf) {
                (MonsterRole::Boss, true) => adjust_damage(m, step),
                (MonsterRole::Boss, false) => adjust_hp(m, step),
                
                (MonsterRole::Brute, true) => adjust_damage(m, step),
                (MonsterRole::Brute, false) => adjust_hp(m, step),
                
                (MonsterRole::Striker, true) => adjust_damage(m, step), // accuracy would be better but dpr is knob for now
                (MonsterRole::Striker, false) => adjust_damage(m, step),
                
                (MonsterRole::Controller, true) => adjust_dc(m, -1.0),
                (MonsterRole::Controller, false) => adjust_hp(m, step),
                
                (MonsterRole::Minion, true) => { if m.count > 1.0 { m.count -= 1.0; } },
                (MonsterRole::Minion, false) => { m.count += 1.0; },
                
                (MonsterRole::Unknown, true) => adjust_damage(m, step),
                (MonsterRole::Unknown, false) => adjust_hp(m, step),
            }
        }
    }
}
