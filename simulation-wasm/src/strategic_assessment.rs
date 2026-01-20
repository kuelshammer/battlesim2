//! Strategic assessment and action economy calculation
//!
//! This module provides functions for assessing combat situations and calculating
//! action economy metrics to guide tactical decisions.

use crate::combat_stats::CombatantStats;
use crate::model::Combattant;
use serde::{Deserialize, Serialize};

/// Conservative DPR calculation mode
///
/// When calculating player DPR, we want to be conservative (underestimate)
/// When calculating monster DPR, we also want to be conservative (overestimate threat)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConservativeMode {
    /// Player attacking monsters - use 75th percentile AC (harder to hit)
    PlayerVsMonsters,
    /// Monster attacking players - use 25th percentile AC (easier to hit)
    MonsterVsPlayers,
}

/// Estimate DPR for a combatant against a group of opponents
///
/// Uses percentile AC of opponents to provide conservative estimates:
/// - PlayerVsMonsters: 75th percentile AC (underestimates player DPR)
/// - MonsterVsPlayers: 25th percentile AC (overestimates monster DPR)
pub fn estimate_dpr_vs_opponents(
    combatant: &Combattant,
    opponents: &[&Combattant],
    mode: ConservativeMode,
) -> f64 {
    if opponents.is_empty() {
        return 0.0;
    }

    // Extract ACs from opponents
    let mut acs: Vec<f64> = opponents.iter().map(|c| c.creature.ac as f64).collect();

    if acs.is_empty() {
        return 0.0;
    }

    acs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Calculate percentile based on mode
    let percentile_ac = match mode {
        ConservativeMode::PlayerVsMonsters => {
            // 75th percentile - harder for players to hit (conservative)
            let idx = ((acs.len() as f64) * 0.75) as usize;
            acs[idx.min(acs.len() - 1)]
        }
        ConservativeMode::MonsterVsPlayers => {
            // 25th percentile - easier for monsters to hit (conservative)
            let idx = ((acs.len() as f64) * 0.25) as usize;
            acs[idx.min(acs.len() - 1)]
        }
    };

    // Use existing CombatantStats to get DPR vs this AC
    let stats = CombatantStats::calculate(&combatant.creature);
    stats.get_dpr_vs_ac(percentile_ac)
}

/// Action economy state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionEconomyState {
    EnemyAdvantage,  // Monsters have action/DPR advantage → Nova tactics
    Even,            // Balanced action economy
    PlayerAdvantage, // Players have action/DPR advantage → Conserve
}

impl std::fmt::Display for ActionEconomyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionEconomyState::EnemyAdvantage => write!(f, "Enemy Advantage"),
            ActionEconomyState::Even => write!(f, "Even"),
            ActionEconomyState::PlayerAdvantage => write!(f, "Player Advantage"),
        }
    }
}

/// Action economy status for an encounter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionEconomyStatus {
    /// Number of conscious players
    pub player_count: usize,

    /// Number of conscious monsters
    pub monster_count: usize,

    /// Estimated rounds until all monsters are killed
    pub rounds_to_kill_monsters: f64,

    /// Estimated rounds until all players are killed
    pub rounds_to_kill_players: f64,

    /// Combined action economy ratio
    pub combined_ratio: f64,

    /// Raw action count ratio (players/monsters)
    pub action_ratio: f64,

    /// Time ratio (who exhausts first)
    pub time_ratio: f64,

    /// Current state
    pub state: ActionEconomyState,
}

impl ActionEconomyStatus {
    /// Get display icon for state
    pub fn icon(&self) -> &str {
        match self.state {
            ActionEconomyState::EnemyAdvantage => "🔴",
            ActionEconomyState::Even => "🟡",
            ActionEconomyState::PlayerAdvantage => "🟢",
        }
    }

    /// Get tactical recommendation
    pub fn recommendation(&self) -> &str {
        match self.state {
            ActionEconomyState::EnemyAdvantage => {
                "Nova! Use AoE and burst damage to reduce their action count fast."
            }
            ActionEconomyState::Even => "Normal tactics. Balance damage and resource conservation.",
            ActionEconomyState::PlayerAdvantage => {
                "Conserve! Use cantrips and basic attacks. Save resources for harder fights."
            }
        }
    }

