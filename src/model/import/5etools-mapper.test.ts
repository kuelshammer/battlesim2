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
        const action = creature.actions[0] as { name: string; toHit: number; dpr: string | number };
        expect(action.name).toBe("Talon");
        expect(action.toHit).toBe(4);
        expect(action.dpr).toBe("1d6 + 2");
    });

    it('should map multiattack correctly', () => {
        const monster: Monster5e = {
            name: "Owlbear",
            hp: { average: 59 },
            ac: [13],
            str: 20, dex: 12, con: 17, int: 3, wis: 12, cha: 7,
            action: [
                {
                    name: "Multiattack",
                    entries: ["The owlbear makes two attacks: one with its beak and one with its claws."]
                },
                {
                    name: "Beak",
                    entries: ["{@hit 7} to hit, reach 5 ft., one target. {@h}10 ({@damage 1d10 + 5}) piercing damage."]
                },
                {
                    name: "Claws",
                    entries: ["{@hit 7} to hit, reach 5 ft., one target. {@h}14 ({@damage 2d8 + 5}) slashing damage."]
                }
            ]
        };

        const creature = mapMonster5eToCreature(monster);
        expect(creature.actions.length).toBe(2);
        const action0 = creature.actions[0] as { name: string; targets: number };
        expect(action0.name).toBe("Beak");
        expect(action0.targets).toBe(2); // OWlbear makes two attacks total, we map it to 'targets' for simplicity in this engine
        const action1 = creature.actions[1] as { name: string; targets: number };
        expect(action1.name).toBe("Claws");
        expect(action1.targets).toBe(2);
    });

    it('should handle Abjurer-style source and nested type', () => {
        const monster: Monster5e = {
            name: "Abjurer Wizard",
            source: "MPMM",
            hp: { average: 104 },
            ac: [12],
            type: { type: "humanoid" },
            str: 9, dex: 14, con: 14, int: 18, wis: 12, cha: 11
        };

        const creature = mapMonster5eToCreature(monster);
        expect(creature.src).toBe("MPMM");
        expect(creature.type).toBe("humanoid");
    });
});
