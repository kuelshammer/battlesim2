use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::enums::{CreatureCondition, TriggerCondition};
use crate::model::Action;

/// Comprehensive event enum covering all combat interactions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    // Combat Events
    ActionStarted { actor_id: String, action_id: String },
    AttackHit { attacker_id: String, target_id: String, damage: f64 },
    AttackMissed { attacker_id: String, target_id: String },
    DamageTaken { target_id: String, damage: f64, damage_type: String },
    DamagePrevented { target_id: String, prevented_amount: f64 },

    // Spell Events
    SpellCast { caster_id: String, spell_id: String, spell_level: u8 },
    SpellSaved { target_id: String, spell_id: String },
    SpellFailed { target_id: String, spell_id: String, reason: String },
    ConcentrationBroken { caster_id: String, reason: String },
    ConcentrationMaintained { caster_id: String, save_dc: f64 },

    // Status Events
    BuffApplied { target_id: String, buff_id: String, source_id: String },
    BuffExpired { target_id: String, buff_id: String },
    BuffRemoved { target_id: String, buff_id: String, source_id: String },
    ConditionAdded { target_id: String, condition: CreatureCondition, source_id: String },
    ConditionRemoved { target_id: String, condition: CreatureCondition, source_id: String },

    // Healing Events
    HealingApplied { target_id: String, amount: f64, source_id: String },
    TempHPGranted { target_id: String, amount: f64, source_id: String },
    TempHPLost { target_id: String, amount: f64 },

    // Life Cycle Events
    UnitDied { unit_id: String, killer_id: Option<String>, damage_type: Option<String> },
    TurnStarted { unit_id: String, round_number: u32 },
    TurnEnded { unit_id: String, round_number: u32 },
    RoundStarted { round_number: u32 },
    RoundEnded { round_number: u32 },
    EncounterStarted { combatant_ids: Vec<String> },
    EncounterEnded { winner: Option<String>, reason: String },

    // Movement Events (future extensibility)
    MovementStarted { unit_id: String, from_position: String, to_position: String },
    MovementInterrupted { unit_id: String, reason: String },
    OpportunityAttack { attacker_id: String, target_id: String, provoked_by: String },

    // Resource Events
    ResourceConsumed { unit_id: String, resource_type: String, amount: f64 },
    ResourceRestored { unit_id: String, resource_type: String, amount: f64 },
    ResourceDepleted { unit_id: String, resource_type: String },

    // Custom Events for user-defined abilities
    Custom { event_type: String, data: HashMap<String, String>, source_id: String },
}

impl Event {
    /// Get the primary actor/source of this event
    pub fn get_source_id(&self) -> Option<String> {
        match self {
            Event::ActionStarted { actor_id, .. } => Some(actor_id.clone()),
            Event::AttackHit { attacker_id, .. } => Some(attacker_id.clone()),
            Event::AttackMissed { attacker_id, .. } => Some(attacker_id.clone()),
            Event::SpellCast { caster_id, .. } => Some(caster_id.clone()),
            Event::BuffApplied { source_id, .. } => Some(source_id.clone()),
            Event::ConditionAdded { source_id, .. } => Some(source_id.clone()),
            Event::HealingApplied { source_id, .. } => Some(source_id.clone()),
            Event::TempHPGranted { source_id, .. } => Some(source_id.clone()),
            Event::OpportunityAttack { attacker_id, .. } => Some(attacker_id.clone()),
            Event::ResourceConsumed { unit_id, .. } => Some(unit_id.clone()),
            Event::Custom { source_id, .. } => Some(source_id.clone()),
            _ => None,
        }
    }

