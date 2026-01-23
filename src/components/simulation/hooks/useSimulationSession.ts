import { useState, useEffect, useRef, useMemo } from "react"
import { z } from "zod"
import { Creature, CreatureSchema, TimelineEvent, TimelineEventSchema, EncounterResult as EncounterResultType } from "@/model/model"
import { SimulationEvent } from "@/model/events"
import { clone, useStoredState } from "@/model/utils"
import { v4 as uuid } from 'uuid'

type PropType = object

const emptyCombat: TimelineEvent = {
    type: 'combat',
    id: uuid(),
    monsters: [],
    monstersSurprised: false,
    playersSurprised: false,
    targetRole: 'Standard',
}

// Sanitization helper: Fix duplicate IDs in players array
const sanitizePlayersParser = (parser: (data: unknown) => Creature[]) => (data: unknown) => {
    const parsed = parser(data);
    if (!parsed) return null;

    const playerIds = new Set<string>();
    const sanitized = parsed.map(p => {
        if (playerIds.has(p.id)) {
            return { ...p, id: uuid() }; // Generate new ID for duplicate
        }
        playerIds.add(p.id);
        return p;
    });

    return sanitized;
};

// Sanitization helper: Fix duplicate IDs in timeline monsters
const sanitizeTimelineParser = (parser: (data: unknown) => TimelineEvent[]) => (data: unknown) => {
    const parsed = parser(data);
    if (!parsed) return null;

    return parsed.map(item => {
        if (item.type !== 'combat') return item;

        const monsterIds = new Set<string>();
        const sanitizedMonsters = item.monsters.map(m => {
            if (monsterIds.has(m.id)) {
                return { ...m, id: uuid() }; // Generate new ID for duplicate
            }
            monsterIds.add(m.id);
            return m;
        });

        return { ...item, monsters: sanitizedMonsters };
    });
};

export interface SimulationSessionState {
    players: Creature[]
    timeline: TimelineEvent[]
    isPlayersLoaded: boolean
    isTimelineLoaded: boolean
    isHighPrecisionLoaded: boolean
}

export interface SimulationSessionActions {
    setPlayers: (players: Creature[]) => void
    setTimeline: (timeline: TimelineEvent[]) => void
    createCombat: () => void
    createShortRest: () => void
    updateTimelineItem: (index: number, newValue: TimelineEvent) => void
    deleteTimelineItem: (index: number) => void
    swapTimelineItems: (index1: number, index2: number) => void
    clearAdventuringDay: () => void
}

export interface SimulationSessionSelectors {
    isEmptyResult: boolean
    combatantNames: Map<string, string>
    actionNames: Map<string, string>
    encounterWeights: number[]
}

export const useSimulationSession = (): SimulationSessionState & SimulationSessionActions & SimulationSessionSelectors => {
    const [players, setPlayers, isPlayersLoaded] = useStoredState<Creature[]>('players', [], sanitizePlayersParser(z.array(CreatureSchema).parse))
    const [timeline, setTimeline, isTimelineLoaded] = useStoredState<TimelineEvent[]>('timeline', [emptyCombat], sanitizeTimelineParser(z.array(TimelineEventSchema).parse))

    // Memoize expensive computations
    const isEmptyResult = useMemo(() => {
        const hasPlayers = !!players.length
        const hasMonsters = !!timeline.find(item => item.type === 'combat' && !!item.monsters.length)
        return !hasPlayers && !hasMonsters
    }, [players.length, timeline])

    // Memoize combatant names map
    const combatantNames = useMemo(() => {
        const names = new Map<string, string>()

        // Add players - IDs are prefixed with 'p-' and numbered if count > 1
        players.forEach((p, group_idx) => {
            for (let i = 0; i < (p.count || 1); i++) {
                const id = `p-${group_idx}-${i}-${p.id}`
                const name = (p.count || 1) > 1 ? `${p.name} ${i + 1}` : p.name
                names.set(id, name)
            }
            // Fallback for base ID - VERY IMPORTANT for resolving WASM partySlots
            names.set(p.id, p.name)
        })

        // Add monsters - IDs include encounter index and are numbered if count > 1
        timeline.forEach((item, step_idx) => {
            if (item.type === 'combat') {
                item.monsters.forEach((m, group_idx) => {
                    for (let i = 0; i < (m.count || 1); i++) {
                        const id = `step${step_idx}-m-${group_idx}-${i}-${m.id}`
                        const name = (m.count || 1) > 1 ? `${m.name} ${i + 1}` : m.name
                        names.set(id, name)
                    }
                    // Fallback for base ID
                    names.set(m.id, m.name)
                })
            }
        })

        return names
    }, [players, timeline])

    // Memoize action names map
    const actionNames = useMemo(() => {
        const names = new Map<string, string>()
        players.forEach(p => p.actions.forEach(a => {
            const name = a.type === 'template' ? a.templateOptions.templateName : a.name
            names.set(a.id, name)
        }))
        timeline.forEach(item => {
            if (item.type === 'combat') {
                item.monsters.forEach(m => m.actions.forEach(a => {
                    const name = a.type === 'template' ? a.templateOptions.templateName : a.name
                    names.set(a.id, name)
                }))
            }
        })
        return names
    }, [players, timeline])

    const encounterWeights = useMemo(() => {
        const weights: number[] = [];
        timeline.forEach(item => {
            if (item.type === 'combat') {
                const role = item.targetRole || 'Standard';
                const weight = role === 'Skirmish' ? 1 : role === 'Standard' ? 2 : role === 'Elite' ? 3 : 4;
                weights.push(weight);
            }
        });
        return weights;
    }, [timeline]);

    const createCombat = () => {
        setTimeline([...timeline, {
            type: 'combat',
            id: uuid(), // Ensure new items have a unique ID
            monsters: [],
            monstersSurprised: false,
            playersSurprised: false,
            targetRole: 'Standard',
        }])
    }

    const createShortRest = () => {
        setTimeline([...timeline, {
            type: 'shortRest',
            id: uuid(),
        }])
    }

    const updateTimelineItem = (index: number, newValue: TimelineEvent) => {
        const timelineClone = clone(timeline)
        timelineClone[index] = newValue
        setTimeline(timelineClone)
    }

    const deleteTimelineItem = (index: number) => {
        if (timeline.length <= 1) return // Must have at least one item
        const timelineClone = clone(timeline)
        timelineClone.splice(index, 1)
        setTimeline(timelineClone)
    }

    const swapTimelineItems = (index1: number, index2: number) => {
        const timelineClone = clone(timeline)
        const tmp = timelineClone[index1]
        timelineClone[index1] = timelineClone[index2]
        timelineClone[index2] = tmp
        setTimeline(timelineClone)
    }

    const clearAdventuringDay = () => {
        setPlayers([])
        setTimeline([emptyCombat])
    }

    return {
        // State
        players,
        timeline,
        isPlayersLoaded,
        isTimelineLoaded,
        isHighPrecisionLoaded: true, // This will be moved to useAutoSimulation

        // Actions
        setPlayers,
        setTimeline,
        createCombat,
        createShortRest,
        updateTimelineItem,
        deleteTimelineItem,
        swapTimelineItems,
        clearAdventuringDay,

        // Selectors
        isEmptyResult,
        combatantNames,
        actionNames,
        encounterWeights,
    }
}