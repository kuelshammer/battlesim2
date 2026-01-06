use serde::{Deserialize, Serialize, Deserializer, Serializer};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;
use crate::resources::{ResourceLedger, ResourceType, ResetType};
use super::formula::DiceFormula;
use super::buff::Buff;
use super::action::{Action, ActionTrigger, Frequency};
use super::types::{AcKnowledge, SerializableResourceLedger};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Creature {
    pub id: String,
    pub arrival: Option<i32>,
    #[serde(default)]
    pub mode: String, // "player", "monster", "custom"
    pub name: String,
    pub count: f64, // TS uses number, but usually integer.
    pub hp: u32,
    #[serde(alias = "AC")]
    pub ac: u32,
    #[serde(rename = "speed_fly")]
    pub speed_fly: Option<f64>,

    // Save bonuses - average is required, individual are optional overrides
    #[serde(rename = "saveBonus")]
    pub save_bonus: f64,
    #[serde(
        default,
        rename = "strSaveBonus",
        skip_serializing_if = "Option::is_none"
    )]
    pub str_save_bonus: Option<f64>,
    #[serde(
        default,
        rename = "dexSaveBonus",
        skip_serializing_if = "Option::is_none"
    )]
    pub dex_save_bonus: Option<f64>,
    #[serde(
        default,
        rename = "conSaveBonus",
        skip_serializing_if = "Option::is_none"
    )]
    pub con_save_bonus: Option<f64>,
    #[serde(
        default,
        rename = "intSaveBonus",
        skip_serializing_if = "Option::is_none"
    )]
    pub int_save_bonus: Option<f64>,
    #[serde(
        default,
        rename = "wisSaveBonus",
        skip_serializing_if = "Option::is_none"
    )]
    pub wis_save_bonus: Option<f64>,
    #[serde(
        default,
        rename = "chaSaveBonus",
        skip_serializing_if = "Option::is_none"
    )]
    pub cha_save_bonus: Option<f64>,

    // Advantage on saves
    #[serde(
        default,
        rename = "conSaveAdvantage",
        skip_serializing_if = "Option::is_none"
    )]
    pub con_save_advantage: Option<bool>,
    #[serde(
        default,
        rename = "saveAdvantage",
        skip_serializing_if = "Option::is_none"
    )]
    pub save_advantage: Option<bool>, // Advantage on ALL saves (e.g. Paladin Aura)

    #[serde(default = "default_initiative_bonus")]
    #[serde(rename = "initiativeBonus")]
    pub initiative_bonus: DiceFormula,

    #[serde(default)]
    #[serde(rename = "initiativeAdvantage")]
    pub initiative_advantage: bool,
    pub actions: Vec<Action>, // This might need to be flexible if templates are involved
    #[serde(default)]
    pub triggers: Vec<ActionTrigger>,
    #[serde(rename = "spellSlots")]
    pub spell_slots: Option<HashMap<String, i32>>,
    #[serde(rename = "classResources")]
    pub class_resources: Option<HashMap<String, i32>>,
    #[serde(rename = "hitDice")]
    pub hit_dice: Option<String>, // Changed from DiceFormula
    #[serde(rename = "conModifier")]
    pub con_modifier: Option<f64>, // New field for constitution modifier to apply to hit dice rolls
    #[serde(default)]
    #[serde(rename = "magicItems")]
    pub magic_items: Vec<String>, // Names of magic items equipped
    #[serde(rename = "maxArcaneWardHp")]
    pub max_arcane_ward_hp: Option<u32>, // Maximum Arcane Ward HP (for Abjuration Wizard)
    #[serde(default)]
    #[serde(rename = "initialBuffs")]
    pub initial_buffs: Vec<Buff>, // Buffs from magic items applied at encounter start
}

fn default_initiative_bonus() -> DiceFormula {
    DiceFormula::Value(0.0)
}