    /// Get the primary target/recipient of this event
    pub fn get_target_id(&self) -> Option<String> {
        match self {
            Event::AttackHit { target_id, .. } => Some(target_id.clone()),
            Event::AttackMissed { target_id, .. } => Some(target_id.clone()),
            Event::DamageTaken { target_id, .. } => Some(target_id.clone()),
            Event::DamagePrevented { target_id, .. } => Some(target_id.clone()),
            Event::SpellSaved { target_id, .. } => Some(target_id.clone()),
            Event::SpellFailed { target_id, .. } => Some(target_id.clone()),
            Event::BuffApplied { target_id, .. } => Some(target_id.clone()),
            Event::BuffExpired { target_id, .. } => Some(target_id.clone()),
            Event::BuffRemoved { target_id, .. } => Some(target_id.clone()),
            Event::ConditionAdded { target_id, .. } => Some(target_id.clone()),
            Event::ConditionRemoved { target_id, .. } => Some(target_id.clone()),
            Event::HealingApplied { target_id, .. } => Some(target_id.clone()),
            Event::TempHPGranted { target_id, .. } => Some(target_id.clone()),
            Event::TempHPLost { target_id, .. } => Some(target_id.clone()),
            Event::UnitDied { unit_id, .. } => Some(unit_id.clone()),
            Event::TurnStarted { unit_id, .. } => Some(unit_id.clone()),
            Event::TurnEnded { unit_id, .. } => Some(unit_id.clone()),
            Event::MovementStarted { unit_id, .. } => Some(unit_id.clone()),
            Event::MovementInterrupted { unit_id, .. } => Some(unit_id.clone()),
            Event::OpportunityAttack { target_id, .. } => Some(target_id.clone()),
            Event::ResourceConsumed { unit_id, .. } => Some(unit_id.clone()),
            Event::ResourceRestored { unit_id, .. } => Some(unit_id.clone()),
            Event::ResourceDepleted { unit_id, .. } => Some(unit_id.clone()),
            _ => None,
        }
    }

    /// Check if this event involves a specific combatant
    pub fn involves_combatant(&self, combatant_id: &str) -> bool {
        self.get_source_id().as_ref().map_or(false, |id| id == combatant_id) ||
        self.get_target_id().as_ref().map_or(false, |id| id == combatant_id)
    }

    /// Get event type as string for filtering and logging
    pub fn get_type(&self) -> &'static str {
        match self {
            Event::ActionStarted { .. } => "ActionStarted",
            Event::AttackHit { .. } => "AttackHit",
            Event::AttackMissed { .. } => "AttackMissed",
            Event::DamageTaken { .. } => "DamageTaken",
            Event::DamagePrevented { .. } => "DamagePrevented",
            Event::SpellCast { .. } => "SpellCast",
            Event::SpellSaved { .. } => "SpellSaved",
            Event::SpellFailed { .. } => "SpellFailed",
            Event::ConcentrationBroken { .. } => "ConcentrationBroken",
            Event::ConcentrationMaintained { .. } => "ConcentrationMaintained",
            Event::BuffApplied { .. } => "BuffApplied",
            Event::BuffExpired { .. } => "BuffExpired",
            Event::BuffRemoved { .. } => "BuffRemoved",
            Event::ConditionAdded { .. } => "ConditionAdded",
            Event::ConditionRemoved { .. } => "ConditionRemoved",
            Event::HealingApplied { .. } => "HealingApplied",
            Event::TempHPGranted { .. } => "TempHPGranted",
            Event::TempHPLost { .. } => "TempHPLost",
            Event::UnitDied { .. } => "UnitDied",
            Event::TurnStarted { .. } => "TurnStarted",
            Event::TurnEnded { .. } => "TurnEnded",
            Event::RoundStarted { .. } => "RoundStarted",
            Event::RoundEnded { .. } => "RoundEnded",
            Event::EncounterStarted { .. } => "EncounterStarted",
            Event::EncounterEnded { .. } => "EncounterEnded",
            Event::MovementStarted { .. } => "MovementStarted",
            Event::MovementInterrupted { .. } => "MovementInterrupted",
            Event::OpportunityAttack { .. } => "OpportunityAttack",
            Event::ResourceConsumed { .. } => "ResourceConsumed",
            Event::ResourceRestored { .. } => "ResourceRestored",
            Event::ResourceDepleted { .. } => "ResourceDepleted",
            Event::Custom { .. } => "Custom",
        }
    }
}

/// Represents a listener that reacts to specific events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventListener {
    pub id: String,
    pub owner_id: String,
    pub trigger_condition: TriggerCondition,
    pub response_action: Action,
    pub priority: i32, // Higher priority reactions go first
    pub uses_per_encounter: Option<u32>,
    pub remaining_uses: u32,
}

impl EventListener {
    pub fn new(
        id: String,
        owner_id: String,
        trigger_condition: TriggerCondition,
        response_action: Action,
        priority: i32,
        uses_per_encounter: Option<u32>,
    ) -> Self {
        let remaining_uses = uses_per_encounter.unwrap_or(u32::MAX);
        Self {
            id,
            owner_id,
            trigger_condition,
            response_action,
            priority,
            uses_per_encounter,
            remaining_uses,
        }
    }

