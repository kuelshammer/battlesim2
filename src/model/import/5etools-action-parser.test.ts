import { describe, it, expect } from 'vitest';
import { parse5eAttack } from './5etools-action-parser';

describe('5etools-action-parser', () => {
    it('should parse a standard melee attack', () => {
        const entry = "{@hit 4} to hit, reach 5 ft., one target. {@h}5 ({@damage 1d6 + 2}) piercing damage.";
        const result = parse5eAttack("Bite", entry);
        
        expect(result.name).toBe("Bite");
        expect(result.toHit).toBe(4);
        expect(result.dpr).toBe(5.5); // 1d6 + 2 = 3.5 + 2 = 5.5
    });

    it('should parse a standard ranged attack', () => {
        const entry = "{@hit 5} to hit, range 150/600 ft., one target. {@h}13 ({@damage 2d8 + 3}) piercing damage.";
        const result = parse5eAttack("Longbow", entry);
        
        expect(result.name).toBe("Longbow");
        expect(result.toHit).toBe(5);
        expect(result.dpr).toBe(12); // 2d8 + 3 = 9 + 3 = 12
    });

    it('should handle entries with multiple damage types (ignoring secondary for now)', () => {
        const entry = "{@hit 7} to hit, reach 5 ft., one target. {@h}10 ({@damage 1d10 + 4}) slashing damage plus 3 ({@damage 1d6}) fire damage.";
        const result = parse5eAttack("Fire Sword", entry);
        
        expect(result.name).toBe("Fire Sword");
        expect(result.toHit).toBe(7);
        expect(result.dpr).toBe(9.5); // 1d10 + 4 = 5.5 + 4 = 9.5
    });
});
