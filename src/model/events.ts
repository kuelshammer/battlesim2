// src/model/events.ts
// This file defines TypeScript types for simulation events, mirroring the Rust Event enum.
// It also provides a utility function to parse raw event strings from WASM into these structured types.

import { CreatureCondition } from "./enums"; // Assuming CreatureCondition is defined here

// --- Event Type Definitions (Mirroring Rust Event Enum) ---

// Base interface for all events (for common properties like type)
export interface BaseEvent {
    type: string; // Corresponds to the Rust enum variant name (e.g., 'AttackHit')
    // Add any other common fields if they exist in the Rust BaseEvent (not explicit here)
}

export interface ActionStartedEvent extends BaseEvent {
    type: 'ActionStarted';
    actor_id: string;
    action_id: string;
}

export interface AttackHitEvent extends BaseEvent {
    type: 'AttackHit';
    attacker_id: string;
    target_id: string;
    damage: number;
}

export interface AttackMissedEvent extends BaseEvent {
    type: 'AttackMissed';
    attacker_id: string;
    target_id: string;
}

export interface DamageTakenEvent extends BaseEvent {
    type: 'DamageTaken';
    target_id: string;
    damage: number;
    damage_type: string;
}

export interface DamagePreventedEvent extends BaseEvent {
    type: 'DamagePrevented';
    target_id: string;
    prevented_amount: number;
}

export interface SpellCastEvent extends BaseEvent {
    type: 'SpellCast';
    caster_id: string;
    spell_id: string;
    spell_level: number;
}

export interface SpellSavedEvent extends BaseEvent {
    type: 'SpellSaved';
    target_id: string;
    spell_id: string;
}

export interface SpellFailedEvent extends BaseEvent {
    type: 'SpellFailed';
    target_id: string;
    spell_id: string;
    reason: string;
}

export interface ConcentrationBrokenEvent extends BaseEvent {
    type: 'ConcentrationBroken';
    caster_id: string;
    reason: string;
}

export interface ConcentrationMaintainedEvent extends BaseEvent {
    type: 'ConcentrationMaintained';
    caster_id: string;
    save_dc: number;
}

export interface BuffAppliedEvent extends BaseEvent {
    type: 'BuffApplied';
    target_id: string;
    buff_id: string;
    source_id: string;
}

export interface BuffExpiredEvent extends BaseEvent {
    type: 'BuffExpired';
    target_id: string;
    buff_id: string;
}

export interface BuffRemovedEvent extends BaseEvent {
    type: 'BuffRemoved';
    target_id: string;
    buff_id: string;
    source_id: string;
}

export interface ConditionAddedEvent extends BaseEvent {
    type: 'ConditionAdded';
    target_id: string;
    condition: CreatureCondition;
    source_id: string;
}

export interface ConditionRemovedEvent extends BaseEvent {
    type: 'ConditionRemoved';
    target_id: string;
    condition: CreatureCondition;
    source_id: string;
}

export interface HealingAppliedEvent extends BaseEvent {
    type: 'HealingApplied';
    target_id: string;
    amount: number;
    source_id: string;
}

export interface TempHPGrantedEvent extends BaseEvent {
    type: 'TempHPGranted';
    target_id: string;
    amount: number;
    source_id: string;
}

export interface TempHPLostEvent extends BaseEvent {
    type: 'TempHPLost';
    target_id: string;
    amount: number;
}

export interface UnitDiedEvent extends BaseEvent {
    type: 'UnitDied';
    unit_id: string;
    killer_id?: string;
    damage_type?: string;
}

export interface TurnStartedEvent extends BaseEvent {
    type: 'TurnStarted';
    unit_id: string;
    round_number: number;
}

export interface TurnEndedEvent extends BaseEvent {
    type: 'TurnEnded';
    unit_id: string;
    round_number: number;
}

export interface RoundStartedEvent extends BaseEvent {
    type: 'RoundStarted';
    round_number: number;
}

export interface RoundEndedEvent extends BaseEvent {
    type: 'RoundEnded';
    round_number: number;
}

export interface EncounterStartedEvent extends BaseEvent {
    type: 'EncounterStarted';
    combatant_ids: string[];
}

export interface EncounterEndedEvent extends BaseEvent {
    type: 'EncounterEnded';
    winner?: string;
    reason: string;
}

export interface MovementStartedEvent extends BaseEvent {
    type: 'MovementStarted';
    unit_id: string;
    from_position: string;
    to_position: string;
}

export interface MovementInterruptedEvent extends BaseEvent {
    type: 'MovementInterrupted';
    unit_id: string;
    reason: string;
}

export interface OpportunityAttackEvent extends BaseEvent {
    type: 'OpportunityAttack';
    attacker_id: string;
    target_id: string;
    provoked_by: string;
}

export interface ResourceConsumedEvent extends BaseEvent {
    type: 'ResourceConsumed';
    unit_id: string;
    resource_type: string;
    amount: number;
}

