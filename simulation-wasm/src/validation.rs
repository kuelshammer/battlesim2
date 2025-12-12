use crate::model::Action;
use crate::context::TurnContext;
use crate::resources::{ActionCost, ActionRequirement};

/// Checks if all requirements for an action are met by a combatant in the given context.
pub fn check_action_requirements(
    action: &Action,
    _context: &TurnContext,
    combatant_id: &str,
) -> bool {
    for requirement in action.base().requirements.iter() {
        if !check_single_requirement(requirement, _context, combatant_id) {
            return false;
        }
    }
    true
}

/// Checks if a single requirement is met by a combatant in the given context.
pub fn check_single_requirement(
    requirement: &ActionRequirement,
    _context: &TurnContext,
    combatant_id: &str,
) -> bool {
    match requirement {
        ActionRequirement::ResourceAvailable { resource_type, resource_val, amount } => {
            let cost_for_check = vec![ActionCost::Discrete { 
                resource_type: resource_type.clone(), 
                resource_val: resource_val.clone(),
                amount: *amount 
            }];
            _context.can_afford(&cost_for_check, combatant_id)
        },
        ActionRequirement::CombatState { condition: combat_condition } => {
            check_combat_condition(combat_condition, _context, combatant_id)
        },
        ActionRequirement::StatusEffect { effect: effect_name } => {
            let effects = _context.get_effects_on_target(combatant_id);
            effects.iter().any(|effect| {
                match &effect.effect_type {
                    crate::context::EffectType::Buff(buff) => buff.display_name.as_ref() == Some(effect_name),
                    crate::context::EffectType::Condition(condition) => format!("{:?}", condition) == *effect_name,
                    _ => false, // Only buffs and conditions are considered for StatusEffect requirement
                }
            })
        },
        ActionRequirement::Custom { description: _description } => {
            // Custom requirements are not evaluated automatically and need external scripting
            // For now, custom requirements always fail if not handled explicitly elsewhere.
            false
        }
    }
}

/// Checks specific combat conditions.
fn check_combat_condition(
    condition: &crate::resources::CombatCondition,
    _context: &TurnContext,
    _combatant_id: &str, // _combatant_id might be needed for some conditions
) -> bool {
    match condition {
        crate::resources::CombatCondition::EnemyInRange(_range) => {
            // Requires a position system. For now, always false.
            false
        },
        crate::resources::CombatCondition::IsSurprised => {
            // Requires a surprised status in combatant state. For now, always false.
            false
        },
        crate::resources::CombatCondition::HasTempHP => {
           if let Some(combatant) = _context.combatants.get(_combatant_id) {
               combatant.temp_hp > 0.0
           } else {
               false
           }
        }
    }
}
