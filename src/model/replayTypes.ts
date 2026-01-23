import type { Event } from './model'

/**
 * # Replay Schema Documentation
 *
 * The replay system uses a normalized hierarchical structure to enable O(1) navigation
 * and efficient parsing. Events are grouped into rounds → turns → actions, with
 * precomputed metadata for constant-time lookups.
 *
 * ## Structure Overview
 *
 * ```
 * Replay
 * ├── rounds: Round[] (chronological)
 * │   └── Round (roundNumber, turns[])
 * │       └── Turn (unitId, roundNumber, actions[])
 * │           └── Action (actorId, actionId, subEvents[])
 * ├── globalEvents: Event[] (events outside round boundaries)
 * └── metadata: ReplayMetadata (precomputed navigation data)
 * ```
 *
 * ## Parsing Flow
 *
 * 1. **Event Processing**: Events are processed chronologically, building nested structure
 * 2. **Boundary Detection**: RoundStarted/RoundEnded and TurnStarted/TurnEnded define boundaries
 * 3. **Action Grouping**: ActionStarted groups sub-events until ActionEnded
 * 4. **Metadata Computation**: Flat indices and offsets computed once after parsing
 *
 * ## Navigation
 *
 * - **Flat Index**: Global action index across entire replay (0-based)
 * - **Hierarchical Indices**: (roundIndex, turnIndex, actionIndex) within their containers
 * - **O(1) Lookups**: All navigation uses precomputed offsets in metadata
 *
 * ## Performance Characteristics
 *
 * - **Parsing**: O(n) where n = number of events
 * - **Navigation**: O(1) for all operations (seek, next, previous)
 * - **Memory**: Linear with number of actions, plus constant metadata overhead
 *
 * @example
 * ```typescript
 * const replay = parseEventsToReplay(events)
 * const action = getActionByFlatIndex(replay, 42) // O(1)
 * const next = getNextAction(replay, 42) // O(1)
 * ```
 */

/**
 * Represents a single action taken by a unit.
 * Contains sub-events that occurred during the action (hits, damage, buffs, etc.)
 */
export interface ReplayAction {
  /** The ID of the unit performing the action */
  actorId: string
  /** The ID of the action being performed */
  actionId: string
  /** All sub-events that occurred during this action, in chronological order */
  subEvents: Event[]
}

/**
 * Represents a single unit's turn within a round.
 * Contains all actions taken by that unit during their turn.
 */
export interface ReplayTurn {
  /** The ID of the unit whose turn this is */
  unitId: string
  /** The round number this turn belongs to */
  roundNumber: number
  /** All actions taken during this turn, in chronological order */
  actions: ReplayAction[]
}

/**
 * Represents a single round of combat.
 * Contains all turns that occurred during that round.
 */
export interface ReplayRound {
  /** The round number (1-indexed) */
  roundNumber: number
  /** All turns that occurred during this round, in chronological order */
  turns: ReplayTurn[]
}

/**
 * Represents a complete combat encounter replay.
 * Contains all rounds and optionally any events outside round boundaries.
 */
export interface Replay {
  /** All rounds of the encounter, in chronological order */
  rounds: ReplayRound[]
  /** Events that occurred outside of round boundaries (e.g., EncounterStarted, EncounterEnded) */
  globalEvents: Event[]
  /** Precomputed metadata for efficient navigation */
  metadata: ReplayMetadata
}

/**
 * Precomputed metadata for efficient replay navigation and lookup.
 *
 * All arrays are indexed by round number (0-based). This metadata enables O(1)
 * conversion between flat indices and hierarchical coordinates.
 *
 * ## Index Relationships
 *
 * - `roundOffsets[r]` = first flat index of round r
 * - `roundActionCounts[r]` = total actions in round r
 * - `turnOffsets[r][t]` = first flat index of turn t within round r (relative to round start)
 * - `turnActionCounts[r][t]` = actions in turn t within round r
 *
 * ## Flat Index Computation
 *
 * ```
 * flatIndex = roundOffsets[roundIndex] + turnOffsets[roundIndex][turnIndex] + actionIndex
 * ```
 *
 * ## Reverse Lookup (Flat → Hierarchical)
 *
 * Uses binary search on `roundOffsets` to find containing round, then linear
 * search within round for turn boundaries.
 */
export interface ReplayMetadata {
  /** Total number of actions across all rounds and turns */
  totalActions: number
  /** Total number of turns across all rounds */
  totalTurns: number
  /** Flat index offset where each round starts (roundIndex -> first action index) */
  roundOffsets: number[]
  /** Cumulative action counts per round (roundIndex -> action count) */
  roundActionCounts: number[]
  /** Flat index offset where each turn starts within its round (roundIndex -> turnIndex -> first action index) */
  turnOffsets: number[][]
  /** Action counts per turn within each round (roundIndex -> turnIndex -> action count) */
  turnActionCounts: number[][]
}

/**
 * Parsing state tracked during event processing.
 */
export interface ParseState {
  /** The current replay being built */
  replay: Replay
  /** The current round being built (if any) */
  currentRound: ReplayRound | null
  /** The current turn being built (if any) */
  currentTurn: ReplayTurn | null
  /** The current action being built (if any) */
  currentAction: ReplayAction | null
}