export interface ResourceRestoredEvent extends BaseEvent {
    type: 'ResourceRestored';
    unit_id: string;
    resource_type: string;
    amount: number;
}

export interface ResourceDepletedEvent extends BaseEvent {
    type: 'ResourceDepleted';
    unit_id: string;
    resource_type: string;
}

export interface CustomEvent extends BaseEvent {
    type: 'Custom';
    event_type: string; // This is the internal Rust type, like "ReactionAction"
    data: { [key: string]: string };
    source_id: string;
}


// --- Union Type for all Events ---
export type SimulationEvent =
    | ActionStartedEvent
    | AttackHitEvent
    | AttackMissedEvent
    | DamageTakenEvent
    | DamagePreventedEvent
    | SpellCastEvent
    | SpellSavedEvent
    | SpellFailedEvent
    | ConcentrationBrokenEvent
    | ConcentrationMaintainedEvent
    | BuffAppliedEvent
    | BuffExpiredEvent
    | BuffRemovedEvent
    | ConditionAddedEvent
    | ConditionRemovedEvent
    | HealingAppliedEvent
    | TempHPGrantedEvent
    | TempHPLostEvent
    | UnitDiedEvent
    | TurnStartedEvent
    | TurnEndedEvent
    | RoundStartedEvent
    | RoundEndedEvent
    | EncounterStartedEvent
    | EncounterEndedEvent
    | MovementStartedEvent
    | MovementInterruptedEvent
    | OpportunityAttackEvent
    | ResourceConsumedEvent
    | ResourceRestoredEvent
    | ResourceDepletedEvent
    | CustomEvent;


// --- Utility function to parse raw event strings into structured types ---
// This function attempts to parse the Rust `Debug` string representation into
// a structured TypeScript object. This is inherently fragile and relies on
// the exact formatting of Rust's `Debug` output.
export function parseEventString(eventString: string): SimulationEvent | null {
    // Regex to capture the event type and the content in curly braces
    const match = eventString.match(/^([A-Za-z]+)\s?\{(.*)\}$/);
    if (!match) {
        console.warn('Failed to parse event string format:', eventString);
        return null;
    }

    const type = match[1];
    const content = match[2];

    try {
        const parsedData: { [key: string]: any } = {};
        // Split content by comma, but only if not inside another { } or [ ]
        // This is a simplified regex and might need refinement for complex nested structures
        const parts = content.split(/,\s*(?![^{]*\}|[^[]*\])/).filter(p => p.trim() !== '');

        for (const part of parts) {
            const [key, value] = part.split(/:\s*(.*)/s).map(s => s.trim()); // Split only on first colon
            if (key && value) {
                // Attempt to parse value as number, boolean, or string
                let processedValue: any = value;
                if (!isNaN(Number(value)) && !value.includes('"')) { // Check for numbers not enclosed in quotes
                    processedValue = Number(value);
                } else if (value === 'true') {
                    processedValue = true;
                } else if (value === 'false') {
                    processedValue = false;
                } else if (value.startsWith('"') && value.endsWith('"')) {
                    processedValue = value.substring(1, value.length - 1); // Remove quotes
                } else if (value.startsWith('[') && value.endsWith(']')) {
                     // Handle array parsing
                     try {
                        processedValue = JSON.parse(value.replace(/([a-zA-Z_]+):/g, '"$1":'));
                     } catch (e) {
                         console.warn('Failed to parse array in event:', value, e);
                         processedValue = value;
                     }
                } else if (value.startsWith('{') && value.endsWith('}')) {
                    // Handle HashMap parsing for CustomEvent
                    // This is very specific for Rust Debug output of HashMap<String, String>
                    if (type === 'Custom') {
                        const mapContent = value.substring(1, value.length - 1); // Remove braces
                        const mapParts = mapContent.split(/,\s*(?![^{]*\})/).filter(p => p.trim() !== '');
                        const mapData: { [k: string]: string } = {};
                        for (const mapPart of mapParts) {
                            const [mapKey, mapVal] = mapPart.split(/:\s*(.*)/s).map(s => s.trim());
                            if (mapKey && mapVal) {
                                mapData[mapKey.substring(1, mapKey.length - 1)] = mapVal.substring(1, mapVal.length - 1);
                            }
                        }
                        processedValue = mapData;
                    } else {
                        // General object parsing (might need more robust solution for nested objects)
                        try {
                            processedValue = JSON.parse(value.replace(/([a-zA-Z_]+):/g, '"$1":'));
                        } catch (e) {
                            console.warn('Failed to parse object in event:', value, e);
                            processedValue = value;
                        }
                    }
                }
                parsedData[key] = processedValue;
            }
        }

        // Construct the specific event type
        const event: SimulationEvent = { type, ...parsedData } as SimulationEvent;
        return event;

    } catch (e) {
        console.error(`Error parsing event string "${eventString}":`, e);
        return null;
    }
}
