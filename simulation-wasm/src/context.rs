use crate::combat_stats::{CombatStatsCache, CombatantStats};
use crate::enums::CreatureCondition;
use crate::events::{Event, EventBus};
use crate::model::{Action, Combattant};
use crate::resources::{ActionCost, ResetType, ResourceLedger};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Central context that maintains all game state during a combat encounter
/// Acts as the "single source of truth" for turn-based resource and event management
#[derive(Debug, Clone)]
pub struct TurnContext {
    // Resource Management - Moved to CombattantState
    // pub resource_ledger: ResourceLedger,

    // Event Tracking
    pub event_bus: EventBus,
    pub round_number: u32,
    pub current_turn_owner: Option<String>,
    pub log_enabled: bool,

    // Combat State
    pub combatants: HashMap<String, CombattantState>,
    pub active_effects: HashMap<String, ActiveEffect>,

    // Performance Optimization
    pub combat_stats_cache: CombatStatsCache, // Cached combat statistics for targeting

    // Environmental Context
    pub battlefield_conditions: Vec<String>, // Simplified to string for now
    pub weather: Option<String>,             // Simplified to string for now
    pub terrain: String,                     // Simplified to string for now

    // Roll Manipulation System
    pub roll_modifications: RollModificationQueue, // Per-combatant pending roll mods

    // Interrupt System
    pub action_interrupted: bool, // Flag to signal that current action should stop
}

/// A queue of pending modifications to be applied to rolls
#[derive(Debug, Clone, Default)]
pub struct RollModificationQueue {
    pub modifications: HashMap<String, Vec<RollModification>>,
}

impl RollModificationQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, unit_id: &str, modification: RollModification) {
        self.modifications
            .entry(unit_id.to_string())
            .or_default()
            .push(modification);
    }

    pub fn take_all(&mut self, unit_id: &str) -> Vec<RollModification> {
        self.modifications.remove(unit_id).unwrap_or_default()
    }
}

/// A pending modification to be applied to a roll
#[derive(Debug, Clone, PartialEq)]
pub enum RollModification {
    /// Add a bonus to the roll (e.g., Bardic Inspiration)
    AddBonus { amount: String }, // DiceFormula string
    /// Force a reroll (e.g., Lucky, Portent)
    Reroll {
        roll_type: String, // "attack", "save", "abilityCheck"
        must_use_second: bool,
    },
    /// Apply advantage/disadvantage
    SetAdvantage {
        roll_type: String,
        advantage: bool, // true = advantage, false = disadvantage
    },
}

/// State of a combatant within the current encounter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombattantState {
    pub id: String,
    pub side: u32, // 0 for Team 1, 1 for Team 2
    pub base_combatant: Combattant,
    pub current_hp: u32,
    pub temp_hp: u32,
    pub conditions: Vec<CreatureCondition>,
    pub concentration: Option<String>, // ID of spell/concentration source
    pub position: crate::model::Position,
    pub resources: ResourceLedger, // Per-combatant resource tracking
    #[serde(default)]
    pub arcane_ward_hp: Option<u32>,
    pub known_ac: HashMap<String, crate::model::AcKnowledge>,
    #[serde(default)]
    pub cumulative_spent: f64,
    #[serde(skip)]
    pub cached_stats: Option<CombatantStats>, // Cached combat statistics
}

/// An active effect applied to a combatant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEffect {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub effect_type: EffectType,
    pub remaining_duration: i32,
    pub conditions: Vec<String>, // Simplified to string for now
}

/// Types of effects that can be applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectType {
    Buff(Box<crate::model::Buff>), // Store full Buff object
    DamageOverTime {
        damage_per_round: f64,
        damage_type: String,
    },
    HealingOverTime {
        healing_per_round: f64,
    },
    Condition(CreatureCondition),
    Custom(String),
}

