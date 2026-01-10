import { useState, useCallback, useMemo, useRef, useEffect } from 'react'
import type { Replay, ReplayAction } from '@/model/replayTypes'

export interface FlattenedAction {
  /** Flattened index across all actions in the replay */
  index: number
  /** Round number this action belongs to */
  roundNumber: number
  /** Round index (0-based) */
  roundIndex: number
  /** Unit ID whose turn this is */
  unitId: string
  /** Turn index within the round (0-based) */
  turnIndex: number
  /** Action data */
  action: ReplayAction
}

export interface UseCombatPlaybackOptions {
  /** Auto-advance playback interval in ms (default: 1000ms) */
  autoAdvanceInterval?: number
  /** Callback when action changes */
  onActionChange?: (action: FlattenedAction | null) => void
}

export interface UseCombatPlaybackResult {
  /** Current state */
  currentRoundIndex: number
  currentTurnIndex: number
  currentActionIndex: number
  currentAction: FlattenedAction | null
  isPlaying: boolean

  /** Flattened list of all actions */
  actions: FlattenedAction[]
  totalActions: number
  totalRounds: number
  progress: number

  /** Navigation controls */
  nextAction: () => void
  prevAction: () => void
  play: () => void
  pause: () => void
  togglePlay: () => void
  seek: (actionIndex: number) => void
  seekToStart: () => void
  seekToEnd: () => void
  seekToRound: (roundIndex: number) => void
}

/**
 * Hook for managing combat replay playback state.
 *
 * Provides a flattened view of all actions across all rounds and turns,
 * making it easy to navigate through the combat chronologically.
 *
 * @param replay - The structured replay data
 * @param options - Playback options
 * @returns Playback state and controls
 */
