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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EnemyTarget {
    #[serde(rename = "enemy with least HP")]
    EnemyWithLeastHP,
    #[serde(rename = "enemy with most HP")]
    EnemyWithMostHP,
    #[serde(rename = "enemy with highest DPR")]
    EnemyWithHighestDPR,
    #[serde(rename = "enemy with lowest AC")]
    EnemyWithLowestAC,
    #[serde(rename = "enemy with highest AC")]
    EnemyWithHighestAC,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AllyTarget {
    #[serde(rename = "ally with the least HP")]
    AllyWithLeastHP,
    #[serde(rename = "ally with the most HP")]
    AllyWithMostHP,
    #[serde(rename = "ally with the highest DPR")]
    AllyWithHighestDPR,
    #[serde(rename = "ally with the lowest AC")]
    AllyWithLowestAC,
    #[serde(rename = "ally with the highest AC")]
    AllyWithHighestAC,
    #[serde(rename = "self")]
    Self_,
}

// Unified target type for templates that can target either allies or enemies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TargetType {
    Enemy(EnemyTarget),  // Try enemy first (more common for templates)
    Ally(AllyTarget),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionType {
    Atk,
    Heal,
    Buff,
    Debuff,
    Template,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuffDuration {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
}
