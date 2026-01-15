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

        // Try UUID suffix (36 chars) if ID looks like Prefix-UUID
        // This handles cases from WASM backend like "step0-m-0-3-UUID" or "p-1-0-UUID"
        if (id.length > 36) {
            const suffix = id.substring(id.length - 36);
            if (names[suffix]) {
                return names[suffix];
            }
        }

        return id;
    }

    static formatRollResult(result: RollResult, targetAc?: number): string {
        // Build the value breakdown (e.g. "12 + 7 + 2")
        const valueBreakdown = result.modifiers
            .map(([, val], index) => {
                const prefix = index === 0 ? (val < 0 ? '-' : '') : (val < 0 ? ' - ' : ' + ');
                return `${prefix}${Math.abs(val)}`;
            })
            .join('');

        // Prettify formula: "1d20+3[PB]" -> "1d20 + 3[PB]"
        const prettyFormula = result.formula
            .replace(/([+-])/g, ' $1 ')
            .replace(/\s+/g, ' ')
            .trim();

        let output = `${prettyFormula} = ${valueBreakdown} = ${result.total}`;

        if (targetAc !== undefined && targetAc > 0) {
            const isHit = result.total >= targetAc;
            output += ` vs. AC ${targetAc} ==> ${isHit ? 'HIT' : 'miss'}`;
        }

        return output;
    }

    static toSummary(event: Event, names: Record<string, string>, actionNames: Record<string, string> = {}): string {
        const getName = (id: string) => this.getUnitName(id, names);
        const getActionName = (id: string) => actionNames[id] || id;

        switch (event.type) {
            case 'AttackHit':
                return `${getName(event.attacker_id)} attacks ${getName(event.target_id)} for ${event.damage} damage.`;
            case 'AttackMissed':
                return `${getName(event.attacker_id)} misses ${getName(event.target_id)}.`;
            case 'TurnStarted':
                return `${getName(event.unit_id)} starts turn (Round ${event.round_number}).`;
            case 'ActionStarted':
                return `${getName(event.actor_id)} uses action: ${getActionName(event.action_id)}.`;
            case 'ActionSkipped':
                return `⚠️ ${getName(event.actor_id)} skipped ${getActionName(event.action_id)}: ${event.reason}`;
            case 'DamageTaken':
                return `${getName(event.target_id)} takes ${event.damage} damage.`;
            case 'HealingApplied':
                return `${getName(event.target_id)} is healed for ${event.amount} HP.`;
            case 'UnitDied':
                return `${getName(event.unit_id)} falls!`;
            case 'RoundStarted':
                return `--- Round ${event.round_number} ---`;
            case 'SpellCast':
                return `${getName(event.caster_id)} casts ${getActionName(event.spell_id)}.`;
            case 'BuffApplied':
                return `${getName(event.target_id)} gains ${getActionName(event.buff_id)}.`;
            case 'ConditionAdded':
                return `${getName(event.target_id)} is ${event.condition}.`;
            case 'ConditionRemoved':
                return `${getName(event.target_id)} is no longer ${event.condition}.`;
            case 'EncounterEnded':
                return `Encounter ended. Winner: ${event.winner || 'None'}.`;
            default:
                return (event as Event & { type: string }).type;
        }
    }

    static toDetails(event: Event, names: Record<string, string>): string { // eslint-disable-line @typescript-eslint/no-unused-vars

        switch (event.type) {
            case 'AttackHit': {
                let details = `Attack Roll: ${event.attack_roll ? this.formatRollResult(event.attack_roll, event.target_ac) : 'N/A'}`;
                if (event.damage_roll) {
                    details += `\nDamage: ${this.formatRollResult(event.damage_roll)}`;
                }
                return details;
            }
            case 'AttackMissed':
                return `Attack Roll: ${event.attack_roll ? this.formatRollResult(event.attack_roll, event.target_ac) : 'N/A'}`;
            case 'ActionStarted': {
                if (!event.decision_trace || Object.keys(event.decision_trace).length === 0) {
                    return "No decision trace available.";
                }
                const scores = Object.entries(event.decision_trace)
                    .sort(([, a], [, b]) => b - a)
                    .map(([name, score]) => `${name}: ${score.toFixed(1)}`)
                    .join("\n");
                return `AI Decision Trace (Scores):\n${scores}`;
            }
            default:
                return JSON.stringify(event, null, 2);
        }
    }
}