    /// Check if this listener can trigger for the given event
    pub fn can_trigger(&self, event: &Event) -> bool {
        if self.remaining_uses == 0 {
            return false;
        }

        // Check trigger condition against event
        match (&self.trigger_condition, event) {
            (TriggerCondition::OnHit, Event::AttackHit { .. }) => true,
            (TriggerCondition::OnBeingAttacked, Event::AttackHit { target_id, .. }) => {
                // Check if the listener's owner is the target
                self.owner_id == *target_id
            },
            (TriggerCondition::OnMiss, Event::AttackMissed { .. }) => true,
            (TriggerCondition::OnBeingDamaged, Event::DamageTaken { target_id, .. }) => {
                self.owner_id == *target_id
            },
            (TriggerCondition::OnAllyAttacked, Event::AttackHit { target_id: _, .. }) => {
                // This would need to check if target is an ally - complex logic for later
                false // Placeholder
            },
            (TriggerCondition::OnEnemyDeath, Event::UnitDied { unit_id: _, .. }) => {
                // This would need to check if unit is an enemy - complex logic for later
                false // Placeholder
            },
            (TriggerCondition::OnCriticalHit, Event::AttackHit { .. }) => {
                // This would need to check if attack was a critical hit
                false // Placeholder
            },
            _ => false,
        }
    }

    /// Use this listener (decrement remaining uses)
    pub fn use_listener(&mut self) {
        if self.remaining_uses > 0 {
            self.remaining_uses -= 1;
        }
    }

    /// Reset remaining uses to maximum
    pub fn reset_uses(&mut self) {
        self.remaining_uses = self.uses_per_encounter.unwrap_or(u32::MAX);
    }
}

/// Central event management system for collecting, dispatching, and processing events
#[derive(Debug, Clone)]
pub struct EventBus {
    pending_events: Vec<Event>,
    event_history: Vec<Event>,
    listeners: HashMap<String, Vec<EventListener>>,
    max_history_size: usize,
}

