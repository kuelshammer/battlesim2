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

    it('should map basic actions correctly', () => {
        const monster: Monster5e = {
            name: "Aarakocra",
            hp: { average: 13 },
            ac: [12],
            str: 10, dex: 14, con: 10, int: 11, wis: 12, cha: 11,
            action: [
                {
                    name: "Talon",
                    entries: ["{@hit 4} to hit, reach 5 ft., one target. {@h}5 ({@damage 1d6 + 2}) piercing damage."]
                }
            ]
        };

        const creature = mapMonster5eToCreature(monster);
        expect(creature.actions.length).toBe(1);
        expect(creature.actions[0].name).toBe("Talon");
        expect(creature.actions[0].toHit).toBe(4);
        expect(creature.actions[0].dpr).toBe(5.5);
    });
});
