import { describe, it, expect } from 'vitest';
import { mapMonster5eToCreature } from './5etools-mapper';
import { Monster5e } from './5etools-schema';

describe('5etools-mapper', () => {
    it('should map core stats correctly', () => {
        const monster: Monster5e = {
            name: "Aarakocra",
            src: "MM",
            hp: { average: 13, formula: "3d8" },
            ac: [12],
            str: 10,
            dex: 14,
            con: 10,
            int: 11,
            wis: 12,
            cha: 11
        };

        const creature = mapMonster5eToCreature(monster);

        expect(creature.name).toBe("Aarakocra");
        expect(creature.hp).toBe(13);
        expect(creature.ac).toBe(12);
        expect(creature.mode).toBe("monster");
        expect(creature.actions).toEqual([]); // Actions mapping is Phase 2
    });

    it('should handle complex AC', () => {
        const monster: Monster5e = {
            name: "Mage",
            hp: { average: 40 },
            ac: [12, { ac: 15, from: ["mage armor"] }],
            str: 9, dex: 14, con: 11, int: 17, wis: 12, cha: 11
        };

        const creature = mapMonster5eToCreature(monster);
        expect(creature.ac).toBe(12); // Should take the first base AC for now
    });

    it('should handle AC as object', () => {
        const monster: Monster5e = {
            name: "Animated Armor",
            hp: { average: 33 },
            ac: [{ ac: 18, from: ["natural armor"] }],
            str: 14, dex: 11, con: 13, int: 1, wis: 3, cha: 1
        };

        const creature = mapMonster5eToCreature(monster);
        expect(creature.ac).toBe(18);
    });
});