impl TurnContext {
    /// Create a new turn context for a combat encounter
    pub fn new(
        combatants: Vec<Combattant>,
        battlefield_conditions: Vec<String>,
        weather: Option<String>,
        terrain: String,
        log_enabled: bool,
    ) -> Self {
        // Create combatant states with individual resource ledgers
        let combatant_states: HashMap<String, CombattantState> = combatants
            .into_iter()
            .map(|c| {
                // Initialize resource ledger from creature definition (Max values)
                let mut resources = c.creature.initialize_ledger();

                // Overwrite current values from initial state (carry-over usage)
                for (key, val) in &c.initial_state.resources.current {
                    resources.current.insert(key.clone(), *val);
                }

                let mut state = CombattantState {
                    id: c.id.clone(),
                    side: c.team,
                    current_hp: c.initial_state.current_hp,
                    temp_hp: c.initial_state.temp_hp.unwrap_or(0),
                    conditions: Vec::new(),
                    concentration: c.initial_state.concentrating_on.clone(),
                    position: crate::model::Position::default(),
                    base_combatant: c.clone(),
                    resources,
                    // Reset Arcane Ward HP to max at the start of each encounter
                    arcane_ward_hp: c.creature.max_arcane_ward_hp,
                    known_ac: c.initial_state.known_ac.clone(),
                    cumulative_spent: c.initial_state.cumulative_spent,
                    cached_stats: None,
                };

                // Sync initial state to base_combatant for immediate use in targeting logic
                state.base_combatant.final_state.current_hp = state.current_hp;
                state.base_combatant.final_state.temp_hp = Some(state.temp_hp);

                (state.id.clone(), state)
            })
            .collect();

        let mut active_effects = HashMap::new();
        for (unit_id, state) in &combatant_states {
            for (buff_id, buff) in &state.base_combatant.initial_state.buffs {
                active_effects.insert(
                    format!("{}_{}", buff_id, unit_id),
                    ActiveEffect {
                        id: format!("{}_{}", buff_id, unit_id),
                        source_id: unit_id.clone(), // Assume self-sourced for initial buffs
                        target_id: unit_id.clone(),
                        effect_type: EffectType::Buff(Box::new(buff.clone())),
                        remaining_duration: match buff.duration {
                            crate::enums::BuffDuration::EntireEncounter => 100,
                            crate::enums::BuffDuration::OneRound => 1,
                            _ => 10,
                        },
                        conditions: Vec::new(),
                    },
                );
            }
        }

        Self {
            event_bus: EventBus::new(if log_enabled { 1000 } else { 0 }, log_enabled), // Keep last 1000 events ONLY if logging is enabled
            round_number: 0,
            current_turn_owner: None,
            log_enabled,
            combatants: combatant_states,
            active_effects,
            combat_stats_cache: CombatStatsCache::new(),
            battlefield_conditions,
            weather,
            terrain,
            roll_modifications: RollModificationQueue::new(),
            action_interrupted: false,
        }
    }

    /// Start a new turn for the specified unit
    pub fn start_new_turn(&mut self, unit_id: String) {
        // Reset interrupt flag at start of turn
        self.action_interrupted = false;

        // Emit turn start event
        self.event_bus.emit_event(Event::TurnStarted {
            unit_id: unit_id.clone(),
            round_number: self.round_number,
        });

        self.current_turn_owner = Some(unit_id.clone());

        // Reset turn-based resources and used actions for the current unit
        if let Some(combatant) = self.combatants.get_mut(&unit_id) {
            combatant.resources.reset_by_type(&ResetType::Turn);
            combatant.base_combatant.final_state.used_actions.clear();
        }
    }

    /// End the current turn
    pub fn end_current_turn(&mut self) {
        if let Some(owner) = self.current_turn_owner.clone() {
            // Emit turn end event
            self.event_bus.emit_event(Event::TurnEnded {
                unit_id: owner.clone(),
                round_number: self.round_number,
            });
        }

        self.current_turn_owner = None;
    }

