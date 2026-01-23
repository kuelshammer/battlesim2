import type { Replay, ReplayAction } from '@/model/replayTypes'
import type { FlattenedAction } from '@/hooks/useCombatPlayback'

/**
 * Gets the next action in the replay from the current flat index.
 * Returns null if at the end.
 */
export function getNextAction(replay: Replay, currentFlatIndex: number): FlattenedAction | null {
  if (currentFlatIndex >= replay.metadata.totalActions - 1) {
    return null
  }

  return getActionByFlatIndex(replay, currentFlatIndex + 1)
}

/**
 * Gets the previous action in the replay from the current flat index.
 * Returns null if at the beginning.
 */
export function getPrevAction(replay: Replay, currentFlatIndex: number): FlattenedAction | null {
  if (currentFlatIndex <= 0) {
    return null
  }

  return getActionByFlatIndex(replay, currentFlatIndex - 1)
}

/**
 * Gets an action by its flat index across the entire replay.
 * Uses precomputed metadata for O(1) lookup.
 */
export function getActionByFlatIndex(replay: Replay, flatIndex: number): FlattenedAction | null {
  if (flatIndex < 0 || flatIndex >= replay.metadata.totalActions) {
    return null
  }

  // Find which round this index belongs to using binary search on roundOffsets
  let roundIndex = 0
  for (let i = 0; i < replay.rounds.length; i++) {
    if (replay.metadata.roundOffsets[i] <= flatIndex) {
      roundIndex = i
    } else {
      break
    }
  }

  const round = replay.rounds[roundIndex]
  if (!round) return null

  // Calculate the action index within this round
  const actionIndexInRound = flatIndex - replay.metadata.roundOffsets[roundIndex]

  // Find which turn this action belongs to
  let turnIndex = 0
  let cumulativeActions = 0
  for (let i = 0; i < round.turns.length; i++) {
    if (cumulativeActions + replay.metadata.turnActionCounts[roundIndex][i] > actionIndexInRound) {
      turnIndex = i
      break
    }
    cumulativeActions += replay.metadata.turnActionCounts[roundIndex][i]
  }

  const turn = round.turns[turnIndex]
  if (!turn) return null

  // Calculate action index within the turn
  const actionIndexInTurn = actionIndexInRound - cumulativeActions
  const action = turn.actions[actionIndexInTurn]

  if (!action) return null

  return {
    index: flatIndex,
    roundNumber: round.roundNumber,
    roundIndex,
    unitId: turn.unitId,
    turnIndex,
    action
  }
}

/**
 * Gets all actions within a specific round.
 * Returns empty array if round index is invalid.
 */
export function getActionsForRound(replay: Replay, roundIndex: number): FlattenedAction[] {
  if (roundIndex < 0 || roundIndex >= replay.rounds.length) {
    return []
  }

  const round = replay.rounds[roundIndex]
  const startIndex = replay.metadata.roundOffsets[roundIndex]
  const endIndex = startIndex + replay.metadata.roundActionCounts[roundIndex]

  const actions: FlattenedAction[] = []
  for (let i = startIndex; i < endIndex; i++) {
    const action = getActionByFlatIndex(replay, i)
    if (action) {
      actions.push(action)
    }
  }

  return actions
}

/**
 * Gets the range of flat indices for a specific round.
 * Returns null if round index is invalid.
 */
export function getActionRangeForRound(replay: Replay, roundIndex: number): { start: number, end: number } | null {
  if (roundIndex < 0 || roundIndex >= replay.rounds.length) {
    return null
  }

  const start = replay.metadata.roundOffsets[roundIndex]
  const end = start + replay.metadata.roundActionCounts[roundIndex] - 1

  return { start, end }
}

/**
 * Finds the flat index of the first action in a specific round.
 * Returns -1 if round index is invalid.
 */
export function getFirstActionIndexForRound(replay: Replay, roundIndex: number): number {
  if (roundIndex < 0 || roundIndex >= replay.rounds.length) {
    return -1
  }

  return replay.metadata.roundOffsets[roundIndex]
}

/**
 * Finds the flat index of the last action in a specific round.
 * Returns -1 if round index is invalid.
 */
export function getLastActionIndexForRound(replay: Replay, roundIndex: number): number {
  const range = getActionRangeForRound(replay, roundIndex)
  return range ? range.end : -1
}

/**
 * Converts round/turn/action indices to a flat index.
 * Returns -1 if any indices are invalid.
 */
export function indicesToFlatIndex(replay: Replay, roundIndex: number, turnIndex: number, actionIndex: number): number {
  if (roundIndex < 0 || roundIndex >= replay.rounds.length) {
    return -1
  }

  const round = replay.rounds[roundIndex]
  if (turnIndex < 0 || turnIndex >= round.turns.length) {
    return -1
  }

  const turn = round.turns[turnIndex]
  if (actionIndex < 0 || actionIndex >= turn.actions.length) {
    return -1
  }

  let flatIndex = replay.metadata.roundOffsets[roundIndex]
  for (let t = 0; t < turnIndex; t++) {
    flatIndex += replay.metadata.turnActionCounts[roundIndex][t]
  }
  flatIndex += actionIndex

  return flatIndex
}

/**
 * Converts a flat index to round/turn/action indices.
 * Returns null if the flat index is invalid.
 */
export function flatIndexToIndices(replay: Replay, flatIndex: number): { roundIndex: number, turnIndex: number, actionIndex: number } | null {
  if (flatIndex < 0 || flatIndex >= replay.metadata.totalActions) {
    return null
  }

  // Find round
  let roundIndex = 0
  for (let i = 0; i < replay.rounds.length; i++) {
    if (replay.metadata.roundOffsets[i] <= flatIndex) {
      roundIndex = i
    } else {
      break
    }
  }

  const actionIndexInRound = flatIndex - replay.metadata.roundOffsets[roundIndex]

  // Find turn within round
  let turnIndex = 0
  let cumulativeActions = 0
  for (let i = 0; i < replay.rounds[roundIndex].turns.length; i++) {
    if (cumulativeActions + replay.metadata.turnActionCounts[roundIndex][i] > actionIndexInRound) {
      turnIndex = i
      break
    }
    cumulativeActions += replay.metadata.turnActionCounts[roundIndex][i]
  }

  const actionIndexInTurn = actionIndexInRound - cumulativeActions

  return { roundIndex, turnIndex, actionIndex: actionIndexInTurn }
}