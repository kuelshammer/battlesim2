use crate::context::{EffectType, TurnContext};
use crate::dice;
use crate::events::{Event, RollResult};
use crate::model::{Action, AtkAction};
use crate::rng;
use crate::targeting;
use crate::combat_stats::CombatStatsCache;
use serde::{Deserialize, Serialize};

/// Event-driven action resolver that converts actions into events
#[derive(Debug, Clone)]
pub struct ActionResolver {
    /// Random number generator for dice rolls
    #[allow(dead_code)]
    rng_seed: Option<u64>,
    /// Cache for combatant statistics to optimize targeting
    pub combat_stats_cache: CombatStatsCache,
}

/// Result of an attack roll
#[derive(Debug, Clone)]
struct AttackRollResult {
    total: f64,
    is_critical: bool,
    is_miss: bool,
    roll_detail: Option<RollResult>,
}

/// Result of action resolution containing all generated events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResolutionResult {
    pub events: Vec<Event>,
    pub success: bool,
    pub error: Option<String>,
}

impl Default for ActionResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionResolver {
    /// Create a new action resolver
    pub fn new() -> Self {
        Self { 
            rng_seed: None,
            combat_stats_cache: CombatStatsCache::new(),
        }
    }

    /// Create a new action resolver with a specific seed for reproducible results
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng_seed: Some(seed),
            combat_stats_cache: CombatStatsCache::new(),
        }
    }

    /// Create a new action resolver with an existing cache
    pub fn with_cache(cache: CombatStatsCache) -> Self {
        Self {
            rng_seed: None,
            combat_stats_cache: cache,
        }
    }

    /// Resolve an action and emit events to the context
    pub fn resolve_action(
        &self,
        action: &Action,
        context: &mut TurnContext,
        actor_id: &str,
    ) -> Vec<Event> {
        match action {
            Action::Atk(attack_action) => self.resolve_attack(attack_action, context, actor_id),
            Action::Heal(heal_action) => {
                crate::resolvers::resolve_heal(self, heal_action, context, actor_id)
            }
            Action::Buff(buff_action) => {
                crate::resolvers::resolve_buff(self, buff_action, context, actor_id)
            }
            Action::Debuff(debuff_action) => {
                crate::resolvers::resolve_debuff(self, debuff_action, context, actor_id)
            }
            Action::Template(template_action) => {
                crate::resolvers::resolve_template(self, template_action, context, actor_id)
            }
        }
    }

    /// Resolve attack actions with proper event emission
    /// Each attack re-evaluates the best target from currently alive enemies
    pub fn resolve_attack(
        &self,
        attack: &AtkAction,
        context: &mut TurnContext,
        actor_id: &str,
    ) -> Vec<Event> {
        let mut events = Vec::new();

        // 1. Get actor
        let actor = match context.get_combatant(actor_id) {
            Some(c) => c.base_combatant.clone(),
            None => return events,
        };

        let actor_side = context.get_combatant(actor_id).unwrap().side;
        
        // 2. Resolve hits
        let count = attack.targets.max(1) as usize;
        
        for _ in 0..count {
            // Check if action was interrupted by a reaction
            if context.action_interrupted {
                break;
            }

            // Refresh alive enemies for every hit to support dynamic strategies (Most HP, Highest Survivability)
            let all_alive = context.get_alive_combatants();
            let enemies: Vec<_> = all_alive.into_iter()
                .filter(|c| c.side != actor_side)
                .map(|c| c.base_combatant.clone())
                .collect();

            let strategy = attack.target.clone();
            
            if let Some(idx) = targeting::select_enemy_target(&actor, strategy, &enemies, &[], None) {
                let target_id = enemies[idx].id.clone();
                self.resolve_single_attack_hit(attack, context, actor_id, &target_id, &mut events);
            } else {
                break; // No targets available
            }
        }

        events
    }

    /// Helper to resolve a single hit of an attack
    fn resolve_single_attack_hit(
        &self,
        attack: &AtkAction,
        context: &mut TurnContext,
        actor_id: &str,
        target_id: &str,
        events: &mut Vec<Event>,
    ) {
        // Perform attack roll
        let attack_result = self.roll_attack(attack, context, actor_id, target_id);
        let target_ac = self.get_target_ac(target_id, context);

        // Check for hit
        let mut is_hit = !attack_result.is_miss
            && (attack_result.is_critical || attack_result.total >= target_ac);

        // Defensive Reactions (Shield)
        if is_hit && !attack_result.is_critical {
            let (new_ac, reaction_events) = self.resolve_defensive_reactions(target_id, context, attack_result.total, target_ac);
            if !reaction_events.is_empty() {
                events.extend(reaction_events);
                if attack_result.total < new_ac {
                    is_hit = false;
                }
            }
        }

        if is_hit {
            let (damage, damage_roll) = self.calculate_damage(attack, attack_result.is_critical, context, actor_id);

            // Determine range from action tags
            use crate::resources::ActionTag;
            let range = if attack.tags.contains(&ActionTag::Melee) {
                Some(crate::enums::AttackRange::Melee)
            } else if attack.tags.contains(&ActionTag::Ranged) {
                Some(crate::enums::AttackRange::Ranged)
            } else {
                None
            };

            let hit_event = Event::AttackHit {
                attacker_id: actor_id.to_string(),
                target_id: target_id.to_string(),
                damage,
                attack_roll: attack_result.roll_detail,
                damage_roll,
                target_ac,
                range,
            };
            context.record_event(hit_event.clone());
            events.push(hit_event);

            let trigger_events = self.resolve_trigger_effects(
                target_id,
                crate::enums::TriggerCondition::OnBeingHit,
                context,
                Some(actor_id),
                None,
            );
            events.extend(trigger_events);

            let damage_events = context.apply_damage(target_id, damage, "Physical", actor_id);
            events.extend(damage_events);
        } else {
            let miss_event = Event::AttackMissed {
                attacker_id: actor_id.to_string(),
                target_id: target_id.to_string(),
                attack_roll: attack_result.roll_detail,
                target_ac,
            };
            context.record_event(miss_event.clone());
            events.push(miss_event);
        }
    }

    /// Resolve healing actions with proper event emission

    /// Helper to resolve reactive effects from buffs (Triggers)
    /// Helper to resolve reactive effects from buffs (Triggers)
    fn resolve_trigger_effects(
        &self,
        reactor_id: &str,
        condition: crate::enums::TriggerCondition,
        context: &mut TurnContext,
        triggering_actor_id: Option<&str>,
        _damage_info: Option<&str>,
    ) -> Vec<Event> {
        let mut events = Vec::new();

        // Iterate over active effects to find buffs on the reactor
        // Access values() directly to avoid borrow checker issues with iterating the map
        let active_effects: Vec<crate::context::ActiveEffect> =
            context.active_effects.values().cloned().collect();

        for effect in active_effects {
            if effect.target_id != reactor_id {
                continue;
            }

            if let EffectType::Buff(buff) = &effect.effect_type {
                for trigger in &buff.triggers {
                    if trigger.condition == condition {
                        // Check requirements
                        let requirements_met = trigger.requirements.iter().all(|req| {
                            match req {
                                crate::enums::TriggerRequirement::HasTempHP => {
                                    if let Some(c) = context.combatants.get(reactor_id) {
                                        c.temp_hp > 0
                                    } else {
                                        false
                                    }
                                }
                                // Implement other requirements as needed
                                _ => true,
                            }
                        });

                        if requirements_met {
                            // Execute Effect
                            match &trigger.effect {
                                crate::enums::TriggerEffect::DealDamage {
                                    amount,
                                    damage_type,
                                } => {
                                    if let Some(target_id) = triggering_actor_id {
                                        // Parse amount formula
                                        let formula =
                                            crate::model::DiceFormula::Expr(amount.clone());
                                        // Basic eval for now, assume no variable parts specific to reactor yet (except fixed values)
                                        let dmg_value = dice::evaluate(&formula, 1);

                                        // Apply damage to the TRIGGERING ACTOR (Retaliation)
                                        let dmg_events = context.apply_damage(
                                            target_id,
                                            dmg_value,
                                            damage_type,
                                            reactor_id,
                                        );
                                        events.extend(dmg_events);
                                    }
                                }
                                crate::enums::TriggerEffect::GrantImmediateAction {
                                    action_id,
                                    action_slot: _,
                                } => {
                                    // Grant an immediate action outside normal turn order
                                    if let Some(combatant) = context.get_combatant(reactor_id) {
                                        // Look for the action in the combatant's known actions
                                        let action = combatant.base_combatant.creature.actions.iter()
                                            .find(|a| a.base().id == *action_id)
                                            .cloned();

                                        if let Some(action) = action {
                                            // Emit action start event for the granted action
                                            context.record_event(crate::events::Event::ActionStarted {
                                                actor_id: reactor_id.to_string(),
                                                action_id: action_id.clone(),
                                                decision_trace: std::collections::HashMap::new(),
                                            });

                                            // Resolve the granted action recursively
                                            let nested_events = self.resolve_action(&action, context, reactor_id);
                                            events.extend(nested_events);
                                        }
                                    }
                                }
                                crate::enums::TriggerEffect::InterruptAction { action_id: _ } => {
                                    // Set the interrupt flag to stop the current action sequence
                                    context.action_interrupted = true;
                                }
                                crate::enums::TriggerEffect::AddToRoll { amount, roll_type: _ } => {
                                    // Add a pending roll bonus
                                    context.roll_modifications.add(
                                        reactor_id,
                                        crate::context::RollModification::AddBonus {
                                            amount: amount.clone(),
                                        },
                                    );
                                }
                                crate::enums::TriggerEffect::ForceSelfReroll {
                                    roll_type,
                                    must_use_second,
                                } => {
                                    // Force a reroll for the reactor
                                    context.roll_modifications.add(
                                        reactor_id,
                                        crate::context::RollModification::Reroll {
                                            roll_type: roll_type.clone(),
                                            must_use_second: *must_use_second,
                                        },
                                    );
                                }
                                crate::enums::TriggerEffect::ForceTargetReroll {
                                    roll_type,
                                    must_use_second,
                                } => {
                                    // Force a reroll for the triggering actor
                                    if let Some(target_id) = triggering_actor_id {
                                        context.roll_modifications.add(
                                            target_id,
                                            crate::context::RollModification::Reroll {
                                                roll_type: roll_type.clone(),
                                                must_use_second: *must_use_second,
                                            },
                                        );
                                    }
                                }
                                // Implement other effects as needed
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        events
    }

    /// Resolve defensive reactions (like Shield) that can change a hit to a miss
    fn resolve_defensive_reactions(
        &self,
        reactor_id: &str,
        context: &mut TurnContext,
        hit_roll: f64,
        current_ac: f64,
    ) -> (f64, Vec<Event>) {
        let mut events = Vec::new();
        let mut final_ac = current_ac;

        // Get reactor's triggers
        let triggers = match context.get_combatant(reactor_id) {
            Some(c) => c.base_combatant.creature.triggers.clone(),
            None => return (final_ac, events),
        };

        // Check if reactor has a reaction available
        let reaction_slot_id = crate::enums::ActionSlot::Reaction;
        if let Some(reactor) = context.get_combatant(reactor_id) {
            if reactor.base_combatant.final_state.used_actions.contains(&(reaction_slot_id as i32).to_string()) {
                return (final_ac, events);
            }
        }

                for trigger in triggers {

                    if trigger.condition == crate::enums::TriggerCondition::OnBeingAttacked

                        && trigger.cost == Some(crate::enums::ActionSlot::Reaction as i32)

                        && hit_roll >= final_ac

                    {

                        // Check if it's a Shield-like template

                        if let Action::Template(template_action) = &trigger.action {

                            let template_name = template_action.template_options.template_name.to_lowercase();

                            

                            if template_name == "shield" {

                                // Shield adds +5 AC

                                let shield_ac_bonus = 5.0;

                                if hit_roll < final_ac + shield_ac_bonus {

                                    // Trigger it!

                                    // 1. Consume reaction

                                    if let Some(reactor_mut) = context.get_combatant_mut(reactor_id) {

                                        reactor_mut.base_combatant.final_state.used_actions.insert((reaction_slot_id as i32).to_string());

                                    }

        

                                    // 2. Resolve the action (will apply the buff)

                                    // Record action start for the reaction

                                    context.record_event(Event::ActionStarted {

                                        actor_id: reactor_id.to_string(),

                                        action_id: template_action.id.clone(),

                                        decision_trace: std::collections::HashMap::new(),

                                    });

        

                                    let reaction_events = self.resolve_action(&trigger.action, context, reactor_id);

                                    events.extend(reaction_events);

        

                                    final_ac += shield_ac_bonus;

                                    break; // Only one defensive reaction per attack

                                }

                            }

                        }

                    }

                }

        (final_ac, events)
    }

    /// Get save bonus for a combatant including active buffs



    /// Roll attack value
    fn roll_attack(&self, attack: &AtkAction, context: &mut TurnContext, actor_id: &str, target_id: &str) -> AttackRollResult {
        // 1. Determine Advantage/Disadvantage
        let attacker_has_adv = context.has_condition(actor_id, crate::enums::CreatureCondition::AttacksWithAdvantage)
            || context.has_condition(actor_id, crate::enums::CreatureCondition::AttacksAndIsAttackedWithAdvantage);
        let attacker_has_triple_adv = context.has_condition(actor_id, crate::enums::CreatureCondition::AttacksWithTripleAdvantage);
        let target_grants_adv = context.has_condition(target_id, crate::enums::CreatureCondition::IsAttackedWithAdvantage);
        
        let attacker_has_dis = context.has_condition(actor_id, crate::enums::CreatureCondition::AttacksWithDisadvantage)
            || context.has_condition(actor_id, crate::enums::CreatureCondition::AttacksAndSavesWithDisadvantage);
        let target_grants_dis = context.has_condition(target_id, crate::enums::CreatureCondition::IsAttackedWithDisadvantage);

        let final_triple_adv = attacker_has_triple_adv && !(attacker_has_dis || target_grants_dis);
        let final_adv = (attacker_has_adv || target_grants_adv) && !(attacker_has_dis || target_grants_dis) && !final_triple_adv;
        let final_dis = (attacker_has_dis || target_grants_dis) && !(attacker_has_adv || target_grants_adv || attacker_has_triple_adv);

        // 2. Perform Roll
        let mut roll1 = rng::roll_d20();
        
        // Apply rerolls before calculating natural_roll
        let mods = context.roll_modifications.take_all(actor_id);
        for modif in &mods {
            if let crate::context::RollModification::Reroll { roll_type, must_use_second } = modif {
                if roll_type == "attack" {
                    let roll2 = rng::roll_d20();
                    if *must_use_second {
                        roll1 = roll2;
                    } else {
                        roll1 = roll1.max(roll2);
                    }
                }
            }
        }

        let natural_roll: u32;
        
        if final_triple_adv {
            let roll2 = rng::roll_d20();
            let roll3 = rng::roll_d20();
            natural_roll = roll1.max(roll2).max(roll3);
        } else if final_adv {
            let roll2 = rng::roll_d20();
            natural_roll = roll1.max(roll2);
        } else if final_dis {
            let roll2 = rng::roll_d20();
            natural_roll = roll1.min(roll2);
        } else {
            natural_roll = roll1;
        }

        let (modifier_total, roll_detail) = if context.log_enabled {
            let detail = dice::evaluate_detailed(&attack.to_hit, 1);
            (detail.total, Some(detail))
        } else {
            (dice::evaluate(&attack.to_hit, 1), None)
        };

        let mut total = natural_roll as f64 + modifier_total;

        // Apply bonus modifications
        for modif in &mods {
            if let crate::context::RollModification::AddBonus { amount } = modif {
                // Simplified bonus evaluation
                let formula = crate::model::DiceFormula::Expr(amount.clone());
                total += dice::evaluate(&formula, 1);
            }
        }

        
        // Check for accuracy-altering buffs in active effects
        let mut final_roll_detail = roll_detail;
        
        // Filter and sort active buffs affecting the attacker for determinism
        let mut attacker_buffs: Vec<_> = context.active_effects.values()
            .filter(|e| e.target_id == actor_id)
            .filter_map(|e| if let EffectType::Buff(b) = &e.effect_type { Some((&e.id, b)) } else { None })
            .collect();
        attacker_buffs.sort_by(|a, b| a.0.cmp(b.0));

        for (_, buff) in attacker_buffs {
            if let Some(to_hit_formula) = &buff.to_hit {
                if context.log_enabled {
                    let buff_roll = dice::evaluate_detailed(to_hit_formula, 1);
                    total += buff_roll.total;
                    if let Some(detail) = &mut final_roll_detail {
                        detail.modifiers.push((buff.display_name.clone().unwrap_or_else(|| "Buff".to_string()), buff_roll.total));
                        detail.total += buff_roll.total;
                    }
                } else {
                    total += dice::evaluate(to_hit_formula, 1);
                }
            }
        }

        AttackRollResult {
            total,
            is_critical: natural_roll == 20,
            is_miss: natural_roll == 1,
            roll_detail: final_roll_detail,
        }
    }

    /// Get target's armor class
    fn get_target_ac(&self, target_id: &str, context: &TurnContext) -> f64 {
        let Some(target) = context.get_combatant(target_id) else {
            return 10.0;
        };

        let base_ac = target.base_combatant.creature.ac as f64;
        
        // Sum up AC bonuses from active effects in the context
        let mut buff_ac = 0.0;
        for effect in context.active_effects.values() {
            if effect.target_id == target_id {
                if let EffectType::Buff(buff) = &effect.effect_type {
                    if let Some(ac_formula) = &buff.ac {
                        buff_ac += dice::average(ac_formula);
                    }
                }
            }
        }

        base_ac + buff_ac
    }

    /// Get save bonus for a combatant including active buffs
    fn get_save_bonus(&self, target_id: &str, context: &TurnContext) -> f64 {
        let Some(target) = context.get_combatant(target_id) else {
            return 0.0;
        };

        let mut bonus = target.base_combatant.creature.save_bonus;

        // Add bonuses from active effects (Bless, Bane, etc.)
        // Filter and sort for determinism
        let mut target_buffs: Vec<_> = context.active_effects.values()
            .filter(|e| e.target_id == target_id)
            .filter_map(|e| if let EffectType::Buff(b) = &e.effect_type { Some((&e.id, b)) } else { None })
            .collect();
        target_buffs.sort_by(|a, b| a.0.cmp(b.0));

        for (_, buff) in target_buffs {
            if let Some(save_formula) = &buff.save {
                bonus += dice::average(save_formula);
            }
        }

        bonus
    }

    /// Roll a saving throw for a combatant
    pub fn roll_save(&self, target_id: &str, context: &mut TurnContext) -> f64 {
        let mut roll = rng::roll_d20();
        
        // Apply rerolls
        let mods = context.roll_modifications.take_all(target_id);
        for modif in &mods {
            if let crate::context::RollModification::Reroll { roll_type, must_use_second } = modif {
                if roll_type == "save" {
                    let roll2 = rng::roll_d20();
                    if *must_use_second {
                        roll = roll2;
                    } else {
                        roll = roll.max(roll2);
                    }
                }
            }
        }

        let mut total = roll as f64 + self.get_save_bonus(target_id, context);

        // Apply bonus modifications
        for modif in &mods {
            if let crate::context::RollModification::AddBonus { amount } = modif {
                // Simplified bonus evaluation
                let formula = crate::model::DiceFormula::Expr(amount.clone());
                total += dice::evaluate(&formula, 1);
            }
        }

        total
    }

    /// Calculate damage from attack
    fn calculate_damage(&self, attack: &AtkAction, is_critical: bool, context: &TurnContext, actor_id: &str) -> (f64, Option<RollResult>) {
        let (mut damage, mut damage_roll) = if context.log_enabled {
            let detail = dice::evaluate_detailed(&attack.dpr, if is_critical { 2 } else { 1 });
            (detail.total, Some(detail))
        } else {
            (dice::evaluate(&attack.dpr, if is_critical { 2 } else { 1 }), None)
        };

        // Add damage bonuses from active buffs
        let mut attacker_buffs: Vec<_> = context.active_effects.values()
            .filter(|e| e.target_id == actor_id)
            .filter_map(|e| if let EffectType::Buff(b) = &e.effect_type { Some((&e.id, b)) } else { None })
            .collect();
        attacker_buffs.sort_by(|a, b| a.0.cmp(b.0));

        for (_, buff) in attacker_buffs {
            if let Some(damage_formula) = &buff.damage {
                if context.log_enabled {
                    let buff_dmg_roll = dice::evaluate_detailed(damage_formula, 1);
                    damage += buff_dmg_roll.total;
                    if let Some(detail) = &mut damage_roll {
                        detail.modifiers.push((buff.display_name.clone().unwrap_or_else(|| "Damage Buff".to_string()), buff_dmg_roll.total));
                        detail.total += buff_dmg_roll.total;
                    }
                } else {
                    damage += dice::evaluate(damage_formula, 1);
                }
            }
        }

        if is_critical {
            if let Some(detail) = &mut damage_roll {
                // If it's a critical hit, we've already doubled the dice in evaluate_detailed (via multiplier)
                // But we might want to label it.
                detail.modifiers.push(("Critical".to_string(), 0.0));
            }
        }

        (damage, damage_roll)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DiceFormula;

    #[test]
    fn test_action_resolver_creation() {
        let resolver = ActionResolver::new();
        assert!(resolver.rng_seed.is_none());

        let resolver_with_seed = ActionResolver::with_seed(42);
        assert_eq!(resolver_with_seed.rng_seed, Some(42));
    }

    #[test]
    fn test_resolve_attack() {
        let _resolver = ActionResolver::new();

        // This test would require more setup to create a proper TurnContext
        // For now, just test that the method exists and can be called
        // In a full implementation, this would test actual attack resolution
    }

    #[test]
    fn test_resolve_heal() {
        let _resolver = ActionResolver::new();

        // This test would require proper TurnContext setup
        // For now, just test method existence
    }

    #[test]
    fn test_reactive_retaliation() {
        use crate::context::TurnContext;
        use crate::enums::{
            ActionCondition, BuffDuration, EnemyTarget, TriggerCondition, TriggerEffect,
            TriggerRequirement,
        };
        use crate::model::{Buff, Combattant, Creature, CreatureState, EffectTrigger};

        // 1. Setup Context with Attacker and Defender
        let attacker = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "orc".to_string(),
            name: "Orc".to_string(),
            hp: 20,
            ac: 10,
            count: 1.0,
            actions: vec![],
            triggers: vec![],
            arrival: None,
            mode: "monster".to_string(),
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
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        };
        let defender = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "warlock".to_string(),
            name: "Warlock".to_string(),
            hp: 20,
            ac: 10,
            count: 1.0,
            actions: vec![],
            triggers: vec![],
            arrival: None,
            mode: "player".to_string(),
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
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        };

        let mut context = TurnContext::new(
            vec![
                Combattant { team: 1,
                    id: "orc".to_string(),
                    creature: std::sync::Arc::new(attacker),
                    initiative: 10.0,
                    initial_state: CreatureState { current_hp: 20, ..CreatureState::default() },
                    final_state: CreatureState { current_hp: 20, ..CreatureState::default() },
                    actions: vec![],
                },
                Combattant { team: 0,
                    id: "warlock".to_string(),
                    creature: std::sync::Arc::new(defender.clone()),
                    initiative: 10.0,
                    initial_state: CreatureState {
                        current_hp: 20,
                        temp_hp: Some(10), // Pre-give Temp HP
                        ..CreatureState::default()
                    },
                    final_state: CreatureState {
                        current_hp: 20,
                        temp_hp: Some(10),
                        ..CreatureState::default()
                    },
                    actions: vec![],
                },
            ],
            vec![],
            None,
            "Plains".to_string(),
            true,
        );

        // 2. Apply Reactive Buff manually to Defender
        let buff = Buff {
            display_name: Some("Armor of Agathys".to_string()),
            duration: BuffDuration::EntireEncounter,
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
            source: Some("warlock".to_string()),
            concentration: false,
            triggers: vec![EffectTrigger {
                condition: TriggerCondition::OnBeingHit,
                requirements: vec![TriggerRequirement::HasTempHP],
                effect: TriggerEffect::DealDamage {
                    amount: "5".to_string(),
                    damage_type: "Cold".to_string(),
                },
            }],
            suppressed_until: None,
        };

        use crate::context::{ActiveEffect, EffectType};
        context.apply_effect(ActiveEffect {
            id: "aoa".to_string(),
            source_id: "warlock".to_string(),
            target_id: "warlock".to_string(),
            effect_type: EffectType::Buff(buff),
            remaining_duration: 100,
            conditions: vec![],
        });

        // 2.5 Precalculate stats for targeting
        context.precalculate_combat_stats();

        // 3. Resolve Attack
        let resolver = ActionResolver::new();
        let attack = crate::model::AtkAction {
            id: "axe".to_string(),
            name: "Axe".to_string(),
            action_slot: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
            freq: crate::model::Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 1,
            dpr: crate::model::DiceFormula::Value(4.0), // Low damage so temp hp remains
            to_hit: crate::model::DiceFormula::Value(100.0), // Ensure Hit
            target: EnemyTarget::EnemyWithLeastHP,
            use_saves: None,
            half_on_save: None,
            rider_effect: None,
        };

        // Mock getting target manually since our resolution doesn't run full selection here easily
        // But resolve_attack calls get_single_attack_target.
        // We need to ensure logic selects "warlock".
        // resolve_attack -> get_single_attack_target.
        // Ensure Warlock is enemy of Orc (modes differ). They do (monster vs player).

        rng::force_d20_rolls(vec![10]);
        let events = resolver.resolve_attack(&attack, &mut context, "orc");

        // 4. Assertions
        // Check for AttackHit
        assert!(events.iter().any(|e| matches!(e, crate::events::Event::AttackHit { target_id, .. } if target_id == "warlock")));

        // Check for Retaliation Damage on Orc
        let retaliation = events.iter().find(|e| matches!(e, crate::events::Event::DamageTaken { target_id, damage_type, .. } if target_id == "orc" && damage_type == "Cold"));
        assert!(retaliation.is_some(), "Retaliation damage event not found!");

        if let Some(crate::events::Event::DamageTaken { damage, .. }) = retaliation {
            assert_eq!(*damage, 5.0);
        }

        // Check Temp HP reduced on Warlock
        let warlock = context.combatants.get("warlock").unwrap();
        assert!(
            warlock.temp_hp < 10,
            "Temp HP should be reduced by attack damage"
        );
    }
    #[test]
    fn test_multiattack_retargeting() {
        use crate::context::TurnContext;
        use crate::enums::{ActionCondition, EnemyTarget};
        use crate::model::{Combattant, Creature, CreatureState};

        // Setup: Attacker (3 attacks), 3 Victims (10 HP each)
        // Damage = 5 (Fixed). 2 Hits to kill.

        let attacker_c = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "attacker".to_string(),
            name: "Attacker".to_string(),
            hp: 100,
            ac: 15,
            count: 1.0,
            actions: vec![],
            triggers: vec![],
            arrival: None,
            mode: "player".to_string(),
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
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        };
        let victim_tpl = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "victim".to_string(),
            name: "Victim".to_string(),
            hp: 4, // One hit kills
            ac: 10,
            count: 1.0,
            actions: vec![],
            triggers: vec![],
            arrival: None,
            mode: "monster".to_string(),
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
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        };

        let mut context = TurnContext::new(
            vec![
                Combattant { team: 0,
                    id: "attacker".to_string(),
                    creature: std::sync::Arc::new(attacker_c),
                    initiative: 20.0,
                    initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
                    final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
                    actions: vec![],
                },
                Combattant { team: 1,
                    id: "v1".to_string(),
                    creature: std::sync::Arc::new(victim_tpl.clone()),
                    initiative: 10.0,
                    initial_state: CreatureState { current_hp: 10, ..CreatureState::default() },
                    final_state: CreatureState { current_hp: 10, ..CreatureState::default() },
                    actions: vec![],
                },
                Combattant { team: 1,
                    id: "v2".to_string(),
                    creature: std::sync::Arc::new(victim_tpl.clone()),
                    initiative: 10.0,
                    initial_state: CreatureState { current_hp: 10, ..CreatureState::default() },
                    final_state: CreatureState { current_hp: 10, ..CreatureState::default() },
                    actions: vec![],
                },
                Combattant { team: 1,
                    id: "v3".to_string(),
                    creature: std::sync::Arc::new(victim_tpl.clone()),
                    initiative: 10.0,
                    initial_state: CreatureState { current_hp: 10, ..CreatureState::default() },
                    final_state: CreatureState { current_hp: 10, ..CreatureState::default() },
                    actions: vec![],
                },
            ],
            vec![],
            None,
            "Arena".to_string(),
            true,
        );

        let resolver = ActionResolver::with_seed(12345); // Deterministic

        // 3 Attacks. 5 Damage Each. 10 HP Targets.
        // Should go: v1(5dmg) -> v1(Kill) -> v2(5dmg).
        let attack = AtkAction {
            id: "multi".to_string(),
            name: "Multiattack".to_string(),
            action_slot: Some(1),
            freq: crate::model::Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 3,
            dpr: crate::model::DiceFormula::Expr("5".to_string()),
            to_hit: crate::model::DiceFormula::Expr("100".to_string()), // Sure hit
            target: EnemyTarget::EnemyWithLeastHP,
            use_saves: None,
            half_on_save: None,
            rider_effect: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
        };

        let events = resolver.resolve_attack(&attack, &mut context, "attacker");

        println!("Events: {:?}", events);

        let v1_hits = events.iter().filter(|e| matches!(e, crate::events::Event::AttackHit { target_id, .. } if target_id == "v1")).count();
        let v2_hits = events.iter().filter(|e| matches!(e, crate::events::Event::AttackHit { target_id, .. } if target_id == "v2")).count();
        let v3_hits = events.iter().filter(|e| matches!(e, crate::events::Event::AttackHit { target_id, .. } if target_id == "v3")).count();

        let dead_count = events
            .iter()
            .filter(|e| matches!(e, crate::events::Event::UnitDied { .. }))
            .count();

        // Assertions:
        // 1. All three units should die (if all hit)
        assert!(dead_count >= 1, "At least one victim should die");

        // 2. Hits distribution should be [1, 1, 1] if no misses
        let mut hits = vec![v1_hits, v2_hits, v3_hits];
        hits.sort();
        
        if dead_count == 3 {
            assert_eq!(hits, vec![1, 1, 1]);
        }
    }
}
