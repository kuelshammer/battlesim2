import { describe, it, expect, beforeEach } from 'vitest'
import { parseEventsToReplay } from './replayParser'
import {
  getActionByFlatIndex,
  getNextAction,
  getPrevAction,
  getActionsForRound,
  getActionRangeForRound,
  getFirstActionIndexForRound,
  getLastActionIndexForRound,
  indicesToFlatIndex,
  flatIndexToIndices
} from './replaySelectors'
import type { Event } from './model'

// Helper to create test events
function createEvent(type: string, data: Record<string, any> = {}): Event {
  return { type, ...data } as Event
}

describe('replaySelectors', () => {
  let replay: ReturnType<typeof parseEventsToReplay>

  beforeEach(() => {
    // Create a test replay with known structure:
    // Round 1: 2 turns, 3 actions total (turn1: 2 actions, turn2: 1 action)
    // Round 2: 1 turn, 1 action
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

  describe('getActionByFlatIndex', () => {
    it('returns correct action for valid indices', () => {
      const action0 = getActionByFlatIndex(replay, 0)
      expect(action0).toBeTruthy()
      expect(action0!.action.actionId).toBe('a1')
      expect(action0!.roundIndex).toBe(0)
      expect(action0!.turnIndex).toBe(0)

      const action2 = getActionByFlatIndex(replay, 2)
      expect(action2).toBeTruthy()
      expect(action2!.action.actionId).toBe('a3')
      expect(action2!.roundIndex).toBe(0)
      expect(action2!.turnIndex).toBe(1)

      const action3 = getActionByFlatIndex(replay, 3)
      expect(action3).toBeTruthy()
      expect(action3!.action.actionId).toBe('a4')
      expect(action3!.roundIndex).toBe(1)
      expect(action3!.turnIndex).toBe(0)
    })

    it('returns null for invalid indices', () => {
      expect(getActionByFlatIndex(replay, -1)).toBeNull()
      expect(getActionByFlatIndex(replay, 4)).toBeNull()
      expect(getActionByFlatIndex(replay, 100)).toBeNull()
    })

    it('performs O(1) lookup for large replays', () => {
      // Create a large replay
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

      const largeReplay = parseEventsToReplay(largeEvents) // 50 rounds * 3 turns * 4 actions = 600 actions

      const startTime = performance.now()
      const action = getActionByFlatIndex(largeReplay, 599) // Last action
      const endTime = performance.now()

      expect(action).toBeTruthy()
      expect(action!.action.actionId).toBe('a4')
      // Should be very fast (< 1ms)
      expect(endTime - startTime).toBeLessThan(1)
    })
  })

  describe('getNextAction and getPrevAction', () => {
    it('navigates correctly within bounds', () => {
      const action1 = getNextAction(replay, 0)
      expect(action1).toBeTruthy()
      expect(action1!.index).toBe(1)

      const action2 = getNextAction(replay, 1)
      expect(action2).toBeTruthy()
      expect(action2!.index).toBe(2)

      const prevAction = getPrevAction(replay, 2)
      expect(prevAction).toBeTruthy()
      expect(prevAction!.index).toBe(1)
    })

    it('returns null at boundaries', () => {
      expect(getNextAction(replay, 3)).toBeNull() // Last action
      expect(getPrevAction(replay, 0)).toBeNull() // First action
    })
  })

  describe('getActionsForRound', () => {
    it('returns all actions for a valid round', () => {
      const round0Actions = getActionsForRound(replay, 0)
      expect(round0Actions).toHaveLength(3)
      expect(round0Actions[0].action.actionId).toBe('a1')
      expect(round0Actions[1].action.actionId).toBe('a2')
      expect(round0Actions[2].action.actionId).toBe('a3')

      const round1Actions = getActionsForRound(replay, 1)
      expect(round1Actions).toHaveLength(1)
      expect(round1Actions[0].action.actionId).toBe('a4')
    })

    it('returns empty array for invalid round', () => {
      expect(getActionsForRound(replay, -1)).toHaveLength(0)
      expect(getActionsForRound(replay, 2)).toHaveLength(0)
    })
  })

  describe('getActionRangeForRound', () => {
    it('returns correct ranges for valid rounds', () => {
      const range0 = getActionRangeForRound(replay, 0)
      expect(range0).toEqual({ start: 0, end: 2 })

      const range1 = getActionRangeForRound(replay, 1)
      expect(range1).toEqual({ start: 3, end: 3 })
    })

    it('returns null for invalid rounds', () => {
      expect(getActionRangeForRound(replay, -1)).toBeNull()
      expect(getActionRangeForRound(replay, 2)).toBeNull()
    })
  })

  describe('getFirstActionIndexForRound and getLastActionIndexForRound', () => {
    it('returns correct indices for valid rounds', () => {
      expect(getFirstActionIndexForRound(replay, 0)).toBe(0)
      expect(getLastActionIndexForRound(replay, 0)).toBe(2)
      expect(getFirstActionIndexForRound(replay, 1)).toBe(3)
      expect(getLastActionIndexForRound(replay, 1)).toBe(3)
    })

    it('returns -1 for invalid rounds', () => {
      expect(getFirstActionIndexForRound(replay, -1)).toBe(-1)
      expect(getLastActionIndexForRound(replay, 2)).toBe(-1)
    })
  })

  describe('indicesToFlatIndex and flatIndexToIndices', () => {
    it('converts correctly between index formats', () => {
      // Test round 0, turn 0, action 0 -> index 0
      expect(indicesToFlatIndex(replay, 0, 0, 0)).toBe(0)

      // Test round 0, turn 0, action 1 -> index 1
      expect(indicesToFlatIndex(replay, 0, 0, 1)).toBe(1)

      // Test round 0, turn 1, action 0 -> index 2
      expect(indicesToFlatIndex(replay, 0, 1, 0)).toBe(2)

      // Test round 1, turn 0, action 0 -> index 3
      expect(indicesToFlatIndex(replay, 1, 0, 0)).toBe(3)

      // Reverse conversions
      expect(flatIndexToIndices(replay, 0)).toEqual({ roundIndex: 0, turnIndex: 0, actionIndex: 0 })
      expect(flatIndexToIndices(replay, 1)).toEqual({ roundIndex: 0, turnIndex: 0, actionIndex: 1 })
      expect(flatIndexToIndices(replay, 2)).toEqual({ roundIndex: 0, turnIndex: 1, actionIndex: 0 })
      expect(flatIndexToIndices(replay, 3)).toEqual({ roundIndex: 1, turnIndex: 0, actionIndex: 0 })
    })

    it('returns -1 for invalid indices', () => {
      expect(indicesToFlatIndex(replay, -1, 0, 0)).toBe(-1)
      expect(indicesToFlatIndex(replay, 0, -1, 0)).toBe(-1)
      expect(indicesToFlatIndex(replay, 0, 0, -1)).toBe(-1)
      expect(indicesToFlatIndex(replay, 2, 0, 0)).toBe(-1) // Round out of bounds
      expect(indicesToFlatIndex(replay, 0, 3, 0)).toBe(-1) // Turn out of bounds
      expect(indicesToFlatIndex(replay, 0, 0, 3)).toBe(-1) // Action out of bounds
    })

    it('returns null for invalid flat indices', () => {
      expect(flatIndexToIndices(replay, -1)).toBeNull()
      expect(flatIndexToIndices(replay, 4)).toBeNull()
    })
  })
})