impl Hash for Creature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.arrival.hash(state);
        self.mode.hash(state);
        self.name.hash(state);
        crate::utilities::hash_f64(self.count, state);
        self.hp.hash(state);
        self.ac.hash(state);
        crate::utilities::hash_opt_f64(self.speed_fly, state);
        crate::utilities::hash_f64(self.save_bonus, state);
        crate::utilities::hash_opt_f64(self.str_save_bonus, state);
        crate::utilities::hash_opt_f64(self.dex_save_bonus, state);
        crate::utilities::hash_opt_f64(self.con_save_bonus, state);
        crate::utilities::hash_opt_f64(self.int_save_bonus, state);
        crate::utilities::hash_opt_f64(self.wis_save_bonus, state);
        crate::utilities::hash_opt_f64(self.cha_save_bonus, state);
        self.con_save_advantage.hash(state);
        self.save_advantage.hash(state);
        self.initiative_bonus.hash(state);
        self.initiative_advantage.hash(state);
        self.actions.hash(state);
        self.triggers.hash(state);
        
        // HashMap hashing needs sorting for determinism
        if let Some(slots) = &self.spell_slots {
            let mut sorted_slots: Vec<_> = slots.iter().collect();
            sorted_slots.sort_by_key(|a| a.0);
            sorted_slots.hash(state);
        } else {
            None::<()>.hash(state);
        }

        if let Some(res) = &self.class_resources {
            let mut sorted_res: Vec<_> = res.iter().collect();
            sorted_res.sort_by_key(|a| a.0);
            sorted_res.hash(state);
        } else {
            None::<()>.hash(state);
        }

        self.hit_dice.hash(state);
        crate::utilities::hash_opt_f64(self.con_modifier, state);
        self.magic_items.hash(state);
        self.max_arcane_ward_hp.hash(state);
        self.initial_buffs.hash(state);
    }
}

impl Creature {
    pub fn initialize_ledger(&self) -> ResourceLedger {
        let mut ledger = ResourceLedger::new();

        // Add standard resources
        ledger.register_resource(
            ResourceType::Action,
            None,
            1.0,
            Some(ResetType::Turn),
        );
        ledger.register_resource(
            ResourceType::BonusAction,
            None,
            1.0,
            Some(ResetType::Turn),
        );
        ledger.register_resource(
            ResourceType::Reaction,
            None,
            1.0,
            Some(ResetType::Round),
        );
        ledger.register_resource(
            ResourceType::Movement,
            None,
            30.0,
            Some(ResetType::Turn),
        ); // Default 30ft, should use self.speed if available

        // Add spell slots
        if let Some(slots) = &self.spell_slots {
            for (level_str, count) in slots {
                // Try to extract digit from string (e.g. "1st" -> 1)
                let cleaned_level = level_str.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
                if let Ok(level) = cleaned_level.parse::<u8>() {
                    let resource_type = ResourceType::SpellSlot; 
                    ledger.register_resource(
                        resource_type,
                        Some(&level.to_string()),
                        *count as f64,
                        Some(ResetType::LongRest),
                    );
                }
            }
        }

        // Add class resources
        if let Some(resources) = &self.class_resources {
            for (name, count) in resources {
                let resource_type = ResourceType::ClassResource;
                
                // Identify common short rest resources
                let reset_type = match name.as_str() {
                    "Ki" | "Action Surge" | "Superiority Dice" | "Wild Shape" | 
                    "Channel Divinity" | "Warlock Spell Slot" | "Bardic Inspiration" |
                    "Second Wind" | "Frenzy" | "Rage" => {
                        // Note: Rage is usually LR, but some implementations might use it differently.
                        // For 5e 2014/2024, Rage is Long Rest. 
                        // Let's stick to the most common SR ones.
                        if name == "Rage" { ResetType::LongRest }
                        else { ResetType::ShortRest }
                    },
                    _ => ResetType::LongRest,
                };

                ledger.register_resource(
                    resource_type,
                    Some(name),
                    *count as f64,
                    Some(reset_type),
                );
            }
        }

        // Add Hit Dice resource
        if let Some(hit_dice_expr) = &self.hit_dice {
            let s = hit_dice_expr.replace(" ", "");
            let mut current_term = String::new();

            for c in s.chars() {
                if c == '+' || c == '-' {
                    if !current_term.is_empty() {
                        register_hit_dice_term(&mut ledger, &current_term);
                        current_term.clear();
                    }
                } else {
                    current_term.push(c);
                }
            }
            if !current_term.is_empty() {
                register_hit_dice_term(&mut ledger, &current_term);
            }
        }

        // Add per-action resources (1/fight, 1/day, Limited, Recharge)
        for action in &self.actions {
            let base = action.base();
            let reset_rule = match &base.freq {
                Frequency::Static(s) if s == "at will" => None,
                Frequency::Static(s) if s == "1/fight" => Some(ResetType::ShortRest),
                Frequency::Static(s) if s == "1/day" => Some(ResetType::LongRest),
                Frequency::Recharge { .. } => Some(ResetType::Encounter),
                Frequency::Limited { reset, .. } => {
                    if reset == "lr" { Some(ResetType::LongRest) }
                    else { Some(ResetType::ShortRest) }
                }
                _ => None,
            };

            if let Some(rule) = reset_rule {
                let max_uses = match &base.freq {
                    Frequency::Limited { uses, .. } => *uses as f64,
                    _ => 1.0,
                };
                ledger.register_resource(
                    ResourceType::ActionUsage,
                    Some(&base.id),
                    max_uses,
                    Some(rule),
                );
            }
        }

        ledger
    }

