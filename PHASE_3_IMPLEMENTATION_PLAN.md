# Phase 3 Implementation Plan: Event Bus & Context

## 🔗 Context
This document details the step-by-step implementation for **Phase 3** of the [Simulation Architecture Redesign](SIMULATION_ARCHITECTURE_REDESIGN.md).

**Goal:** Define the event system and turn context that actions interact with, creating the "nervous system" for reactive abilities and complex game state tracking.

---

## 🛠️ Step-by-Step Implementation

### Part A: Core Event System (Backend - Rust)

**File:** `simulation-wasm/src/events.rs` (new file)

1.  **Create Event Enum:**
    ```rust
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Event {
        // Combat Events
        ActionStarted { actor_id: String, action_id: String },
        AttackHit { attacker_id: String, target_id: String, damage: f64 },
        AttackMissed { attacker_id: String, target_id: String },
        DamageTaken { target_id: String, damage: f64, damage_type: String },

        // Spell Events
        SpellCast { caster_id: String, spell_id: String, spell_level: u8 },
        SpellSaved { target_id: String, spell_id: String },
        ConcentrationBroken { caster_id: String, reason: String },

        // Status Events
        BuffApplied { target_id: String, buff_id: String, source_id: String },
        BuffExpired { target_id: String, buff_id: String },
        ConditionAdded { target_id: String, condition: CreatureCondition },
        ConditionRemoved { target_id: String, condition: CreatureCondition },

        // Life Cycle Events
        UnitDied { unit_id: String, killer_id: Option<String> },
        TurnStarted { unit_id: String, round_number: u32 },
        TurnEnded { unit_id: String, round_number: u32 },
        RoundStarted { round_number: u32 },
        RoundEnded { round_number: u32 },

        // Movement Events (future extensibility)
        MovementStarted { unit_id: String, from_position: String, to_position: String },
        MovementInterrupted { unit_id: String, reason: String },
    }
    ```

2.  **Create Event Bus:**
    ```rust
    #[derive(Debug, Clone)]
    pub struct EventBus {
        pending_events: Vec<Event>,
        event_history: Vec<Event>,
        listeners: HashMap<String, Vec<EventListener>>,
    }

    #[derive(Debug, Clone)]
    pub struct EventListener {
        pub id: String,
        pub owner_id: String,
        pub trigger_condition: TriggerCondition,
        pub response_template: ActionTemplate, // What action to trigger
    }

    impl EventBus {
        pub fn new() -> Self;
        pub fn emit_event(&mut self, event: Event);
        pub fn register_listener(&mut self, listener: EventListener);
        pub fn process_pending_events(&mut self) -> Vec<(String, ActionTemplate)>; // Returns triggered reactions
        pub fn get_recent_events(&self, count: usize) -> &[Event];
        pub fn clear_pending(&mut self);
    }
    ```

### Part B: Turn Context System (Backend - Rust)

**File:** `simulation-wasm/src/context.rs` (new file)

1.  **Create TurnContext Struct:**
    ```rust
    #[derive(Debug, Clone)]
    pub struct TurnContext {
        // Resource Management
        pub resource_ledger: ResourceLedger,

        // Event Tracking
        pub event_bus: EventBus,
        pub round_number: u32,
        pub current_turn_owner: String,

        // Combat State
        pub combatants: HashMap<String, CombattantState>,
        pub active_effects: HashMap<String, ActiveEffect>,

        // Environmental Context
        pub battlefield_conditions: Vec<BattlefieldCondition>,
        pub weather: Option<WeatherCondition>,
        pub terrain: TerrainType,
    }

    #[derive(Debug, Clone)]
    pub struct ActiveEffect {
        pub id: String,
        pub source_id: String,
        pub target_id: String,
        pub effect_type: EffectType,
        pub remaining_duration: i32,
        pub conditions: Vec<ActionRequirement>,
    }

    #[derive(Debug, Clone)]
    pub enum EffectType {
        Buff(Buff),
        DamageOverTime { damage_per_round: f64, damage_type: String },
        HealingOverTime { healing_per_round: f64 },
        Condition(CreatureCondition),
        Custom(String),
    }
    ```

2.  **Implement Context Management:**
    ```rust
    impl TurnContext {
        pub fn new(combatants: Vec<Combattant>) -> Self;
        pub fn start_new_turn(&mut self, unit_id: String);
        pub fn end_current_turn(&mut self);
        pub fn advance_round(&mut self);

        // Resource Access
        pub fn can_afford(&self, costs: &[ActionCost], unit_id: &str) -> bool;
        pub fn pay_costs(&mut self, costs: &[ActionCost], unit_id: &str) -> Result<(), String>;

        // Event Integration
        pub fn record_event(&mut self, event: Event);
        pub fn check_reactions(&mut self, triggering_event: &Event) -> Vec<(String, ActionTemplate)>;

        // Effect Management
        pub fn apply_effect(&mut self, effect: ActiveEffect);
        pub fn update_effects(&mut self); // Called at end of turn
        pub fn get_effects_on_target(&self, target_id: &str) -> &[ActiveEffect];
    }
    ```

