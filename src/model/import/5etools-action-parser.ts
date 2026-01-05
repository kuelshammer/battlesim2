import { evaluateDiceFormula } from "../dice";
import { v4 as uuid } from 'uuid';
import { AtkAction } from "../model";

export function parse5eAttack(name: string, entry: string): AtkAction {
    // Extract toHit bonus: {@hit 4}
    const hitMatch = entry.match(/{@hit\s+(\d+)}/);
    const toHit = hitMatch ? parseInt(hitMatch[1]) : 0;

    // Extract damage formula: {@damage 1d6 + 2}
    // Note: We only take the first damage formula for simplicity
    const damageMatch = entry.match(/{@damage\s+([^}]+)}/);
    let dpr: string | number = 0;
    if (damageMatch) {
        // Store the raw formula string so the engine can roll it
        dpr = damageMatch[1];
    }

    return {
        id: uuid(),
        name,
        type: "atk",
        actionSlot: 0,
        cost: [],
        requirements: [],
        tags: [],
        freq: "at will",
        condition: "default",
        dpr: dpr,
        toHit: toHit,
        target: "enemy with most HP",
        targets: 1,
    };
}

export function parse5eMultiattack(entry: string): { total: number, details: string } | null {
    // Standard pattern: "makes X attacks"
    const numberMap: Record<string, number> = {
        "one": 1, "two": 2, "three": 3, "four": 4, "five": 5
    };

    const match = entry.match(/makes\s+(\w+)\s+(?:melee\s+)?attacks?/i);
    if (match) {
        const numStr = match[1].toLowerCase();
        const total = numberMap[numStr] || parseInt(numStr) || 1;
        
        // Extract details after the colon if present
        let details = "";
        const colonIndex = entry.indexOf(":");
        if (colonIndex !== -1) {
            details = entry.substring(colonIndex + 1).replace(/\.$/, "").trim();
            // Simplify "one with its bite and one with its claws" -> "bite and claws"
            details = details.replace(/one with its\s+/g, "").replace(/its\s+/g, "");
        } else {
            // e.g. "makes three melee attacks" -> details: "melee attacks"
            details = entry.split("makes")[1].trim().replace(/\.$/, "");
            // remove the number word from details
            details = details.replace(new RegExp(`^${numStr}\\s+`, 'i'), "");
        }

        return { total, details };
    }

    return null;
}