    /// Calculate the survivability score for UI slot ordering (Tank → Glass Cannon)
    pub fn max_survivability_score(&self) -> f64 {
        self.max_survivability_score_vs_attack(5)
    }

    /// Calculate survivability score against a specific monster attack bonus
    pub fn max_survivability_score_vs_attack(&self, monster_attack_bonus: i32) -> f64 {
        let hit_chance = calculate_hit_chance(self.ac, monster_attack_bonus);
        let rage_multiplier = if self.is_barbarian() { 2.0 } else { 1.0 };

        // EHP = HP / hit_chance × rage_multiplier
        (self.hp as f64 / hit_chance * rage_multiplier).round()
    }

    /// Check if this creature is a Barbarian (has Rage class resource)
    fn is_barbarian(&self) -> bool {
        // Check class_resources for Rage
        if let Some(resources) = &self.class_resources {
            return resources.contains_key("Rage");
        }
        false
    }
}

// Helper function to register a single hit dice term (e.g., "3d8")
fn register_hit_dice_term(ledger: &mut ResourceLedger, term: &str) {
    let cleaned_term = if let Some(bracket_pos) = term.find('[') {
        &term[..bracket_pos]
    } else {
        term
    };

    if cleaned_term.contains('d') {
        let parts: Vec<&str> = cleaned_term.split('d').collect();
        if parts.len() == 2 {
            let count = parts[0].parse::<i32>().unwrap_or(1); // "d8" -> count 1
            let count = if count == 0 && parts[0].is_empty() {
                1
            } else {
                count
            };
            let sides = parts[1].parse::<i32>().unwrap_or(6); // Default to d6 if parse fails

            let resource_type = match sides {
                6 => ResourceType::HitDiceD6,
                8 => ResourceType::HitDiceD8,
                10 => ResourceType::HitDiceD10,
                12 => ResourceType::HitDiceD12,
                _ => {
                    eprintln!(
                        "Warning: Unsupported hit dice size 'd{}' for term '{}'",
                        sides, term
                    );
                    return;
                }
            };

            ledger.register_resource(
                resource_type,
                None,
                count as f64,
                Some(ResetType::LongRest),
            );
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreatureState {
    #[serde(rename = "currentHp")]
    pub current_hp: u32,
    #[serde(rename = "tempHp")]
    pub temp_hp: Option<u32>,
    pub buffs: HashMap<String, Buff>,

    // Use SerializableResourceLedger for frontend compatibility
    #[serde(default = "default_serializable_resource_ledger")]
    pub resources: SerializableResourceLedger,

    #[serde(rename = "upcomingBuffs")]
    pub upcoming_buffs: HashMap<String, Buff>,
    #[serde(rename = "usedActions")]
    pub used_actions: HashSet<String>,
    #[serde(rename = "concentratingOn")]
    pub concentrating_on: Option<String>,
    pub actions_used_this_encounter: HashSet<String>,
    #[serde(rename = "bonusActionUsed")]
    pub bonus_action_used: bool,
    #[serde(default)]
    pub known_ac: HashMap<String, AcKnowledge>,
    #[serde(
        rename = "arcaneWardHp",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub arcane_ward_hp: Option<u32>,
    #[serde(rename = "cumulativeSpent", default)]
    pub cumulative_spent: f64,
}

fn default_serializable_resource_ledger() -> SerializableResourceLedger {
    SerializableResourceLedger::from(ResourceLedger::new())
}

impl Default for CreatureState {
    fn default() -> Self {
        CreatureState {
            current_hp: 0,
            temp_hp: None,
            buffs: HashMap::new(),
            resources: default_serializable_resource_ledger(),
            upcoming_buffs: HashMap::new(),
            used_actions: HashSet::new(),
            concentrating_on: None,
            actions_used_this_encounter: HashSet::new(),
            bonus_action_used: false,
            known_ac: HashMap::new(),
            arcane_ward_hp: None,
            cumulative_spent: 0.0,
        }
    }
}

impl CreatureState {
    pub fn has_rage_active(&self) -> bool {
        if self.buffs.contains_key("Rage") {
            return true;
        }
        if let Some(&current) = self.resources.current.get("Rage") {
            if current > 0.0 {
                return true;
            }
        }
        false
    }
}

fn calculate_hit_chance(ac: u32, attack_bonus: i32) -> f64 {
    let roll_needed = ac as i32 - attack_bonus;
    if roll_needed <= 1 {
        0.95
    } else if roll_needed >= 20 {
        0.05
    } else {
        (21 - roll_needed) as f64 / 20.0
    }
}

#[derive(Debug, PartialEq)]
pub struct Combattant {
    pub id: String,
    pub team: u32, // 0 for Team 1 (Players), 1 for Team 2 (Monsters) - defaults to 0
    pub creature: Arc<Creature>,
    pub initiative: f64, // defaults to 0.0
    pub initial_state: CreatureState,
    pub final_state: CreatureState,
    pub actions: Vec<CombattantAction>,
}

impl Clone for Combattant {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            team: self.team,
            creature: Arc::clone(&self.creature),
            initiative: self.initiative,
            initial_state: self.initial_state.clone(),
            final_state: self.final_state.clone(),
            actions: self.actions.clone(),
        }
    }
}

impl Serialize for Combattant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Combattant", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("team", &self.team)?;
        state.serialize_field("creature", self.creature.as_ref())?;
        state.serialize_field("initiative", &self.initiative)?;
        state.serialize_field("initialState", &self.initial_state)?;
        state.serialize_field("finalState", &self.final_state)?;
        state.serialize_field("actions", &self.actions)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Combattant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CombattantHelper {
            id: String,
            #[serde(default)]
            team: u32,
            creature: Creature,
            #[serde(default)]
            initiative: f64,
            #[serde(rename = "initialState")]
            initial_state: CreatureState,
            #[serde(rename = "finalState")]
            final_state: CreatureState,
            actions: Vec<CombattantAction>,
        }

        let helper = CombattantHelper::deserialize(deserializer)?;
        Ok(Combattant {
            id: helper.id,
            team: helper.team,
            creature: Arc::new(helper.creature),
            initiative: helper.initiative,
            initial_state: helper.initial_state,
            final_state: helper.final_state,
            actions: helper.actions,
        })
    }
}

impl Combattant {
    pub fn current_survivability_score(&self) -> f64 {
        self.current_survivability_score_vs_attack(5)
    }

    pub fn current_survivability_score_vs_attack(&self, monster_attack_bonus: i32) -> f64 {
        let hit_chance = calculate_hit_chance(self.creature.ac, monster_attack_bonus);
        let rage_multiplier = if self.final_state.has_rage_active() { 2.0 } else { 1.0 };
        let total_hp = self.final_state.current_hp as f64 + self.final_state.temp_hp.unwrap_or(0) as f64;
        (total_hp / hit_chance * rage_multiplier).round()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombattantAction {
    pub action: Action,
    pub targets: HashMap<String, i32>,
}