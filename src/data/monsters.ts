// @ts-nocheck - Temporarily excluded: legacy actions need migration to new schema
import { Creature } from "../model/model"

/**
 * A small set of default monsters kept statically to support
 * templates and initial state without a network fetch.
 * The full library is now in public/data/monsters.json.
 */
export const DefaultMonsters: Creature[] = [
    {
        "id": "670d1340-c829-4746-b112-d96960ebd650",
        "mode": "monster",
        "name": "Bandit",
        "type": "humanoid",
        "src": "MM p.343",
        "cr": "1/8",
        "hp": 11,
        "ac": 12,
        "actions": [
            {
                "id": "932121f1-a8ab-4ad1-9c3a-3de0423ff70b",
                "name": "Scimitar",
                "type": "atk",
                "actionSlot": 0,
                "freq": "at will",
                "condition": "default",
                "dpr": 4.5,
                "toHit": 3,
                "target": "enemy with most HP",
                "targets": 1
            },
            {
                "id": "06e35845-60e7-4569-90de-3b57e1c6e1c4",
                "name": "Light Crossbow",
                "type": "atk",
                "actionSlot": 0,
                "freq": "at will",
                "condition": "default",
                "dpr": 5.5,
                "toHit": 3,
                "target": "enemy with most HP",
                "targets": 1
            }
        ],
        "count": 1,
        "saveBonus": 0.0625
    },
    {
        "id": "5ce452a5-0310-4520-9b89-7438c75567e7",
        "mode": "monster",
        "name": "Dire Wolf",
        "type": "beast",
        "src": "MM p.321",
        "cr": "1",
        "hp": 37,
        "ac": 14,
        "actions": [
            {
                "id": "5ce452a5-0310-4520-9b89-7438c75567e7",
                "name": "Bite",
                "type": "atk",
                "actionSlot": 0,
                "freq": "at will",
                "condition": "default",
                "dpr": 10,
                "toHit": 5,
                "target": "enemy with most HP",
                "targets": 1
            }
        ],
        "count": 1,
        "saveBonus": 0.5
    },
    {
        "id": "e6f4d0c0-359f-4104-8155-0b2e361fc629",
        "mode": "monster",
        "name": "Giant Constrictor Snake",
        "type": "beast",
        "src": "MM p.324",
        "cr": "2",
        "hp": 60,
        "ac": 12,
        "actions": [
            {
                "id": "e6f4d0c0-359f-4104-8155-0b2e361fc629",
                "name": "Bite",
                "type": "atk",
                "actionSlot": 0,
                "freq": "at will",
                "condition": "default",
                "dpr": 11,
                "toHit": 6,
                "target": "enemy with most HP",
                "targets": 1
            }
        ],
        "count": 1,
        "saveBonus": 1
    },
    {
        "id": "b17786ef-1e13-48ad-9264-f5447c3c4f3b",
        "mode": "monster",
        "name": "Giant Scorpion",
        "type": "beast",
        "src": "MM p.327",
        "cr": "3",
        "hp": 52,
        "ac": 15,
        "actions": [
            {
                "id": "b17786ef-1e13-48ad-9264-f5447c3c4f3b",
                "name": "Claw x 2 & Sting",
                "type": "atk",
                "actionSlot": 0,
                "freq": "at will",
                "condition": "default",
                "dpr": 42.5,
                "toHit": 4,
                "target": "enemy with most HP",
                "targets": 1
            }
        ],
        "count": 1,
        "saveBonus": 1.5
    },
    {
        "id": "23c0a9dc-6a83-4689-8f50-b4995bf8d95a",
        "mode": "monster",
        "name": "Fire Elemental",
        "type": "elemental",
        "src": "MM p.125",
        "cr": "5",
        "hp": 102,
        "ac": 13,
        "actions": [
            {
                "id": "23c0a9dc-6a83-4689-8f50-b4995bf8d95a",
                "name": "Touch x 2",
                "type": "atk",
                "actionSlot": 0,
                "freq": "at will",
                "condition": "default",
                "dpr": 15.5,
                "toHit": 6,
                "target": "enemy with most HP",
                "targets": 2
            }
        ],
        "count": 1,
        "saveBonus": 2.5
    }
]

export function getMonster(name: string): Creature | undefined {
    return DefaultMonsters.find(monster => (monster.name === name))
}
