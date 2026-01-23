import { describe, it, expect } from 'vitest'
import { parseEventsToReplay } from './replayParser'
import type { Event } from './model'

// Helper to create test events
function createEvent(type: string, data: Record<string, any> = {}): Event {
  return { type, ...data } as Event
}

describe('parseEventsToReplay', () => {
  it('parses a simple encounter with one round and one action', () => {
    const events: Event[] = [
      createEvent('EncounterStarted'),
      createEvent('RoundStarted', { round_number: 1 }),
      createEvent('TurnStarted', { unit_id: 'unit1', round_number: 1 }),
      createEvent('ActionStarted', { actor_id: 'unit1', action_id: 'attack' }),
      createEvent('AttackHit', { attacker_id: 'unit1', target_id: 'unit2', damage: 10 }),
      createEvent('ActionEnded'),
      createEvent('TurnEnded'),
      createEvent('RoundEnded'),
      createEvent('EncounterEnded')
    ]

    const replay = parseEventsToReplay(events)

    expect(replay.rounds).toHaveLength(1)
    expect(replay.rounds[0].roundNumber).toBe(1)
    expect(replay.rounds[0].turns).toHaveLength(1)
    expect(replay.rounds[0].turns[0].unitId).toBe('unit1')
    expect(replay.rounds[0].turns[0].actions).toHaveLength(1)
    expect(replay.rounds[0].turns[0].actions[0].actorId).toBe('unit1')
    expect(replay.rounds[0].turns[0].actions[0].actionId).toBe('attack')
    expect(replay.rounds[0].turns[0].actions[0].subEvents).toHaveLength(3) // ActionStarted + AttackHit + ActionEnded

    expect(replay.metadata.totalActions).toBe(1)
    expect(replay.metadata.totalTurns).toBe(1)
    expect(replay.metadata.roundOffsets).toEqual([0])
    expect(replay.metadata.roundActionCounts).toEqual([1])
  })

  it('parses multiple rounds with multiple actions', () => {
    const events: Event[] = [
      createEvent('EncounterStarted'),
      createEvent('RoundStarted', { round_number: 1 }),
      createEvent('TurnStarted', { unit_id: 'unit1', round_number: 1 }),
      createEvent('ActionStarted', { actor_id: 'unit1', action_id: 'attack1' }),
      createEvent('AttackHit', { attacker_id: 'unit1', target_id: 'unit2', damage: 10 }),
      createEvent('ActionEnded'),
      createEvent('TurnEnded'),
      createEvent('RoundEnded'),
      createEvent('RoundStarted', { round_number: 2 }),
      createEvent('TurnStarted', { unit_id: 'unit1', round_number: 2 }),
      createEvent('ActionStarted', { actor_id: 'unit1', action_id: 'attack2' }),
      createEvent('AttackHit', { attacker_id: 'unit1', target_id: 'unit2', damage: 15 }),
      createEvent('ActionEnded'),
      createEvent('TurnEnded'),
      createEvent('TurnStarted', { unit_id: 'unit2', round_number: 2 }),
      createEvent('ActionStarted', { actor_id: 'unit2', action_id: 'attack3' }),
      createEvent('AttackMiss', { attacker_id: 'unit2', target_id: 'unit1' }),
      createEvent('ActionEnded'),
      createEvent('TurnEnded'),
      createEvent('RoundEnded'),
      createEvent('EncounterEnded')
    ]

    const replay = parseEventsToReplay(events)

    expect(replay.rounds).toHaveLength(2)
    expect(replay.metadata.totalActions).toBe(3)
    expect(replay.metadata.totalTurns).toBe(3)
    expect(replay.metadata.roundOffsets).toEqual([0, 1])
    expect(replay.metadata.roundActionCounts).toEqual([1, 2])
    expect(replay.metadata.turnOffsets).toEqual([[0], [0, 1]])
    expect(replay.metadata.turnActionCounts).toEqual([[1], [1, 1]])
  })

  it('handles large replay with 1000+ actions efficiently', () => {
    // Create a large replay with 100 rounds, each with ~10 actions
    const events: Event[] = [createEvent('EncounterStarted')]

    for (let round = 1; round <= 100; round++) {
      events.push(createEvent('RoundStarted', { round_number: round }))

      for (let turn = 1; turn <= 2; turn++) {
        const unitId = `unit${turn}`
        events.push(createEvent('TurnStarted', { unit_id: unitId, round_number: round }))

        for (let action = 1; action <= 5; action++) {
          const actionId = `action${action}`
          events.push(createEvent('ActionStarted', { actor_id: unitId, action_id: actionId }))
          events.push(createEvent('AttackHit', { attacker_id: unitId, target_id: `unit${turn === 1 ? 2 : 1}`, damage: 10 }))
          events.push(createEvent('ActionEnded'))
        }

        events.push(createEvent('TurnEnded'))
      }

      events.push(createEvent('RoundEnded'))
    }

    events.push(createEvent('EncounterEnded'))

    const startTime = performance.now()
    const replay = parseEventsToReplay(events)
    const endTime = performance.now()

    // Should parse in reasonable time (less than 100ms for 1000 actions)
    expect(endTime - startTime).toBeLessThan(100)

    expect(replay.rounds).toHaveLength(100)
    expect(replay.metadata.totalActions).toBe(1000) // 100 rounds * 2 turns * 5 actions
    expect(replay.metadata.totalTurns).toBe(200)
  })

  it('computes correct metadata offsets for complex replay', () => {
    const events: Event[] = [
      createEvent('EncounterStarted'),
      // Round 1: 2 turns, 3 actions total (turn1: 2 actions, turn2: 1 action)
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
      // Round 2: 1 turn, 1 action
      createEvent('RoundStarted', { round_number: 2 }),
      createEvent('TurnStarted', { unit_id: 'unit1', round_number: 2 }),
      createEvent('ActionStarted', { actor_id: 'unit1', action_id: 'a4' }),
      createEvent('ActionEnded'),
      createEvent('TurnEnded'),
      createEvent('RoundEnded'),
      createEvent('EncounterEnded')
    ]

    const replay = parseEventsToReplay(events)

    expect(replay.metadata.totalActions).toBe(4)
    expect(replay.metadata.roundOffsets).toEqual([0, 3])
    expect(replay.metadata.roundActionCounts).toEqual([3, 1])
    expect(replay.metadata.turnOffsets).toEqual([[0, 2], [0]])
    expect(replay.metadata.turnActionCounts).toEqual([[2, 1], [1]])
  })

  it('handles events outside round boundaries', () => {
    const events: Event[] = [
      createEvent('EncounterStarted'),
      createEvent('BuffApplied', { target_id: 'unit1', buff_id: 'bless' }), // Outside rounds
      createEvent('RoundStarted', { round_number: 1 }),
      createEvent('TurnStarted', { unit_id: 'unit1', round_number: 1 }),
      createEvent('ActionStarted', { actor_id: 'unit1', action_id: 'attack' }),
      createEvent('ActionEnded'),
      createEvent('TurnEnded'),
      createEvent('RoundEnded'),
      createEvent('ConcentrationBroken', { unit_id: 'unit1' }), // Outside rounds
      createEvent('EncounterEnded')
    ]

    const replay = parseEventsToReplay(events)

    expect(replay.globalEvents).toHaveLength(4) // EncounterStarted, BuffApplied, ConcentrationBroken, EncounterEnded
    expect(replay.rounds).toHaveLength(1)
    expect(replay.rounds[0].turns[0].actions).toHaveLength(1)
  })

  it('creates implicit actions for standalone events', () => {
    const events: Event[] = [
      createEvent('EncounterStarted'),
      createEvent('RoundStarted', { round_number: 1 }),
      createEvent('TurnStarted', { unit_id: 'unit1', round_number: 1 }),
      createEvent('BuffApplied', { target_id: 'unit1', buff_id: 'bless' }), // Should create implicit action
      createEvent('TurnEnded'),
      createEvent('RoundEnded'),
      createEvent('EncounterEnded')
    ]

    const replay = parseEventsToReplay(events)

    expect(replay.rounds[0].turns[0].actions).toHaveLength(1)
    expect(replay.rounds[0].turns[0].actions[0].actionId).toBe('implicit')
    expect(replay.rounds[0].turns[0].actions[0].actorId).toBe('unit1')
    expect(replay.rounds[0].turns[0].actions[0].subEvents).toHaveLength(1)
    expect(replay.rounds[0].turns[0].actions[0].subEvents[0].type).toBe('BuffApplied')
  })
})