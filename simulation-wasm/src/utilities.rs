use rand::Rng;
use crate::model::{Creature, DiceFormula};
use crate::dice; // Import the dice module

pub fn roll_initiative(c: &Creature) -> f64 {
    let roll = if c.initiative_advantage {
        let r1 = rand::thread_rng().gen_range(1..=20);
        let r2 = rand::thread_rng().gen_range(1..=20);
        r1.max(r2) as f64
    } else {
        rand::thread_rng().gen_range(1..=20) as f64
    };

    let bonus = dice::evaluate(&c.initiative_bonus, 1); // Use dice::evaluate with multiplier 1

    roll + bonus
}