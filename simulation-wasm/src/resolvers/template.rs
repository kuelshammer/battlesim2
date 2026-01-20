use crate::context::{ActiveEffect, EffectType, TurnContext};
use crate::events::Event;
use crate::model::{Action, Buff, TemplateAction};
use crate::{action_resolver::ActionResolver, targeting};
use std::collections::HashMap;

pub fn resolve(
    resolver: &ActionResolver,
    template_action: &TemplateAction,
    context: &mut TurnContext,
    actor_id: &str,
) -> Vec<Event> {
    let mut events = Vec::new();
    let template_name = template_action
        .template_options
        .template_name
        .to_lowercase();

    // 1. Get actor and teammates/enemies for targeting
    let actor = match context.get_combatant(actor_id) {
        Some(c) => c.base_combatant.clone(),
        None => return events,
    };

    let actor_side = context.get_combatant(actor_id).unwrap().side;

    let all_combatants = context.get_alive_combatants();
    let (allies, enemies): (Vec<_>, Vec<_>) = all_combatants
        .into_iter()
        .map(|c| c.base_combatant.clone())
        .partition(|c| context.get_combatant(&c.id).unwrap().side == actor_side);

    // 2. Get smart targets from targeting module
    let mut template_action_to_resolve = template_action.clone();

    // Ensure default target type if missing
    if template_action_to_resolve.template_options.target.is_none() {
        let default_target = if template_name == "bane"
            || template_name == "hex"
            || template_name == "hunter's mark"
            || template_name == "hypnotic pattern"
        {
            use crate::enums::{EnemyTarget, TargetType};
            TargetType::Enemy(EnemyTarget::EnemyWithLeastHP)
        } else {
            use crate::enums::{AllyTarget, TargetType};
            TargetType::Ally(AllyTarget::AllyWithLeastHP)
        };
        template_action_to_resolve.template_options.target = Some(default_target);
    }

    let target_indices = targeting::get_targets(
        &actor,
        &Action::Template(template_action_to_resolve.clone()),
        &allies,
        &enemies,
    );

    for (is_enemy, idx) in target_indices {
        // Check for interruption
        if context.action_interrupted {
            break;
        }

        let target_id = if is_enemy {
            if idx < enemies.len() {
                enemies[idx].id.clone()
            } else {
                continue;
            }
        } else if idx < allies.len() {
            allies[idx].id.clone()
        } else {
            continue;
        };

        // Apply effect based on template name
        let mut buff = Buff {
            display_name: Some(template_action.template_options.template_name.clone()),
            duration: crate::enums::BuffDuration::EntireEncounter,
            ac: None,
            to_hit: None,
            damage: None,
            damage_reduction: None,
            damage_multiplier: None,
            damage_taken_multiplier: None,
            dc: None,
            save: None,
            condition: None,
            magnitude: None,
            source: Some(actor_id.to_string()),
            concentration: true,
            triggers: Vec::new(),
            suppressed_until: None,
        };

        // Customize buff based on name
        match template_name.as_str() {
            "bless" => {
                buff.to_hit = Some(crate::model::DiceFormula::Expr("1d4".to_string()));
                buff.save = Some(crate::model::DiceFormula::Expr("1d4".to_string()));
            }
            "bane" => {
                buff.to_hit = Some(crate::model::DiceFormula::Expr("-1d4".to_string()));
                buff.save = Some(crate::model::DiceFormula::Expr("-1d4".to_string()));
            }
            "haste" => {
                buff.ac = Some(crate::model::DiceFormula::Value(2.0));
            }
            "shield" => {
                buff.ac = Some(crate::model::DiceFormula::Value(5.0));
                buff.duration = crate::enums::BuffDuration::OneRound;
                buff.concentration = false;
            }
            _ => {}
        }

        // Perform saving throw for debuffs (bane)
        let mut should_apply = true;
        if template_name == "bane" {
            let total_save = resolver.roll_save(&target_id, context);
            let save_dc = template_action.template_options.save_dc.unwrap_or(13.0);

            if total_save >= save_dc {
                should_apply = false;
                events.push(Event::SpellSaved {
                    target_id: target_id.clone(),
                    spell_id: template_action.template_options.template_name.clone(),
                });
            }
        }

        if should_apply {
            let effect = ActiveEffect {
                id: format!(
                    "{}-{}-{}",
                    template_action.template_options.template_name, actor_id, target_id
                ),
                source_id: actor_id.to_string(),
                target_id: target_id.clone(),
                effect_type: EffectType::Buff(Box::new(buff)),
                remaining_duration: 10,
                conditions: Vec::new(),
            };

            events.push(context.apply_effect(effect));
        }
    }

    events.push(Event::Custom {
        event_type: "TemplateActionExecuted".to_string(),
        data: {
            let mut data = HashMap::new();
            data.insert(
                "template_name".to_string(),
                template_action.template_options.template_name.clone(),
            );
            data.insert("actor_id".to_string(), actor_id.to_string());
            data
        },
        source_id: actor_id.to_string(),
    });

    events
}
