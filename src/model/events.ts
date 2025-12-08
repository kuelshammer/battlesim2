// src/model/events.ts
// This file provides a utility function to parse raw event strings from WASM into structured types.
// It uses the Event type defined in model.ts as the source of truth.

import { Event } from "./model";

// Re-export Event as SimulationEvent for backward compatibility if needed, 
// or just use Event everywhere.
export type SimulationEvent = Event;

// --- Utility function to parse raw event strings into structured types ---
// This function attempts to parse the Rust `Debug` string representation into
// a structured TypeScript object. This is inherently fragile and relies on
// the exact formatting of Rust's `Debug` output.
export function parseEventString(eventString: string): Event | null {
    try {
        const parsedObject = JSON.parse(eventString);

        // Serde default for enums is {"VariantName": {data}}
        // We need to transform it into {type: "VariantName", ...data}
        const eventType = Object.keys(parsedObject)[0];
        const eventData = parsedObject[eventType];

        const structuredEvent: Event = {
            type: eventType,
            ...eventData
        } as Event; // Cast to Event to match the discriminated union

        return structuredEvent;
    } catch (e) {
        console.warn('Failed to parse event JSON string:', eventString, e);
        return null;
    }
}
