import { describe, it, expect } from 'vitest';
import { FileLogger } from './fileLogger';
import { Event } from './model';

describe('FileLogger', () => {
    const mockEvents: Event[] = [
        {
            type: 'TurnStarted',
            unit_id: 'player1',
            round_number: 1,
            timestamp: 0
        },
        {
            type: 'AttackHit',
            attacker_id: 'player1',
            target_id: 'monster1',
            damage: 10,
            attack_roll: {
                formula: '1d20+5',
                rolls: [{ die: 20, value: 15 }],
                modifiers: [['d20', 15], ['Proficiency', 3], ['Strength', 2]],
                total: 20
            },
            damage_roll: {
                formula: '1d8+2',
                rolls: [{ die: 8, value: 8 }],
                modifiers: [['Base', 2]],
                total: 10
            },
            timestamp: 1
        }
    ];

    const names = {
        'player1': 'Alice',
        'monster1': 'Goblin'
    };

    it('should format events into a human-readable log string', () => {
        const logger = new FileLogger(names);
        const logOutput = logger.formatLog(mockEvents);

        expect(logOutput).toContain('Alice starts turn (Round 1)');
        expect(logOutput).toContain('Alice attacks Goblin for 10 damage');
        expect(logOutput).toContain('Attack Roll: 1d20 [15] + 3 (Proficiency) + 2 (Strength) = 20');
        expect(logOutput).toContain('Damage: 1d8 [8] + 2 (Base) = 10');
    });

    it('should include raw JSON for debugging if requested', () => {
        const logger = new FileLogger(names, { includeRaw: true });
        const logOutput = logger.formatLog(mockEvents);

        expect(logOutput).toContain('"type": "AttackHit"');
    });
});
