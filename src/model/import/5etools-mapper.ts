import { Creature, Action, CreatureType } from "../model";
import { Monster5eImport } from "./5etools-schema";
import { v4 as uuid } from 'uuid';
import { parse5eAttack, parse5eMultiattack } from "./5etools-action-parser";
import { evaluateDiceFormula } from "../dice";

export function mapMonster5eToCreature(monster: Monster5eImport): Creature {
    // Extract base AC: usually the first number in the array
    let ac = 10;
    if (monster.ac && monster.ac.length > 0) {
        const firstAc = monster.ac[0];
        if (typeof firstAc === 'number') {
            ac = firstAc;
        } else if (typeof firstAc === 'object' && firstAc && firstAc.ac) {
            ac = firstAc.ac;
        }
    }

    // Extract HP
    let hp = 0;
    if (monster.hp && monster.hp.average) {
        hp = monster.hp.average;
    }

    // Map actions
    const actions: Action[] = [];
    let multiattackInfo: { total: number, details: string } | null = null;

    if (monster.action) {
        // First pass: find multiattack
        for (const act of monster.action) {
            if (act && act.name === "Multiattack" && act.entries && act.entries.length > 0) {
                multiattackInfo = parse5eMultiattack(act.entries[0]);
            }
        }

        // Second pass: map attacks
        for (const act of monster.action) {
            if (!act || act.name === "Multiattack") continue;

            if (act.entries && act.entries.length > 0 && typeof act.entries[0] === 'string') {
                const parsedAction = parse5eAttack(act.name || "Unknown", act.entries[0]);
                // Cast to any because TS struggles with DiceFormula string|number comparison, use evaluate for safety
                if (evaluateDiceFormula(parsedAction.toHit, 0.5) > 0 || evaluateDiceFormula(parsedAction.dpr, 0.5) > 0) {
                    // Apply multiattack: if this attack is mentioned in multiattack details,
                    // or if it's the only attack and total > 1
                    if (multiattackInfo && act.name) {
                        const isMentioned = multiattackInfo.details.toLowerCase().includes(act.name.toLowerCase());
                        if (isMentioned || (monster.action.length === 2 && multiattackInfo.total > 1)) {
                            parsedAction.targets = multiattackInfo.total;
                        }
                    }
                    actions.push(parsedAction);
                }
            }
        }
    }

    // Calculate save bonus (average of ability modifiers as a placeholder)
    // In a real scenario, we would parse 'save' property or calculate from abilities
    const strMod = Math.floor(((monster.str || 10) - 10) / 2);
    const dexMod = Math.floor(((monster.dex || 10) - 10) / 2);
    const conMod = Math.floor(((monster.con || 10) - 10) / 2);
    const intMod = Math.floor(((monster.int || 10) - 10) / 2);
    const wisMod = Math.floor(((monster.wis || 10) - 10) / 2);
    const chaMod = Math.floor(((monster.cha || 10) - 10) / 2);

    const avgModifier = (strMod + dexMod + conMod + intMod + wisMod + chaMod) / 6;

    // Map type: could be string or object { type: "humanoid" }
    let creatureType: CreatureType | undefined = undefined;
    if (typeof monster.type === 'string') {
        creatureType = monster.type as CreatureType;
    } else if (typeof monster.type === 'object' && monster.type && monster.type.type) {
        creatureType = monster.type.type as CreatureType;
    }

    return {
        id: uuid(),
        mode: "monster",
        name: monster.name || "Unknown",
        src: monster.source,
        type: creatureType,
        count: 1,
        hp: hp,
        ac: ac,
        actions: actions,
        saveBonus: parseFloat(avgModifier.toFixed(3)),
        strSaveBonus: strMod,
        dexSaveBonus: dexMod,
        conSaveBonus: conMod,
        intSaveBonus: intMod,
        wisSaveBonus: wisMod,
        chaSaveBonus: chaMod,
        magicItems: [],
        initialBuffs: [],
    };
}