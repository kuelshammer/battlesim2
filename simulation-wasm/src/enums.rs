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
                matches!(event, Event::AttackHit { .. })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum TriggerRequirement {
    #[serde(rename = "damageType")]
    DamageType(String),
    #[serde(rename = "range")]
    Range(i32),
    #[serde(rename = "hasTempHP")]
    HasTempHP,
    #[serde(rename = "actionTag")]
    ActionTag(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
