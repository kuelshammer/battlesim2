use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DiceFormula {
    Value(f64),
    Expr(String),
}

impl Hash for DiceFormula {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DiceFormula::Value(v) => {
                0.hash(state);
                crate::utils::hash_f64(*v, state);
            }
            DiceFormula::Expr(s) => {
                1.hash(state);
                s.hash(state);
            }
        }
    }
}
