import { Event, RollResult } from "./model";

export class LogFormatter {
    static getUnitName(id: string, names: Record<string, string>): string {
        if (names[id]) return names[id];
        // Try UUID prefix (36 chars) if ID looks like UUID-Suffix
        if (id.length > 36 && id[36] === '-') {
            const baseId = id.substring(0, 36);
            if (names[baseId]) {
                return names[baseId];
            }
        }
        return id;
    }

    static formatRollResult(result: RollResult): string {
        const rolls = result.rolls.map(r => r.value).join(', ');
        
        // Find d20 or base die roll
        const d20Roll = result.modifiers.find(([name]) => name === 'd20');
        const otherMods = result.modifiers
            .filter(([name]) => name !== 'd20' && name !== 'Base' && name !== 'Critical')
            .map(([name, val]) => `${val >= 0 ? '+ ' : '- '}${Math.abs(val)} (${name})`)
            .join(' ');
        
        // Base value (like static damage modifier)
        const baseMod = result.modifiers.find(([name]) => name === 'Base');
        let baseModStr = '';
        if (baseMod && baseMod[1] !== 0) {
            // If it's the only mod besides d20/dice, we might just call it a modifier or its name
            // But usually 'Base' comes from DiceFormula::Value
        }

        const dicePart = result.formula.split(/[+\-\[]/)[0];
        let output = `${dicePart} [${rolls}]`;
        if (otherMods) {
            output += ` ${otherMods}`;
        }
        output += ` = ${result.total}`;
        
        return output;
    }

    static toSummary(event: Event, names: Record<string, string>): string {
        const getName = (id: string) => this.getUnitName(id, names);

        switch (event.type) {
            case 'AttackHit':
                return `${getName(event.attacker_id)} attacks ${getName(event.target_id)} for ${event.damage} damage.`;
            case 'AttackMissed':
                return `${getName(event.attacker_id)} misses ${getName(event.target_id)}.`;
            case 'TurnStarted':
                return `${getName(event.unit_id)} starts turn (Round ${event.round_number}).`;
            case 'ActionStarted':
                return `${getName(event.unit_id)} uses action: ${event.action_name}.`;
            case 'DamageTaken':
                return `${getName(event.target_id)} takes ${event.amount} damage.`;
            case 'HealingApplied':
                return `${getName(event.target_id)} is healed for ${event.amount} HP.`;
            case 'UnitDied':
                return `${getName(event.unit_id)} falls!`;
            case 'RoundStarted':
                return `--- Round ${event.round_number} ---`;
            case 'SpellCast':
                return `${getName(event.unit_id)} casts ${event.spell_name} on ${getName(event.target_id)}.`;
            case 'BuffApplied':
                return `${getName(event.target_id)} gains ${event.buff_name}.`;
            case 'ConditionAdded':
                return `${getName(event.target_id)} is ${event.condition}.`;
            case 'ConditionRemoved':
                return `${getName(event.target_id)} is no longer ${event.condition}.`;
            case 'EncounterEnded':
                return `Encounter ended after ${event.rounds} rounds. Winner: ${event.winner || 'None'}.`;
            default:
                return event.type;
        }
    }

    static toDetails(event: Event, names: Record<string, string>): string {
        const getName = (id: string) => this.getUnitName(id, names);

        switch (event.type) {
            case 'AttackHit': {
                let details = `Attack Roll: ${event.attack_roll ? this.formatRollResult(event.attack_roll) : 'N/A'}`;
                if (event.damage_roll) {
                    details += `\nDamage: ${this.formatRollResult(event.damage_roll)}`;
                }
                return details;
            }
            case 'AttackMissed':
                return `Attack Roll: ${event.attack_roll ? this.formatRollResult(event.attack_roll) : 'N/A'}`;
            default:
                return JSON.stringify(event, null, 2);
        }
    }
}
