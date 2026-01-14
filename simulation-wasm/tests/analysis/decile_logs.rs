use simulation_wasm::model::*;
use simulation_wasm::decile_analysis::run_encounter_analysis_with_logs;
use simulation_wasm::events::Event;
use std::collections::HashMap;

#[test]
fn test_cumulative_log_sorting() {
    let mut runs = Vec::new();
    
    // Create 100 dummy runs
    for i in 0..100 {
        let mut encounters = Vec::new();
        let mut events = Vec::new();
        
        // Encounter 1
        let hp1 = i as u32; // Higher i means better E1
        encounters.push(EncounterResult {
            stats: HashMap::new(),
            rounds: vec![Round {
                team1: vec![dummy_combatant("p1", hp1)],
                team2: vec![],
            }],
            target_role: TargetRole::Standard,
        });
        events.push(Event::EncounterStarted { combatant_ids: vec!["p1".to_string()] });
        events.push(Event::Custom { event_type: "E1".to_string(), data: HashMap::new(), source_id: "p1".to_string() });
        events.push(Event::EncounterEnded { winner: Some("team1".to_string()), reason: "Victory".to_string() });

        // Encounter 2
        let hp2 = (100 - i) as u32; // Higher i means WORSE E2
        encounters.push(EncounterResult {
            stats: HashMap::new(),
            rounds: vec![Round {
                team1: vec![dummy_combatant("p1", hp2)],
                team2: vec![],
            }],
            target_role: TargetRole::Standard,
        });
        events.push(Event::EncounterStarted { combatant_ids: vec!["p1".to_string()] });
        events.push(Event::Custom { event_type: "E2".to_string(), data: HashMap::new(), source_id: "p1".to_string() });
        events.push(Event::EncounterEnded { winner: Some("team1".to_string()), reason: "Victory".to_string() });

        runs.push(SimulationRun {
            result: SimulationResult {
                encounters,
                score: Some(i as f64), // Global score (matches E1)
                num_combat_encounters: 2,
                seed: i as u64,
            },
            events,
        });
    }

    // Analysis for E1 (cumulative E1)
    // Runs are sorted by E1 score (hp1 = i). 
    // Decile 1 (5%) should be run with i ~ 5.
    let output1 = run_encounter_analysis_with_logs(&mut runs, 0, "Enc 1", 1, 0);
    assert_eq!(output1.decile_logs.len(), 11);
    
    // Check slicing: Log for E1 should only contain E1 events
    for log in &output1.decile_logs {
        assert!(log.iter().any(|e| matches!(e, Event::Custom { event_type, .. } if event_type == "E1")));
        assert!(!log.iter().any(|e| matches!(e, Event::Custom { event_type, .. } if event_type == "E2")));
    }

    // Analysis for E2 (cumulative E1 + E2)
    // Cumulative score = hp1 + hp2 = i + (100 - i) = 100 (Constant!)
    // If scores are constant, sorting order might be stable or random.
    // Let's make E2 slightly different to ensure sorting.
    for (i, run) in runs.iter_mut().enumerate().take(100) {
        let hp2 = (100 - i) as u32;
        run.result.encounters[1].rounds[0].team1[0].final_state.current_hp = hp2;
    }
    
    let output2 = run_encounter_analysis_with_logs(&mut runs, 1, "Enc 2", 1, 0);
    
    // Check slicing: Log for E2 should only contain E2 events
    for log in &output2.decile_logs {
        assert!(!log.iter().any(|e| matches!(e, Event::Custom { event_type, .. } if event_type == "E1")));
        assert!(log.iter().any(|e| matches!(e, Event::Custom { event_type, .. } if event_type == "E2")));
    }
}

fn dummy_combatant(id: &str, hp: u32) -> Combattant {
    Combattant {
        id: id.to_string(),
        team: 0,
        creature: std::sync::Arc::new(Creature {
            id: id.to_string(),
            arrival: None,
            mode: "player".to_string(), magic_items: vec![], max_arcane_ward_hp: None, initial_buffs: vec![],
            name: id.to_string(),
            count: 1.0,
            hp: 100,
            ac: 10,
            speed_fly: None,
            save_bonus: 0.0,
            str_save_bonus: None, dex_save_bonus: None, con_save_bonus: None,
            int_save_bonus: None, wis_save_bonus: None, cha_save_bonus: None,
            con_save_advantage: None, save_advantage: None,
            initiative_bonus: DiceFormula::Value(0.0),
            initiative_advantage: false,
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        }),
        initiative: 10.0,
        initial_state: CreatureState::default(),
        final_state: CreatureState {
            current_hp: hp,
            ..CreatureState::default()
        },
        actions: vec![],
    }
}