    /// Advance to the next round
    pub fn advance_round(&mut self) {
        self.round_number += 1;

        // Emit round start event
        self.event_bus.emit_event(Event::RoundStarted {
            round_number: self.round_number,
        });

        // Reset round-based resources
        for combatant in self.combatants.values_mut() {
            combatant.resources.reset_by_type(&ResetType::Round);
        }

        // Update all effects
        self.update_effects();
    }

    /// Check if a combatant can afford the specified costs
    pub fn can_afford(&self, costs: &[ActionCost], unit_id: &str) -> bool {
        let combatant = match self.combatants.get(unit_id) {
            Some(c) => c,
            None => return false, // Combatant not found
        };

        for cost in costs {
            match cost {
                ActionCost::Discrete {
                    resource_type,
                    resource_val,
                    amount,
                } => {
                    if !combatant.resources.has(
                        resource_type.clone(),
                        resource_val.as_deref(),
                        *amount,
                    ) {
                        return false;
                    }
                }
                ActionCost::Variable {
                    resource_type,
                    resource_val,
                    min: _min,
                    max,
                } => {
                    if !combatant.resources.has(
                        resource_type.clone(),
                        resource_val.as_deref(),
                        *max,
                    ) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Pay the specified costs for a combatant
    pub fn pay_costs(&mut self, costs: &[ActionCost], unit_id: &str) -> Result<(), String> {
        let combatant = match self.combatants.get_mut(unit_id) {
            Some(c) => c,
            None => return Err(format!("Combatant {} not found", unit_id)),
        };

        for cost in costs {
            match cost {
                ActionCost::Discrete {
                    resource_type,
                    resource_val,
                    amount,
                } => {
                    if let Err(e) = combatant.resources.consume(
                        resource_type.clone(),
                        resource_val.as_deref(),
                        *amount,
                    ) {
                        return Err(format!("Failed to pay cost: {}", e));
                    }

                    // Emit resource consumed event
                    self.event_bus.emit_event(Event::ResourceConsumed {
                        unit_id: unit_id.to_string(),
                        resource_type: resource_type.to_key(resource_val.as_deref()),
                        amount: *amount,
                    });

                    // If it's a movement resource, also emit UnitMoved
                    if matches!(resource_type, crate::resources::ResourceType::Movement) {
                        let current_pos = combatant.position;

                        self.event_bus.emit_event(Event::UnitMoved {
                            creature_id: unit_id.to_string(),
                            from_position: Some((current_pos.x as i32, current_pos.y as i32)),
                            to_position: Some((current_pos.x as i32, current_pos.y as i32)), // Position doesn't change yet in pay_costs
                            distance: *amount as u32,
                        });
                    }

                    // Track cumulative expenditure
                    let key = resource_type.to_key(resource_val.as_deref());
                    let weight = crate::intensity_calculation::get_resource_weight(
                        &key,
                        &combatant.resources.reset_rules,
                        combatant
                            .base_combatant
                            .creature
                            .con_modifier
                            .unwrap_or(0.0),
                    );
                    combatant.cumulative_spent += *amount * weight;
                }
                ActionCost::Variable {
                    resource_type,
                    resource_val,
                    min: _min,
                    max,
                } => {
                    if let Err(e) = combatant.resources.consume(
                        resource_type.clone(),
                        resource_val.as_deref(),
                        *max,
                    ) {
                        return Err(format!("Failed to pay cost: {}", e));
                    }

                    // Emit resource consumed event
                    self.event_bus.emit_event(Event::ResourceConsumed {
                        unit_id: unit_id.to_string(),
                        resource_type: resource_type.to_key(resource_val.as_deref()),
                        amount: *max,
                    });

                    // If it's a movement resource, also emit UnitMoved
                    if matches!(resource_type, crate::resources::ResourceType::Movement) {
                        let current_pos = combatant.position;

                        self.event_bus.emit_event(Event::UnitMoved {
                            creature_id: unit_id.to_string(),
                            from_position: Some((current_pos.x as i32, current_pos.y as i32)),
                            to_position: Some((current_pos.x as i32, current_pos.y as i32)), // Position doesn't change yet in pay_costs
                            distance: *max as u32,
                        });
                    }

                    // Track cumulative expenditure
                    let key = resource_type.to_key(resource_val.as_deref());
                    let weight = crate::intensity_calculation::get_resource_weight(
                        &key,
                        &combatant.resources.reset_rules,
                        combatant
                            .base_combatant
                            .creature
                            .con_modifier
                            .unwrap_or(0.0),
                    );
                    combatant.cumulative_spent += *max * weight;
                }
            }
        }
        Ok(())
    }

    /// Record an event in the system
    pub fn record_event(&mut self, event: Event) {
        self.event_bus.emit_event(event);
    }

    /// Process pending events and return triggered reactions
    pub fn process_events(&mut self) -> Vec<(String, Action)> {
        let reactions = self.event_bus.process_pending_events();

        // Convert reaction actions - this is a placeholder implementation
        // In a full implementation, you'd resolve templates through an action resolver
        // For now, just return the reactions as-is
        // TODO: Implement proper template to action conversion
        let converted_reactions: Vec<(String, Action)> = reactions;

        converted_reactions
    }

    /// Apply an active effect to a target
    /// Apply an active effect to a target
    pub fn apply_effect(&mut self, effect: ActiveEffect) -> Event {
        // Determine event based on effect type
        let event = match &effect.effect_type {
            EffectType::Buff(buff) => {
                // Handle concentration
                if buff.concentration {
                    if let Some(source) = self.combatants.get_mut(&effect.source_id) {
                        // If already concentrating on something else, we should break it
                        // But for now, just overwrite the concentration ID
                        source.concentration =
                            Some(buff.display_name.clone().unwrap_or(effect.id.clone()));
                    }
                }

                Event::BuffApplied {
                    target_id: effect.target_id.clone(),
                    buff_id: buff.display_name.clone().unwrap_or(effect.id.clone()),
                    source_id: effect.source_id.clone(),
                }
            }
            EffectType::Condition(condition) => Event::ConditionAdded {
                target_id: effect.target_id.clone(),
                condition: *condition,
                source_id: effect.source_id.clone(),
            },
            _ => Event::Custom {
                event_type: "EffectApplied".to_string(),
                data: {
                    let mut data = HashMap::new();
                    data.insert("effect_id".to_string(), effect.id.clone());
                    data.insert("target_id".to_string(), effect.target_id.clone());
                    data
                },
                source_id: effect.source_id.clone(),
            },
        };

        self.event_bus.emit_event(event.clone());
        self.active_effects.insert(effect.id.clone(), effect);

        event
    }

    /// Update all active effects (called at end of turn)
    pub fn update_effects(&mut self) {
        let mut effects_to_remove = Vec::new();

        // Collect effects to apply to avoid borrow checker issues
        // Sort by ID for deterministic processing order
        let mut effects_to_apply: Vec<(String, ActiveEffect)> = self
            .active_effects
            .iter()
            .map(|(id, effect)| (id.clone(), effect.clone()))
            .collect();
        effects_to_apply.sort_by(|a, b| a.0.cmp(&b.0));

        for (effect_id, effect) in effects_to_apply {
            // Apply effect logic based on type
            match &effect.effect_type {
                EffectType::DamageOverTime {
                    damage_per_round,
                    damage_type,
                } => {
                    // Apply damage through unified method
                    let _events = self.apply_damage(
                        &effect.target_id,
                        *damage_per_round,
                        damage_type,
                        &effect.source_id,
                    );
                }
                EffectType::HealingOverTime { healing_per_round } => {
                    // Apply healing through unified method
                    let _event = self.apply_healing(
                        &effect.target_id,
                        *healing_per_round,
                        false,
                        &effect.source_id,
                    );
                }
                EffectType::Condition(condition) => {
                    // Ensure condition is applied
                    if let Some(combatant) = self.combatants.get_mut(&effect.target_id) {
                        if !combatant.conditions.contains(condition) {
                            combatant.conditions.push(*condition);
                        }
                    }
                }
                _ => {} // Buffs and Custom effects handled elsewhere
            }

            // Decrease duration
            if effect.remaining_duration <= 0 {
                effects_to_remove.push(effect_id.clone());
            }
        }

        // Remove expired effects
        for effect_id in effects_to_remove {
            if let Some(effect) = self.active_effects.remove(&effect_id) {
                // Emit effect expiration event
                self.event_bus.emit_event(Event::Custom {
                    event_type: "EffectExpired".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("effect_id".to_string(), effect_id.clone());
                        data.insert("target_id".to_string(), effect.target_id.clone());
                        data
                    },
                    source_id: effect.source_id,
                });
            }
        }
    }

    /// Get all active effects on a specific target
    pub fn get_effects_on_target(&self, target_id: &str) -> Vec<&ActiveEffect> {
        self.active_effects
            .values()
            .filter(|effect| effect.target_id == target_id)
            .collect()
    }

    /// Get combatant state by ID
    pub fn get_combatant(&self, combatant_id: &str) -> Option<&CombattantState> {
        self.combatants.get(combatant_id)
    }

    /// Get mutable combatant state by ID
    pub fn get_combatant_mut(&mut self, combatant_id: &str) -> Option<&mut CombattantState> {
        self.combatants.get_mut(combatant_id)
    }

    /// Check if a combatant has a specific condition from active effects or permanent conditions
    pub fn has_condition(&self, target_id: &str, condition: CreatureCondition) -> bool {
        // 1. Check permanent conditions on the combatant
        if let Some(combatant) = self.combatants.get(target_id) {
            if combatant.conditions.contains(&condition) {
                return true;
            }
        }

        // 2. Check active effects (temporary buffs/debuffs)
        self.active_effects
            .values()
            .filter(|e| e.target_id == target_id)
            .any(|e| {
                if let EffectType::Condition(c) = &e.effect_type {
                    *c == condition
                } else if let EffectType::Buff(b) = &e.effect_type {
                    b.condition == Some(condition)
                } else {
                    false
                }
            })
    }

    /// Check if a combatant is alive (using standardized HP threshold >= 0.5)
    pub fn is_combatant_alive(&self, combatant_id: &str) -> bool {
        self.combatants
            .get(combatant_id)
            .is_some_and(|c| c.current_hp > 0)
    }

    /// Get all alive combatants (using standardized HP threshold >= 0.5)
    pub fn get_alive_combatants(&self) -> Vec<&CombattantState> {
        let mut combatants: Vec<&CombattantState> = self
            .combatants
            .values()
            .filter(|c| c.current_hp > 0)
            .collect();

        // Sort by ID for deterministic order (HashMap iteration is random in Rust)
        combatants.sort_by(|a, b| a.id.cmp(&b.id));
        combatants
    }

    /// Apply damage to a combatant with proper event emission
    /// This is the unified way to apply damage - all damage should go through this method
    pub fn apply_damage(
        &mut self,
        target_id: &str,
        damage: f64,
        damage_type: &str,
        source_id: &str,
    ) -> Vec<Event> {
        let mut events = Vec::new();

        if let Some(combatant) = self.combatants.get_mut(target_id) {
            let mut remaining_damage = damage;

            // 1. Arcane Ward Absorption
            if let Some(ward) = combatant.arcane_ward_hp {
                if ward > 0 {
                    let absorbed = (ward as f64).min(remaining_damage);
                    combatant.arcane_ward_hp = Some(((ward as f64) - absorbed).round() as u32);
                    remaining_damage = (remaining_damage - absorbed).max(0.0);

                    // Could emit WardAbsorbed event here if Event enum supported it
                }
            }

            // 2. Temporary HP Absorption
            if combatant.temp_hp > 0 {
                let absorbed = (combatant.temp_hp as f64).min(remaining_damage);
                combatant.temp_hp = ((combatant.temp_hp as f64) - absorbed).round() as u32;
                remaining_damage = (remaining_damage - absorbed).max(0.0);
            }

            // 3. Apply remaining damage to HP
            let actual_damage = remaining_damage;
            combatant.current_hp = ((combatant.current_hp as f64) - actual_damage)
                .max(0.0)
                .round() as u32;

            // Sync with base_combatant for targeting modules
            combatant.base_combatant.final_state.current_hp = combatant.current_hp;
            combatant.base_combatant.final_state.temp_hp = Some(combatant.temp_hp);

            events.push(Event::DamageTaken {
                target_id: target_id.to_string(),
                damage: actual_damage,
                damage_type: damage_type.to_string(),
            });

            // Check if combatant died (using standardized threshold == 0)
            if combatant.current_hp == 0 {
                events.push(Event::UnitDied {
                    unit_id: target_id.to_string(),
                    killer_id: Some(source_id.to_string()),
                    damage_type: Some(damage_type.to_string()),
                });

                // Immediately remove all buffs from the dead combatant
                self.remove_all_buffs_from_source(target_id);

                // Remove from combat stats cache to optimize targeting
                self.remove_combatant_from_cache(target_id);
            }
        }

        // Emit all events to the event bus
        for event in &events {
            self.event_bus.emit_event(event.clone());
        }

        events
    }

    /// Apply healing to a combatant with proper event emission
    /// This is the unified way to apply healing - all healing should go through this method
    pub fn apply_healing(
        &mut self,
        target_id: &str,
        amount: f64,
        is_temp_hp: bool,
        source_id: &str,
    ) -> Event {
        if let Some(combatant) = self.combatants.get_mut(target_id) {
            let event = if is_temp_hp {
                combatant.temp_hp = ((combatant.temp_hp as f64) + amount).round() as u32;
                combatant.base_combatant.final_state.temp_hp = Some(combatant.temp_hp);
                Event::TempHPGranted {
                    target_id: target_id.to_string(),
                    amount,
                    source_id: source_id.to_string(),
                }
            } else {
                let max_hp = combatant.base_combatant.creature.hp as f64;
                let current_hp = combatant.current_hp as f64;
                // Calculate healing amount, capping at max HP
                let actual_healing = (current_hp + amount).min(max_hp) - current_hp;

                // Update HP
                combatant.current_hp = (current_hp + actual_healing).round() as u32;
                combatant.base_combatant.final_state.current_hp = combatant.current_hp;

                Event::HealingApplied {
                    target_id: target_id.to_string(),
                    amount: actual_healing,
                    source_id: source_id.to_string(),
                }
            };

            // Emit the event to the event bus
            self.event_bus.emit_event(event.clone());
            event
        } else {
            let event = Event::HealingApplied {
                target_id: target_id.to_string(),
                amount: 0.0,
                source_id: source_id.to_string(),
            };
            self.event_bus.emit_event(event.clone());
            event
        }
    }

    /// Remove all buffs from a specific source (used when caster dies)
    /// This ensures no "zombie buffs" persist after caster death
    pub fn remove_all_buffs_from_source(&mut self, source_id: &str) {
        let mut effects_to_remove = Vec::new();

        // Find all effects from the specified source
        for (effect_id, effect) in &self.active_effects {
            if effect.source_id == source_id {
                effects_to_remove.push(effect_id.clone());
            }
        }

        // Remove the effects
        for effect_id in effects_to_remove {
            self.active_effects.remove(&effect_id);

            // Emit buff removal event
            self.event_bus.emit_event(Event::Custom {
                event_type: "BuffRemoved".to_string(),
                data: {
                    let mut data = HashMap::new();
                    data.insert("effect_id".to_string(), effect_id.clone());
                    data.insert("source_id".to_string(), source_id.to_string());
                    data
                },
                source_id: source_id.to_string(),
            });
        }
    }

    /// Get statistics about the current context
    pub fn get_stats(&self) -> ContextStats {
        ContextStats {
            round_number: self.round_number,
            current_turn_owner: self.current_turn_owner.clone(),
            total_combatants: self.combatants.len(),
            alive_combatants: self.get_alive_combatants().len(),
            total_effects: self.active_effects.len(),
            pending_events: self.event_bus.pending_count(),
        }
    }

    /// Get cached combat statistics for a combatant
    pub fn get_combatant_stats(&mut self, combatant_id: &str) -> Option<&CombatantStats> {
        if let Some(combatant_state) = self.combatants.get(combatant_id) {
            // Get or calculate stats through the cache
            let stats = self
                .combat_stats_cache
                .get_stats(&combatant_state.base_combatant);

            // Also cache in the combatant state for quick access
            if let Some(state) = self.combatants.get_mut(combatant_id) {
                state.cached_stats = Some(stats.clone());
            }

            Some(stats)
        } else {
            None
        }
    }

    /// Pre-calculate combat statistics for all combatants
    pub fn precalculate_combat_stats(&mut self) {
        let combatants: Vec<Combattant> = self
            .combatants
            .values()
            .map(|state| state.base_combatant.clone())
            .collect();

        self.combat_stats_cache
            .precalculate_for_combatants(&combatants);

        // Update cached stats in each combatant state
        for state in self.combatants.values_mut() {
            if let Some(stats) = self
                .combat_stats_cache
                .get_stats_by_id(&state.base_combatant.creature.id)
            {
                state.cached_stats = Some(stats.clone());
            }
        }
    }

    /// Invalidate combat stats cache (call when combatants die or state changes significantly)
    pub fn invalidate_combat_stats_cache(&mut self) {
        self.combat_stats_cache.mark_dirty();

        // Clear cached stats from all combatant states
        for state in self.combatants.values_mut() {
            state.cached_stats = None;
        }
    }

    /// Remove combatant from stats cache (e.g., when combatant dies)
    pub fn remove_combatant_from_cache(&mut self, combatant_id: &str) {
        if let Some(combatant_state) = self.combatants.get(combatant_id) {
            let creature_id = &combatant_state.base_combatant.creature.id;
            self.combat_stats_cache.remove_creature(creature_id);
        }

        // Clear cached stats from the combatant state
        if let Some(state) = self.combatants.get_mut(combatant_id) {
            state.cached_stats = None;
        }
    }

    /// Get combat stats cache statistics
    pub fn get_cache_stats(&self) -> crate::combat_stats::CacheStats {
        self.combat_stats_cache.get_cache_stats()
    }
}

/// Statistics about the turn context
#[derive(Debug, Clone)]
pub struct ContextStats {
    pub round_number: u32,
    pub current_turn_owner: Option<String>,
    pub total_combatants: usize,
    pub alive_combatants: usize,
    pub total_effects: usize,
    pub pending_events: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Combattant, Creature, CreatureState};
    use crate::resources::ActionCost;

    #[test]
    fn test_turn_context_creation() {
        let creature = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "player1".to_string(),
            name: "Player 1".to_string(),
            count: 1.0,
            hp: 30,
            ac: 15,
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
            initiative_bonus: crate::model::DiceFormula::Value(0.0),
            initiative_advantage: false,
            actions: Vec::new(),
            triggers: Vec::new(),
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "monster".to_string(),
        };

        let combatants = vec![Combattant {
            team: 0,
            id: "player1".to_string(),
            creature: std::sync::Arc::new(creature),
            initiative: 10.0,
            initial_state: CreatureState {
                current_hp: 30,
                ..CreatureState::default()
            },
            final_state: CreatureState {
                current_hp: 30,
                ..CreatureState::default()
            },
            actions: Vec::new(),
        }];

        let context = TurnContext::new(combatants, Vec::new(), None, "Plains".to_string(), true);

        assert_eq!(context.round_number, 0);
        assert!(context.current_turn_owner.is_none());
        assert_eq!(context.combatants.len(), 1);
        assert!(context.is_combatant_alive("player1"));
    }

    #[test]
    fn test_turn_management() {
        let creature = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "player1".to_string(),
            name: "Player 1".to_string(),
            count: 1.0,
            hp: 30,
            ac: 15,
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
            initiative_bonus: crate::model::DiceFormula::Value(0.0),
            initiative_advantage: false,
            actions: Vec::new(),
            triggers: Vec::new(),
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "monster".to_string(),
        };

        let combatants = vec![Combattant {
            team: 0,
            id: "player1".to_string(),
            creature: std::sync::Arc::new(creature),
            initiative: 10.0,
            initial_state: CreatureState {
                current_hp: 30,
                ..CreatureState::default()
            },
            final_state: CreatureState {
                current_hp: 30,
                ..CreatureState::default()
            },
            actions: Vec::new(),
        }];

        let mut context =
            TurnContext::new(combatants, Vec::new(), None, "Plains".to_string(), true);

        context.start_new_turn("player1".to_string());
        assert_eq!(context.current_turn_owner, Some("player1".to_string()));

        context.end_current_turn();
        assert!(context.current_turn_owner.is_none());

        context.advance_round();
        assert_eq!(context.round_number, 1);
    }

