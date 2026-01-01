use crate::dice;
use crate::model::Creature;
use crate::rng; // Import rng module
use std::hash::{Hash, Hasher};

pub fn hash_f64<H: Hasher>(val: f64, state: &mut H) {
    val.to_bits().hash(state);
}

pub fn hash_opt_f64<H: Hasher>(val: Option<f64>, state: &mut H) {
    val.map(|v| v.to_bits()).hash(state);
}

pub fn roll_initiative(c: &Creature) -> f64 {
    let roll = if c.initiative_advantage {
        let r1 = rng::roll_d20();
        let r2 = rng::roll_d20();
        r1.max(r2) as f64
    } else {
        rng::roll_d20() as f64
    };

    let bonus = dice::evaluate(&c.initiative_bonus, 1); // Use dice::evaluate with multiplier 1

    roll + bonus
}
