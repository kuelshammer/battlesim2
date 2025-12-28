/**
 * Example encounters for onboarding and demonstration
 *
 * These are pre-built scenarios that showcase different aspects of the simulator:
 * - Level 1: Simple encounter (Goblin Ambush)
 * - Level 3: Mixed combat (Orc Raid)
 * - Level 10: Complex boss fight (Dragon Encounter)
 */

import { v4 as uuid } from 'uuid'

/**
 * Goblin Ambush - A classic Level 1 D&D scenario
 *
 * 4 Level 1 Adventurers vs 4 Goblins
 * This demonstrates basic simulation features with a simple, balanced encounter.
 */
export const goblinAmbush = {
  name: 'Goblin Ambush',
  description: 'A classic Level 1 encounter. Four goblins ambush the party on a forest road.',
  level: 1,
  difficulty: 'Easy',
  players: [
    {
      id: uuid(),
      mode: 'player',
      name: 'Fighter',
      class: { type: 'fighter', level: 1, options: {} },
      hp: 12,
      ac: 16,
      count: 1,
      saveBonus: 2,
      initiativeBonus: 3,
      actions: [
        {
          id: uuid(),
          name: 'Longsword',
          actionSlot: 0,
          type: 'atk',
          freq: 'at will',
          condition: 'default',
          dpr: '1d8+3',
          toHit: 5,
          target: 'enemy with least HP',
          targets: 1,
          cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
          requirements: [],
          tags: ['Melee', 'Weapon', 'Attack', 'Damage'],
        },
      ],
      triggers: [],
    },
    {
      id: uuid(),
      mode: 'player',
      name: 'Cleric',
      class: { type: 'cleric', level: 1, options: {} },
      hp: 8,
      ac: 16,
      count: 1,
      saveBonus: 2,
      initiativeBonus: 2,
      actions: [
        {
          id: uuid(),
          name: 'Mace',
          actionSlot: 0,
          type: 'atk',
          freq: 'at will',
          condition: 'default',
          dpr: '1d6+2',
          toHit: 4,
          target: 'enemy with least HP',
          targets: 1,
          cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
          requirements: [],
          tags: ['Melee', 'Weapon', 'Attack', 'Damage'],
        },
      ],
      triggers: [],
    },
  ],
  encounters: [
    {
      id: uuid(),
      type: 'combat',
      monsters: [
        {
          id: uuid(),
          mode: 'monster',
          name: 'Goblin',
          type: 'humanoid',
          cr: '1/4',
          hp: 7,
          ac: 15,
          count: 4,
          saveBonus: 1,
          actions: [
            {
              id: uuid(),
              name: 'Scimitar',
              actionSlot: 0,
              type: 'atk',
              freq: 'at will',
              condition: 'default',
              dpr: '1d6+2',
              toHit: 4,
              target: 'enemy with least HP',
              targets: 1,
              cost: [],
              requirements: [],
              tags: ['Melee', 'Weapon', 'Attack', 'Damage'],
            },
          ],
          triggers: [],
        },
      ],
      playersSurprised: false,
      monstersSurprised: true, // Goblins ambush from stealth
      targetRole: 'Standard',
    },
  ],
}

/**
 * Orc Raid - A Level 3 mixed combat scenario
 *
 * 4 Level 3 Adventurers vs 4 Orcs + 1 Orc Eye of Gruumsh
 * Demonstrates mixed melee combat with a support spellcaster.
 */
export const orcRaid = {
  name: 'Orc Raid',
  description: 'A brutal orc warband raids a village. Includes melee warriors and a shaman.',
  level: 3,
  difficulty: 'Medium',
  players: [
    {
      id: uuid(),
      mode: 'player',
      name: 'Fighter',
      class: { type: 'fighter', level: 3, options: {} },
      hp: 31,
      ac: 18,
      count: 1,
      saveBonus: 3,
      initiativeBonus: 3,
      actions: [
        {
          id: uuid(),
          name: 'Greatsword x2',
          actionSlot: 0,
          type: 'atk',
          freq: 'at will',
          condition: 'default',
          dpr: '2d6+3',
          toHit: 6,
          target: 'enemy with least HP',
          targets: 1,
          cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
          requirements: [],
          tags: ['Melee', 'Weapon', 'Attack', 'Damage'],
        },
      ],
      triggers: [],
    },
    {
      id: uuid(),
      mode: 'player',
      name: 'Ranger',
      class: { type: 'ranger', level: 3, options: {} },
      hp: 27,
      ac: 15,
      count: 1,
      saveBonus: 3,
      initiativeBonus: 3,
      actions: [
        {
          id: uuid(),
          name: 'Longbow',
          actionSlot: 0,
          type: 'atk',
          freq: 'at will',
          condition: 'default',
          dpr: '1d8+4',
          toHit: 6,
          target: 'enemy with least HP',
          targets: 1,
          cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
          requirements: [],
          tags: ['Ranged', 'Weapon', 'Attack', 'Damage'],
        },
      ],
      triggers: [],
    },
  ],
  encounters: [
    {
      id: uuid(),
      type: 'combat',
      monsters: [
        {
          id: uuid(),
          mode: 'monster',
          name: 'Orc',
          type: 'humanoid',
          cr: '1/2',
          hp: 15,
          ac: 13,
          count: 4,
          saveBonus: 1,
          actions: [
            {
              id: uuid(),
              name: 'Greataxe',
              actionSlot: 0,
              type: 'atk',
              freq: 'at will',
              condition: 'default',
              dpr: '1d12+3',
              toHit: 5,
              target: 'enemy with least HP',
              targets: 1,
              cost: [],
              requirements: [],
              tags: ['Melee', 'Weapon', 'Attack', 'Damage'],
            },
          ],
          triggers: [],
        },
      ],
      playersSurprised: false,
      monstersSurprised: false,
      targetRole: 'Standard',
    },
  ],
}

