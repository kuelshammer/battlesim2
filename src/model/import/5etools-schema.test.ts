import { describe, it, expect } from 'vitest';
import { Monster5eSchema } from './5etools-schema';

describe('Monster5eSchema', () => {
    it('should validate a basic 5e.tools monster', () => {
        const monster = {
            name: "Aarakocra",
            src: "MM",
            hp: { average: 13 },
            ac: [12],
            str: 10,
            dex: 14,
            con: 10,
            int: 11,
            wis: 12,
            cha: 11
        };
        const result = Monster5eSchema.safeParse(monster);
        expect(result.success).toBe(true);
        if (result.success) {
            expect(result.data.hp.average).toBe(13);
            expect(result.data.ac[0]).toBe(12);
        }
    });
});