    #[test]
    fn test_resource_management() {
        let creature = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "player1".to_string(),
            name: "Player 1".to_string(),
            count: 1.0,
            hp: 30,
            ac: 15,
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
            initiative_bonus: crate::model::DiceFormula::Value(0.0),
            initiative_advantage: false,
            actions: Vec::new(),
            triggers: Vec::new(),
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "monster".to_string(),
        };

        let combatants = vec![Combattant {
            team: 0,
            id: "player1".to_string(),
            creature: std::sync::Arc::new(creature),
            initiative: 10.0,
            initial_state: CreatureState {
                current_hp: 30,
                ..CreatureState::default()
            },
            final_state: CreatureState {
                current_hp: 30,
                ..CreatureState::default()
            },
            actions: Vec::new(),
        }];

        let mut context =
            TurnContext::new(combatants, Vec::new(), None, "Plains".to_string(), true);

        let costs = vec![ActionCost::Discrete {
            resource_type: crate::resources::ResourceType::Action,
            resource_val: None,
            amount: 1.0,
        }];

        assert!(context.can_afford(&costs, "player1"));

        let result = context.pay_costs(&costs, "player1");
        assert!(result.is_ok());

        assert!(!context.can_afford(&costs, "player1"));
    }
    #[test]
    fn test_effect_management() {
        let creature = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "player1".to_string(),
            name: "Player 1".to_string(),
            count: 1.0,
            hp: 30,
            ac: 15,
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
            initiative_bonus: crate::model::DiceFormula::Value(0.0),
            initiative_advantage: false,
            actions: Vec::new(),
            triggers: Vec::new(),
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "monster".to_string(),
        };

        let combatants = vec![Combattant {
            team: 0,
            id: "player1".to_string(),
            creature: std::sync::Arc::new(creature),
            initiative: 10.0,
            initial_state: CreatureState {
                current_hp: 30,
                ..CreatureState::default()
            },
            final_state: CreatureState {
                current_hp: 30,
                ..CreatureState::default()
            },
            actions: Vec::new(),
        }];

        let mut context =
            TurnContext::new(combatants, Vec::new(), None, "Plains".to_string(), true);

        let effect = ActiveEffect {
            id: "test_effect".to_string(),
            source_id: "source".to_string(),
            target_id: "player1".to_string(),
            effect_type: EffectType::DamageOverTime {
                damage_per_round: 5.0,
                damage_type: "Fire".to_string(),
            },
            remaining_duration: 3,
            conditions: Vec::new(),
        };

        context.apply_effect(effect);
        assert_eq!(context.active_effects.len(), 1);

        let effects = context.get_effects_on_target("player1");
        assert_eq!(effects.len(), 1);

        // Update effects (should apply damage)
        context.update_effects();
        let combatant = context
            .get_combatant("player1")
            .expect("player1 combatant should exist in test context");
        assert_eq!(combatant.current_hp, 25); // 30 - 5 = 25
    }
}