    /// Format rounds to exhaustion for display
    pub fn format_rounds(&self) -> String {
        format!(
            "You {:.1} rounds | Them {:.1} rounds",
            self.rounds_to_kill_monsters, self.rounds_to_kill_players
        )
    }
}

/// Calculate action economy status for current combat state
///
/// Factors in:
/// - Action count (who has more combatants acting)
/// - DPR (who deals more damage per round)
/// - HP (who has more staying power)
///
/// Returns analysis of "who wins the race to exhaustion?"
pub fn calculate_action_economy(
    players: &[&Combattant],
    monsters: &[&Combattant],
) -> ActionEconomyStatus {
    // Filter to conscious combatants only
    let conscious_players: Vec<&Combattant> = players
        .iter()
        .filter(|p| p.final_state.current_hp > 0)
        .copied()
        .collect();

    let conscious_monsters: Vec<&Combattant> = monsters
        .iter()
        .filter(|m| m.final_state.current_hp > 0)
        .copied()
        .collect();

    let player_count = conscious_players.len();
    let monster_count = conscious_monsters.len();

    // Handle edge cases
    if monster_count == 0 {
        return ActionEconomyStatus {
            player_count,
            monster_count: 0,
            rounds_to_kill_monsters: 0.0,
            rounds_to_kill_players: f64::INFINITY,
            combined_ratio: f64::INFINITY,
            action_ratio: f64::INFINITY,
            time_ratio: 0.0,
            state: ActionEconomyState::PlayerAdvantage,
        };
    }

    if player_count == 0 {
        return ActionEconomyStatus {
            player_count: 0,
            monster_count,
            rounds_to_kill_monsters: f64::INFINITY,
            rounds_to_kill_players: 0.0,
            combined_ratio: 0.0,
            action_ratio: 0.0,
            time_ratio: f64::INFINITY,
            state: ActionEconomyState::EnemyAdvantage,
        };
    }

    // Calculate total HP
    let total_player_hp: f64 = conscious_players
        .iter()
        .map(|p| p.final_state.current_hp as f64)
        .sum();

    let total_monster_hp: f64 = conscious_monsters
        .iter()
        .map(|m| m.final_state.current_hp as f64)
        .sum();

    // Calculate DPR (using conservative estimates)
    let player_dpr: f64 = conscious_players
        .iter()
        .copied()
        .map(|p| {
            estimate_dpr_vs_opponents(
                p,
                conscious_monsters.as_slice(),
                ConservativeMode::PlayerVsMonsters,
            )
        })
        .sum();

    let monster_dpr: f64 = conscious_monsters
        .iter()
        .copied()
        .map(|m| {
            estimate_dpr_vs_opponents(
                m,
                conscious_players.as_slice(),
                ConservativeMode::MonsterVsPlayers,
            )
        })
        .sum();

    // Calculate rounds to exhaustion ("who wins the race?")
    let rounds_to_kill_monsters = if player_dpr > 0.0 {
        total_monster_hp / player_dpr
    } else {
        f64::INFINITY
    };

    let rounds_to_kill_players = if monster_dpr > 0.0 {
        total_player_hp / monster_dpr
    } else {
        f64::INFINITY
    };

    // Calculate ratios
    // time_ratio: rounds_to_kill_monsters / rounds_to_kill_players
    //   < 1.0 = players finish first (advantage)
    //   > 1.0 = monsters finish first (disadvantage)
    //
    // Note: Action count is already factored into time_ratio through DPR:
    // - More players = higher player_dpr = fewer rounds to kill monsters
    // - More monsters = higher monster_dpr = fewer rounds to kill players
    // So time_ratio already captures the action economy effect
    let action_ratio = player_count as f64 / monster_count as f64;
    let time_ratio = rounds_to_kill_monsters / rounds_to_kill_players;

    // Use time_ratio directly as combined_ratio
    // The DPR estimation is already conservative (75th percentile for players, 25th for monsters)
    // so no additional factors needed
    let combined_ratio = time_ratio;

    // Determine state based on combined ratio
    // time_ratio < 1.0 = players finish first = Player Advantage
    // time_ratio > 1.0 = monsters finish first = Enemy Advantage
    //
    // Thresholds:
    // < 0.6 = Player Advantage (players clearly winning, >1.6x faster)
    // 0.6 - 1.5 = Even (roughly balanced, 0.67x to 1.5x speed difference)
    // > 1.5 = Enemy Advantage (monsters clearly winning, >1.5x faster)
    let state = if combined_ratio < 0.6 {
        ActionEconomyState::PlayerAdvantage
    } else if combined_ratio > 1.5 {
        ActionEconomyState::EnemyAdvantage
    } else {
        ActionEconomyState::Even
    };

    ActionEconomyStatus {
        player_count,
        monster_count,
        rounds_to_kill_monsters,
        rounds_to_kill_players,
        combined_ratio,
        action_ratio,
        time_ratio,
        state,
    }
}

