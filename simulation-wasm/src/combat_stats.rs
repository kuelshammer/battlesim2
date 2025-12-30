use crate::dice;
use crate::model::{Combattant, Creature};
use std::collections::HashMap;

/// Pre-calculated combat statistics for a creature type
#[derive(Debug, Clone)]
pub struct CombatantStats {
    /// Creature identifier
    pub creature_id: String,
    
    /// Base Armor Class
    pub ac: f64,
    
    /// Estimated Damage Per Round against baseline AC (15)
    pub dpr: f64,
    
    /// Hit probability against baseline AC (15)
    pub hit_probability: f64,
    
    /// Best action DPR (for action economy decisions)
    pub best_action_dpr: f64,
    
    /// Best bonus action DPR
    pub best_bonus_action_dpr: f64,
    
    /// Total effective DPR (action + bonus action)
    pub total_dpr: f64,
    
    /// Number of attack actions available
    pub attack_count: usize,
    
    /// Has bonus action attacks
    pub has_bonus_attacks: bool,
}

impl Default for CombatantStats {
    fn default() -> Self {
        Self {
            creature_id: "unknown".to_string(),
            ac: 10.0,
            dpr: 0.0,
            hit_probability: 0.5,
            best_action_dpr: 0.0,
            best_bonus_action_dpr: 0.0,
            total_dpr: 0.0,
            attack_count: 0,
            has_bonus_attacks: false,
        }
    }
}

impl CombatantStats {
    /// Calculate combat statistics for a creature
    pub fn calculate(creature: &Creature) -> Self {
        const BASELINE_AC: f64 = 15.0;
        
        let mut action_dpr: f64 = 0.0;
        let mut bonus_action_dpr: f64 = 0.0;
        let mut attack_count = 0;
        let mut has_bonus_attacks = false;
        
        for action in &creature.actions {
            if let crate::model::Action::Atk(attack) = action {
                attack_count += 1;
                
                // Calculate base damage per hit
                let damage_per_hit = match &attack.dpr {
                    crate::model::DiceFormula::Value(v) => *v,
                    crate::model::DiceFormula::Expr(e) => dice::parse_average(e),
                };
                
                // Calculate to_hit bonus
                let to_hit_bonus = match &attack.to_hit {
                    crate::model::DiceFormula::Value(v) => *v,
                    crate::model::DiceFormula::Expr(e) => dice::parse_average(e),
                };
                
                // Calculate hit probability against baseline AC
                let needed_roll = BASELINE_AC - to_hit_bonus;
                let hit_chance = if needed_roll <= 1.0 {
                    0.95 // Auto-hit except on nat 1
                } else if needed_roll >= 20.0 {
                    0.05 // Only nat 20
                } else {
                    (21.0 - needed_roll) / 20.0
                };
                
                // Account for number of targets
                let num_targets = attack.targets.max(1) as f64;
                
                // Calculate expected DPR for this action
                let expected_dpr = damage_per_hit * hit_chance * num_targets;
                
                // Categorize by action cost
                let is_bonus_action = attack.action_slot == Some(1);
                
                if is_bonus_action {
                    bonus_action_dpr = bonus_action_dpr.max(expected_dpr);
                    has_bonus_attacks = true;
                } else {
                    action_dpr = action_dpr.max(expected_dpr);
                }
            }
        }
        
        let total_dpr = action_dpr + bonus_action_dpr;
        
        Self {
            creature_id: creature.id.clone(),
            ac: creature.ac as f64,
            dpr: total_dpr,
            hit_probability: if attack_count > 0 {
                // Average hit probability across all attacks
                let total_hit_prob = creature.actions.iter()
                    .filter_map(|action| {
                        if let crate::model::Action::Atk(attack) = action {
                            let to_hit_bonus = match &attack.to_hit {
                                crate::model::DiceFormula::Value(v) => *v,
                                crate::model::DiceFormula::Expr(e) => dice::parse_average(e),
                            };
                            let needed_roll = BASELINE_AC - to_hit_bonus;
                            Some(if needed_roll <= 1.0 {
                                0.95
                            } else if needed_roll >= 20.0 {
                                0.05
                            } else {
                                (21.0 - needed_roll) / 20.0
                            })
                        } else {
                            None
                        }
                    })
                    .sum::<f64>();
                total_hit_prob / attack_count as f64
            } else {
                0.0
            },
            best_action_dpr: action_dpr,
            best_bonus_action_dpr: bonus_action_dpr,
            total_dpr,
            attack_count,
            has_bonus_attacks,
        }
    }
    
