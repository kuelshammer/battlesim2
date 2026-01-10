import type { Event } from './model'

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
