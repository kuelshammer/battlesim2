import { z } from 'zod'
import { v4 as uuid } from 'uuid'
import { AllyTargetSchema, BuffDurationSchema, ChallengeRatingSchema, ClassesSchema, ActionConditionSchema, CreatureTypeSchema, EnemyTargetSchema, CreatureConditionSchema, EnemyTargetList, AllyTargetList, TriggerConditionSchema, ResourceTypeSchema, ResetTypeSchema, ActionTagSchema } from './enums'
import { ClassOptionsSchema } from './classOptions'
import { validateDiceFormula } from './dice'
import type { ActionTemplateName } from '../data/actions'

export const DiceFormulaSchema = z.number().or(z.custom<string>((data) => {
    if (typeof data !== 'string') return false

    return validateDiceFormula(data)
}))

export const FrequencyList = ['at will', '1/fight', '1/day'] as const
export const FrequencySchema = z.enum(FrequencyList).or(z.discriminatedUnion('reset', [
    z.object({
        reset: z.literal('recharge'),
        cooldownRounds: z.number().min(2),
    }),
    z.object({
        reset: z.enum(['sr', 'lr']),
        uses: z.number().min(1)
    })
]))
export type Frequency = z.infer<typeof FrequencySchema>;

// New Phase 2 schemas for flexible action system
export const ActionCostSchema = z.discriminatedUnion('type', [
    z.object({
        type: z.literal('Discrete'),
        resourceType: ResourceTypeSchema,
        resourceVal: z.string().optional(),
        amount: z.number(),
    }),
    z.object({
        type: z.literal('Variable'),
        resourceType: ResourceTypeSchema,
        resourceVal: z.string().optional(),
        min: z.number(),
        max: z.number(),
    }),
])
export type ActionCost = z.infer<typeof ActionCostSchema>

export const ActionRequirementSchema = z.discriminatedUnion('type', [
    z.object({
        type: z.literal('ResourceAvailable'),
        resourceType: ResourceTypeSchema,
        resourceVal: z.string().optional(),
        amount: z.number(),
    }),
    z.object({
        type: z.literal('CombatState'),
        condition: z.enum(['EnemyInRange', 'IsSurprised', 'HasTempHP']),
        value: z.number().optional(),
    }),
    z.object({
        type: z.literal('StatusEffect'),
        effect: z.string(),
    }),
    z.object({
        type: z.literal('Custom'),
        description: z.string(),
    }),
])
export type ActionRequirement = z.infer<typeof ActionRequirementSchema>

const BuffSchema = z.object({
    displayName: z.string().optional(),
    duration: BuffDurationSchema.optional().default('entire encounter'),

    ac: DiceFormulaSchema.optional(),
    toHit: DiceFormulaSchema.optional(),
    damage: DiceFormulaSchema.optional(),
    damageReduction: DiceFormulaSchema.optional(),
    damageMultiplier: z.number().optional(),
    damageTakenMultiplier: z.number().optional(),
    dc: DiceFormulaSchema.optional(),
    save: DiceFormulaSchema.optional(),
    condition: CreatureConditionSchema.optional(),

    // Odds that the buff was applied. All of the effects are multiplied by this value. Default 1.
    magnitude: z.number().optional(),
    concentration: z.boolean().optional(),
}).passthrough()

// Not to be used directly. See ActionSchema
const ActionSchemaBase = z.object({
    id: z.string().optional().default(() => uuid()),
    name: z.string().optional().default('Action'),

    // Legacy field - kept for backward compatibility during transition
    actionSlot: z.number().optional(), // Will be deprecated in favor of cost/requirements/tags

    // New fields replacing actionSlot
    cost: z.array(ActionCostSchema).default([]),
    requirements: z.array(ActionRequirementSchema).default([]),
    tags: z.array(ActionTagSchema).default([]),

    freq: FrequencySchema.optional().default('at will'),
    condition: ActionConditionSchema.optional().default('default'),
    targets: z.number().default(1),
}).passthrough()

const AtkActionSchema = ActionSchemaBase.merge(z.object({
    type: z.literal('atk'),
    dpr: DiceFormulaSchema.optional().default(0),
    toHit: DiceFormulaSchema.optional().default(0),
    target: EnemyTargetSchema.optional().default('enemy with most HP'),
    useSaves: z.boolean().optional(), // If false or undefined, action.targets becomes the number of hits, and the action can now target the same creature multiple times
    halfOnSave: z.boolean().optional(), // Only useful if useSaves == true

    // TODO: add other types of rider effects, like extra damage if the target fails a save, for example
    riderEffect: z.object({
        dc: z.number().optional(), // TODO: make it so if the dc is undefined, the rider effect applies without a save
        buff: BuffSchema,
    }).optional(),
})).passthrough()

