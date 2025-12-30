# Regression Test Scenarios Report

Date: 2025-12-30
Agent: Senior Frontend Architect & Avant-Garde UI Designer

## Overview
This report documents three critical missing simulation scenarios identified in `Battlesim2-2bz`. These scenarios have been verified using the native `sim_cli` and are ready to be integrated into the regression test suite.

## 1. Total Party Kill (TPK) Scenario
Tests the simulation's ability to handle the end of an adventure when all players are defeated.

**Verification:** Confirmed `UnitDied` events for all players and simulation termination.

### JSON Definition
```json
{
  "players": [
    {
      "id": "weak_player",
      "name": "Commoner",
      "count": 1,
      "hp": 4,
      "ac": 10,
      "saveBonus": 0,
      "initiativeBonus": 0,
      "actions": [
        {
          "id": "club",
          "name": "Club",
          "type": "atk",
          "dpr": "1d4",
          "toHit": "1d20+2",
          "target": "enemy with least HP",
          "freq": "at will",
          "condition": "default",
          "targets": 1
        }
      ]
    }
  ],
  "encounters": [
    {
      "monsters": [
        {
          "id": "deadly_boss",
          "name": "Ancient Red Dragon",
          "count": 1,
          "hp": 546,
          "ac": 22,
          "saveBonus": 10,
          "initiativeBonus": 0,
          "actions": [
            {
              "id": "fire_breath",
              "name": "Fire Breath",
              "type": "atk",
              "dpr": "91",
              "toHit": "1d20+14",
              "target": "enemy with least HP",
              "freq": "at will",
              "condition": "default",
              "targets": 1
            }
          ]
        }
      ]
    }
  ]
}
```

## 2. Multi-Encounter Adventure Scenario
Tests the `timeline` functionality, including multiple combat steps and short rests.

**Verification:** Confirmed execution of `step0` (Combat), `step1` (ShortRest - implicit), and `step2` (Combat).

### JSON Definition
```json
{
  "players": [
    {
      "id": "fighter1",
      "name": "Fighter 1",
      "count": 1,
      "hp": 30,
      "ac": 18,
      "saveBonus": 3,
      "initiativeBonus": 2,
      "actions": [
        {
          "id": "longsword1",
          "name": "Longsword",
          "type": "atk",
          "dpr": "1d8+3",
          "toHit": "1d20+5",
          "target": "enemy with least HP",
          "freq": "at will",
          "condition": "default",
          "targets": 1
        }
      ]
    }
  ],
  "timeline": [
    {
      "type": "combat",
      "monsters": [
        {
          "id": "goblin1",
          "name": "Goblin",
          "count": 2,
          "hp": 7,
          "ac": 15,
          "saveBonus": 0,
          "actions": [
            {
              "id": "scimitar1",
              "name": "Scimitar",
              "type": "atk",
              "dpr": "1d6+2",
              "toHit": "1d20+4",
              "target": "enemy with least HP",
              "freq": "at will",
              "condition": "default",
              "targets": 1
            }
          ]
        }
      ]
    },
    {
      "type": "shortRest",
      "id": "sr1"
    },
    {
      "type": "combat",
      "monsters": [
        {
          "id": "bugbear1",
          "name": "Bugbear",
          "count": 1,
          "hp": 27,
          "ac": 16,
          "saveBonus": 1,
          "actions": [
            {
              "id": "morningstar1",
              "name": "Morningstar",
              "type": "atk",
              "dpr": "2d8+2",
              "toHit": "1d20+4",
              "target": "enemy with least HP",
              "freq": "at will",
              "condition": "default",
              "targets": 1
            }
          ]
        }
      ]
    }
  ]
}
```

## 3. Reaction Chain Scenario
Tests the `triggers` system and reaction resource consumption (e.g., using the `Shield` spell).

**Verification:** Confirmed `ActionStarted` event for `shield_act` triggered by `on being attacked`.

### JSON Definition
```json
{
  "players": [
    {
      "id": "wizard",
      "name": "Wizard",
      "count": 1,
      "hp": 20,
      "ac": 12,
      "saveBonus": 0,
      "initiativeBonus": 2,
      "actions": [
        {
          "id": "firebolt",
          "name": "Firebolt",
          "type": "atk",
          "dpr": "1d10",
          "toHit": "1d20+5",
          "target": "enemy with least HP",
          "freq": "at will",
          "condition": "default",
          "targets": 1
        }
      ],
      "triggers": [
        {
          "id": "trigger_shield",
          "condition": "on being attacked",
          "cost": 4,
          "action": {
            "id": "shield_act",
            "type": "template",
            "templateOptions": {
              "templateName": "Shield"
            },
            "freq": "at will",
            "condition": "default",
            "targets": 1
          }
        }
      ],
      "spellSlots": {
        "1st": 4
      }
    }
  ],
  "encounters": [
    {
      "monsters": [
        {
          "id": "goblin",
          "name": "Goblin Archer",
          "count": 3,
          "hp": 7,
          "ac": 15,
          "saveBonus": 0,
          "actions": [
            {
              "id": "shortbow",
              "name": "Shortbow",
              "type": "atk",
              "dpr": "1d6+2",
              "toHit": "1d20+4",
              "target": "enemy with least HP",
              "freq": "at will",
              "condition": "default",
              "targets": 1
            }
          ]
        }
      ]
    }
  ]
}
```