/**
 * Dragon Encounter - A Level 10 boss fight scenario
 *
 * 4 Level 10 Adventurers vs 1 Adult Red Dragon
 * Demonstrates legendary actions, AOE damage, and high-level combat.
 */
export const dragonEncounter = {
  name: 'Red Dragon Fight',
  description: 'An epic boss fight against an Adult Red Dragon with legendary actions and breath weapon.',
  level: 10,
  difficulty: 'Hard',
  players: [
    {
      id: uuid(),
      mode: 'player',
      name: 'Fighter (Champion)',
      class: { type: 'fighter', level: 10, options: {} },
      hp: 96,
      ac: 20,
      count: 1,
      saveBonus: 6,
      initiativeBonus: 6,
      actions: [
        {
          id: uuid(),
          name: 'Greatsword x3',
          actionSlot: 0,
          type: 'atk',
          freq: 'at will',
          condition: 'default',
          dpr: '2d6+6',
          toHit: 11,
          target: 'enemy with least HP',
          targets: 3,
          cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
          requirements: [],
          tags: ['Melee', 'Weapon', 'Attack', 'Damage'],
        },
      ],
      triggers: [],
    },
    {
      id: uuid(),
      mode: 'player',
      name: 'Wizard',
      class: { type: 'wizard', level: 10, options: {} },
      hp: 52,
      ac: 14,
      count: 1,
      saveBonus: 5,
      initiativeBonus: 4,
      spellSlots: {
        '1': 4,
        '2': 3,
        '3': 3,
        '4': 3,
        '5': 2,
      },
      actions: [
        {
          id: uuid(),
          name: 'Fireball',
          actionSlot: 0,
          type: 'atk',
          freq: { reset: 'sr', uses: 3 },
          condition: 'default',
          dpr: '8d6',
          toHit: 15, // DC instead of to-hit
          target: 'all enemies',
          targets: 10,
          cost: [{ type: 'Discrete', resourceType: 'Action', amount: 1 }],
          requirements: [],
          tags: ['Spell', 'Fire', 'Evocation', 'AoE', 'Attack', 'Damage'],
        },
      ],
      triggers: [],
    },
  ],
  encounters: [
    {
      id: uuid(),
      type: 'combat',
      monsters: [
        {
          id: uuid(),
          mode: 'monster',
          name: 'Adult Red Dragon',
          type: 'dragon',
          cr: '14',
          hp: 189,
          ac: 19,
          count: 1,
          saveBonus: 9,
          actions: [
            {
              id: uuid(),
              name: 'Multiattack',
              actionSlot: 0,
              type: 'atk',
              freq: 'at will',
              condition: 'default',
              dpr: '2d10+10',
              toHit: 12,
              target: 'enemy with least HP',
              targets: 2,
              cost: [],
              requirements: [],
              tags: ['Melee', 'Weapon', 'Attack', 'Damage'],
            },
            {
              id: uuid(),
              name: 'Breath Weapon',
              actionSlot: 0,
              type: 'atk',
              freq: { reset: 'recharge', cooldownRounds: 5 },
              condition: 'default',
              dpr: '18d6',
              toHit: 18, // DEX save DC
              target: 'all enemies',
              targets: 10,
              cost: [],
              requirements: [],
              tags: ['Fire', 'AoE', 'Attack', 'Damage'],
            },
          ],
          triggers: [],
        },
      ],
      playersSurprised: false,
      monstersSurprised: false,
      targetRole: 'Boss',
    },
  ],
}

/**
 * All example encounters
 */
export const exampleEncounters = [
  goblinAmbush,
  orcRaid,
  dragonEncounter,
] as const

/**
 * Get an example encounter by name
 */
export function getExampleEncounter(name: string): typeof exampleEncounters[number] | undefined {
  return exampleEncounters.find((ex) => ex.name === name)
}