    /// Get estimated DPR against a specific target AC
    pub fn get_dpr_vs_ac(&self, target_ac: f64) -> f64 {
        if self.attack_count == 0 {
            return 0.0;
        }
        
        // Adjust DPR based on target AC difference from baseline
        let ac_diff = target_ac - 15.0;
        let hit_chance_adjustment = if ac_diff <= 0.0 {
            1.0 // No penalty for easier targets
        } else {
            // Each point of AC above baseline reduces hit chance by 5%
            (1.0 - (ac_diff * 0.05)).max(0.05) // Minimum 5% hit chance
        };
        
        self.total_dpr * hit_chance_adjustment
    }
}

/// Cache for combatant statistics to avoid repeated calculations
#[derive(Debug, Clone)]
pub struct CombatStatsCache {
    /// Cached statistics by creature ID
    stats_cache: HashMap<String, CombatantStats>,
    
    /// Cache invalidation flag - set to true when combatants change
    dirty: bool,
}

impl CombatStatsCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            stats_cache: HashMap::new(),
            dirty: false,
        }
    }
    
    /// Get or calculate combat statistics for a combatant
    pub fn get_stats(&mut self, combatant: &Combattant) -> &CombatantStats {
        let creature_id = &combatant.creature.id;
        
        if self.dirty {
            self.stats_cache.clear();
            self.dirty = false;
        }

        self.stats_cache.entry(creature_id.clone()).or_insert_with(|| {
            CombatantStats::calculate(&combatant.creature)
        })
    }
    
    /// Get cached stats for a creature ID, returns None if not cached
    pub fn get_stats_by_id(&self, creature_id: &str) -> Option<&CombatantStats> {
        self.stats_cache.get(creature_id)
    }
    
    /// Pre-calculate stats for all combatants
    pub fn precalculate_for_combatants(&mut self, combatants: &[Combattant]) {
        for combatant in combatants {
            let creature_id = &combatant.creature.id;
            if !self.stats_cache.contains_key(creature_id) || self.dirty {
                let stats = CombatantStats::calculate(&combatant.creature);
                self.stats_cache.insert(creature_id.clone(), stats);
            }
        }
        self.dirty = false;
    }
    
    /// Mark cache as dirty (needs recalculation)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    
    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.stats_cache.clear();
        self.dirty = false;
    }
    
    /// Remove stats for a specific creature (e.g., when creature dies)
    pub fn remove_creature(&mut self, creature_id: &str) {
        self.stats_cache.remove(creature_id);
    }
    
    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            cached_creatures: self.stats_cache.len(),
            is_dirty: self.dirty,
        }
    }
}

/// Statistics about the cache itself
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of creature types cached
    pub cached_creatures: usize,
    
    /// Whether cache needs recalculation
    pub is_dirty: bool,
}