export function useCombatPlayback(
  replay: Replay | null,
  options: UseCombatPlaybackOptions = {}
): UseCombatPlaybackResult {
  const {
    autoAdvanceInterval = 1000,
    onActionChange
  } = options

  // State
  const [currentRoundIndex, setCurrentRoundIndex] = useState(0)
  const [currentTurnIndex, setCurrentTurnIndex] = useState(0)
  const [currentActionIndex, setCurrentActionIndex] = useState(0)
  const [isPlaying, setIsPlaying] = useState(false)

  // Refs for interval tracking
  const intervalRef = useRef<number | null>(null)
  const onActionChangeRef = useRef(onActionChange)

  // Update callback ref
  useEffect(() => {
    onActionChangeRef.current = onActionChange
  }, [onActionChange])

  // Flatten all actions into a chronological list
  const actions: FlattenedAction[] = useMemo(() => {
    if (!replay) return []

    const flattened: FlattenedAction[] = []
    let globalIndex = 0

    for (let rIndex = 0; rIndex < replay.rounds.length; rIndex++) {
      const round = replay.rounds[rIndex]
      for (let tIndex = 0; tIndex < round.turns.length; tIndex++) {
        const turn = round.turns[tIndex]
        for (let aIndex = 0; aIndex < turn.actions.length; aIndex++) {
          const action = turn.actions[aIndex]
          flattened.push({
            index: globalIndex++,
            roundNumber: round.roundNumber,
            roundIndex: rIndex,
            unitId: turn.unitId,
            turnIndex: tIndex,
            action
          })
        }
      }
    }

    return flattened
  }, [replay])

  const totalActions = actions.length
  const totalRounds = replay?.rounds.length || 0

  // Get current action based on indices
  const currentAction: FlattenedAction | null = useMemo(() => {
    if (!replay || actions.length === 0) return null

    // Find the action at the current position
    // We need to rebuild the path from indices to find the correct action
    const round = replay.rounds[currentRoundIndex]
    if (!round) {
      // If round index is out of bounds, use the last action
      return actions[actions.length - 1]
    }

    const turn = round.turns[currentTurnIndex]
    if (!turn) {
      // If turn index is out of bounds, find the last action in this round
      const lastActionInRound = actions.filter(a => a.roundIndex === currentRoundIndex).pop()
      return lastActionInRound || null
    }

    const action = turn.actions[currentActionIndex]
    if (!action) {
      // If action index is out of bounds, find the last action in this turn
      const lastActionInTurn = actions.find(
        a => a.roundIndex === currentRoundIndex && a.turnIndex === currentTurnIndex
      )
      // If not found, find the last action before this position
      if (!lastActionInTurn) {
        const candidates = actions.filter(
          a => a.roundIndex < currentRoundIndex ||
          (a.roundIndex === currentRoundIndex && a.turnIndex < currentTurnIndex)
        )
        return candidates[candidates.length - 1] || null
      }
      return lastActionInTurn
    }

    // Find the matching flattened action
    return actions.find(
      a => a.roundIndex === currentRoundIndex &&
          a.turnIndex === currentTurnIndex &&
          a.action === action
    ) || null
  }, [replay, actions, currentRoundIndex, currentTurnIndex, currentActionIndex])

  // Navigation controls
  const nextAction = useCallback(() => {
    const currentFlatIndex = currentAction?.index ?? -1
    if (currentFlatIndex < totalActions - 1) {
      const next = actions[currentFlatIndex + 1]
      if (next) {
        setCurrentRoundIndex(next.roundIndex)
        setCurrentTurnIndex(next.turnIndex)
        // Calculate action index within the turn
        const turn = replay?.rounds[next.roundIndex]?.turns[next.turnIndex]
        if (turn) {
          const actionInTurnIndex = turn.actions.findIndex(a => a === next.action)
          setCurrentActionIndex(actionInTurnIndex >= 0 ? actionInTurnIndex : 0)
        }
      }
    }
  }, [currentAction, actions, totalActions, replay])

  const prevAction = useCallback(() => {
    const currentFlatIndex = currentAction?.index ?? 0
    if (currentFlatIndex > 0) {
      const prev = actions[currentFlatIndex - 1]
      if (prev) {
        setCurrentRoundIndex(prev.roundIndex)
        setCurrentTurnIndex(prev.turnIndex)
        const turn = replay?.rounds[prev.roundIndex]?.turns[prev.turnIndex]
        if (turn) {
          const actionInTurnIndex = turn.actions.findIndex(a => a === prev.action)
          setCurrentActionIndex(actionInTurnIndex >= 0 ? actionInTurnIndex : 0)
        }
      }
    }
  }, [currentAction, actions, replay])

  // Auto-play logic
  useEffect(() => {
    if (isPlaying && intervalRef.current === null) {
      intervalRef.current = window.setInterval(() => {
        const currentFlatIndex = currentAction?.index ?? -1
        if (currentFlatIndex >= totalActions - 1) {
          // Reached the end, pause
          setIsPlaying(false)
          return
        }
        // Call nextAction to handle round/turn boundaries properly
        nextAction()
      }, autoAdvanceInterval)
    } else if (!isPlaying && intervalRef.current !== null) {
      clearInterval(intervalRef.current)
      intervalRef.current = null
    }

    return () => {
      if (intervalRef.current !== null) {
        clearInterval(intervalRef.current)
        intervalRef.current = null
      }
    }
  }, [isPlaying, autoAdvanceInterval, currentAction, totalActions, nextAction])

  // Trigger callback when action changes
  useEffect(() => {
    if (onActionChangeRef.current) {
      onActionChangeRef.current(currentAction)
    }
  }, [currentAction])

  // Calculate progress (0-1)
  const progress = useMemo(() => {
    if (totalActions === 0) return 0
    return (currentAction?.index ?? 0) / (totalActions - 1)
  }, [currentAction, totalActions])

  const play = useCallback(() => {
    setIsPlaying(true)
  }, [])

  const pause = useCallback(() => {
    setIsPlaying(false)
  }, [])

  const togglePlay = useCallback(() => {
    setIsPlaying(prev => !prev)
  }, [])

  const seek = useCallback((actionIndex: number) => {
    const target = actions[actionIndex]
    if (target) {
      setCurrentRoundIndex(target.roundIndex)
      setCurrentTurnIndex(target.turnIndex)
      const turn = replay?.rounds[target.roundIndex]?.turns[target.turnIndex]
      if (turn) {
        const actionInTurnIndex = turn.actions.findIndex(a => a === target.action)
        setCurrentActionIndex(actionInTurnIndex >= 0 ? actionInTurnIndex : 0)
      }
    }
  }, [actions, replay])

  const seekToStart = useCallback(() => {
    setCurrentRoundIndex(0)
    setCurrentTurnIndex(0)
    setCurrentActionIndex(0)
  }, [])

  const seekToEnd = useCallback(() => {
    const last = actions[actions.length - 1]
    if (last) {
      setCurrentRoundIndex(last.roundIndex)
      setCurrentTurnIndex(last.turnIndex)
      const turn = replay?.rounds[last.roundIndex]?.turns[last.turnIndex]
      if (turn) {
        setCurrentActionIndex(turn.actions.length - 1)
      }
    }
  }, [actions, replay])

  const seekToRound = useCallback((roundIndex: number) => {
    const firstActionInRound = actions.find(a => a.roundIndex === roundIndex)
    if (firstActionInRound) {
      seek(firstActionInRound.index)
    }
  }, [actions, seek])

  return {
    currentRoundIndex,
    currentTurnIndex,
    currentActionIndex,
    currentAction,
    isPlaying,
    actions,
    totalActions,
    totalRounds,
    progress,
    nextAction,
    prevAction,
    play,
    pause,
    togglePlay,
    seek,
    seekToStart,
    seekToEnd,
    seekToRound
  }
}