const HealActionSchema = ActionSchemaBase.merge(z.object({
    type: z.literal('heal'),
    amount: DiceFormulaSchema.optional().default(0),
    tempHP: z.boolean().optional(),
    target: AllyTargetSchema.optional().default('ally with least HP'),
})).passthrough()

const BuffActionSchema = ActionSchemaBase.merge(z.object({
    type: z.literal('buff'),
    target: AllyTargetSchema.optional().default('self'),

    buff: BuffSchema,
})).passthrough()

const DebuffActionSchema = ActionSchemaBase.merge(z.object({
    type: z.literal('debuff'),
    target: EnemyTargetSchema.optional().default('enemy with least HP'),
    saveDC: z.number().optional().default(10),

    buff: BuffSchema,
})).passthrough()



const TemplateActionSchema = z.object({
    type: z.literal('template'),
    id: z.string(),
    freq: FrequencySchema,
    condition: ActionConditionSchema,

    // Legacy field - kept for backward compatibility during transition
    actionSlot: z.number().optional(), // Override the template's default action slot

    // New fields replacing actionSlot
    cost: z.array(ActionCostSchema).default([]),
    requirements: z.array(ActionRequirementSchema).default([]),
    tags: z.array(ActionTagSchema).default([]),

    templateOptions: z.object({
        target: AllyTargetSchema.or(EnemyTargetSchema).optional(),
        templateName: z.string() as z.ZodType<ActionTemplateName>, // Use string and cast to ActionTemplateName
        toHit: DiceFormulaSchema.optional(),
        saveDC: z.number().optional(),
        amount: DiceFormulaSchema.optional(),
    }),
})

// Like a regular Action, but without the possibility of it being a TemplateAction
export const FinalActionSchema = z.discriminatedUnion('type', [
    HealActionSchema,
    AtkActionSchema,
    BuffActionSchema,
    DebuffActionSchema,
])

const ActionSchema = z.discriminatedUnion('type', [
    HealActionSchema,
    AtkActionSchema,
    BuffActionSchema,
    DebuffActionSchema,
    TemplateActionSchema,
])

// Creature is the definition of the creature. It's what the user inputs.
// Combattant (see below) is the representation of a creature during the simulation. 
// A new combattant is created for each instance of the creature, and for each round of combat.
export const CreatureSchema = z.object({
    id: z.string(),
    arrival: z.number().optional(), // Which round is the creature added (optional, default: round 0)

    mode: z.enum(['player', 'monster', 'custom']), // This determines which UI is opened when the user clicks on "modify creature"

    // Properties for monsters. Not used by the simulator, but by the monster search UI.
    type: CreatureTypeSchema.optional(),
    cr: ChallengeRatingSchema.optional(),
    src: z.string().optional(),

    // Properties for player characters. Not used by the simulator, but used by the PC template UI.
    class: z.object({
        type: ClassesSchema,
        level: z.number(),
        options: ClassOptionsSchema,
    }).passthrough().optional(),

    // Properties of the creature, which are used by the simulator
    name: z.string(),
    count: z.coerce.number(),
    hp: z.coerce.number(), // Removed .int() constraint
    ac: z.coerce.number(),
    speed_fly: z.coerce.number().optional(),

    // Save bonuses - average is required, individual are optional overrides
    saveBonus: z.coerce.number().default(0), // Average save bonus (default to 0 if missing)
    strSaveBonus: z.coerce.number().optional(),
    dexSaveBonus: z.coerce.number().optional(),
    conSaveBonus: z.coerce.number().optional(),
    intSaveBonus: z.coerce.number().optional(),
    wisSaveBonus: z.coerce.number().optional(),
    chaSaveBonus: z.coerce.number().optional(),

    // Advantage on saves
    conSaveAdvantage: z.boolean().optional(), // For Resilient (Con), War Caster, etc.
    saveAdvantage: z.boolean().optional(), // For Paladin Aura, advantage on ALL saves

    initiativeBonus: DiceFormulaSchema.optional(),
    initiativeAdvantage: z.boolean().optional(),
    actions: z.array(ActionSchema),
    triggers: z.array(z.object({
        id: z.string(),
        condition: TriggerConditionSchema,
        action: ActionSchema,
        cost: z.number().optional(), // ActionSlot
    }).passthrough()).optional(),
    // New fields for Phase 5 resource management
    spellSlots: z.record(z.string(), z.coerce.number()).optional(),
    classResources: z.record(z.string(), z.coerce.number()).optional(),
    hitDice: DiceFormulaSchema.optional(), // New field for hit dice for short rest healing
    conModifier: z.coerce.number().optional(), // New field for constitution modifier to apply to hit dice rolls
}).passthrough()

