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
        // Force cast because the parsing logic is generic but the type is specific
        const event = { type, ...parsedData } as unknown as Event;
        return event;

    } catch (e) {
        console.error(`Error parsing event string "${eventString}":`, e);
        return null;
    }
}
