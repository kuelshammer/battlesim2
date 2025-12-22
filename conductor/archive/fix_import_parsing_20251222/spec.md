# Spec: Robust 5e.tools JSON Parsing

## Overview
The current JSON import fails if the input text contains any non-JSON characters (like a leading 'j', markdown ticks, or whitespace). This track will implement a cleaning utility to ensure the parser extracts the valid JSON object from the provided string.

## Technical Goals
- Implement a `cleanJsonInput` utility function.
- Handle leading/trailing garbage characters.
- Strip markdown code blocks if present.
- Support common 5e.tools copy-paste artifacts.

## Acceptance Criteria
- Pasting `j{ "name": "..." }` should successfully import.
- Pasting ` ```json { ... } ``` ` should successfully import.
- The user is notified if the string contains NO valid JSON structure.
