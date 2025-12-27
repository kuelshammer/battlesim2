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

    it('handles composite IDs from WASM backend (monster)', () => {
        const uuid = '670d1340-c829-4746-b112-d96960ebd650';
        const namesWithUUID = { [uuid]: 'Goblin 2' };
        // step0-m-0-3-UUID
        const id = `step0-m-0-3-${uuid}`;
        const summary = LogFormatter.toSummary({
            type: 'TurnStarted',
            unit_id: id,
            round_number: 1
        }, namesWithUUID);
        expect(summary).toContain('Goblin 2');
    });

    it('handles composite IDs from WASM backend (player)', () => {
        const uuid = '61ecead5-6832-43fb-852e-9b7336070757';
        const namesWithUUID = { [uuid]: 'Fighter 1' };
        // p-1-0-UUID
        const id = `p-1-0-${uuid}`;
        const summary = LogFormatter.toSummary({
            type: 'TurnStarted',
            unit_id: id,
            round_number: 1
        }, namesWithUUID);
        expect(summary).toContain('Fighter 1');
    });

    it('handles composite IDs with exact numbering match', () => {
        const uuid = '670d1340-c829-4746-b112-d96960ebd650';
        const id = `step0-m-0-3-${uuid}`;
        const namesWithNumbering = { 
            [uuid]: 'Aarakocra',
            [id]: 'Aarakocra 4' 
        };
        const summary = LogFormatter.toSummary({
            type: 'TurnStarted',
            unit_id: id,
            round_number: 1
        }, namesWithNumbering);
        expect(summary).toContain('Aarakocra 4');
    });

    it('generates details for an attack hit with target AC', () => {
        const hitWithAC: Event = {
            ...mockAttackHit,
            target_ac: 15
        };
        // Mock formula in action_resolver prepends 1d20
        hitWithAC.attack_roll!.formula = '1d20+5';
        const details = LogFormatter.toDetails(hitWithAC, names);
        expect(details).toContain('Attack Roll: 1d20 + 5 = 13 + 5 = 18 vs. AC 15 ==> HIT');
    });

    it('generates details for an attack miss with target AC', () => {
        const mockMiss: Event = {
            type: 'AttackMissed',
            attacker_id: 'attacker-1',
            target_id: 'target-1',
            attack_roll: {
                total: 8,
                rolls: [{ sides: 20, value: 3 }],
                modifiers: [['d20', 3], ['Str', 5]],
                formula: '1d20+5'
            },
            target_ac: 15
        };
        const details = LogFormatter.toDetails(mockMiss, names);
        expect(details).toContain('Attack Roll: 1d20 + 5 = 3 + 5 = 8 vs. AC 15 ==> miss');
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
        expect(details).toContain('Attack Roll: 1d20 + 5 = 3 + 5 = 8');
    });

    it('handles Bless bonus in details', () => {
        const details = LogFormatter.toDetails(mockBlessAttack, names);
        expect(details).toContain('1d20 + 3 + 1d4 + 2 = 12 + 3 + 2 + 2 = 19');
    });

    it('handles ActionStarted summary', () => {
        const actionNames = {
            'a1-uuid': 'Longsword',
            's1-uuid': 'Fireball'
        };
        const event: Event = {
            type: 'ActionStarted',
            actor_id: 'p1',
            action_id: 'a1-uuid'
        };
        const summary = LogFormatter.toSummary(event, names, actionNames);
        expect(summary).toBe('Alice uses action: Longsword.');
    });

    it('handles SpellCast summary with action resolution', () => {
        const actionNames = {
            's1-uuid': 'Fireball'
        };
        const event: Event = {
            type: 'SpellCast',
            caster_id: 'p1',
            spell_id: 's1-uuid',
            target_id: 'target-1',
            spell_level: 3
        };
        const summary = LogFormatter.toSummary(event, names, actionNames);
        expect(summary).toBe('Alice casts Fireball.');
    });

    it('handles DamageTaken summary', () => {
        const event: Event = {
            type: 'DamageTaken',
            target_id: 'p1',
            damage: 15,
            damage_type: 'slashing'
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
