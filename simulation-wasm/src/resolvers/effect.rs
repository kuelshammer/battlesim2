use crate::context::{ActiveEffect, EffectType, TurnContext};
use crate::events::Event;
use crate::model::{Action, BuffAction, DebuffAction};
use crate::{targeting, action_resolver::ActionResolver};

pub fn resolve_buff(
    _resolver: &ActionResolver,
    buff_action: &BuffAction,
    context: &mut TurnContext,
    actor_id: &str,
) -> Vec<Event> {
    let mut events = Vec::new();

    // 1. Get actor and teammates/enemies for targeting
    let actor = match context.get_combatant(actor_id) {
        Some(c) => c.base_combatant.clone(),
        None => return events,
    };

    let actor_side = context.get_combatant(actor_id).unwrap().side;
    
    let all_combatants = context.get_alive_combatants();
    let (allies, enemies): (Vec<_>, Vec<_>) = all_combatants.into_iter()
        .map(|c| c.base_combatant.clone())
        .partition(|c| context.get_combatant(&c.id).unwrap().side == actor_side);

    // 2. Get smart targets from targeting module
    let target_indices = targeting::get_targets(&actor, &Action::Buff(buff_action.clone()), &allies, &enemies);

    for (is_enemy, idx) in target_indices {
        // Check for interruption
        if context.action_interrupted {
            break;
        }

        let target_id = if is_enemy {
            if idx < enemies.len() { enemies[idx].id.clone() } else { continue }
        } else {
            if idx < allies.len() { allies[idx].id.clone() } else { continue }
        };

        let effect = ActiveEffect {
            id: format!("{}-{}-{}", buff_action.name, actor_id, target_id),
            source_id: actor_id.to_string(),
            target_id: target_id.clone(),
            effect_type: EffectType::Buff(buff_action.buff.clone()),
            remaining_duration: 10,
            conditions: Vec::new(),
        };

        events.push(context.apply_effect(effect));
    }

    events
}

pub fn resolve_debuff(
    resolver: &ActionResolver,
    debuff_action: &DebuffAction,
    context: &mut TurnContext,
    actor_id: &str,
) -> Vec<Event> {
    let mut events = Vec::new();

    // 1. Get actor and teammates/enemies for targeting
    let actor = match context.get_combatant(actor_id) {
        Some(c) => c.base_combatant.clone(),
        None => return events,
    };

    let actor_side = context.get_combatant(actor_id).unwrap().side;
    
    let all_combatants = context.get_alive_combatants();
    let (allies, enemies): (Vec<_>, Vec<_>) = all_combatants.into_iter()
        .map(|c| c.base_combatant.clone())
        .partition(|c| context.get_combatant(&c.id).unwrap().side == actor_side);

    // 2. Get smart targets from targeting module
    let target_indices = targeting::get_targets(&actor, &Action::Debuff(debuff_action.clone()), &allies, &enemies);

    for (is_enemy, idx) in target_indices {
        // Check for interruption
        if context.action_interrupted {
            break;
        }

        let target_id = if is_enemy {
            if idx < enemies.len() { enemies[idx].id.clone() } else { continue }
        } else {
            if idx < allies.len() { allies[idx].id.clone() } else { continue }
        };

        // 1. Perform saving throw
        let total_save = resolver.roll_save(&target_id, context);

        if total_save < debuff_action.save_dc {
            // Save failed! Apply buff
            let effect = ActiveEffect {
                id: format!("{}-{}-{}", debuff_action.name, actor_id, target_id),
                source_id: actor_id.to_string(),
                target_id: target_id.clone(),
                effect_type: EffectType::Buff(debuff_action.buff.clone()),
                remaining_duration: 10,
                conditions: Vec::new(),
            };

            events.push(context.apply_effect(effect));
        } else {
            // Save succeeded
            events.push(Event::SpellSaved {
                target_id: target_id.clone(),
                spell_id: debuff_action.name.clone(),
            });
        }
    }

    events
}

pub fn resolve_effect(
    _resolver: &ActionResolver,
    effect_action: &impl EffectAction,
    context: &mut TurnContext,
    actor_id: &str,
    is_buff: bool,
) -> Vec<Event> {
    let events = Vec::new();

    let targets = effect_action.get_targets(context, actor_id);

    for target_id in targets {
        // Check for interruption
        if context.action_interrupted {
            break;
        }

        let effect_id = effect_action.base().id.clone();

        apply_effect_to_combatant(&target_id, &effect_id, actor_id, is_buff, context);
    }

    events
}

fn apply_effect_to_combatant(
    target_id: &str,
    effect_id: &str,
    source_id: &str,
    is_buff: bool,
    context: &mut TurnContext,
) {
    let effect_type = if is_buff {
        panic!("Buffs must use resolve_buff directly to preserve data");
    } else {
        EffectType::Condition(crate::enums::CreatureCondition::Incapacitated)
    };

    let effect = ActiveEffect {
        id: format!("{}_{}", effect_id, source_id),
        source_id: source_id.to_string(),
        target_id: target_id.to_string(),
        effect_type,
        remaining_duration: 5,
        conditions: Vec::new(),
    };

    context.apply_effect(effect);
}

pub trait EffectAction {
    fn base(&self) -> crate::model::ActionBase;
    fn get_targets(&self, context: &mut TurnContext, actor_id: &str) -> Vec<String>;
}

impl EffectAction for BuffAction {
    fn base(&self) -> crate::model::ActionBase {
        self.base()
    }

    fn get_targets(&self, context: &mut TurnContext, actor_id: &str) -> Vec<String> {
        self.get_simple_targets(context, actor_id)
    }
}

impl EffectAction for DebuffAction {
    fn base(&self) -> crate::model::ActionBase {
        self.base()
    }

    fn get_targets(&self, context: &mut TurnContext, actor_id: &str) -> Vec<String> {
        self.get_simple_targets(context, actor_id)
    }
}

trait SimpleTargeting {
    fn get_simple_targets(&self, context: &mut TurnContext, actor_id: &str) -> Vec<String>;
}

impl SimpleTargeting for BuffAction {
    fn get_simple_targets(&self, context: &mut TurnContext, actor_id: &str) -> Vec<String> {
        let actor_mode = context
            .get_combatant(actor_id)
            .map(|c| c.base_combatant.creature.mode.clone())
            .unwrap_or_default();

        context
            .combatants
            .values()
            .filter(|c| context.is_combatant_alive(&c.id))
            .filter(|c| c.base_combatant.creature.mode == actor_mode)
            .take(self.targets.max(1) as usize)
            .map(|c| c.id.clone())
            .collect()
    }
}

impl SimpleTargeting for DebuffAction {
    fn get_simple_targets(&self, context: &mut TurnContext, actor_id: &str) -> Vec<String> {
        let actor_mode = context
            .get_combatant(actor_id)
            .map(|c| c.base_combatant.creature.mode.clone())
            .unwrap_or_default();

        context
            .combatants
            .values()
            .filter(|c| c.id != actor_id)
            .filter(|c| context.is_combatant_alive(&c.id))
            .filter(|c| c.base_combatant.creature.mode != actor_mode)
            .take(self.targets.max(1) as usize)
            .map(|c| c.id.clone())
            .collect()
    }
}
