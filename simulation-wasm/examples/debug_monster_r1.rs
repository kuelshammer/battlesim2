use simulation_wasm::enums::*;
use simulation_wasm::execution;
use simulation_wasm::model::*; // This is run_monte_carlo from simulation.rs

fn create_fighter(id: &str, name: &str, init_bonus: f64, init_advantage: bool) -> Creature {
    Creature {
        id: id.to_string(),
        name: name.to_string(),
        count: 1.0,
        hp: 100.0,
        ac: 10.0,
        speed_fly: None,
        save_bonus: 3.0,
        str_save_bonus: None,
        dex_save_bonus: None,
        con_save_bonus: Some(3.0),
        int_save_bonus: None,
        wis_save_bonus: None,
        cha_save_bonus: None,
        con_save_advantage: None,
        save_advantage: None,
        initiative_bonus: model::DiceFormula::Value(init_bonus),
        initiative_advantage: init_advantage,
        actions: vec![
            Action::Atk(AtkAction {
                id: format!("{}_atk", id),
                name: "Basic Attack".to_string(),
                action_slot: Some(0), // Action
                cost: vec![],         // Add missing fields
                requirements: vec![], // Add missing fields
                tags: vec![],         // Add missing fields
                freq: Frequency::Static("at will".to_string()),
                condition: ActionCondition::Default,
                targets: 2,                       // Extra Attack
                dpr: DiceFormula::Value(10.0),    // Fixed 10 damage
                to_hit: DiceFormula::Value(10.0), // +10 to hit
                target: EnemyTarget::EnemyWithLeastHP,
                use_saves: None,
                half_on_save: None,
                rider_effect: None,
            }),
            // Action Surge
            Action::Atk(AtkAction {
                id: format!("{}_surge", id),
                name: "Action Surge Attack".to_string(),
                action_slot: Some(5), // ActionSlots['Other 1'] (for Action Surge)
                cost: vec![],         // Add missing fields
                requirements: vec![], // Add missing fields
                tags: vec![],         // Add missing fields
                freq: Frequency::Static("1/fight".to_string()), // Usable once per fight
                condition: ActionCondition::Default, // Condition not fully implemented yet
                targets: 2,           // Extra Attack again
                dpr: DiceFormula::Value(10.0), // Fixed 10 damage
                to_hit: DiceFormula::Value(10.0), // +10 to hit
                target: EnemyTarget::EnemyWithLeastHP,
                use_saves: None,
                half_on_save: None,
                rider_effect: None,
            }),
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
fn main() {
    let fast_fighter = create_fighter("fast", "Fast Fighter", 10.0, false); // +10 init, no advantage
    let slow_fighter_creature = create_fighter("slow", "Slow Fighter", 0.0, false); // +0 init, no advantage

    let encounter = Encounter {
        monsters: vec![slow_fighter_creature], // Use the created creature
        players_surprised: Some(false),
        monsters_surprised: Some(false),
        short_rest: Some(false),
        players_precast: None,
        monsters_precast: None,
    };

    let players = vec![fast_fighter];
    let iterations = 1; // Single iteration for debugging

    println!("Running {} simulations...", iterations);
    let results = simulation::run_monte_carlo(&players, &[encounter], iterations);

    // This test is for debugging purposes, so we don't need extensive result processing
    // Just a single simulation trace.
    let encounter_result = &results[0][0];
    println!("\n--- Encounter Result Trace ---");
    for (i, round) in encounter_result.rounds.iter().enumerate() {
        println!("Round {}:", i + 1);
        for c in &round.team1 {
            println!(
                "  Team 1 - {}: HP {:.1}",
                c.creature.name, c.final_state.current_hp
            );
            println!(
                "    Actions: {:?}",
                c.actions
                    .iter()
                    .map(|ca| ca.action.base().name.clone())
                    .collect::<Vec<_>>()
            );
        }
        for c in &round.team2 {
            println!(
                "  Team 2 - {}: HP {:.1}",
                c.creature.name, c.final_state.current_hp
            );
            println!(
                "    Actions: {:?}",
                c.actions
                    .iter()
                    .map(|ca| ca.action.base().name.clone())
                    .collect::<Vec<_>>()
            );
        }
    }
}