impl EventBus {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            pending_events: Vec::new(),
            event_history: Vec::new(),
            listeners: HashMap::new(),
            max_history_size,
        }
    }

    /// Emit an event to the bus (adds to pending events)
    pub fn emit_event(&mut self, event: Event) {
        self.pending_events.push(event);
    }

    /// Register a new event listener
    pub fn register_listener(&mut self, listener: EventListener) {
        let owner_id = listener.owner_id.clone();
        self.listeners.entry(owner_id).or_insert_with(Vec::new).push(listener);
    }

    /// Process all pending events and return triggered reactions
    pub fn process_pending_events(&mut self) -> Vec<(String, Action)> {
        let mut triggered_reactions = Vec::new();

        // Collect all pending events first
        let pending_events: Vec<Event> = self.pending_events.drain(..).collect();

        // Add events to history
        for event in &pending_events {
            self.event_history.push(event.clone());
        }

        // Trim history if it exceeds max size
        if self.event_history.len() > self.max_history_size {
            let remove_count = self.event_history.len() - self.max_history_size;
            self.event_history.drain(0..remove_count);
        }

        // Check all listeners for reactions to each event
        for event in pending_events {
            triggered_reactions.extend(self.check_listeners_for_event(&event));
        }

        // Sort reactions by priority (higher first) and by owner for consistency
        triggered_reactions.sort_by(|a, b| {
            // First sort by priority (descending)
            let priority_cmp = b.1.base().action_slot.unwrap_or(0).cmp(&a.1.base().action_slot.unwrap_or(0));
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }
            // Then by owner ID for consistent ordering
            a.0.cmp(&b.0)
        });

        triggered_reactions
    }

    /// Check all listeners for reactions to a specific event
    fn check_listeners_for_event(&mut self, event: &Event) -> Vec<(String, Action)> {
        let mut triggered = Vec::new();

        // Clone listeners to avoid borrowing issues
        let listeners_clone = self.listeners.clone();

        for (owner_id, listeners) in &listeners_clone {
            for listener in listeners {
                if listener.can_trigger(event) {
                    triggered.push((owner_id.clone(), listener.response_action.clone()));
                }
            }
        }

        triggered
    }

    /// Get recent events from history
    pub fn get_recent_events(&self, count: usize) -> &[Event] {
        let start = if self.event_history.len() > count {
            self.event_history.len() - count
        } else {
            0
        };
        &self.event_history[start..]
    }

    /// Get all events from history
    pub fn get_all_events(&self) -> &[Event] {
        &self.event_history
    }

    /// Get events filtered by type
    pub fn get_events_by_type(&self, event_type: &str) -> Vec<&Event> {
        self.event_history
            .iter()
            .filter(|event| event.get_type() == event_type)
            .collect()
    }

    /// Get events involving a specific combatant
    pub fn get_events_for_combatant(&self, combatant_id: &str) -> Vec<&Event> {
        self.event_history
            .iter()
            .filter(|event| event.involves_combatant(combatant_id))
            .collect()
    }

    /// Get the number of pending events
    pub fn pending_count(&self) -> usize {
        self.pending_events.len()
    }

    /// Clear all pending events (use with caution)
    pub fn clear_pending(&mut self) {
        self.pending_events.clear();
    }

    /// Reset all listeners (for new encounters)
    pub fn reset_listeners(&mut self) {
        for listeners in self.listeners.values_mut() {
            for listener in listeners {
                listener.reset_uses();
            }
        }
    }

    /// Remove a specific listener
    pub fn remove_listener(&mut self, owner_id: &str, listener_id: &str) -> bool {
        if let Some(listeners) = self.listeners.get_mut(owner_id) {
            let original_len = listeners.len();
            listeners.retain(|listener| listener.id != listener_id);
            return listeners.len() < original_len;
        }
        false
    }

    /// Get all listeners for a specific owner
    pub fn get_listeners_for_owner(&self, owner_id: &str) -> &[EventListener] {
        self.listeners.get(owner_id).map_or(&[], |listeners| listeners.as_slice())
    }

    /// Get statistics about the event bus
    pub fn get_stats(&self) -> EventBusStats {
        let total_listeners: usize = self.listeners.values().map(|listeners| listeners.len()).sum();
        let pending_count = self.pending_events.len();
        let history_count = self.event_history.len();

        EventBusStats {
            total_listeners,
            pending_count,
            history_count,
            max_history_size: self.max_history_size,
        }
    }
}

/// Statistics about the event bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusStats {
    pub total_listeners: usize,
    pub pending_count: usize,
    pub history_count: usize,
    pub max_history_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    // Removed unused imports: ActionCost, ResourceType
    // use crate::resources::{ActionCost, ResourceType}; 

    #[test]
    fn test_event_creation() {
        let event = Event::AttackHit {
            attacker_id: "attacker".to_string(),
            target_id: "target".to_string(),
            damage: 10.0,
        };

        assert_eq!(event.get_source_id(), Some("attacker".to_string()));
        assert_eq!(event.get_target_id(), Some("target".to_string()));
        assert_eq!(event.get_type(), "AttackHit");
        assert!(event.involves_combatant("attacker"));
        assert!(event.involves_combatant("target"));
        assert!(!event.involves_combatant("other"));
    }

    #[test]
    fn test_event_listener() {
        // This would need a proper Action instance for full testing
        // For now, just test the basic logic
        let event = Event::AttackHit {
            attacker_id: "attacker".to_string(),
            target_id: "target".to_string(),
            damage: 10.0,
        };

        assert_eq!(event.get_type(), "AttackHit");
    }

    #[test]
    fn test_event_bus() {
        let mut bus = EventBus::new(100);

        // Test emitting events
        bus.emit_event(Event::RoundStarted { round_number: 1 });
        bus.emit_event(Event::RoundStarted { round_number: 2 });

        assert_eq!(bus.pending_count(), 2);
        assert_eq!(bus.get_all_events().len(), 0); // Not processed yet

        // Process events
        let reactions = bus.process_pending_events();

        assert_eq!(bus.pending_count(), 0);
        assert_eq!(bus.get_all_events().len(), 2);
        assert_eq!(reactions.len(), 0); // No listeners registered yet

        // Test recent events
        let recent = bus.get_recent_events(1);
        assert_eq!(recent.len(), 1);
        match &recent[0] {
            Event::RoundStarted { round_number, .. } => assert_eq!(*round_number, 2),
            _ => panic!("Expected RoundStarted event"),
        }
    }
}