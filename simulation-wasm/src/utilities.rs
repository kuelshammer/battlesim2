use crate::dice;
use crate::model::Creature;
use crate::rng; // Import rng module
use rand::Rng; // Import Rng trait for gen_range
use std::hash::{Hash, Hasher};

pub fn hash_f64<H: Hasher>(val: f64, state: &mut H) {
    val.to_bits().hash(state);
}

pub fn hash_opt_f64<H: Hasher>(val: Option<f64>, state: &mut H) {
    val.map(|v| v.to_bits()).hash(state);
}

pub fn roll_initiative(c: &Creature) -> f64 {
    let mut rng = rng::get_rng();

    let roll = if c.initiative_advantage {
        let r1 = rng.gen_range(1..=20);
        let r2 = rng.gen_range(1..=20);
        r1.max(r2) as f64
    } else {
        rng.gen_range(1..=20) as f64
    };

    let bonus = dice::evaluate(&c.initiative_bonus, 1); // Use dice::evaluate with multiplier 1

    roll + bonus
}
