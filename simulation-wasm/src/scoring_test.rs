#[cfg(test)]
mod tests {
    use crate::model::*;
    use crate::aggregation::calculate_efficiency_score;
    use crate::events::Event;
    use std::collections::HashMap;

    fn create_mock_combattant(id: &str, current_hp: u32) -> Combattant {
        let creature = Creature {
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

        // Expected: 100 * (1 + (15-10)/20) * 1 = 100 * 1.25 * 1 = 125
        let score = fighter.max_survivability_score();
        assert_eq!(score, 125.0, "Fighter with AC 15 should have EHP of 125");
    }

    #[test]
    fn test_survivability_score_barbarian_with_rage() {
        // Barbarian: 100 HP, AC 15, has Rage class resource
        let mut resources = HashMap::new();
        resources.insert("Rage".to_string(), 2); // 2 Rage uses per day

        let barbarian = Creature {
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

        // Expected: 100 * (1 + (15-10)/20) * 2 = 100 * 1.25 * 2 = 250
        let score = barbarian.max_survivability_score();
        assert_eq!(score, 250.0, "Barbarian with Rage should have 2x EHP");
    }

    #[test]
    fn test_survivability_score_ac_10_baseline() {
        // Wizard: 40 HP, AC 10 (no armor)
        let wizard = Creature {
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

        // Expected: 40 * (1 + (10-10)/20) * 1 = 40 * 1.0 * 1 = 40
        let score = wizard.max_survivability_score();
        assert_eq!(score, 40.0, "Wizard with AC 10 should have EHP equal to HP");
    }

    #[test]
    fn test_survivability_score_high_ac() {
        // Paladin: 120 HP, AC 20 (plate armor)
        let paladin = Creature {
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

        // Expected: 120 * (1 + (20-10)/20) * 1 = 120 * 1.5 * 1 = 180
        let score = paladin.max_survivability_score();
        assert_eq!(score, 180.0, "Paladin with AC 20 should have 1.5x EHP");
    }

    #[test]
    fn test_current_survivability_with_rage_active() {
        // Barbarian with Rage currently active
        let mut resources = HashMap::new();
        resources.insert("Rage".to_string(), 2);

        let barbarian = Creature {
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

        // Expected: 80 * (1 + (15-10)/20) * 2 = 80 * 1.25 * 2 = 200
        let score = combatant.current_survivability_score();
        assert_eq!(score, 200.0, "Barbarian with Rage active should have 2x EHP on current HP");
    }

    #[test]
    fn test_current_survivability_rage_expired() {
        // Barbarian with Rage expired (no buff, no resources)
        let mut resources = HashMap::new();
        resources.insert("Rage".to_string(), 0); // Used all Rage

        let barbarian = Creature {
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

        // Expected: 80 * (1 + (15-10)/20) * 1 = 80 * 1.25 * 1 = 100
        // (No Rage multiplier since it's not active)
        let score = combatant.current_survivability_score();
        assert_eq!(score, 100.0, "Barbarian without Rage active should have normal EHP");
    }
}