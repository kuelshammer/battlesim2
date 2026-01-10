import type { Event } from './model'
import type {
  Replay,
  ReplayRound,
  ReplayTurn,
  ReplayAction,
  ParseState
} from './replayTypes'

/**
 * Parses a chronological array of events into a structured replay representation.
 *
 * The parser groups events by:
 * - Round: RoundStarted / RoundEnded events define boundaries
 * - Turn: TurnStarted / TurnEnded events define boundaries
 * - Action: ActionStarted / ActionSkipped start an action context where sub-events are collected
 *
 * Events outside of expected boundaries (e.g., EncounterStarted, EncounterEnded) are
 * collected into globalEvents for reference.
 *
 * @param events - Chronological array of combat events
 * @returns A structured Replay containing rounds and global events
 */
export function parseEventsToReplay(events: Event[]): Replay {
  const state: ParseState = {
    replay: {
      rounds: [],
      globalEvents: []
    },
    currentRound: null,
    currentTurn: null,
    currentAction: null
  }

  for (const event of events) {
    processEvent(event, state)
  }

  // Close any open contexts
  closeCurrentAction(state)
  closeCurrentTurn(state)
  closeCurrentRound(state)

  return state.replay
}

/**
 * Processes a single event and updates the parsing state accordingly.
 */
function processEvent(event: Event, state: ParseState): void {
  switch (event.type) {
    // Life cycle boundary events
    case 'EncounterStarted':
    case 'EncounterEnded':
      // Add to global events (outside rounds)
      state.replay.globalEvents.push(event)
      break

    case 'RoundStarted':
      closeCurrentRound(state)
      startRound(event.round_number, state)
      break

    case 'RoundEnded':
      closeCurrentRound(state)
      break

    case 'TurnStarted':
      closeCurrentTurn(state)
      startTurn(event.unit_id, event.round_number, state)
      break

    case 'TurnEnded':
      closeCurrentTurn(state)
      break

    // Action boundary events
    case 'ActionStarted':
      closeCurrentAction(state)
      startAction(event.actor_id, event.action_id, state)
      // ActionStarted itself goes into sub-events
      addEventToCurrentContext(event, state)
      break

    case 'ActionSkipped':
      closeCurrentAction(state)
      startAction(event.actor_id, event.action_id, state)
      addEventToCurrentContext(event, state)
      closeCurrentAction(state) // Skipped actions have no sub-events
      break

    // All other events go into the current context (action, turn, or round)
    default:
      addEventToCurrentContext(event, state)
      break
  }
}

/**
 * Starts a new round in the parsing state.
 */
function startRound(roundNumber: number, state: ParseState): void {
  state.currentRound = {
    roundNumber,
    turns: []
  }
}

/**
 * Closes the current round and adds it to the replay if it exists.
 */
function closeCurrentRound(state: ParseState): void {
  if (!state.currentRound) return

  // Close any remaining turn/action
  closeCurrentAction(state)
  closeCurrentTurn(state)

  // Only add non-empty rounds
  if (state.currentRound.turns.length > 0) {
    state.replay.rounds.push(state.currentRound)
  }

  state.currentRound = null
}

/**
 * Starts a new turn in the parsing state.
 */
function startTurn(unitId: string, roundNumber: number, state: ParseState): void {
  if (!state.currentRound) {
    // Turn started without a round - create a round first
    startRound(roundNumber, state)
  }

  state.currentTurn = {
    unitId,
    roundNumber,
    actions: []
  }
}

/**
 * Closes the current turn and adds it to the current round if it exists.
 */
function closeCurrentTurn(state: ParseState): void {
  if (!state.currentTurn || !state.currentRound) return

  // Close any remaining action
  closeCurrentAction(state)

  // Only add non-empty turns
  if (state.currentTurn.actions.length > 0) {
    state.currentRound.turns.push(state.currentTurn)
  }

  state.currentTurn = null
}

/**
 * Starts a new action in the parsing state.
 */
function startAction(actorId: string, actionId: string, state: ParseState): void {
  if (!state.currentTurn) {
    // Action started without a turn - events will go to globalEvents
    return
  }

  state.currentAction = {
    actorId,
    actionId,
    subEvents: []
  }
}

/**
 * Closes the current action and adds it to the current turn if it exists.
 */
function closeCurrentAction(state: ParseState): void {
  if (!state.currentAction || !state.currentTurn) return

  // Only add non-empty actions
  if (state.currentAction.subEvents.length > 0) {
    state.currentTurn.actions.push(state.currentAction)
  }

  state.currentAction = null
}

/**
 * Adds an event to the current context (action, turn, round, or global).
 * Priority: Action > Turn > Round > Global
 */
function addEventToCurrentContext(event: Event, state: ParseState): void {
  // Try to add to current action
  if (state.currentAction) {
    state.currentAction.subEvents.push(event)
    return
  }

  // Try to add to current turn
  if (state.currentTurn) {
    // Events outside an action context are added as a standalone "no-action" action
    // This handles events like buffs applied outside of actions
    if (shouldCreateImplicitAction(event)) {
      state.currentTurn.actions.push({
        actorId: state.currentTurn.unitId,
        actionId: 'implicit',
        subEvents: [event]
      })
    }
    return
  }

  // Try to add to current round (as an implicit turn)
  if (state.currentRound) {
    // Events outside a turn context are added as an implicit turn
    // This is rare but handles edge cases
    const unitId = extractUnitIdFromEvent(event)
    if (unitId) {
      state.currentRound.turns.push({
        unitId,
        roundNumber: state.currentRound.roundNumber,
        actions: [{
          actorId: unitId,
          actionId: 'implicit',
          subEvents: [event]
        }]
      })
    }
    return
  }

  // Fall back to global events
  state.replay.globalEvents.push(event)
}

/**
 * Determines if an event should create an implicit action context.
 * Some events (like BuffApplied, ConditionAdded) make sense as standalone actions.
 */
function shouldCreateImplicitAction(event: Event): boolean {
  // Events that make sense as standalone actions
  const standaloneEventTypes = [
    'BuffApplied',
    'BuffExpired',
    'ConditionAdded',
    'ConditionRemoved',
    'HealingApplied',
    'TempHPGranted',
    'SpellCast',
    'ConcentrationBroken'
  ]

  return standaloneEventTypes.includes(event.type)
}

/**
 * Extracts a unit ID from an event if possible.
 * Used for creating implicit turns when events appear outside turn boundaries.
 */
function extractUnitIdFromEvent(event: Event): string | null {
  // Try common ID fields
  if ('unit_id' in event) return event.unit_id as string
  if ('actor_id' in event) return event.actor_id
  if ('attacker_id' in event) return event.attacker_id
  if ('target_id' in event) return event.target_id
  if ('caster_id' in event) return event.caster_id
  if ('source_id' in event) return event.source_id

  return null
}

/**
 * Legacy function for backward compatibility.
 * Parses events and returns only the rounds array.
 * @deprecated Use parseEventsToReplay instead for full replay data.
 */
export function parseEventsToReplayRounds(events: Event[]): ReplayRound[] {
  const replay = parseEventsToReplay(events)
  return replay.rounds
}
