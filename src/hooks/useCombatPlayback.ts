import { useState, useCallback, useMemo, useRef, useEffect, useReducer } from 'react'
import type { Replay, ReplayAction } from '@/model/replayTypes'
import {
  getActionByFlatIndex,
  getNextAction,
  getPrevAction,
  getFirstActionIndexForRound,
  flatIndexToIndices
} from '@/model/replaySelectors'

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

// Playback state managed by reducer
interface PlaybackState {
  currentFlatIndex: number
  isPlaying: boolean
}

type PlaybackAction =
  | { type: 'SEEK'; flatIndex: number }
  | { type: 'NEXT' }
  | { type: 'PREV' }
  | { type: 'PLAY' }
  | { type: 'PAUSE' }
  | { type: 'TOGGLE_PLAY' }
  | { type: 'SEEK_TO_START' }
  | { type: 'SEEK_TO_END' }
  | { type: 'SEEK_TO_ROUND'; roundIndex: number }

function playbackReducer(state: PlaybackState, action: PlaybackAction, replay: Replay | null): PlaybackState {
  const totalActions = replay?.metadata.totalActions ?? 0

  switch (action.type) {
    case 'SEEK':
      return {
        ...state,
        currentFlatIndex: Math.max(0, Math.min(action.flatIndex, totalActions - 1))
      }

    case 'NEXT':
      const nextIndex = Math.min(state.currentFlatIndex + 1, totalActions - 1)
      return { ...state, currentFlatIndex: nextIndex }

    case 'PREV':
      const prevIndex = Math.max(state.currentFlatIndex - 1, 0)
      return { ...state, currentFlatIndex: prevIndex }

    case 'PLAY':
      return { ...state, isPlaying: true }

    case 'PAUSE':
      return { ...state, isPlaying: false }

    case 'TOGGLE_PLAY':
      return { ...state, isPlaying: !state.isPlaying }

    case 'SEEK_TO_START':
      return { ...state, currentFlatIndex: 0 }

    case 'SEEK_TO_END':
      return { ...state, currentFlatIndex: Math.max(0, totalActions - 1) }

    case 'SEEK_TO_ROUND':
      const roundStartIndex = getFirstActionIndexForRound(replay!, action.roundIndex)
      if (roundStartIndex >= 0) {
        return { ...state, currentFlatIndex: roundStartIndex }
      }
      return state

    default:
      return state
  }
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

  // Use reducer for playback state
  const [state, dispatch] = useReducer(
    (state: PlaybackState, action: PlaybackAction) => playbackReducer(state, action, replay),
    { currentFlatIndex: 0, isPlaying: false }
  )

  // Refs for interval tracking
  const intervalRef = useRef<number | null>(null)
  const onActionChangeRef = useRef(onActionChange)

  // Update callback ref
  useEffect(() => {
    onActionChangeRef.current = onActionChange
  }, [onActionChange])

  // Get total counts from metadata
  const totalActions = replay?.metadata.totalActions ?? 0
  const totalRounds = replay?.rounds.length || 0

  // Memoize flattened actions list using selectors for O(1) access
  const actions: FlattenedAction[] = useMemo(() => {
    if (!replay || totalActions === 0) return []

    const flattened: FlattenedAction[] = []
    for (let i = 0; i < totalActions; i++) {
      const action = getActionByFlatIndex(replay, i)
      if (action) {
        flattened.push(action)
      }
    }
    return flattened
  }, [replay, totalActions])

  // Get current action using O(1) selector
  const currentAction: FlattenedAction | null = useMemo(() => {
    if (!replay || totalActions === 0) return null
    return getActionByFlatIndex(replay, state.currentFlatIndex)
  }, [replay, state.currentFlatIndex, totalActions])

  // Derive round/turn/action indices from flat index
  const currentIndices = useMemo(() => {
    if (!replay) return { roundIndex: 0, turnIndex: 0, actionIndex: 0 }
    const indices = flatIndexToIndices(replay, state.currentFlatIndex)
    return indices || { roundIndex: 0, turnIndex: 0, actionIndex: 0 }
  }, [replay, state.currentFlatIndex])

  // Navigation controls
  const nextAction = useCallback(() => {
    dispatch({ type: 'NEXT' })
  }, [])

  const prevAction = useCallback(() => {
    dispatch({ type: 'PREV' })
  }, [])

  // Auto-play logic
  useEffect(() => {
    if (state.isPlaying && intervalRef.current === null) {
      intervalRef.current = window.setInterval(() => {
        if (state.currentFlatIndex >= totalActions - 1) {
          // Reached the end, pause
          dispatch({ type: 'PAUSE' })
          return
        }
        // Advance to next action
        dispatch({ type: 'NEXT' })
      }, autoAdvanceInterval)
    } else if (!state.isPlaying && intervalRef.current !== null) {
      clearInterval(intervalRef.current)
      intervalRef.current = null
    }

    return () => {
      if (intervalRef.current !== null) {
        clearInterval(intervalRef.current)
        intervalRef.current = null
      }
    }
  }, [state.isPlaying, state.currentFlatIndex, autoAdvanceInterval, totalActions, dispatch])

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
    dispatch({ type: 'PLAY' })
  }, [])

  const pause = useCallback(() => {
    dispatch({ type: 'PAUSE' })
  }, [])

  const togglePlay = useCallback(() => {
    dispatch({ type: 'TOGGLE_PLAY' })
  }, [])

  const seek = useCallback((actionIndex: number) => {
    dispatch({ type: 'SEEK', flatIndex: actionIndex })
  }, [])

  const seekToStart = useCallback(() => {
    dispatch({ type: 'SEEK_TO_START' })
  }, [])

  const seekToEnd = useCallback(() => {
    dispatch({ type: 'SEEK_TO_END' })
  }, [])

  const seekToRound = useCallback((roundIndex: number) => {
    dispatch({ type: 'SEEK_TO_ROUND', roundIndex })
  }, [])

  return {
    currentRoundIndex: currentIndices.roundIndex,
    currentTurnIndex: currentIndices.turnIndex,
    currentActionIndex: currentIndices.actionIndex,
    currentAction,
    isPlaying: state.isPlaying,
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