const TeamSchema = z.array(CreatureSchema)

// Simplified ResourceLedger for frontend display
export const ResourceLedgerSchema = z.object({
    current: z.record(z.string(), z.number()),
    max: z.record(z.string(), z.number()),
})
export type ResourceLedger = z.infer<typeof ResourceLedgerSchema>

const CreatureStateSchema = z.object({
    currentHp: z.number(),
    tempHp: z.number().optional(),
    buffs: z.map(z.string(), BuffSchema),
    resources: ResourceLedgerSchema,
    upcomingBuffs: z.map(z.string(), BuffSchema),
    usedActions: z.set(z.string()),
    concentratingOn: z.string().nullable().optional(),
    arcaneWardHp: z.number().optional(),
})

const CombattantSchema = z.object({
    id: z.string(),
    initiative: z.number().optional(),
    creature: CreatureSchema,
    initialState: CreatureStateSchema,
    finalState: CreatureStateSchema,

    // Actions taken by the creature on that round. Initially empty, will be filled by the simulator
    actions: z.array(z.object({
        action: FinalActionSchema,
        targets: z.map(z.string(), z.number()),
    })),
})

const RoundSchema = z.object({
    team1: z.array(CombattantSchema),
    team2: z.array(CombattantSchema),
})

export const EncounterSchema = z.object({
    id: z.string().optional().default(() => uuid()),
    monsters: TeamSchema,
    playersSurprised: z.boolean().optional(),
    monstersSurprised: z.boolean().optional(),
    shortRest: z.boolean().optional(),
    playersPrecast: z.boolean().optional(),
    monstersPrecast: z.boolean().optional(),
})
const EncounterStatsSchema = z.object({
    damageDealt: z.number(),
    damageTaken: z.number(),

    healGiven: z.number(),
    healReceived: z.number(),

    charactersBuffed: z.number(),
    buffsReceived: z.number(),

    charactersDebuffed: z.number(),
    debuffsReceived: z.number(),

    timesUnconscious: z.number(),
})

const EncounterResultSchema = z.object({
    stats: z.map(z.string(), EncounterStatsSchema),
    rounds: z.array(RoundSchema),
})
const SimulationResultSchema = z.object({
    encounters: z.array(EncounterResultSchema),
})

export type DiceFormula = z.infer<typeof DiceFormulaSchema>
export type Buff = z.infer<typeof BuffSchema>
export type EnemyTarget = z.infer<typeof EnemyTargetSchema>
export type AllyTarget = z.infer<typeof AllyTargetSchema>
export type AtkAction = z.infer<typeof AtkActionSchema>
export type HealAction = z.infer<typeof HealActionSchema>
export type BuffAction = z.infer<typeof BuffActionSchema>
export type DebuffAction = z.infer<typeof DebuffActionSchema>
export type TemplateAction = z.infer<typeof TemplateActionSchema>
export type Action = z.infer<typeof ActionSchema>
export type FinalAction = z.infer<typeof FinalActionSchema>
export type Creature = z.infer<typeof CreatureSchema>
export type Team = z.infer<typeof TeamSchema>
export type CreatureState = z.infer<typeof CreatureStateSchema>
export type Combattant = z.infer<typeof CombattantSchema>
export type Round = z.infer<typeof RoundSchema>
export type EncounterStats = z.infer<typeof EncounterStatsSchema>
export type Encounter = z.infer<typeof EncounterSchema>
export type EncounterResult = z.infer<typeof EncounterResultSchema>
export type SimulationResult = z.infer<typeof SimulationResultSchema>

