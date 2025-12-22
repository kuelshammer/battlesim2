use crate::model::{Creature, MonsterRole, Action};
use crate::combat_stats::CombatantStats;

pub fn detect_role(creature: &Creature, encounter_total_hp: f64, party_dpr: f64) -> MonsterRole {
    // 🐜 Minion: HP < 20% of Party DPR, Count >= 4
    if (creature.hp as f64) < (party_dpr * 0.2) && creature.count >= 4.0 {
        return MonsterRole::Minion;
    }

    // 👑 Boss: Legendary Actions OR > 50% of Total HP
    let has_legendary = creature.actions.iter().any(|a| a.base().name.to_lowercase().contains("legendary"));
    if has_legendary || (creature.hp as f64 * creature.count) > (encounter_total_hp * 0.5) {
        return MonsterRole::Boss;
    }

    let stats = CombatantStats::calculate(creature);

    // 🛡️ Brute: Melee Only, High HP, Low AC
    let is_melee_only = !creature.actions.iter().any(|a| {
        if let Action::Atk(atk) = a {
            atk.name.to_lowercase().contains("ranged") || atk.name.to_lowercase().contains("bow")
        } else {
            false
        }
    });
    if is_melee_only && creature.ac < 14 {
        return MonsterRole::Brute;
    }

    // 🏹 Striker: Ranged, High Mobility, High To-Hit
    let is_ranged = creature.actions.iter().any(|a| {
        if let Action::Atk(atk) = a {
            atk.name.to_lowercase().contains("ranged") || atk.name.to_lowercase().contains("bow")
        } else {
            false
        }
    });
    if is_ranged && stats.hit_probability > 0.6 {
        return MonsterRole::Striker;
    }

    // 🧙‍♂️ Controller: Spellcasting, Conditions
    let has_conditions = creature.actions.iter().any(|a| {
        match a {
            Action::Debuff(_) => true,
            Action::Atk(atk) => atk.rider_effect.is_some(),
            _ => false
        }
    });
    if has_conditions {
        return MonsterRole::Controller;
    }

    MonsterRole::Unknown
}

/// Applies a numeric adjustment to a creature's HP
pub fn adjust_hp(creature: &mut Creature, percentage: f64) {
    let current = creature.hp as f64;
    let next = (current * (1.0 + percentage)).round();
    creature.hp = next.max(1.0) as u32;
}

/// Applies a numeric adjustment to a creature's damage output
pub fn adjust_damage(creature: &mut Creature, percentage: f64) {
    for action in &mut creature.actions {
        if let Action::Atk(atk) = action {
            let current = match &atk.dpr {
                crate::model::DiceFormula::Value(v) => *v,
                crate::model::DiceFormula::Expr(e) => crate::dice::parse_average(e),
            };
            let next = current * (1.0 + percentage);
            atk.dpr = crate::model::DiceFormula::Value(next);
        }
    }
}

/// Applies a numeric adjustment to a creature's save DCs
pub fn adjust_dc(creature: &mut Creature, delta: f64) {
    for action in &mut creature.actions {
        match action {
            Action::Debuff(debuff) => {
                debuff.save_dc = (debuff.save_dc + delta).max(1.0);
            },
            Action::Atk(atk) => {
                if let Some(rider) = &mut atk.rider_effect {
                    rider.dc = (rider.dc + delta).max(1.0);
                }
            },
            _ => {}
        }
    }
}
