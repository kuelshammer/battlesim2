import { describe, it, expect } from 'vitest';
import { CreatureSchema } from './model';

describe('CreatureSchema partial initialization', () => {
    it('should allow parsing a creature with missing HP and AC if they have defaults', () => {
        const partialCreature = {
            id: 'test-id',
            mode: 'custom',
            name: 'Test Monster',
            count: 1,
            actions: []
        };
        
        const result = CreatureSchema.safeParse(partialCreature);
        expect(result.success).toBe(true);
        if (result.success) {
            expect(result.data.hp).toBeDefined();
            expect(result.data.ac).toBeDefined();
        }
    });
});
