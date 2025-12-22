use simulation_wasm::model::*;
use simulation_wasm::enums::*;
use simulation_wasm::auto_balancer::AutoBalancer;

#[test]
fn test_black_dragon_auto_balance() {
    // 1. Create 2 Level 10 Fighters (Approx 100 HP each)
    let fighter = Creature {
        id: "fighter".to_string(),
        name: "Fighter".to_string(),
        hp: 100,
        ac: 18,
        count: 2.0,
        arrival: None,
        mode: "player".to_string(),
        speed_fly: None,
        save_bonus: 3.0,
        str_save_bonus: None,
        dex_save_bonus: None,
        con_save_bonus: None,
        int_save_bonus: None,
        wis_save_bonus: None,
        cha_save_bonus: None,
        con_save_advantage: None,
        save_advantage: None,
        initiative_bonus: DiceFormula::Value(3.0),
        initiative_advantage: false,
        actions: vec![
            Action::Atk(AtkAction {
                id: "multiattack".to_string(),
                name: "Multiattack (2x)".to_string(),
                action_slot: None,
                cost: vec![],
                requirements: vec![],
                tags: vec![],
                freq: Frequency::Static("at will".to_string()),
                condition: ActionCondition::Default,
                targets: 2,
                dpr: DiceFormula::Value(12.0),
                to_hit: DiceFormula::Value(9.0),
                target: EnemyTarget::EnemyWithLeastHP,
                use_saves: Some(false),
                half_on_save: None,
                rider_effect: None,
            })
        ],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
    };

    // 2. Create Adult Black Dragon (Broken fight for 2 Fighters)
    let dragon = Creature {
        id: "dragon".to_string(),
        name: "Adult Black Dragon".to_string(),
        hp: 195,
        ac: 19,
        count: 1.0,
        arrival: None,
        mode: "monster".to_string(),
        speed_fly: Some(80.0),
        save_bonus: 5.0,
        str_save_bonus: None,
        dex_save_bonus: None,
        con_save_bonus: None,
        int_save_bonus: None,
        wis_save_bonus: None,
        cha_save_bonus: None,
        con_save_advantage: None,
        save_advantage: None,
        initiative_bonus: DiceFormula::Value(2.0),
        initiative_advantage: false,
        actions: vec![
            Action::Template(TemplateAction {
                id: "breath".to_string(),
                name: "Acid Breath".to_string(),
                action_slot: None,
                cost: vec![],
                requirements: vec![],
                tags: vec![],
                freq: Frequency::Recharge { reset: "recharge".to_string(), cooldown_rounds: 5 },
                condition: ActionCondition::Default,
                targets: 2,
                template_options: TemplateOptions {
                    template_name: "Line".to_string(),
                    target: None,
                    save_dc: Some(18.0),
                    amount: Some(DiceFormula::Value(54.0)), // 12d8
                },
            }),
            Action::Atk(AtkAction {
                id: "bite".to_string(),
                name: "Bite".to_string(),
                action_slot: None,
                cost: vec![],
                requirements: vec![],
                tags: vec![],
                freq: Frequency::Static("at will".to_string()),
                condition: ActionCondition::Default,
                targets: 1,
                dpr: DiceFormula::Value(15.0),
                to_hit: DiceFormula::Value(11.0),
                target: EnemyTarget::EnemyWithMostHP,
                use_saves: Some(false),
                half_on_save: None,
                rider_effect: None,
            })
        ],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
    };

    let mut balancer = AutoBalancer::new();
    balancer.target_simulations = 251; // Much faster for testing
    balancer.max_iterations = 15;
    
    let timeline = vec![TimelineStep::Combat(Encounter {
        monsters: vec![dragon.clone()],
        players_surprised: None,
        monsters_surprised: None,
        players_precast: None,
        monsters_precast: None,
        target_role: TargetRole::Boss,
    })];

    let (optimized_monsters, final_analysis) = balancer.balance_encounter(vec![fighter], vec![dragon.clone()], timeline, 0);

    println!("Initial Dragon Breath: {:?}", dragon.actions[0]);
    println!("Optimized Dragon Breath: {:?}", optimized_monsters[0].actions[0]);
    println!("Final Safety Grade: {:?}", final_analysis.safety_grade);
    println!("Final Intensity Tier: {:?}", final_analysis.intensity_tier);

    // Assertions
    // 1. The dragon should have been nerfed
    if let Action::Template(t) = &optimized_monsters[0].actions[0] {
        if let Some(DiceFormula::Expr(expr)) = &t.template_options.amount {
            // Reconstruct damage converts back to dice string. 
            // 54 average damage is ~12d8. 
            // A nerf should result in a smaller number or fewer dice.
            println!("Optimized damage expr: {}", expr);
        }
        
        // Check if DC was nerfed
        if let Some(dc) = t.template_options.save_dc {
            assert!(dc <= 18.0);
        }
    }

    // 2. Safety grade should be at least C (it was likely F initially)
    assert!(
        matches!(final_analysis.safety_grade, simulation_wasm::decile_analysis::SafetyGrade::A | simulation_wasm::decile_analysis::SafetyGrade::B | simulation_wasm::decile_analysis::SafetyGrade::C),
        "Expected Grade A, B or C, got {:?}", final_analysis.safety_grade
    );
}
