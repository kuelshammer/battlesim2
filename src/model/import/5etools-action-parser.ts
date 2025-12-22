import { evaluateDiceFormula } from "../dice";

export function parse5eAttack(name: string, entry: string): any {
    // Extract toHit bonus: {@hit 4}
    const hitMatch = entry.match(/{@hit\s+(\d+)}/);
    const toHit = hitMatch ? parseInt(hitMatch[1]) : 0;

    // Extract damage formula: {@damage 1d6 + 2}
    // Note: We only take the first damage formula for simplicity in the initial version
    const damageMatch = entry.match(/{@damage\s+([^}]+)}/);
    let dpr = 0;
    if (damageMatch) {
        const formula = damageMatch[1];
        try {
            // Use 0.5 luck for "average" damage calculation
            dpr = evaluateDiceFormula(formula, 0.5);
        } catch (e) {
            console.error(`Failed to evaluate dice formula: ${formula}`, e);
        }
    }

    return {
        name,
        type: "atk",
        actionSlot: 0,
        freq: "at will",
        condition: "default",
        dpr: dpr,
        toHit: toHit,
        target: "enemy with most HP",
        targets: 1,
    };
}