### Part C: Integration with Existing Systems (Backend - Rust)

**Files:** `simulation-wasm/src/simulation.rs`, `simulation-wasm/src/resolution.rs`

1.  **Update Simulation Engine:**
    *   Modify `execute_round()` to create and maintain a `TurnContext`
    *   Replace direct action execution with context-managed execution
    *   Integrate event bus processing into turn sequence

2.  **Update Resolution System:**
    *   Convert `resolve_attack()` to emit events instead of directly applying effects
    *   Create event handlers for common combat outcomes
    *   Ensure all state changes go through the context system

3.  **Update Actions System:**
    *   Modify `get_actions()` to check `Action.requirements` against context
    *   Create `validate_action()` that checks costs against resource ledger
    *   Implement reaction system integration

### Part D: Frontend Event Visualization (TypeScript)

**File:** `src/components/combat/EventLog.tsx` (new file)

1.  **Create Event Log Component:**
    *   Real-time display of combat events
    *   Filtering by event type and combatant
    *   Expandable details for complex events
    *   Highlighting of important events (critical hits, deaths, etc.)

2.  **Event Type Icons and Formatting:**
    *   Visual indicators for different event categories
    *   Color coding by damage type or effect type
    *   Timeline view for turn-by-turn analysis

### Part E: Reaction System Foundation (Backend - Rust)

**File:** `simulation-wasm/src/reactions.rs` (new file)

1.  **Create Reaction Manager:**
    ```rust
    #[derive(Debug, Clone)]
    pub struct ReactionManager {
        available_reactions: HashMap<String, Vec<ReactionTemplate>>,
        used_reactions: HashMap<String, HashSet<String>>, // Track used reactions per turn
    }

    #[derive(Debug, Clone)]
    pub struct ReactionTemplate {
        pub id: String,
        pub name: String,
        pub trigger_event_type: EventType,
        pub trigger_condition: TriggerCondition,
        pub response_action: Action,
        pub cost: Vec<ActionCost>,
        pub priority: i32, // Higher priority reactions go first
        pub uses_per_round: Option<u32>,
    }

    impl ReactionManager {
        pub fn check_reactions(&mut self, event: &Event, context: &TurnContext) -> Vec<ReactionTemplate>;
        pub fn execute_reaction(&mut self, reaction: &ReactionTemplate, context: &mut TurnContext);
        pub fn reset_reactions(&mut self);
    }
    ```

---

## ✅ Definition of Done (Phase 3)

Phase 3 is complete when:

1.  [ ] **Event System:** Full `Event` enum and `EventBus` implementation working
2.  [ ] **Turn Context:** `TurnContext` struct manages resource ledger and event history
3.  [ ] **Integration:** Simulation engine uses context for all state management
4.  [ ] **Event Broadcasting:** Actions emit events that are properly recorded and dispatched
5.  [ ] **Reaction Foundation:** Basic reaction template system in place (execution comes in Phase 4)
6.  [ ] **Frontend Support:** Event log component displays real-time combat events
7.  [ ] **No Breaking Changes:** Existing functionality preserved during integration

---

## 🔧 Technical Considerations

### Performance Optimization
- **Event Filtering:** Implement efficient event filtering to avoid processing irrelevant events
- **History Management:** Limit event history size with configurable retention policies
- **Batch Processing:** Process multiple events in batches during complex interactions

### Error Handling
- **Event Validation:** Ensure emitted events contain required data
- **Reaction Conflicts:** Handle cases where multiple reactions compete for the same trigger
- **Resource Overdraft:** Prevent situations where reactions exceed available resources

### Testing Strategy
- **Event Replay:** System to replay and debug specific event sequences
- **Isolation Testing:** Test event system independently from full simulation
- **Edge Cases:** Cover complex multi-trigger scenarios and reaction chains

---

## 🎯 Integration with Phase 4

Phase 3 establishes the **nervous system** that Phase 4's **execution engine** will use:

- **Event-driven triggers** will replace hardcoded reaction logic
- **Context-aware requirements** will enable sophisticated AI decision-making
- **Resource-ledger integration** will provide flexible action economy management
- **Reaction system** will lay foundation for complex response chains

This creates the infrastructure for truly dynamic, reactive combat where abilities can respond to any game event in user-defined ways.