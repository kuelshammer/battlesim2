import { Creature } from "../model";
import { Monster5e } from "./5etools-schema";
import { v4 as uuid } from 'uuid';

export function mapMonster5eToCreature(monster: Monster5e): Creature {
    // Extract base AC: usually the first number in the array
    let ac = 10;
    if (monster.ac && monster.ac.length > 0) {
        const firstAc = monster.ac[0];
        if (typeof firstAc === 'number') {
            ac = firstAc;
        } else if (typeof firstAc === 'object' && firstAc.ac) {
            ac = firstAc.ac;
        }
    }

    // Extract HP
    const hp = monster.hp.average || 0;

    // Calculate save bonus (average of ability modifiers as a placeholder)
    // In a real scenario, we would parse 'save' property or calculate from abilities
    const strMod = Math.floor(((monster.str || 10) - 10) / 2);
    const dexMod = Math.floor(((monster.dex || 10) - 10) / 2);
    const conMod = Math.floor(((monster.con || 10) - 10) / 2);
    const intMod = Math.floor(((monster.int || 10) - 10) / 2);
    const wisMod = Math.floor(((monster.wis || 10) - 10) / 2);
    const chaMod = Math.floor(((monster.cha || 10) - 10) / 2);

    const avgModifier = (strMod + dexMod + conMod + intMod + wisMod + chaMod) / 6;

    return {
        id: uuid(),
        mode: "monster",
        name: monster.name,
        count: 1,
        hp: hp,
        ac: ac,
        actions: [],
        saveBonus: parseFloat(avgModifier.toFixed(3)),
        strSaveBonus: strMod,
        dexSaveBonus: dexMod,
        conSaveBonus: conMod,
        intSaveBonus: intMod,
        wisSaveBonus: wisMod,
        chaSaveBonus: chaMod,
    } as any; // Cast to any until all optional fields are handled correctly
}