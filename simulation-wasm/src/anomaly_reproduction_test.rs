#[cfg(test)]
mod tests {
    use crate::model::*;
    use crate::safe_aggregation::aggregate_results_safe;
    use crate::run_single_event_driven_simulation;
    use crate::enums::EnemyTarget;

    fn create_fighter(id: &str, initiative: f64) -> Creature {
        Creature {
            id: id.to_string(),
            name: format!("Fighter {}", id),
            hp: 100,
            ac: 10, // Easy to hit
            count: 1.0,
            mode: "player".to_string(),
            arrival: None,
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
            initiative_bonus: DiceFormula::Value(initiative),
            initiative_advantage: false,
            actions: vec![
                Action::Atk(AtkAction {
                    id: format!("atk-{}", id),
                    name: "Attack".to_string(),
                    action_slot: None,
                    cost: vec![],
                    requirements: vec![],
                    tags: vec![],
                    freq: Frequency::Static("at will".to_string()),
                    condition: ActionCondition::Default,
                    targets: 1,
                    dpr: DiceFormula::Value(20.0), // High damage
                    target: EnemyTarget::EnemyWithLeastHP,
                    to_hit: DiceFormula::Value(100.0), // Guaranteed hit
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
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
        }
    }

    #[test]
    fn test_round_1_anomaly_reproduction() {
        let p1 = create_fighter("p1", 20.0); // Fast
        let p2 = create_fighter("p2", 0.0);  // Slow
        
        // Setup as an encounter where p1 is player and p2 is monster
        let mut monster = p2.clone();
        monster.mode = "monster".to_string();
        
        let players = vec![p1];
        let timeline = vec![TimelineStep::Combat(Encounter {
            monsters: vec![monster],
            target_role: TargetRole::Standard,
            players_surprised: None,
            monsters_surprised: None,
            players_precast: None,
            monsters_precast: None,
        })];

        let iterations = 10;
        let mut results = Vec::new();

        for _i in 0..iterations {
            let (res, _) = run_single_event_driven_simulation(&players, &timeline, true);
            results.push(res);
        }

        let aggregated = aggregate_results_safe(&results).expect("Aggregation failed");
        
        assert!(!aggregated.is_empty(), "Aggregated rounds should not be empty");
        
        let round1 = &aggregated[0];
        let aggregated_p1 = round1.team1.iter().find(|c| c.creature.id == "p1").expect("P1 not found");
        let aggregated_p2 = round1.team2.iter().find(|c| c.creature.id == "p2").expect("P2 not found");

        println!("Round 1 Aggregated P1 HP: {}/{}", aggregated_p1.final_state.current_hp, aggregated_p1.creature.hp);
        println!("Round 1 Aggregated P2 HP: {}/{}", aggregated_p2.final_state.current_hp, aggregated_p2.creature.hp);

        // P1 should have taken damage from P2 in Round 1
        assert!(aggregated_p1.final_state.current_hp < aggregated_p1.creature.hp, 
            "P1 should have taken damage in Round 1, but HP is {}", aggregated_p1.final_state.current_hp);
        
        // P2 should also have taken damage from P1 in Round 1
        assert!(aggregated_p2.final_state.current_hp < aggregated_p2.creature.hp,
            "P2 should have taken damage in Round 1, but HP is {}", aggregated_p2.final_state.current_hp);
    }
}
