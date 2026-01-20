pub mod attack;
pub mod effect;
pub mod heal;
pub mod template;

pub use attack::resolve as resolve_attack;
pub use effect::*;
pub use heal::resolve as resolve_heal;
pub use template::resolve as resolve_template;
