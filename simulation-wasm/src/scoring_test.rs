#[cfg(test)]
mod tests {
    use crate::model::*;
    use crate::aggregation::calculate_efficiency_score;
    use crate::events::Event;
    use std::collections::HashMap;

    fn create_mock_combattant(id: &str, current_hp: u32) -> Combattant {
        let creature = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: id.to_string(),
            name: id.to_string(),
            count: 1.0,
            hp: 100,
            ac: 15,
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
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "monster".to_string(),
        };

        Combattant {
            team: 0,
            id: id.to_string(),
            initiative: 10.0,
            creature: std::sync::Arc::new(creature),
            initial_state: CreatureState::default(),
            final_state: CreatureState {
                current_hp,
                ..CreatureState::default()
            },
            actions: vec![],
        }
    }

    #[test]
    fn test_efficiency_scoring_logic() {
        // Run A: High HP, High Resource Spend (1st level spell)
        // Run B: Slightly Lower HP, Low Resource Spend (Cantrips only)
        
        let run_a_result = SimulationResult {
            encounters: vec![EncounterResult {
                stats: HashMap::new(),
                rounds: vec![Round {
                    team1: vec![create_mock_combattant("p1", 100)],
                    team2: vec![],
                }],
                target_role: TargetRole::Standard,
            }],
            score: None,
            num_combat_encounters: 1,
            seed: 12345,
        };
        let run_a_events = vec![
            Event::SpellCast { caster_id: "p1".to_string(), spell_id: "cure_wounds".to_string(), spell_level: 1 }
        ];

        let run_b_result = SimulationResult {
            encounters: vec![EncounterResult {
                stats: HashMap::new(),
                rounds: vec![Round {
                    team1: vec![create_mock_combattant("p1", 90)], // Lower HP
                    team2: vec![],
                }],
                target_role: TargetRole::Standard,
            }],
            score: None,
            num_combat_encounters: 1,
            seed: 12346,
        };
        let run_b_events = vec![]; // No resources spent

        let score_a = calculate_efficiency_score(&run_a_result, &run_a_events);
        let score_b = calculate_efficiency_score(&run_b_result, &run_b_events);

        // Score A: 1,000,000 + 100 - 15 (Lvl 1 Spell) = 1,000,085
        // Score B: 1,000,000 + 90 - 0 = 1,000,090
        
        println!("Score A (Spell): {}", score_a);
        println!("Score B (Low HP): {}", score_b);
        
        assert!(score_b > score_a, "Run B (more efficient) should score higher than Run A despite lower HP");
    }

    #[test]
    fn test_hit_die_penalty() {
        let result = SimulationResult {
            encounters: vec![EncounterResult {
                stats: HashMap::new(),
                rounds: vec![Round {
                    team1: vec![create_mock_combattant("p1", 100)],
                    team2: vec![],
                }],
                target_role: TargetRole::Standard,
            }],
            score: None,
            num_combat_encounters: 1,
            seed: 12347,
        };
        let events = vec![
            Event::ResourceConsumed {
                unit_id: "p1".to_string(),
                resource_type: "HitDice".to_string(),
                amount: 2.0,
            }
        ];

        let score = calculate_efficiency_score(&result, &events);
        // Base: 1,000,000 + 100 - (2 * 15) = 1,000,070
        assert_eq!(score, 1_000_070.0);
    }

    #[test]
    fn test_survivability_score_basic() {
        // Fighter: 100 HP, AC 15, no Rage
        let fighter = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "fighter".to_string(),
            name: "Fighter".to_string(),
            count: 1.0,
            hp: 100,
            ac: 15,
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
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "player".to_string(),
        };

        // Expected: AC 15 vs +5 → 55% hit chance → 100 / 0.55 = 181.8 EHP -> Rounded to 182
        let score = fighter.max_survivability_score();
        assert_eq!(score, 182.0, "Fighter with AC 15 should have EHP of 182, got {}", score);
    }

    #[test]
    fn test_survivability_score_barbarian_with_rage() {
        // Barbarian: 100 HP, AC 15, has Rage class resource
        let mut resources = HashMap::new();
        resources.insert("Rage".to_string(), 2); // 2 Rage uses per day

        let barbarian = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "barbarian".to_string(),
            name: "Barbarian".to_string(),
            count: 1.0,
            hp: 100,
            ac: 15,
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
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: Some(resources),
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "player".to_string(),
        };

        // Expected: AC 15 vs +5 → 55% hit chance → 100 / 0.55 × 2 = 363.6 EHP -> Rounded to 364
        let score = barbarian.max_survivability_score();
        assert_eq!(score, 364.0, "Barbarian with Rage should have 364 EHP, got {}", score);
    }

    #[test]
    fn test_survivability_score_ac_10_baseline() {
        // Wizard: 40 HP, AC 10 (no armor)
        let wizard = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "wizard".to_string(),
            name: "Wizard".to_string(),
            count: 1.0,
            hp: 40,
            ac: 10,
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
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "player".to_string(),
        };

        // Expected: AC 10 vs +5 → 80% hit chance → 40 / 0.8 = 50 EHP
        let score = wizard.max_survivability_score();
        assert_eq!(score, 50.0, "Wizard with AC 10 should have EHP of 50");
    }

    #[test]
    fn test_survivability_score_high_ac() {
        // Paladin: 120 HP, AC 20 (plate armor)
        let paladin = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "paladin".to_string(),
            name: "Paladin".to_string(),
            count: 1.0,
            hp: 120,
            ac: 20,
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
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "player".to_string(),
        };

        // Expected: AC 20 vs +5 → 30% hit chance → 120 / 0.3 = 400 EHP
        let score = paladin.max_survivability_score();
        assert_eq!(score, 400.0, "Paladin with AC 20 should have EHP of 400");
    }

    #[test]
    fn test_current_survivability_with_rage_active() {
        // Barbarian with Rage currently active
        let mut resources = HashMap::new();
        resources.insert("Rage".to_string(), 2);

        let barbarian = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "barbarian".to_string(),
            name: "Barbarian".to_string(),
            count: 1.0,
            hp: 100,
            ac: 15,
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
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: Some(resources),
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "player".to_string(),
        };

        // Create a combatant with Rage active (current HP reduced to 80)
        let mut rage_buffs = HashMap::new();
        rage_buffs.insert("Rage".to_string(), Buff {
            display_name: Some("Rage".to_string()),
            duration: crate::enums::BuffDuration::OneRound,
            ac: None,
            to_hit: None,
            damage: None,
            damage_reduction: None,
            damage_multiplier: None,
            damage_taken_multiplier: Some(0.5), // Barbarians take half damage from physical attacks
            dc: None,
            save: None,
            condition: None,
            magnitude: None,
            source: Some("barbarian".to_string()),
            concentration: false,
            triggers: vec![],
        });

        let mut rage_resources = HashMap::new();
        rage_resources.insert("Rage".to_string(), 2.0);
        let rage_ledger = SerializableResourceLedger {
            current: rage_resources,
            max: HashMap::new(),
        };

        let combatant = Combattant {
            team: 0,
            id: "barbarian".to_string(),
            initiative: 10.0,
            creature: std::sync::Arc::new(barbarian),
            initial_state: CreatureState::default(),
            final_state: CreatureState {
                current_hp: 80, // Took some damage
                buffs: rage_buffs,
                resources: rage_ledger,
                ..CreatureState::default()
            },
            actions: vec![],
        };

        // Expected: 80 HP / 0.55 hit chance × 2 (Rage) = 290.9 EHP -> Rounded to 291
        let score = combatant.current_survivability_score();
        assert_eq!(score, 291.0, "Barbarian with Rage active should have 291 EHP on current HP, got {}", score);
    }

    #[test]
    fn test_current_survivability_rage_expired() {
        // Barbarian with Rage expired (no buff, no resources)
        let mut resources = HashMap::new();
        resources.insert("Rage".to_string(), 0); // Used all Rage

        let barbarian = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "barbarian".to_string(),
            name: "Barbarian".to_string(),
            count: 1.0,
            hp: 100,
            ac: 15,
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
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: Some(resources),
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "player".to_string(),
        };

        let combatant = Combattant {
            team: 0,
            id: "barbarian".to_string(),
            initiative: 10.0,
            creature: std::sync::Arc::new(barbarian),
            initial_state: CreatureState::default(),
            final_state: CreatureState {
                current_hp: 80,
                buffs: HashMap::new(), // No Rage buff
                resources: SerializableResourceLedger {
                    current: HashMap::new(), // Rage at 0
                    max: HashMap::new(),
                },
                ..CreatureState::default()
            },
            actions: vec![],
        };

        // Expected: 80 HP / 0.55 hit chance × 1 (no Rage) = 145.5 EHP -> Rounded to 145
        // (No Rage multiplier since it's not active)
        let score = combatant.current_survivability_score();
        assert_eq!(score, 145.0, "Barbarian without Rage active should have 145 EHP, got {}", score);
    }

    #[test]
    fn test_survivability_edge_case_nat20_only() {
        // Monk with high AC (26) - only hit on nat 20
        let monk = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "monk".to_string(),
            name: "Monk".to_string(),
            count: 1.0,
            hp: 80,
            ac: 26,
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
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "player".to_string(),
        };

        // Expected: AC 26 vs +5 → need 21+ → only nat 20 hits (5%) → 80 / 0.05 = 1600 EHP
        let score = monk.max_survivability_score();
        assert_eq!(score, 1600.0, "Monk with AC 26 should have EHP of 1600 (nat 20 only)");
    }

    #[test]
    fn test_survivability_edge_case_nat1_only_misses() {
        // Commoner with negative AC scenario (AC 4)
        // Hit on anything but nat 1
        let commoner = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "commoner".to_string(),
            name: "Commoner".to_string(),
            count: 1.0,
            hp: 20,
            ac: 4,
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
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "player".to_string(),
        };

        // Expected: AC 4 vs +5 → need -1 or better → hit on 2-20 (95%) → 20 / 0.95 = 21.05 EHP -> Rounded to 21
        let score = commoner.max_survivability_score();
        assert_eq!(score, 21.0, "Commoner with AC 4 should have EHP of 21, got {}", score);
    }

    #[test]
    fn test_survivability_vs_high_attack_bonus() {
        // Fighter vs high-level monster (+10 attack)
        let fighter = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "fighter".to_string(),
            name: "Fighter".to_string(),
            count: 1.0,
            hp: 100,
            ac: 15,
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
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "player".to_string(),
        };

        // Expected: AC 15 vs +10 → need 5+ → hit on 5-20 (80%) → 100 / 0.8 = 125 EHP
        let score = fighter.max_survivability_score_vs_attack(10);
        assert_eq!(score, 125.0, "Fighter AC 15 vs +10 attack should have EHP of 125");
    }
}