#[cfg(test)]
#[allow(clippy::useless_vec)]
mod tests {
    use super::*;
    use crate::enums::EnemyTarget;
    use crate::model::{Action, AtkAction, Creature, DiceFormula, Frequency};

    #[test]
    fn test_estimate_dpr_vs_opponents_empty() {
        let creature = Creature {
            id: "test".to_string(),
            name: "Test".to_string(),
            hp: 50,
            ac: 15,
            arrival: None,
            mode: "monster".to_string(),
            count: 1.0,
            speed_fly: None,
            save_bonus: 0.0,
            str_save_bonus: None,
            dex_save_bonus: None,
            con_save_bonus: None,
            int_save_bonus: None,
            wis_save_bonus: None,
            cha_save_bonus: None,
            con_save_advantage: None,
            save_advantage: None,
            initiative_bonus: DiceFormula::Value(0.0),
            initiative_advantage: false,
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            magic_items: vec![],
            max_arcane_ward_hp: None,
            initial_buffs: vec![],
        };

        let combatant = Combattant {
            id: "test_combatant".to_string(),
            team: 0,
            creature: std::sync::Arc::new(creature),
            initiative: 10.0,
            initial_state: Default::default(),
            final_state: Default::default(),
            actions: vec![],
        };

        let opponents: Vec<&Combattant> = vec![];
        let result =
            estimate_dpr_vs_opponents(&combatant, &opponents, ConservativeMode::PlayerVsMonsters);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_estimate_dpr_percentile_calculation() {
        // Create monsters with different ACs: [10, 12, 15, 18, 20]
        let monsters: Vec<Combattant> = (0..5)
            .map(|i| {
                let ac = 10 + i * 2 + if i < 3 { 0 } else { 3 }; // 10, 12, 15, 18, 20
                let creature = Creature {
                    id: format!("m{}", i),
                    name: format!("Monster {}", i),
                    hp: 20,
                    ac,
                    arrival: None,
                    mode: "monster".to_string(),
                    count: 1.0,
                    speed_fly: None,
                    save_bonus: 0.0,
                    str_save_bonus: None,
                    dex_save_bonus: None,
                    con_save_bonus: None,
                    int_save_bonus: None,
                    wis_save_bonus: None,
                    cha_save_bonus: None,
                    con_save_advantage: None,
                    save_advantage: None,
                    initiative_bonus: DiceFormula::Value(0.0),
                    initiative_advantage: false,
                    actions: vec![],
                    triggers: vec![],
                    spell_slots: None,
                    class_resources: None,
                    hit_dice: None,
                    con_modifier: None,
                    magic_items: vec![],
                    max_arcane_ward_hp: None,
                    initial_buffs: vec![],
                };
                Combattant {
                    id: format!("monster_{}", i),
                    team: 1,
                    creature: std::sync::Arc::new(creature),
                    initiative: 10.0,
                    initial_state: Default::default(),
                    final_state: Default::default(),
                    actions: vec![],
                }
            })
            .collect();

        // Create a player with +10 to hit (80% hit chance vs AC 15)
        // and 10 DPR
        let mut player_creature = Creature {
            id: "player".to_string(),
            name: "Player".to_string(),
            hp: 50,
            ac: 16,
            arrival: None,
            mode: "player".to_string(),
            count: 1.0,
            speed_fly: None,
            save_bonus: 0.0,
            str_save_bonus: None,
            dex_save_bonus: None,
            con_save_bonus: None,
            int_save_bonus: None,
            wis_save_bonus: None,
            cha_save_bonus: None,
            con_save_advantage: None,
            save_advantage: None,
            initiative_bonus: DiceFormula::Value(2.0),
            initiative_advantage: false,
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            magic_items: vec![],
            max_arcane_ward_hp: None,
            initial_buffs: vec![],
        };

        // Add an attack action
        player_creature.actions.push(Action::Atk(AtkAction {
            id: "attack".to_string(),
            name: "Attack".to_string(),
            action_slot: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
            freq: Frequency::Static("at will".to_string()),
            condition: crate::enums::ActionCondition::Default,
            targets: 1,
            dpr: DiceFormula::Value(10.0),   // 10 DPR vs AC 15
            to_hit: DiceFormula::Value(5.0), // +5 to hit
            target: EnemyTarget::EnemyWithLeastHP,
            use_saves: None,
            half_on_save: None,
            rider_effect: None,
        }));

        let player = Combattant {
            id: "player".to_string(),
            team: 0,
            creature: std::sync::Arc::new(player_creature),
            initiative: 10.0,
            initial_state: Default::default(),
            final_state: Default::default(),
            actions: vec![],
        };

        let monster_refs: Vec<&Combattant> = monsters.iter().collect();

        // Test PlayerVsMonsters (75th percentile = AC 18)
        let dpr_vs_75th =
            estimate_dpr_vs_opponents(&player, &monster_refs, ConservativeMode::PlayerVsMonsters);

        // Expected calculation:
        // - to_hit = 5.0, needed_roll vs AC 15 = 10, hit_chance = (21-10)/20 = 0.55
        // - Expected DPR vs AC 15 = 10.0 * 0.55 = 5.5
        // - AC 18 is 3 points harder than baseline: adjustment = 1.0 - (3 * 0.05) = 0.85
        // - Final DPR = 5.5 * 0.85 ≈ 4.675
        assert!(dpr_vs_75th < 5.5);
        assert!(dpr_vs_75th > 4.0);

        // Test MonsterVsPlayers (25th percentile = AC 12)
        // (This would be used when a monster attacks the players)
        // For now just verify it runs and returns a value
        let _dpr_vs_25th =
            estimate_dpr_vs_opponents(&player, &monster_refs, ConservativeMode::MonsterVsPlayers);
    }

    #[test]
    fn test_action_economy_outnumbered_but_winning() {
        // Test case where players are outnumbered but winning the damage race
        // Players: 4, Monsters: 8
        // Players kill monsters in 6 rounds, monsters kill players in 13 rounds
        // Expected: Even or PlayerAdvantage (NOT EnemyAdvantage!)

        let players: Vec<Combattant> = (0..4)
            .map(|i| {
                let creature = Creature {
                    id: format!("player_{}", i),
                    name: format!("Player {}", i),
                    hp: 50,
                    ac: 16,
                    arrival: None,
                    mode: "player".to_string(),
                    count: 1.0,
                    speed_fly: None,
                    save_bonus: 0.0,
                    str_save_bonus: None,
                    dex_save_bonus: None,
                    con_save_bonus: None,
                    int_save_bonus: None,
                    wis_save_bonus: None,
                    cha_save_bonus: None,
                    con_save_advantage: None,
                    save_advantage: None,
                    initiative_bonus: DiceFormula::Value(0.0),
                    initiative_advantage: false,
                    actions: vec![],
                    triggers: vec![],
                    spell_slots: None,
                    class_resources: None,
                    hit_dice: None,
                    con_modifier: None,
                    magic_items: vec![],
                    max_arcane_ward_hp: None,
                    initial_buffs: vec![],
                };
                Combattant {
                    id: format!("player_{}", i),
                    team: 0,
                    creature: std::sync::Arc::new(creature),
                    initiative: 10.0,
                    initial_state: crate::model::CreatureState {
                        current_hp: 50,
                        ..Default::default()
                    },
                    final_state: crate::model::CreatureState {
                        current_hp: 50,
                        ..Default::default()
                    },
                    actions: vec![],
                }
            })
            .collect();

        let monsters: Vec<Combattant> = (0..8)
            .map(|i| {
                let creature = Creature {
                    id: format!("monster_{}", i),
                    name: format!("Monster {}", i),
                    hp: 30,
                    ac: 13,
                    arrival: None,
                    mode: "monster".to_string(),
                    count: 1.0,
                    speed_fly: None,
                    save_bonus: 0.0,
                    str_save_bonus: None,
                    dex_save_bonus: None,
                    con_save_bonus: None,
                    int_save_bonus: None,
                    wis_save_bonus: None,
                    cha_save_bonus: None,
                    con_save_advantage: None,
                    save_advantage: None,
                    initiative_bonus: DiceFormula::Value(0.0),
                    initiative_advantage: false,
                    actions: vec![],
                    triggers: vec![],
                    spell_slots: None,
                    class_resources: None,
                    hit_dice: None,
                    con_modifier: None,
                    magic_items: vec![],
                    max_arcane_ward_hp: None,
                    initial_buffs: vec![],
                };
                Combattant {
                    id: format!("monster_{}", i),
                    team: 1,
                    creature: std::sync::Arc::new(creature),
                    initiative: 10.0,
                    initial_state: crate::model::CreatureState {
                        current_hp: 30,
                        ..Default::default()
                    },
                    final_state: crate::model::CreatureState {
                        current_hp: 30,
                        ..Default::default()
                    },
                    actions: vec![],
                }
            })
            .collect();

        let player_refs: Vec<&Combattant> = players.iter().collect();
        let monster_refs: Vec<&Combattant> = monsters.iter().collect();

        let result = calculate_action_economy(&player_refs, &monster_refs);

        // Players are outnumbered (4 vs 8) but winning the race
        // Should NOT be EnemyAdvantage!
        assert_ne!(result.state, ActionEconomyState::EnemyAdvantage,
            "Players should not be at disadvantage when they're winning the race (6.1 vs 13.6 rounds)");

        // The state should be Even or PlayerAdvantage
        // (Even makes sense given they're outnumbered but winning)
        assert!(
            result.state == ActionEconomyState::Even
                || result.state == ActionEconomyState::PlayerAdvantage,
            "Expected Even or PlayerAdvantage, got {:?}",
            result.state
        );
    }

    #[test]
    fn test_action_economy_edge_cases() {
        // Empty combat
        let players: &[&Combattant] = &[];
        let monsters: &[&Combattant] = &[];
        let result = calculate_action_economy(players, monsters);
        assert_eq!(result.player_count, 0);
        assert_eq!(result.monster_count, 0);
        assert_eq!(result.state, ActionEconomyState::PlayerAdvantage); // Default fallback
    }

    #[test]
    fn test_action_economy_state_display() {
        // Test display methods
        let status = ActionEconomyStatus {
            player_count: 4,
            monster_count: 8,
            rounds_to_kill_monsters: 6.1,
            rounds_to_kill_players: 13.6,
            combined_ratio: 0.21,
            action_ratio: 0.5,
            time_ratio: 0.45,
            state: ActionEconomyState::EnemyAdvantage,
        };

        assert_eq!(status.icon(), "🔴");
        assert_eq!(status.state.to_string(), "Enemy Advantage");
        assert!(status.recommendation().contains("Nova"));

        let formatted = status.format_rounds();
        assert!(formatted.contains("6.1"));
        assert!(formatted.contains("13.6"));
    }

    // Helper function to create a basic combatant
    fn create_combatant(
        id: String,
        name: String,
        hp: u32,
        ac: u32,
        team: u32,
        dpr: f64,
        to_hit: f64,
    ) -> Combattant {
        let mut actions = vec![];
        if dpr > 0.0 {
            actions.push(Action::Atk(AtkAction {
                id: format!("attack_{}", id),
                name: "Attack".to_string(),
                action_slot: None,
                cost: vec![],
                requirements: vec![],
                tags: vec![],
                freq: Frequency::Static("at will".to_string()),
                condition: crate::enums::ActionCondition::Default,
                targets: 1,
                dpr: DiceFormula::Value(dpr),
                to_hit: DiceFormula::Value(to_hit),
                target: EnemyTarget::EnemyWithLeastHP,
                use_saves: None,
                half_on_save: None,
                rider_effect: None,
            }));
        }

        Combattant {
            id: id.clone(),
            team,
            creature: std::sync::Arc::new(Creature {
                id,
                name,
                hp,
                ac,
                arrival: None,
                mode: if team == 0 { "player" } else { "monster" }.to_string(),
                count: 1.0,
                speed_fly: None,
                save_bonus: 0.0,
                str_save_bonus: None,
                dex_save_bonus: None,
                con_save_bonus: None,
                int_save_bonus: None,
                wis_save_bonus: None,
                cha_save_bonus: None,
                con_save_advantage: None,
                save_advantage: None,
                initiative_bonus: DiceFormula::Value(0.0),
                initiative_advantage: false,
                actions,
                triggers: vec![],
                spell_slots: None,
                class_resources: None,
                hit_dice: None,
                con_modifier: None,
                magic_items: vec![],
                max_arcane_ward_hp: None,
                initial_buffs: vec![],
            }),
            initiative: 10.0,
            initial_state: crate::model::CreatureState {
                current_hp: hp,
                ..Default::default()
            },
            final_state: crate::model::CreatureState {
                current_hp: hp,
                ..Default::default()
            },
            actions: vec![],
        }
    }

    #[test]
    fn test_20_goblins_vs_2_fighters() {
        // Task case 1: 20 goblins vs 2 lvl 10 fighters
        // Many weak monsters vs few strong players
        // Due to AC differences (fighters AC 20, goblins AC 15) and to-hit chances,
        // fighters have significant advantage despite action count

        let players: Vec<Combattant> = (0..2)
            .map(|i| {
                // Lvl 10 fighter: 120 HP, AC 20, ~40 DPR (multiple attacks)
                create_combatant(
                    format!("fighter_{}", i),
                    format!("Fighter {}", i),
                    120,
                    20,
                    0,
                    40.0,
                    10.0,
                )
            })
            .collect();

        let monsters: Vec<Combattant> = (0..20)
            .map(|i| {
                // Goblin: 7 HP, AC 15, ~8 DPR
                create_combatant(
                    format!("goblin_{}", i),
                    format!("Goblin {}", i),
                    7,
                    15,
                    1,
                    8.0,
                    4.0,
                )
            })
            .collect();

        let player_refs: Vec<&Combattant> = players.iter().collect();
        let monster_refs: Vec<&Combattant> = monsters.iter().collect();

        let result = calculate_action_economy(&player_refs, &monster_refs);

        // 20 goblins with ~160 total DPR vs 2 fighters with ~80 DPR
        // But goblins have low to-hit (4.0) vs fighters' high AC (20)
        // While fighters have good to-hit (10.0) vs goblins' low AC (15)
        // Result favors players due to accuracy advantage
        assert_eq!(result.player_count, 2);
        assert_eq!(result.monster_count, 20);

        // Should be PlayerAdvantage - fighters are much more accurate
        assert_eq!(result.state, ActionEconomyState::PlayerAdvantage);
    }

    #[test]
    fn test_1_dragon_vs_4_commoners() {
        // Task case 2: 1 ancient dragon vs 4 lvl 1 commoners
        // One very strong monster vs many very weak players
        // Expected: Enemy Advantage (single monster too powerful)

        let players: Vec<Combattant> = (0..4)
            .map(|i| {
                // Lvl 1 commoner: 8 HP, AC 10, ~4 DPR
                create_combatant(
                    format!("commoner_{}", i),
                    format!("Commoner {}", i),
                    8,
                    10,
                    0,
                    4.0,
                    2.0,
                )
            })
            .collect();

        let monsters: Vec<Combattant> = vec![
            // Ancient dragon: 400 HP, AC 22, ~120 DPR (breath + multiple attacks)
            create_combatant(
                "ancient_dragon".to_string(),
                "Ancient Red Dragon".to_string(),
                400,
                22,
                1,
                120.0,
                15.0,
            ),
        ];

        let player_refs: Vec<&Combattant> = players.iter().collect();
        let monster_refs: Vec<&Combattant> = monsters.iter().collect();

        let result = calculate_action_economy(&player_refs, &monster_refs);

        assert_eq!(result.player_count, 4);
        assert_eq!(result.monster_count, 1);

        // Dragon is overwhelmingly powerful
        assert_eq!(result.state, ActionEconomyState::EnemyAdvantage);
    }

    #[test]
    fn test_4_vs_4_even_match() {
        // Task case 3: 4 vs 4 balanced match
        // Expected: Even state (balanced action economy and damage)

        let players: Vec<Combattant> = (0..4)
            .map(|i| {
                // Balanced player: 50 HP, AC 16, ~15 DPR
                create_combatant(
                    format!("player_{}", i),
                    format!("Player {}", i),
                    50,
                    16,
                    0,
                    15.0,
                    7.0,
                )
            })
            .collect();

        let monsters: Vec<Combattant> = (0..4)
            .map(|i| {
                // Balanced monster: 50 HP, AC 15, ~15 DPR
                create_combatant(
                    format!("monster_{}", i),
                    format!("Monster {}", i),
                    50,
                    15,
                    1,
                    15.0,
                    6.0,
                )
            })
            .collect();

        let player_refs: Vec<&Combattant> = players.iter().collect();
        let monster_refs: Vec<&Combattant> = monsters.iter().collect();

        let result = calculate_action_economy(&player_refs, &monster_refs);

        assert_eq!(result.player_count, 4);
        assert_eq!(result.monster_count, 4);

        // Should be roughly Even
        assert_eq!(result.state, ActionEconomyState::Even);
    }

    #[test]
    fn test_threshold_boundaries() {
        // Test the exact threshold boundaries (0.6 and 1.5)
        // time_ratio = (monster_hp / player_dpr) / (player_hp / monster_dpr)

        // Helper to create creatures with specific HP/DPR to hit exact ratios
        let create_test_scenario =
            |player_hp: u32, player_dpr: f64, monster_hp: u32, monster_dpr: f64| {
                let players = vec![create_combatant(
                    "player1".to_string(),
                    "Player 1".to_string(),
                    player_hp,
                    15,
                    0,
                    player_dpr,
                    5.0,
                )];

                let monsters = vec![create_combatant(
                    "monster1".to_string(),
                    "Monster 1".to_string(),
                    monster_hp,
                    15,
                    1,
                    monster_dpr,
                    5.0,
                )];

                let player_refs: Vec<&Combattant> = players.iter().collect();
                let monster_refs: Vec<&Combattant> = monsters.iter().collect();

                calculate_action_economy(&player_refs, &monster_refs)
            };

        // Test Player Advantage threshold (< 0.6)
        // time_ratio = (50/100) / (100/100) = 0.5 (players much faster)
        let result = create_test_scenario(100, 100.0, 50, 100.0);
        assert!(
            result.combined_ratio < 0.6,
            "combined_ratio should be < 0.6, got {}",
            result.combined_ratio
        );
        assert_eq!(result.state, ActionEconomyState::PlayerAdvantage);

        // Test Even lower boundary (~0.6)
        // time_ratio = (60/100) / (100/100) = 0.6
        let result = create_test_scenario(100, 100.0, 60, 100.0);
        assert!(
            result.combined_ratio >= 0.6,
            "combined_ratio should be >= 0.6, got {}",
            result.combined_ratio
        );
        assert!(
            result.state == ActionEconomyState::Even
                || result.state == ActionEconomyState::PlayerAdvantage
        );

        // Test Even range (0.6 - 1.5)
        // time_ratio = (100/100) / (100/100) = 1.0 (balanced)
        let result = create_test_scenario(100, 100.0, 100, 100.0);
        assert!(result.combined_ratio >= 0.6 && result.combined_ratio <= 1.5);
        assert_eq!(result.state, ActionEconomyState::Even);

        // Test Enemy Advantage threshold (> 1.5)
        // time_ratio = (150/100) / (100/100) = 1.5 (monsters faster)
        let result = create_test_scenario(100, 100.0, 150, 100.0);
        assert!(
            result.combined_ratio >= 1.5,
            "combined_ratio should be >= 1.5, got {}",
            result.combined_ratio
        );
        assert!(
            result.state == ActionEconomyState::Even
                || result.state == ActionEconomyState::EnemyAdvantage
        );

        // Test clearly Enemy Advantage (> 1.5)
        // time_ratio = (200/100) / (100/100) = 2.0 (monsters much faster)
        let result = create_test_scenario(100, 100.0, 200, 100.0);
        assert!(
            result.combined_ratio > 1.5,
            "combined_ratio should be > 1.5, got {}",
            result.combined_ratio
        );
        assert_eq!(result.state, ActionEconomyState::EnemyAdvantage);
    }
}
