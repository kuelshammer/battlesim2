import { describe, it, expect } from 'vitest';
import { cleanJsonInput } from './utils';

describe('import-utils', () => {
    it('should handle clean JSON', () => {
        const input = '{"name": "Test"}';
        expect(cleanJsonInput(input)).toBe('{"name": "Test"}');
    });

    it('should handle leading/trailing whitespace', () => {
        const input = '   {"name": "Test"}   \n';
        expect(cleanJsonInput(input)).toBe('{"name": "Test"}');
    });

    it('should handle leading "j" artifact', () => {
        const input = 'j{"name": "Test"}';
        expect(cleanJsonInput(input)).toBe('{"name": "Test"}');
    });

    it('should handle markdown code blocks', () => {
        const input = '```json\n{"name": "Test"}\n```';
        expect(cleanJsonInput(input)).toBe('{"name": "Test"}');
    });

    it('should handle markdown code blocks without language', () => {
        const input = '```\n{"name": "Test"}\n```';
        expect(cleanJsonInput(input)).toBe('{"name": "Test"}');
    });

    it('should extract JSON from a larger string of garbage', () => {
        const input = 'Random text before { "name": "found" } and garbage after';
        expect(cleanJsonInput(input)).toBe('{ "name": "found" }');
    });

    it('should return empty string if no JSON object is found', () => {
        const input = 'just some text with no braces';
        expect(cleanJsonInput(input)).toBe('');
    });
});
