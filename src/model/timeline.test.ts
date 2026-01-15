import { describe, it, expect } from 'vitest';
import { TimelineEventSchema } from './model';

describe('Timeline Models', () => {
    it('should validate a combat encounter as a timeline event', () => {
        const combat = {
            type: 'combat',
            id: 'test-combat',
            monsters: []
        };
        const result = TimelineEventSchema.safeParse(combat);
        expect(result.success).toBe(true);
    });

    it('should validate a short rest as a timeline event', () => {
        const rest = {
            type: 'shortRest',
            id: 'test-rest'
        };
        const result = TimelineEventSchema.safeParse(rest);
        expect(result.success).toBe(true);
    });
});
