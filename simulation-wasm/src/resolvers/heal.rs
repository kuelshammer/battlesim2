use crate::context::TurnContext;
use crate::events::Event;
use crate::model::{Action, HealAction};
use crate::{dice, targeting};

pub fn resolve(
    _resolver: &crate::action_resolver::ActionResolver,
    heal: &HealAction,
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
    let target_indices = targeting::get_targets(&actor, &Action::Heal(heal.clone()), &allies, &enemies);

    let heal_amount = dice::evaluate(&heal.amount, 1);
    let is_temp_hp = heal.temp_hp.unwrap_or(false);

    for (is_enemy, idx) in target_indices {
        // Check for interruption
        if context.action_interrupted {
            break;
        }

        let target_id = if is_enemy {
            if idx < enemies.len() { enemies[idx].id.clone() } else { continue }
        } else if idx < allies.len() {
            allies[idx].id.clone()
        } else {
            continue
        };

        // Apply healing through TurnContext (unified method)
        let healing_event = context.apply_healing(&target_id, heal_amount, is_temp_hp, actor_id);
        events.push(healing_event);
    }

    events
}