// Phase 3: Event System
export const EventSchema = z.discriminatedUnion('type', [
    // Combat Events
    z.object({
        type: z.literal('ActionStarted'),
        actor_id: z.string(),
        action_id: z.string(),
    }),
    z.object({
        type: z.literal('AttackHit'),
        attacker_id: z.string(),
        target_id: z.string(),
        damage: z.number(),
    }),
    z.object({
        type: z.literal('AttackMissed'),
        attacker_id: z.string(),
        target_id: z.string(),
    }),
    z.object({
        type: z.literal('DamageTaken'),
        target_id: z.string(),
        damage: z.number(),
        damage_type: z.string(),
    }),
    z.object({
        type: z.literal('DamagePrevented'),
        target_id: z.string(),
        prevented_amount: z.number(),
    }),
    z.object({
        type: z.literal('HealingApplied'),
        target_id: z.string(),
        amount: z.number(),
        source_id: z.string(),
    }),
    z.object({
        type: z.literal('TempHPGranted'),
        target_id: z.string(),
        amount: z.number(),
        source_id: z.string(),
    }),

    // Spell Events
    z.object({
        type: z.literal('SpellCast'),
        caster_id: z.string(),
        spell_id: z.string(),
        spell_level: z.number(),
    }),
    z.object({
        type: z.literal('SpellSaved'),
        target_id: z.string(),
        spell_id: z.string(),
    }),
    z.object({
        type: z.literal('ConcentrationBroken'),
        caster_id: z.string(),
        reason: z.string(),
    }),

    // Status Events
    z.object({
        type: z.literal('BuffApplied'),
        target_id: z.string(),
        buff_id: z.string(),
        source_id: z.string(),
    }),
    z.object({
        type: z.literal('BuffExpired'),
        target_id: z.string(),
        buff_id: z.string(),
    }),
    z.object({
        type: z.literal('ConditionAdded'),
        target_id: z.string(),
        condition: CreatureConditionSchema,
        source_id: z.string(),
    }),
    z.object({
        type: z.literal('ConditionRemoved'),
        target_id: z.string(),
        condition: CreatureConditionSchema,
    }),

    // Life Cycle Events
    z.object({
        type: z.literal('UnitDied'),
        unit_id: z.string(),
        killer_id: z.string().optional(),
        damage_type: z.string().optional(),
    }),
    z.object({
        type: z.literal('TurnStarted'),
        unit_id: z.string(),
        round_number: z.number(),
    }),
    z.object({
        type: z.literal('TurnEnded'),
        unit_id: z.string(),
        round_number: z.number(),
    }),
    z.object({
        type: z.literal('RoundStarted'),
        round_number: z.number(),
    }),
    z.object({
        type: z.literal('RoundEnded'),
        round_number: z.number(),
    }),
    z.object({
        type: z.literal('EncounterStarted'),
        combatant_ids: z.array(z.string()),
    }),
    z.object({
        type: z.literal('EncounterEnded'),
        winner: z.string().optional(),
        rounds: z.number(),
    }),

    // Custom/Other
    z.object({
        type: z.literal('Custom'),
        event_type: z.string(),
        data: z.record(z.string(), z.string()),
        source_id: z.string(),
    }),
])

export type Event = z.infer<typeof EventSchema>

// Decile Analysis Types
export const CombatantVisualizationSchema = z.object({
    name: z.string(),
    maxHp: z.number().int(),
    startHp: z.number().int(),
    currentHp: z.number().int(),
    isDead: z.boolean(),
    isPlayer: z.boolean(),
    hpPercentage: z.number(),
})

export const DecileStatsSchema = z.object({
    decile: z.number(),
    label: z.string(),
    medianSurvivors: z.number(),
    partySize: z.number(),
    totalHpLost: z.number(),
    hpLostPercent: z.number(),
    winRate: z.number(),
    // New fields for 5-Timeline Dashboard
    medianRunVisualization: z.array(CombatantVisualizationSchema),
    medianRunData: EncounterResultSchema.optional().nullable(),
    battleDurationRounds: z.number(),
})

export const AggregateOutputSchema = z.object({
    scenarioName: z.string(),
    totalRuns: z.number(),
    deciles: z.array(DecileStatsSchema),
    globalMedian: DecileStatsSchema.optional().nullable(),
}).passthrough()

export const FullAnalysisOutputSchema = z.object({
    overall: AggregateOutputSchema,
    encounters: z.array(AggregateOutputSchema),
})

export const FullSimulationOutputSchema = z.object({
    results: z.array(SimulationResultSchema),
    analysis: FullAnalysisOutputSchema,
    firstRunEvents: z.array(EventSchema),
})

export type CombatantVisualization = z.infer<typeof CombatantVisualizationSchema>
export type DecileStats = z.infer<typeof DecileStatsSchema>
export type AggregateOutput = z.infer<typeof AggregateOutputSchema>
export type FullAnalysisOutput = z.infer<typeof FullAnalysisOutputSchema>
export type FullSimulationOutput = z.infer<typeof FullSimulationOutputSchema>
