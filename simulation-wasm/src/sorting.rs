//! Player slot assignment based on contextual survivability scores
//!
//! This module handles the "Shield Wall" ordering system that determines the fixed
//! visual order of players in the UI, from Tank (Slot 1/Left) to Glass Cannon (Slot N/Right).

use crate::model::{Creature, TimelineStep};

/// Represents a player's assigned position in the UI layout
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSlot {
    /// Position index (0 = Tank/Left, N = Glass Cannon/Right)
    pub position: usize,
    /// Player ID from creature definition
    pub player_id: String,
    /// Survivability score used for ordering (higher = more tanky)
    pub survivability_score: i32,
}

/// Extract all attack bonuses from a creature's actions
///
/// Parses through attack actions to collect to_hit bonuses.
/// For dice formulas (e.g., "1d20+5"), extracts the static bonus.
pub fn extract_attack_bonuses_from_creature(creature: &Creature) -> Vec<i32> {
    let mut bonuses = Vec::new();

    for action in &creature.actions {
        // Only check attack actions
        if let crate::model::Action::Atk(attack) = action {
            // Extract to_hit bonus if present
            match &attack.to_hit {
                crate::model::DiceFormula::Value(bonus) => {
                    bonuses.push(*bonus as i32);
                }
                crate::model::DiceFormula::Expr(expr) => {
                    // Try to parse expressions like "1d20+5" to extract the bonus
                    // Look for + or - followed by a number at the end
                    if let Some(pos) = expr.rfind('+').or_else(|| expr.rfind('-')) {
                        if let Ok(bonus_str) = expr[pos..].parse::<f64>() {
                            bonuses.push(bonus_str as i32);
                        }
                    }
                }
            }
        }
    }

    bonuses
}

/// Calculate the average attack bonus across all monsters in the timeline
///
/// Iterates through all combat encounters and collects attack bonuses from
/// all monsters, then returns the mean average.
///
/// # Arguments
/// * `timeline` - The timeline containing all encounters for the day
///
/// # Returns
/// Average attack bonus as f64, or +5.0 as default if no attacks found
pub fn calculate_average_attack_bonus(timeline: &[TimelineStep]) -> f64 {
    let mut all_bonuses = Vec::new();

    for step in timeline {
        if let crate::model::TimelineStep::Combat(combat) = step {
            // Collect bonuses from all monsters in this encounter
            for monster in &combat.monsters {
                let bonuses = extract_attack_bonuses_from_creature(monster);
                all_bonuses.extend(bonuses);
            }
        }
    }

    // If no attack bonuses found, use default +5
    if all_bonuses.is_empty() {
        return 5.0;
    }

    // Calculate average
    let sum: i32 = all_bonuses.iter().sum();
    sum as f64 / all_bonuses.len() as f64
}

/// Assign players to slots based on survivability scores
///
/// Sorts players from highest survivability (Tank) to lowest (Glass Cannon)
/// and assigns them to numbered positions for consistent UI ordering.
///
/// # Arguments
/// * `players` - Slice of player creatures to assign
/// * `avg_attack_bonus` - Average monster attack bonus for the day
///
/// # Returns
/// Vector of PlayerSlot sorted by position (0 = Tank)
pub fn assign_party_slots(players: &[Creature], avg_attack_bonus: i32) -> Vec<PlayerSlot> {
    let mut scored_players: Vec<(String, f64)> = players
        .iter()
        .map(|player| {
            let score = player.max_survivability_score_vs_attack(avg_attack_bonus);
            (player.id.clone(), score)
        })
        .collect();

    // Sort by survivability score DESCENDING, then by player_id for tie-breaking
    scored_players.sort_by(|a, b| {
        b.1
            .partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    // Assign positions
    scored_players
        .into_iter()
        .enumerate()
        .map(|(position, (player_id, score))| PlayerSlot {
            position,
            player_id,
            survivability_score: score as i32,
        })
        .collect()
}

