import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useCombatPlayback } from './useCombatPlayback'
import { parseEventsToReplay } from '@/model/replayParser'
import type { Event } from '@/model/model'

// Helper to create test events
function createEvent(type: string, data: Record<string, any> = {}): Event {
  return { type, ...data } as Event
}

describe('useCombatPlayback', () => {
  let replay: ReturnType<typeof parseEventsToReplay>

  beforeEach(() => {
    // Create a test replay with 4 actions across 2 rounds
    const events: Event[] = [
      createEvent('EncounterStarted'),
      createEvent('RoundStarted', { round_number: 1 }),
      createEvent('TurnStarted', { unit_id: 'unit1', round_number: 1 }),
      createEvent('ActionStarted', { actor_id: 'unit1', action_id: 'a1' }),
      createEvent('ActionEnded'),
      createEvent('ActionStarted', { actor_id: 'unit1', action_id: 'a2' }),
      createEvent('ActionEnded'),
      createEvent('TurnEnded'),
      createEvent('TurnStarted', { unit_id: 'unit2', round_number: 1 }),
      createEvent('ActionStarted', { actor_id: 'unit2', action_id: 'a3' }),
      createEvent('ActionEnded'),
      createEvent('TurnEnded'),
      createEvent('RoundEnded'),
      createEvent('RoundStarted', { round_number: 2 }),
      createEvent('TurnStarted', { unit_id: 'unit1', round_number: 2 }),
      createEvent('ActionStarted', { actor_id: 'unit1', action_id: 'a4' }),
      createEvent('ActionEnded'),
      createEvent('TurnEnded'),
      createEvent('RoundEnded'),
      createEvent('EncounterEnded')
    ]

    replay = parseEventsToReplay(events)
  })

  it('initializes with correct state', () => {
    const { result } = renderHook(() => useCombatPlayback(replay))

    expect(result.current.currentRoundIndex).toBe(0)
    expect(result.current.currentTurnIndex).toBe(0)
    expect(result.current.currentActionIndex).toBe(0)
    expect(result.current.currentAction).toBeTruthy()
    expect(result.current.currentAction!.action.actionId).toBe('a1')
    expect(result.current.isPlaying).toBe(false)
    expect(result.current.totalActions).toBe(4)
    expect(result.current.totalRounds).toBe(2)
    expect(result.current.actions).toHaveLength(4)
    expect(result.current.progress).toBe(0)
  })

  it('handles null replay gracefully', () => {
    const { result } = renderHook(() => useCombatPlayback(null))

    expect(result.current.currentRoundIndex).toBe(0)
    expect(result.current.currentTurnIndex).toBe(0)
    expect(result.current.currentActionIndex).toBe(0)
    expect(result.current.currentAction).toBeNull()
    expect(result.current.isPlaying).toBe(false)
    expect(result.current.totalActions).toBe(0)
    expect(result.current.totalRounds).toBe(0)
    expect(result.current.actions).toHaveLength(0)
    expect(result.current.progress).toBe(0)
  })

  describe('navigation controls', () => {
    it('nextAction advances to next action', () => {
      const { result } = renderHook(() => useCombatPlayback(replay))

      act(() => {
        result.current.nextAction()
      })

      expect(result.current.currentActionIndex).toBe(1)
      expect(result.current.currentAction!.action.actionId).toBe('a2')
      expect(result.current.progress).toBe(0.3333333333333333) // 1/3
    })

    it('prevAction goes to previous action', () => {
      const { result } = renderHook(() => useCombatPlayback(replay))

      act(() => {
        result.current.seek(2) // Go to action index 2
      })

      expect(result.current.currentAction!.action.actionId).toBe('a3')

      act(() => {
        result.current.prevAction()
      })

      expect(result.current.currentAction!.action.actionId).toBe('a2')
    })

    it('seek jumps to specific action', () => {
      const { result } = renderHook(() => useCombatPlayback(replay))

      act(() => {
        result.current.seek(3)
      })

      expect(result.current.currentActionIndex).toBe(0)
      expect(result.current.currentRoundIndex).toBe(1)
      expect(result.current.currentTurnIndex).toBe(0)
      expect(result.current.currentAction!.action.actionId).toBe('a4')
      expect(result.current.progress).toBe(1)
    })

    it('seekToStart goes to first action', () => {
      const { result } = renderHook(() => useCombatPlayback(replay))

      act(() => {
        result.current.seek(3)
        result.current.seekToStart()
      })

      expect(result.current.currentAction!.action.actionId).toBe('a1')
      expect(result.current.progress).toBe(0)
    })

    it('seekToEnd goes to last action', () => {
      const { result } = renderHook(() => useCombatPlayback(replay))

      act(() => {
        result.current.seekToEnd()
      })

      expect(result.current.currentAction!.action.actionId).toBe('a4')
      expect(result.current.progress).toBe(1)
    })

    it('seekToRound jumps to first action of round', () => {
      const { result } = renderHook(() => useCombatPlayback(replay))

      act(() => {
        result.current.seekToRound(1)
      })

      expect(result.current.currentRoundIndex).toBe(1)
      expect(result.current.currentAction!.action.actionId).toBe('a4')
    })
  })

  describe('playback controls', () => {
    beforeEach(() => {
      vi.useFakeTimers()
    })

    afterEach(() => {
      vi.restoreAllMocks()
    })

    it('play starts playback', () => {
      const { result } = renderHook(() => useCombatPlayback(replay))

      act(() => {
        result.current.play()
      })

      expect(result.current.isPlaying).toBe(true)
    })

    it('pause stops playback', () => {
      const { result } = renderHook(() => useCombatPlayback(replay))

      act(() => {
        result.current.play()
        result.current.pause()
      })

      expect(result.current.isPlaying).toBe(false)
    })

    it('togglePlay toggles playback state', () => {
      const { result } = renderHook(() => useCombatPlayback(replay))

      act(() => {
        result.current.togglePlay()
      })

      expect(result.current.isPlaying).toBe(true)

      act(() => {
        result.current.togglePlay()
      })

      expect(result.current.isPlaying).toBe(false)
    })

    it('auto-advances during playback', () => {
      const { result } = renderHook(() => useCombatPlayback(replay))

      act(() => {
        result.current.play()
      })

      expect(result.current.isPlaying).toBe(true)
      expect(result.current.currentAction!.action.actionId).toBe('a1')

      // Advance timer by 1000ms (default interval)
      act(() => {
        vi.advanceTimersByTime(1000)
      })

      expect(result.current.currentAction!.action.actionId).toBe('a2')
    })

    it('pauses at end of replay', () => {
      const { result } = renderHook(() => useCombatPlayback(replay))

      act(() => {
        result.current.seek(3) // Go to last action
        result.current.play()
      })

      // Advance timer - should pause at end
      act(() => {
        vi.advanceTimersByTime(1000)
      })

      expect(result.current.isPlaying).toBe(false)
      expect(result.current.currentAction!.action.actionId).toBe('a4')
    })
  })

  describe('performance with large replays', () => {
    it('handles 1000+ actions efficiently', () => {
      // Create large replay
      const largeEvents: Event[] = [createEvent('EncounterStarted')]
      for (let round = 1; round <= 50; round++) {
        largeEvents.push(createEvent('RoundStarted', { round_number: round }))
        for (let turn = 1; turn <= 3; turn++) {
          const unitId = `unit${turn}`
          largeEvents.push(createEvent('TurnStarted', { unit_id: unitId, round_number: round }))
          for (let action = 1; action <= 4; action++) {
            largeEvents.push(createEvent('ActionStarted', { actor_id: unitId, action_id: `a${action}` }))
            largeEvents.push(createEvent('ActionEnded'))
          }
          largeEvents.push(createEvent('TurnEnded'))
        }
        largeEvents.push(createEvent('RoundEnded'))
      }
      largeEvents.push(createEvent('EncounterEnded'))

      const largeReplay = parseEventsToReplay(largeEvents)

      const { result } = renderHook(() => useCombatPlayback(largeReplay))

      expect(result.current.totalActions).toBe(600) // 50 * 3 * 4

      // Test seeking performance
      const startTime = performance.now()
      act(() => {
        result.current.seek(599) // Last action
      })
      const endTime = performance.now()

      expect(result.current.currentAction!.action.actionId).toBe('a4')
      // Should be very fast (< 5ms)
      expect(endTime - startTime).toBeLessThan(5)
    })
  })

  describe('callbacks', () => {
    it('calls onActionChange when action changes', () => {
      const onActionChange = vi.fn()
      const { result } = renderHook(() =>
        useCombatPlayback(replay, { onActionChange })
      )

      act(() => {
        result.current.nextAction()
      })

      expect(onActionChange).toHaveBeenCalledWith(result.current.currentAction)
    })
  })
})