impl Default for CombatStatsCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Action, AtkAction, DiceFormula, Frequency, ActionCondition};
    
    fn create_test_creature(id: &str, ac: f64, damage: f64, to_hit: f64) -> Creature {
        Creature {
            id: id.to_string(),
            name: id.to_string(),
            count: 1.0,
            hp: 30,
            ac: ac as u32,
            speed_fly: None,
            save_bonus: 0.0,
            str_save_bonus: None,
            dex_save_bonus: None,
            con_save_bonus: None,
            int_save_bonus: None,
            wis_save_bonus: None,
            cha_save_bonus: None,
            con_save_advantage: None,
            save_advantage: None,
            initiative_bonus: DiceFormula::Value(0.0),
            initiative_advantage: false,
            actions: vec![
                Action::Atk(AtkAction {
                    id: "attack".to_string(),
                    name: "Attack".to_string(),
                    action_slot: None,
                    cost: vec![],
                    requirements: vec![],
                    tags: vec![],
                    freq: Frequency::Static("at will".to_string()),
                    condition: ActionCondition::Default,
                    targets: 1,
                    dpr: DiceFormula::Value(damage),
                    to_hit: DiceFormula::Value(to_hit),
                    target: crate::enums::EnemyTarget::EnemyWithLeastHP,
                    use_saves: None,
                    half_on_save: None,
                    rider_effect: None,
                })
            ],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "monster".to_string(),
        }
    }
    
    #[test]
    fn test_combatant_stats_calculation() {
        let creature = create_test_creature("goblin", 15.0, 10.0, 5.0);
        let stats = CombatantStats::calculate(&creature);
        
        assert_eq!(stats.creature_id, "goblin");
        assert_eq!(stats.ac, 15.0);
        assert!(stats.dpr > 0.0);
        assert_eq!(stats.attack_count, 1);
        assert!(!stats.has_bonus_attacks);
    }
    
    #[test]
    fn test_dpr_vs_ac_adjustment() {
        let creature = create_test_creature("goblin", 15.0, 10.0, 5.0);
        let stats = CombatantStats::calculate(&creature);
        
        // Against baseline AC (15), should be full DPR
        let baseline_dpr = stats.get_dpr_vs_ac(15.0);
        let higher_ac_dpr = stats.get_dpr_vs_ac(20.0);
        let lower_ac_dpr = stats.get_dpr_vs_ac(10.0);
        
        assert_eq!(baseline_dpr, stats.total_dpr);
        assert!(higher_ac_dpr < baseline_dpr);
        assert_eq!(lower_ac_dpr, stats.total_dpr); // No penalty for easier targets
    }

    #[test]
    fn test_cache_operations() {
        let mut cache = CombatStatsCache::new();
        let creature = create_test_creature("goblin", 15.0, 10.0, 5.0);
        let combatant = Combattant { team: 0,
            id: "goblin1".to_string(),
            creature: std::sync::Arc::new(creature.clone()),
            initiative: 10.0,
            initial_state: crate::model::CreatureState::default(),
            final_state: crate::model::CreatureState::default(),
            actions: vec![],
        };
        
        // Initially empty
        assert_eq!(cache.get_cache_stats().cached_creatures, 0);
        
        // Get stats should calculate and cache
        let stats_cloned = cache.get_stats(&combatant).clone();
        assert_eq!(stats_cloned.creature_id, "goblin");
        assert_eq!(cache.get_cache_stats().cached_creatures, 1);
        
        // Second get should use cache
        let stats2_cloned = cache.get_stats(&combatant).clone();
        assert_eq!(stats_cloned.creature_id, stats2_cloned.creature_id);
        assert_eq!(cache.get_cache_stats().cached_creatures, 1);
        
        // Mark dirty and get again should recalculate
        cache.mark_dirty();
        assert!(cache.get_cache_stats().is_dirty);
        let stats3_cloned = cache.get_stats(&combatant).clone();
        assert!(!cache.get_cache_stats().is_dirty);
        assert_eq!(stats_cloned.creature_id, stats3_cloned.creature_id);
    }
    
    #[test]
    fn test_precalculate_for_combatants() {
        let mut cache = CombatStatsCache::new();

        let creature1 = create_test_creature("goblin", 15.0, 10.0, 5.0);
        let creature2 = create_test_creature("orc", 12.0, 15.0, 3.0);

        let combatants = vec![
            Combattant { team: 0,
                id: "goblin1".to_string(),
                creature: std::sync::Arc::new(creature1),
                initiative: 10.0,
                initial_state: crate::model::CreatureState::default(),
                final_state: crate::model::CreatureState::default(),
                actions: vec![],
            },
            Combattant { team: 0,
                id: "orc1".to_string(),
                creature: std::sync::Arc::new(creature2),
                initiative: 5.0,
                initial_state: crate::model::CreatureState::default(),
                final_state: crate::model::CreatureState::default(),
                actions: vec![],
            },
        ];
        
        cache.precalculate_for_combatants(&combatants);
        
        assert_eq!(cache.get_cache_stats().cached_creatures, 2);
        assert!(!cache.get_cache_stats().is_dirty);
        
        // Should be able to get stats without recalculation
        assert!(cache.get_stats_by_id("goblin").is_some());
        assert!(cache.get_stats_by_id("orc").is_some());
    }
}