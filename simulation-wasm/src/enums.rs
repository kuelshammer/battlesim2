use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionSlot {
    Action = 0,
    BonusAction = 1,
    Reaction = 4,
    LegendaryAction = 2,
    LairAction = 3,
    Other1 = 5,
    Other2 = 6,
    WhenReducedTo0HP = -1,
    WhenReducingAnEnemyTo0HP = -2,
    BeforeTheEncounterStarts = -3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttackRange {
    Melee,
    Ranged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EnemyTarget {
    #[serde(rename = "enemy with least HP", alias = "enemy with the least HP")]
    EnemyWithLeastHP,
    #[serde(rename = "enemy with most HP", alias = "enemy with the most HP")]
    EnemyWithMostHP,
    #[serde(rename = "enemy with highest DPR", alias = "enemy with the highest DPR")]
    EnemyWithHighestDPR,
    #[serde(rename = "enemy with lowest AC", alias = "enemy with the lowest AC")]
    EnemyWithLowestAC,
    #[serde(rename = "enemy with highest AC", alias = "enemy with the highest AC")]
    EnemyWithHighestAC,
    #[serde(rename = "enemy with highest survivability", alias = "enemy with the highest survivability")]
    EnemyWithHighestSurvivability,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AllyTarget {
    #[serde(rename = "ally with least HP", alias = "ally with the least HP")]
    AllyWithLeastHP,
    #[serde(rename = "ally with most HP", alias = "ally with the most HP")]
    AllyWithMostHP,
    #[serde(rename = "ally with highest DPR", alias = "ally with the highest DPR")]
    AllyWithHighestDPR,
    #[serde(rename = "ally with lowest AC", alias = "ally with the lowest AC")]
    AllyWithLowestAC,
    #[serde(rename = "ally with highest AC", alias = "ally with the highest AC")]
    AllyWithHighestAC,
    #[serde(rename = "self")]
    Self_,
}

// Unified target type for templates that can target either allies or enemies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TargetType {
    Enemy(EnemyTarget), // Try enemy first (more common for templates)
    Ally(AllyTarget),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionCondition {
    Default,
    #[serde(rename = "ally at 0 HP")]
    AllyAt0HP,
    #[serde(rename = "ally under half HP")]
    AllyUnderHalfHP,
    #[serde(rename = "ally below 25% HP")]
    AllyBelow25PercentHP,
    #[serde(rename = "ally below 50% HP")]
    AllyBelow50PercentHP,
    #[serde(rename = "ally below 75% HP")]
    AllyBelow75PercentHP,
    #[serde(rename = "any ally injured")]
    AnyAllyInjured,
    #[serde(rename = "any ally needs healing")]
    AnyAllyNeedsHealing,
    #[serde(rename = "any ally below 50% HP")]
    AnyAllyBelow50PercentHP,
    #[serde(rename = "is available")]
    IsAvailable,
    #[serde(rename = "is under half HP")]
    IsUnderHalfHP,
    #[serde(rename = "has no THP")]
    HasNoTHP,
    #[serde(rename = "not used yet")]
    NotUsedYet,
    #[serde(rename = "enemy count one")]
    EnemyCountOne,
    #[serde(rename = "enemy count multiple")]
    EnemyCountMultiple,
    #[serde(rename = "buff not active")]
    BuffNotActive,
    #[serde(rename = "not concentrating")]
    NotConcentrating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CreatureCondition {
    Blinded,
    Frightened,
    Incapacitated,
    Invisible,
    Paralyzed,
    Petrified,
    Poisoned,
    Restrained,
    Stunned,
    Unconscious,
    Exhausted,
    #[serde(rename = "Attacks with Advantage")]
    AttacksWithAdvantage,
    #[serde(rename = "Attacks with Disadvantage")]
    AttacksWithDisadvantage,
    #[serde(rename = "Attacks with Triple Advantage")]
    AttacksWithTripleAdvantage,
    #[serde(rename = "Is attacked with Advantage")]
    IsAttackedWithAdvantage,
    #[serde(rename = "Is attacked with Disadvantage")]
    IsAttackedWithDisadvantage,
    #[serde(rename = "Attacks and is attacked with Advantage")]
    AttacksAndIsAttackedWithAdvantage,
    #[serde(rename = "Attacks and saves with Disadvantage")]
    AttacksAndSavesWithDisadvantage,
    #[serde(rename = "Saves with Advantage")]
    SavesWithAdvantage,
    #[serde(rename = "Save with Disadvantage")]
    SaveWithDisadvantage,
    #[serde(rename = "Is Concentrating")]
    IsConcentrating,
    #[serde(rename = "Is Surprised")]
    IsSurprised,
    #[serde(rename = "Is Prone")]
    IsProne,
    #[serde(rename = "Is Hidden")]
    IsHidden,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionType {
    Atk,
    Heal,
    Buff,
    Debuff,
    Template,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuffDuration {
    #[serde(rename = "instant")]
    Instant,
    #[serde(rename = "until next attack made")]
    UntilNextAttackMade,
    #[serde(rename = "until next attack taken")]
    UntilNextAttackTaken,
    #[serde(rename = "1 round")]
    OneRound,
    #[serde(rename = "repeat the save each round")]
    RepeatTheSaveEachRound,
    #[serde(rename = "entire encounter")]
    EntireEncounter,
}

// Re-insert TriggerCondition here
// Note: No Eq/Hash due to f64 in DamageExceedsPercent (f64 doesn't implement Eq/Hash)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TriggerCondition {
    #[serde(rename = "on hit")]
    OnHit, // e.g. Divine Smite
    #[serde(rename = "on being attacked")]
    OnBeingAttacked, // e.g. Shield Spell, Cutting Words
    #[serde(rename = "on miss")]
    OnMiss, // e.g. Precision Attack
    #[serde(rename = "on being damaged")]
    OnBeingDamaged, // e.g. Hellish Rebuke
    #[serde(rename = "on ally attacked")]
    OnAllyAttacked, // e.g. Sentinel
    #[serde(rename = "on enemy death")]
    OnEnemyDeath, // e.g. Great Weapon Master, Dark One's Blessing
    #[serde(rename = "on critical hit")]
    OnCriticalHit, // e.g. Divine Smite (crit fishing)
    #[serde(rename = "on being hit")]
    OnBeingHit, // e.g. Armor of Agathys that requires a hit but not necessarily damage
    #[serde(rename = "on cast spell")]
    OnCastSpell, // e.g. Counterspell, Silvery Barbs
    #[serde(rename = "on save failed")]
    OnSaveFailed, // e.g. Portent, Silvery Barbs
    #[serde(rename = "on save succeeded")]
    OnSaveSucceeded, // e.g. Lucky, Magic Resonance
    #[serde(rename = "on enemy moved")]
    OnEnemyMoved, // e.g. Opportunity Attack
    #[serde(rename = "enemy entered reach")]
    EnemyEnteredReach, // e.g. Polearm Master - enemy moved from >5ft to <=5ft
    #[serde(rename = "enemy left reach")]
    EnemyLeftReach, // e.g. Opportunity Attack - enemy moved from <=5ft to >5ft
    #[serde(rename = "on ability check")]
    OnAbilityCheck, // e.g. Bardic Inspiration, Luck
    #[serde(rename = "on concentration broken")]
    OnConcentrationBroken, // e.g. War Caster, Concentration focus

    // Composite triggers
    #[serde(rename = "and")]
    And { conditions: Vec<TriggerCondition> },
    #[serde(rename = "or")]
    Or { conditions: Vec<TriggerCondition> },
    #[serde(rename = "not")]
    Not { condition: Box<TriggerCondition> },

    // State conditions
    #[serde(rename = "enemyCountAtLeast")]
    EnemyCountAtLeast { count: i32 },
    #[serde(rename = "damageExceedsPercent")]
    DamageExceedsPercent { threshold: f64 },
    #[serde(rename = "attackWasMelee")]
    AttackWasMelee,
    #[serde(rename = "belowHpPercent")]
    BelowHpPercent { threshold: f64 },
    #[serde(rename = "aboveHpPercent")]
    AboveHpPercent { threshold: f64 },
    #[serde(rename = "hasTempHp")]
    HasTempHp,
    #[serde(rename = "hasReactionAvailable")]
    HasReactionAvailable,
}

impl TriggerCondition {
    /// Evaluate if this trigger condition matches the given event
    pub fn evaluate(&self, event: &crate::events::Event) -> bool {
        use crate::events::Event;

        match self {
            // Simple event type checks
            TriggerCondition::OnHit => matches!(event, Event::AttackHit { .. }),
            TriggerCondition::OnBeingAttacked => matches!(event, Event::AttackHit { .. }),
            TriggerCondition::OnMiss => matches!(event, Event::AttackMissed { .. }),
            TriggerCondition::OnBeingDamaged => matches!(event, Event::DamageTaken { .. }),
            TriggerCondition::OnAllyAttacked => matches!(event, Event::AttackHit { .. }),
            TriggerCondition::OnEnemyDeath => matches!(event, Event::UnitDied { .. }),
            TriggerCondition::OnCriticalHit => matches!(event, Event::AttackHit { .. }),
            TriggerCondition::OnBeingHit => matches!(event, Event::AttackHit { .. }),
            TriggerCondition::OnCastSpell => {
                matches!(event, Event::CastSpell { .. } | Event::SpellCast { .. })
            }
            TriggerCondition::OnSaveFailed => {
                matches!(event, Event::SaveResult { succeeded: false, .. })
            }
            TriggerCondition::OnSaveSucceeded => {
                matches!(event, Event::SaveResult { succeeded: true, .. })
            }
            TriggerCondition::OnEnemyMoved => {
                matches!(event, Event::UnitMoved { .. })
            }
            TriggerCondition::EnemyEnteredReach => {
                // Check if enemy moved from >5ft to <=5ft (for Polearm Master)
                if let Event::UnitMoved { from_position, to_position, .. } = event {
                    // Calculate distances from origin (0,0) - 5ft reach in D&D is one 5ft square
                    // In grid terms, adjacent squares are 5ft apart (Manhattan-like distance)
                    let from_dist = from_position
                        .map(|(x, y)| (x.abs() + y.abs()) as f64 * 5.0)
                        .unwrap_or(f64::MAX);
                    let to_dist = to_position
                        .map(|(x, y)| (x.abs() + y.abs()) as f64 * 5.0)
                        .unwrap_or(f64::MAX);

                    // Was outside reach (>5ft) and now within reach (<=5ft)
                    from_dist > 5.0 && to_dist <= 5.0
                } else {
                    false
                }
            }
            TriggerCondition::EnemyLeftReach => {
                // Check if enemy moved from <=5ft to >5ft (for Opportunity Attack)
                if let Event::UnitMoved { from_position, to_position, .. } = event {
                    // Calculate distances from origin (0,0)
                    let from_dist = from_position
                        .map(|(x, y)| (x.abs() + y.abs()) as f64 * 5.0)
                        .unwrap_or(f64::MAX);
                    let to_dist = to_position
                        .map(|(x, y)| (x.abs() + y.abs()) as f64 * 5.0)
                        .unwrap_or(f64::MAX);

                    // Was within reach (<=5ft) and now outside reach (>5ft)
                    from_dist <= 5.0 && to_dist > 5.0
                } else {
                    false
                }
            }
            TriggerCondition::OnAbilityCheck => {
                matches!(event, Event::AbilityCheckMade { .. })
            }
            TriggerCondition::OnConcentrationBroken => {
                matches!(event, Event::ConcentrationBroken { .. })
            }

            // Composite triggers - recursive evaluation
            TriggerCondition::And { conditions } => {
                conditions.iter().all(|c| c.evaluate(event))
            }
            TriggerCondition::Or { conditions } => {
                conditions.iter().any(|c| c.evaluate(event))
            }
            TriggerCondition::Not { condition } => {
                !condition.evaluate(event)
            }

            // State conditions - require additional context
            // These return false for now as they need combat state
            TriggerCondition::EnemyCountAtLeast { count: _ } => false,
            TriggerCondition::DamageExceedsPercent { threshold: _ } => {
                matches!(event, Event::DamageTaken { .. })
            }
            TriggerCondition::AttackWasMelee => {
                matches!(event, Event::AttackHit { range: Some(AttackRange::Melee), .. })
            }
            TriggerCondition::BelowHpPercent { threshold: _ } => {
                // TODO: Requires combat state to check HP percentage
                // Implementation: (current_hp / max_hp) * 100.0 < threshold
                false
            }
            TriggerCondition::AboveHpPercent { threshold: _ } => {
                // TODO: Requires combat state to check HP percentage
                // Implementation: (current_hp / max_hp) * 100.0 >= threshold
                false
            }
            TriggerCondition::HasTempHp => {
                // TODO: Requires combat state to check temp_hp
                // Implementation: temp_hp.unwrap_or(0) > 0
                false
            }
            TriggerCondition::HasReactionAvailable => {
                // TODO: Requires combat state to check reaction usage
                // Implementation: Check if combatant has reaction remaining this round
                false
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TriggerRequirement {
    #[serde(rename = "damageType")]
    DamageType(String),
    #[serde(rename = "range")]
    Range(i32),
    #[serde(rename = "hasTempHP")]
    HasTempHP,
    #[serde(rename = "actionTag")]
    ActionTag(String),
    #[serde(rename = "withinRange")]
    WithinRange { max_distance: f64 },
}

impl std::hash::Hash for TriggerRequirement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::mem::discriminant;
        match self {
            TriggerRequirement::DamageType(s) => {
                discriminant(self).hash(state);
                s.hash(state);
            }
            TriggerRequirement::Range(i) => {
                discriminant(self).hash(state);
                i.hash(state);
            }
            TriggerRequirement::HasTempHP => {
                discriminant(self).hash(state);
            }
            TriggerRequirement::ActionTag(s) => {
                discriminant(self).hash(state);
                s.hash(state);
            }
            TriggerRequirement::WithinRange { max_distance } => {
                discriminant(self).hash(state);
                crate::utilities::hash_f64(*max_distance, state);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TriggerEffect {
    #[serde(rename = "Damage")]
    DealDamage {
        amount: String, // DiceFormula string representation
        #[serde(rename = "damageType")]
        damage_type: String,
    },
    #[serde(rename = "ReduceDamage")]
    ReduceDamage { amount: String },
    #[serde(rename = "RestoreResource")]
    RestoreResource { resource: String, amount: String },
    #[serde(rename = "SuppressBuff")]
    SuppressBuff {
        #[serde(rename = "buffId")]
        buff_id: String,
        duration: BuffDuration,
    },
    #[serde(rename = "ApplyBuff")]
    ApplyBuff {
        buff: String, // buff ID to apply
        target: TriggerTarget,
    },
    #[serde(rename = "RemoveBuff")]
    RemoveBuff {
        #[serde(rename = "buffId")]
        buff_id: String,
        target: TriggerTarget,
    },
    #[serde(rename = "Chain")]
    Chain {
        effects: Vec<TriggerEffect>,
    },
    #[serde(rename = "AddToRoll")]
    AddToRoll {
        amount: String, // DiceFormula string representation
        #[serde(rename = "rollType")]
        roll_type: String, // "attack", "save", "abilityCheck", etc.
    },
    #[serde(rename = "ForceSelfReroll")]
    ForceSelfReroll {
        #[serde(rename = "rollType")]
        roll_type: String, // "attack", "save", "abilityCheck", etc.
        #[serde(rename = "mustUseSecond")]
        must_use_second: bool, // If true, must use second roll
    },
    #[serde(rename = "ForceTargetReroll")]
    ForceTargetReroll {
        #[serde(rename = "rollType")]
        roll_type: String, // "attack", "save", "abilityCheck", etc.
        #[serde(rename = "mustUseSecond")]
        must_use_second: bool, // If true, must use second roll
    },
    #[serde(rename = "InterruptAction")]
    InterruptAction {
        #[serde(rename = "actionId")]
        action_id: String, // ID of action to interrupt
    },
    #[serde(rename = "GrantImmediateAction")]
    GrantImmediateAction {
        #[serde(rename = "actionId")]
        action_id: String, // ID of action to grant immediately
        #[serde(rename = "actionSlot")]
        action_slot: String, // "bonusAction", "reaction", etc.
    },
    #[serde(rename = "RedirectAttack")]
    RedirectAttack {
        #[serde(rename = "newTargetId")]
        new_target_id: String,
    },
    #[serde(rename = "SplitDamage")]
    SplitDamage {
        #[serde(rename = "targetId")]
        target_id: String,
        #[serde(rename = "percent")]
        percent: f64,
    },
    #[serde(rename = "SetAdvantageOnNext")]
    SetAdvantageOnNext {
        #[serde(rename = "rollType")]
        roll_type: String,
        #[serde(rename = "advantage")]
        advantage: bool,
    },
    #[serde(rename = "ConsumeReaction")]
    ConsumeReaction {
        #[serde(rename = "targetId")]
        target_id: String,
    },
}

impl std::hash::Hash for TriggerEffect {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            TriggerEffect::DealDamage { amount, damage_type } => {
                amount.hash(state);
                damage_type.hash(state);
            }
            TriggerEffect::ReduceDamage { amount } => amount.hash(state),
            TriggerEffect::RestoreResource { resource, amount } => {
                resource.hash(state);
                amount.hash(state);
            }
            TriggerEffect::SuppressBuff { buff_id, duration } => {
                buff_id.hash(state);
                duration.hash(state);
            }
            TriggerEffect::ApplyBuff { buff, target } => {
                buff.hash(state);
                target.hash(state);
            }
            TriggerEffect::RemoveBuff { buff_id, target } => {
                buff_id.hash(state);
                target.hash(state);
            }
            TriggerEffect::Chain { effects } => {
                effects.hash(state);
            }
            TriggerEffect::AddToRoll { amount, roll_type } => {
                amount.hash(state);
                roll_type.hash(state);
            }
            TriggerEffect::ForceSelfReroll { roll_type, must_use_second } => {
                roll_type.hash(state);
                must_use_second.hash(state);
            }
            TriggerEffect::ForceTargetReroll { roll_type, must_use_second } => {
                roll_type.hash(state);
                must_use_second.hash(state);
            }
            TriggerEffect::InterruptAction { action_id } => action_id.hash(state),
            TriggerEffect::GrantImmediateAction { action_id, action_slot } => {
                action_id.hash(state);
                action_slot.hash(state);
            }
            TriggerEffect::RedirectAttack { new_target_id } => new_target_id.hash(state),
            TriggerEffect::SplitDamage { target_id, percent } => {
                target_id.hash(state);
                crate::utilities::hash_f64(*percent, state);
            }
            TriggerEffect::SetAdvantageOnNext { roll_type, advantage } => {
                roll_type.hash(state);
                advantage.hash(state);
            }
            TriggerEffect::ConsumeReaction { target_id } => target_id.hash(state),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TriggerTarget {
    Self_,
    Attacker,
    Target,
}

impl TriggerEffect {
    /// Apply this effect to the target combatant
    ///
    /// This is a placeholder implementation - full implementation requires
    /// access to combat state, damage resolution, and buff management systems.
    ///
    /// Returns Ok(()) if the effect was applied, Err with message if it failed.
    pub fn apply_effect(
        &self,
        _target_id: &str,
        _context: &mut crate::context::TurnContext,
    ) -> Result<(), String> {
        match self {
            TriggerEffect::DealDamage { amount, damage_type } => {
                // TODO: Parse damage formula and apply damage to target
                // This requires integration with the damage resolution system
                Err(format!("DealDamage not yet implemented: {} {}", amount, damage_type))
            }
            TriggerEffect::ReduceDamage { amount } => {
                // TODO: Parse reduction formula and apply to next damage
                Err(format!("ReduceDamage not yet implemented: {}", amount))
            }
            TriggerEffect::RestoreResource { resource, amount } => {
                // TODO: Parse amount formula and restore to resource
                Err(format!("RestoreResource not yet implemented: {} {}", resource, amount))
            }
            TriggerEffect::SuppressBuff { buff_id, duration } => {
                // Calculate the round when suppression should end
                let current_round = _context.round_number;
                let suppressed_until = match duration {
                    BuffDuration::OneRound => Some(current_round + 1),
                    BuffDuration::Instant => Some(current_round),
                    _ => {
                        // For other durations, suppress until end of current round
                        // TODO: Implement proper duration handling for UntilNextAttackTaken, etc.
                        Some(current_round + 1)
                    }
                };

                // Access the target combatant's buffs and set suppressed_until
                if let Some(combatant_state) = _context.combatants.get_mut(_target_id) {
                    if let Some(buff) = combatant_state.base_combatant.final_state.buffs.get_mut(buff_id) {
                        buff.suppressed_until = suppressed_until;
                        return Ok(());
                    }
                }
                Err(format!("SuppressBuff failed: buff '{}' not found on target '{}'", buff_id, _target_id))
            }
            TriggerEffect::ApplyBuff { buff, target } => {
                // TODO: Apply buff to target (Self_, Attacker, or Target)
                Err(format!("ApplyBuff not yet implemented: {} {:?}", buff, target))
            }
            TriggerEffect::RemoveBuff { buff_id, target } => {
                // TODO: Remove buff from target
                Err(format!("RemoveBuff not yet implemented: {} {:?}", buff_id, target))
            }
            TriggerEffect::Chain { effects } => {
                // TODO: Apply each effect in sequence
                Err(format!("Chain not yet implemented: {} effects", effects.len()))
            }
            TriggerEffect::AddToRoll { amount, roll_type } => {
                // TODO: Add bonus to next roll of specified type
                // Requires tracking pending roll modifiers in combat state
                Err(format!("AddToRoll not yet implemented: +{} to {}", amount, roll_type))
            }
            TriggerEffect::ForceSelfReroll { roll_type, must_use_second } => {
                // TODO: Force the triggering combatant to reroll
                // Requires roll modification system
                Err(format!("ForceSelfReroll not yet implemented: {} (must_use: {})", roll_type, must_use_second))
            }
            TriggerEffect::ForceTargetReroll { roll_type, must_use_second } => {
                // TODO: Force the target to reroll
                // Requires roll modification system
                Err(format!("ForceTargetReroll not yet implemented: {} (must_use: {})", roll_type, must_use_second))
            }
            TriggerEffect::InterruptAction { action_id } => {
                // TODO: Interrupt the specified action
                // Requires action interruption system
                Err(format!("InterruptAction not yet implemented: {}", action_id))
            }
            TriggerEffect::GrantImmediateAction { action_id, action_slot } => {
                // TODO: Grant an immediate action outside normal turn order
                // Requires action grant system
                Err(format!("GrantImmediateAction not yet implemented: {} ({})", action_id, action_slot))
            }
            TriggerEffect::RedirectAttack { new_target_id } => {
                Err(format!("RedirectAttack not yet implemented: {}", new_target_id))
            }
            TriggerEffect::SplitDamage { target_id, percent } => {
                Err(format!("SplitDamage not yet implemented: {} ({}%)", target_id, percent))
            }
            TriggerEffect::SetAdvantageOnNext { roll_type, advantage } => {
                Err(format!("SetAdvantageOnNext not yet implemented: {} ({})", roll_type, advantage))
            }
            TriggerEffect::ConsumeReaction { target_id } => {
                Err(format!("ConsumeReaction not yet implemented: {}", target_id))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Action,
    BonusAction,
    Reaction,
    Movement,
    SpellSlot,
    ClassResource,
    HitDiceD6,  // New
    HitDiceD8,  // New
    HitDiceD10, // New
    HitDiceD12, // New
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResetType {
    Turn,
    Round,
    ShortRest,
    LongRest,
    Encounter,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChallengeRating {
    Zero,
    Quarter,
    Half,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
    Thirteen,
    Fourteen,
    Fifteen,
    Sixteen,
    Seventeen,
    Eighteen,
    Nineteen,
    Twenty,
    TwentyOne,
    TwentyTwo,
    TwentyThree,
    TwentyFour,
    TwentyFive,
    Thirty,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Classes {
    Barbarian,
    Bard,
    Cleric,
    Druid,
    Fighter,
    Monk,
    Paladin,
    Ranger,
    Rogue,
    Sorcerer,
    Warlock,
    Wizard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CreatureType {
    Aberration,
    Beast,
    Celestial,
    Construct,
    Dragon,
    Elemental,
    Fey,
    Fiend,
    Giant,
    Humanoid,
    Monstrosity,
    Ooze,
    Plant,
    Undead,
}
