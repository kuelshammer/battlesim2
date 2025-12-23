import { describe, it, expect } from 'vitest';
import { LogFormatter } from './logFormatter';
import { Event } from './model';

describe('LogFormatter', () => {
    const mockAttackHit: Event = {
        type: 'AttackHit',
        attacker_id: 'attacker-1',
        target_id: 'target-1',
        damage: 10,
        attack_roll: {
            total: 18,
            rolls: [{ sides: 20, value: 13 }],
            modifiers: [['d20', 13], ['Str', 5]],
            formula: '1d20+5'
        },
        damage_roll: {
            total: 10,
            rolls: [{ sides: 8, value: 5 }],
            modifiers: [['Base', 5], ['Str', 5]],
            formula: '1d8+5'
        }
    };

    const names = {
        'attacker-1': 'Bob',
        'target-1': 'Goblin 7',
        'p1': 'Alice'
    };

    const mockBlessAttack: Event = {
        type: 'AttackHit',
        attacker_id: 'p1',
        target_id: 'target-1',
        damage: 8,
        attack_roll: {
            total: 19,
            rolls: [{ sides: 20, value: 12 }],
            modifiers: [['d20', 12], ['Prof', 3], ['Bless', 2], ['Str', 2]],
            formula: '1d20+3+1d4+2'
        }
    };

    it('generates a summary for an attack hit', () => {
        const summary = LogFormatter.toSummary(mockAttackHit, names);
        expect(summary).toBe('Bob attacks Goblin 7 for 10 damage.');
    });

    it('handles UUID suffixes in names', () => {
        const uuid = '550e8400-e29b-41d4-a716-446655440000';
        const namesWithUUID = { [uuid]: 'Player' };
        const summary = LogFormatter.toSummary({
            type: 'TurnStarted',
            unit_id: `${uuid}-1`,
            round_number: 1
        }, namesWithUUID);
        expect(summary).toContain('Player');
    });

    it('generates details for an attack hit', () => {
        const details = LogFormatter.toDetails(mockAttackHit, names);
        expect(details).toContain('Attack Roll: 1d20 [13] + 5 (Str) = 18');
        expect(details).toContain('Damage: 1d8 [5] + 5 (Str) = 10');
    });

    it('falls back to IDs if names are missing', () => {
        const summary = LogFormatter.toSummary(mockAttackHit, {});
        expect(summary).toBe('attacker-1 attacks target-1 for 10 damage.');
    });

    it('handles AttackMissed', () => {
        const mockMiss: Event = {
            type: 'AttackMissed',
            attacker_id: 'attacker-1',
            target_id: 'target-1',
            attack_roll: {
                total: 8,
                rolls: [{ sides: 20, value: 3 }],
                modifiers: [['d20', 3], ['Str', 5]],
                formula: '1d20+5'
            }
        };
        const summary = LogFormatter.toSummary(mockMiss, names);
        expect(summary).toBe('Bob misses Goblin 7.');
        
        const details = LogFormatter.toDetails(mockMiss, names);
        expect(details).toContain('Attack Roll: 1d20 [3] + 5 (Str) = 8');
    });

    it('handles Bless bonus in details', () => {
        const details = LogFormatter.toDetails(mockBlessAttack, names);
        expect(details).toContain('+ 2 (Bless)');
    });

    it('handles ActionStarted summary', () => {
        const event: Event = {
            type: 'ActionStarted',
            unit_id: 'p1',
            action_name: 'Action Surge'
        };
        const summary = LogFormatter.toSummary(event, names);
        expect(summary).toBe('Alice uses action: Action Surge.');
    });

    it('handles DamageTaken summary', () => {
        const event: Event = {
            type: 'DamageTaken',
            target_id: 'p1',
            amount: 15
        };
        const summary = LogFormatter.toSummary(event, names);
        expect(summary).toBe('Alice takes 15 damage.');
    });

    it('handles RoundStarted summary', () => {
        const event: Event = {
            type: 'RoundStarted',
            round_number: 2
        };
        const summary = LogFormatter.toSummary(event, names);
        expect(summary).toBe('--- Round 2 ---');
    });
});
