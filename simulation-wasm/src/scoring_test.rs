#[cfg(test)]
mod tests {
    use crate::model::*;
    use crate::aggregation::calculate_efficiency_score;
    use crate::events::Event;
    use std::collections::{HashMap, HashSet};

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
            creature,
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
            }],
            score: None,
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
            }],
            score: None,
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
}