import { describe, it, expect } from 'vitest';
import { parse5eAttack, parse5eMultiattack } from './5etools-action-parser';

describe('5etools-action-parser', () => {
    it('should parse a standard melee attack', () => {
        const entry = "{@hit 4} to hit, reach 5 ft., one target. {@h}5 ({@damage 1d6 + 2}) piercing damage.";
        const result = parse5eAttack("Bite", entry);
        
        expect(result.id).toBeDefined();
        expect(result.name).toBe("Bite");
        expect(result.toHit).toBe(4);
        expect(result.dpr).toBe("1d6 + 2");
    });

    it('should parse a standard ranged attack', () => {
        const entry = "{@hit 5} to hit, range 150/600 ft., one target. {@h}13 ({@damage 2d8 + 3}) piercing damage.";
        const result = parse5eAttack("Longbow", entry);
        
        expect(result.name).toBe("Longbow");
        expect(result.toHit).toBe(5);
        expect(result.dpr).toBe("2d8 + 3");
    });

    it('should handle entries with multiple damage types (ignoring secondary for now)', () => {
        const entry = "{@hit 7} to hit, reach 5 ft., one target. {@h}10 ({@damage 1d10 + 4}) slashing damage plus 3 ({@damage 1d6}) fire damage.";
        const result = parse5eAttack("Fire Sword", entry);
        
        expect(result.name).toBe("Fire Sword");
        expect(result.toHit).toBe(7);
        expect(result.dpr).toBe("1d10 + 4");
    });

    describe('parse5eMultiattack', () => {
        it('should parse simple multiattack (The creature makes two attacks: one with its bite and one with its claws.)', () => {
            const entry = "The creature makes two attacks: one with its bite and one with its claws.";
            const result = parse5eMultiattack(entry);
            expect(result).toEqual({ total: 2, details: "bite and claws" });
        });

        it('should parse numeric multiattack (The creature makes three melee attacks.)', () => {
            const entry = "The creature makes three melee attacks.";
            const result = parse5eMultiattack(entry);
            expect(result).toEqual({ total: 3, details: "melee attacks" });
        });
    });
});
