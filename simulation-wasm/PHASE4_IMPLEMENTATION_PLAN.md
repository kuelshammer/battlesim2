# Phase 4: Execution Engine & Integration Implementation Plan

## Overview

This phase transforms the Phase 3 foundation into a working execution engine that integrates the event and context systems into the main simulation loop. We'll update the core simulation to emit events instead of direct effects, create the action resolution engine that properly processes actions and reactions, and establish the proper sequencing for turn-based combat with reactive abilities.

## Core Objectives

1. **Integrate Event System** - Update simulation.rs to use TurnContext and EventBus
2. **Create Action Resolution Engine** - Build the core execution engine that processes actions and reactions
3. **Update Resolution System** - Modify resolution.rs to emit events instead of direct effects
4. **Establish Proper Turn Sequencing** - Implement the correct order of operations for combat rounds
5. **Add Frontend Components** - Create EventLog.tsx and update UI for new event-driven system

## Implementation Components

### 4.1 Action Execution Engine

**File**: `simulation-wasm/src/execution.rs` (NEW)

The ActionExecutionEngine will be the central coordinator for all action processing:

```rust
pub struct ActionExecutionEngine {
    context: TurnContext,
    reaction_manager: ReactionManager,
    action_resolver: ActionResolver,
}

impl ActionExecutionEngine {
    pub fn new(combatants: Vec<Combattant>) -> Self;

    /// Execute a full combat encounter until completion
    pub fn execute_encounter(&mut self) -> EncounterResult;

    /// Process a single turn for a combatant
    pub fn execute_turn(&mut self, combatant_id: &str, actions: Vec<Action>) -> TurnResult;

    /// Process an action and all resulting reactions
    pub fn execute_action_with_reactions(&mut self, actor_id: &str, action: Action) -> ActionResult;

    /// Process reaction phase after an action
    pub fn process_reaction_phase(&mut self, triggering_event: &Event) -> Vec<ReactionResult>;
}
```

Key Features:
- Coordinates action resolution with reaction system
- Maintains proper turn sequencing (Action → Reaction → Effect Resolution)
- Handles resource management through TurnContext
- Integrates with EventBus for reactive abilities

### 4.2 Enhanced Action Resolver

**File**: `simulation-wasm/src/resolution.rs` (MODIFY)

Update the existing action resolution system to emit events instead of applying direct effects:

```rust
impl ActionResolver {
    /// Resolve an action and emit events to the context
    pub fn resolve_action(&self, action: &Action, context: &mut TurnContext, actor_id: &str) -> Vec<Event>;

    /// Resolve attack actions with proper event emission
    pub fn resolve_attack(&self, attack: &AtkAction, context: &mut TurnContext, actor_id: &str) -> Vec<Event>;

    /// Resolve healing actions with proper event emission
    pub fn resolve_heal(&self, heal: &HealAction, context: &mut TurnContext, actor_id: &str) -> Vec<Event>;

    /// Resolve buff/debuff actions with proper event emission
    pub fn resolve_effect(&self, effect_action: &impl EffectAction, context: &mut TurnContext, actor_id: &str) -> Vec<Event>;

    /// Apply damage with proper event emission and reaction triggers
    pub fn apply_damage(&self, target_id: &str, damage: f64, damage_type: &str, context: &mut TurnContext) -> Vec<Event>;
}
```

Key Changes:
- All resolution methods now return Vec<Event> instead of applying direct effects
- Proper integration with TurnContext for state management
- Support for reaction triggers through event emission
- Damage application with armor class calculations and critical hit handling

### 4.3 Integrated Simulation Loop

**File**: `simulation-wasm/src/simulation.rs` (MODIFY)

Update the main simulation loop to use the new execution engine:

```rust
pub fn run_single_simulation(combatants: Vec<Combattant>) -> SimulationResult {
    let mut engine = ActionExecutionEngine::new(combatants);

    // Initialize encounter
    engine.context.record_event(Event::EncounterStarted {
        combatant_ids: engine.context.combatants.keys().cloned().collect(),
    });

    // Main combat loop
    while !is_encounter_complete(&engine.context) {
        engine.context.advance_round();

        let initiative_order = calculate_initiative_order(&engine.context);

        for combatant_id in initiative_order {
            if !engine.context.is_combatant_alive(&combatant_id) {
                continue;
            }

            // Execute turn with all actions and reactions
            let turn_result = engine.execute_combatant_turn(&combatant_id);

            // Process end-of-turn effects
            engine.context.update_effects();
        }
    }

    // Generate final results
    generate_encounter_results(&engine.context)
}

fn execute_combatant_turn(&mut self, combatant_id: &str) -> TurnResult {
    self.context.start_new_turn(combatant_id.to_string());

    // Execute AI or player actions
    let actions = self.select_actions_for_combatant(combatant_id);

    let mut action_results = Vec::new();
    for action in actions {
        let result = self.execute_action_with_reactions(combatant_id, action);
        action_results.push(result);

        // Process reaction phase after each action
        self.process_reaction_phase(&result.triggering_event);
    }

    self.context.end_current_turn();
    TurnResult {
        combatant_id: combatant_id.to_string(),
        action_results,
        effects_applied: self.get_effects_for_combatant(combatant_id),
    }
}
```

### 4.4 Enhanced Context Integration

**File**: `simulation-wasm/src/context.rs` (MODIFY)

Add missing methods to TurnContext for full integration:

```rust
impl TurnContext {
    /// Get all combatants sorted by initiative
    pub fn get_initiative_order(&self) -> Vec<String> {
        let mut combatants: Vec<_> = self.combatants.values().collect();
        combatants.sort_by(|a, b| b.base_combatant.initiative.partial_cmp(&a.base_combatant.initiative).unwrap());
        combatants.into_iter().map(|c| c.id.clone()).collect()
    }

    /// Check if encounter is complete (all but one side defeated)
    pub fn is_encounter_complete(&self) -> bool {
        let alive_combatants = self.get_alive_combatants();
        if alive_combatants.len() <= 1 {
            return true;
        }

        // Check if all alive combatants are on same team
        let first_team = alive_combatants[0].base_combatant.team;
        alive_combatants.iter().all(|c| c.base_combatant.team == first_team)
    }

    /// Apply damage to a combatant with proper event emission
    pub fn apply_damage(&mut self, target_id: &str, damage: f64, damage_type: &str, source_id: &str) -> Vec<Event> {
        let mut events = Vec::new();

        if let Some(combatant) = self.combatants.get_mut(target_id) {
            let actual_damage = damage;
            combatant.current_hp = (combatant.current_hp - actual_damage).max(0.0);

            events.push(Event::DamageTaken {
                target_id: target_id.to_string(),
                damage: actual_damage,
                damage_type: damage_type.to_string(),
            });

            if combatant.current_hp <= 0.0 {
                events.push(Event::UnitDied {
                    unit_id: target_id.to_string(),
                    killer_id: Some(source_id.to_string()),
                    damage_type: Some(damage_type.to_string()),
                });
            }
        }

        events
    }

    /// Apply healing to a combatant with proper event emission
    pub fn apply_healing(&mut self, target_id: &str, amount: f64, source_id: &str) -> Event {
        if let Some(combatant) = self.combatants.get_mut(target_id) {
            let max_hp = combatant.base_combatant.creature.hp;
            let actual_healing = (combatant.current_hp + amount).min(max_hp) - combatant.current_hp;
            combatant.current_hp += actual_healing;

            Event::HealingApplied {
                target_id: target_id.to_string(),
                amount: actual_healing,
                source_id: source_id.to_string(),
            }
        } else {
            Event::HealingApplied {
                target_id: target_id.to_string(),
                amount: 0.0,
                source_id: source_id.to_string(),
            }
        }
    }
}
```

### 4.5 Reaction System Integration

**File**: `simulation-wasm/src/reactions.rs` (MODIFY)

Enhance the reaction system with proper action execution:

```rust
impl ReactionManager {
    /// Execute all triggered reactions for an event
    pub fn execute_triggered_reactions(
        &mut self,
        event: &Event,
        context: &mut TurnContext
    ) -> Vec<ReactionResult> {
        let triggered = self.check_reactions(event, context);
        let mut results = Vec::new();

        for (combatant_id, reaction) in triggered {
            // Check if combatant can still react (might be dead, etc.)
            if !context.is_combatant_alive(&combatant_id) {
                continue;
            }

            match self.execute_reaction(&combatant_id, reaction, context) {
                Ok(()) => {
                    results.push(ReactionResult {
                        combatant_id,
                        reaction_id: reaction.id.clone(),
                        success: true,
                        events_generated: context.event_bus.get_recent_events(5).to_vec(),
                    });
                },
                Err(e) => {
                    results.push(ReactionResult {
                        combatant_id,
                        reaction_id: reaction.id.clone(),
                        success: false,
                        error: Some(e),
                        events_generated: Vec::new(),
                    });
                }
            }
        }

        results
    }

    /// Get reactions that would trigger for an event (without executing)
    pub fn get_triggered_reactions(&self, event: &Event, context: &TurnContext) -> Vec<(String, &ReactionTemplate)> {
        let mut triggered = Vec::new();

        for (combatant_id, reactions) in &self.available_reactions {
            if !context.is_combatant_alive(combatant_id) {
                continue;
            }

            for reaction in reactions {
                if self.can_trigger_reaction(combatant_id, reaction, event, context) {
                    triggered.push((combatant_id.clone(), reaction));
                }
            }
        }

        // Sort by priority
        triggered.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
        triggered
    }
}

#[derive(Debug, Clone)]
pub struct ReactionResult {
    pub combatant_id: String,
    pub reaction_id: String,
    pub success: bool,
    pub events_generated: Vec<Event>,
    pub error: Option<String>,
}
```

### 4.6 Frontend Event Log Component

**File**: `frontend/src/components/EventLog.tsx` (NEW)

Create a React component to display combat events in real-time:

```tsx
interface EventLogProps {
  events: Event[];
  maxHeight?: number;
  filterByType?: string[];
  highlightCombatant?: string;
}

const EventLog: React.FC<EventLogProps> = ({
  events,
  maxHeight = 400,
  filterByType,
  highlightCombatant
}) => {
  const filteredEvents = useMemo(() => {
    let filtered = events;

    if (filterByType && filterByType.length > 0) {
      filtered = filtered.filter(event => filterByType.includes(event.get_type()));
    }

    return filtered.slice(-50); // Show last 50 events
  }, [events, filterByType]);

  const getEventIcon = (eventType: string) => {
    switch (eventType) {
      case 'AttackHit': return '⚔️';
      case 'AttackMissed': return '❌';
      case 'DamageTaken': return '💔';
      case 'HealingApplied': return '💚';
      case 'UnitDied': return '💀';
      case 'RoundStarted': return '🔄';
      case 'TurnStarted': return '➡️';
      default: return '📝';
    }
  };

  const getEventColor = (event: Event) => {
    if (highlightCombatant && event.involves_combatant(highlightCombatant)) {
      return 'highlighted';
    }

    switch (event.get_type()) {
      case 'AttackHit': return 'damage';
      case 'HealingApplied': return 'healing';
      case 'UnitDied': return 'death';
      case 'RoundStarted': return 'system';
      default: return 'default';
    }
  };

  return (
    <div className="event-log" style={{ maxHeight, overflowY: 'auto' }}>
      <div className="event-log-header">
        <h3>Combat Log</h3>
        <EventFilter
          availableTypes={getUniqueEventTypes(events)}
          onFilterChange={setFilterByType}
        />
      </div>

      <div className="event-list">
        {filteredEvents.map((event, index) => (
          <div
            key={index}
            className={`event-item ${getEventColor(event)}`}
            title={`${event.get_type()}: ${formatEventDetails(event)}`}
          >
            <span className="event-icon">
              {getEventIcon(event.get_type())}
            </span>
            <span className="event-text">
              {formatEventMessage(event)}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
};

// Helper functions
const formatEventMessage = (event: Event): string => {
  switch (event.get_type()) {
    case 'AttackHit':
      return `${event.get_source_id()} hits ${event.get_target_id()} for ${(event as any).damage} damage!`;
    case 'HealingApplied':
      return `${event.get_target_id()} healed for ${(event as any).amount} HP!`;
    case 'UnitDied':
      return `${event.get_target_id()} has been defeated!`;
    case 'RoundStarted':
      return `Round ${(event as any).round_number} begins!`;
    case 'TurnStarted':
      return `${event.get_source_id()}'s turn begins!`;
    default:
      return `${event.get_type()}: ${formatEventDetails(event)}`;
  }
};
```

### 4.7 WASM Interface Updates

**File**: `simulation-wasm/src/lib.rs` (MODIFY)

Update the WASM interface to support the new event-driven simulation:

```rust
#[wasm_bindgen]
pub fn run_simulation_with_events(players: JsValue, encounters: JsValue, iterations: usize) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let encounters: Vec<Encounter> = serde_wasm_bindgen::from_value(encounters)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse encounters: {}", e)))?;

    let mut results = Vec::new();

    for i in 0..iterations {
        let result = simulation::run_single_simulation_with_events(
            &players,
            &encounters[i % encounters.len()]
        );
        results.push(result);
    }

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);

    serde::Serialize::serialize(&results, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize results: {}", e)))
}

#[wasm_bindgen]
pub fn run_single_combat_simulation_with_events(
    players: JsValue,
    encounter: JsValue
) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let encounter: Encounter = serde_wasm_bindgen::from_value(encounter)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse encounter: {}", e)))?;

    let result = simulation::run_single_simulation_with_events(&players, &encounter);

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);

    serde::Serialize::serialize(&result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
}

#[derive(Serialize, Deserialize)]
pub struct EventDrivenSimulationResult {
    pub winner: Option<String>,
    pub total_rounds: u32,
    pub events: Vec<Event>,
    pub final_combatant_states: Vec<CombattantState>,
    pub statistics: EncounterStatistics,
}

#[derive(Serialize, Deserialize)]
pub struct EncounterStatistics {
    pub total_damage_dealt: HashMap<String, f64>,
    pub total_healing_dealt: HashMap<String, f64>,
    pub attacks_landed: HashMap<String, u32>,
    pub attacks_missed: HashMap<String, u32>,
    pub reactions_triggered: u32,
    pub critical_hits: u32,
}
```

## Implementation Steps

### Step 4.1: Create Action Execution Engine
1. Create `execution.rs` with ActionExecutionEngine
2. Implement basic action processing with reaction integration
3. Add proper error handling and resource management
4. Create comprehensive unit tests
5. Commit: "feat: Create ActionExecutionEngine for coordinated action processing"

### Step 4.2: Update Action Resolution System
1. Modify `resolution.rs` to emit events instead of direct effects
2. Update all resolution methods to return Vec<Event>
3. Ensure proper integration with TurnContext
4. Update tests to verify event emission
5. Commit: "feat: Update action resolution to emit events"

### Step 4.3: Integrate Simulation Loop
1. Update `simulation.rs` to use ActionExecutionEngine
2. Implement proper turn sequencing with reaction phases
3. Add encounter completion detection
4. Update WASM interface functions
5. Add comprehensive integration tests
6. Commit: "feat: Integrate execution engine into simulation loop"

### Step 4.4: Enhance TurnContext
1. Add missing methods to TurnContext
2. Implement damage and healing application with event emission
3. Add encounter completion detection
4. Update context tests
5. Commit: "feat: Enhance TurnContext with full integration support"

### Step 4.5: Complete Reaction System
1. Update ReactionManager with proper action execution
2. Add ReactionResult struct for detailed feedback
3. Implement reaction triggering without execution (preview)
4. Add comprehensive reaction tests
5. Commit: "feat: Complete reaction system with action execution"

### Step 4.6: Create Frontend Event Log
1. Create `EventLog.tsx` component
2. Implement event filtering and highlighting
3. Add event formatting utilities
4. Update main simulation interface to use EventLog
5. Add styling for different event types
6. Commit: "feat: Create EventLog frontend component"

### Step 4.7: Update WASM Interface
1. Update `lib.rs` with event-driven simulation functions
2. Add EventDrivenSimulationResult struct
3. Implement encounter statistics collection
4. Update frontend to use new WASM functions
5. Add end-to-end tests
6. Commit: "feat: Update WASM interface for event-driven simulation"

### Step 4.8: Integration Testing & Validation
1. Create comprehensive integration tests
2. Test full combat encounters with reactions
3. Validate event emission and processing
4. Test WASM compilation and frontend integration
5. Performance testing and optimization
6. Commit: "feat: Complete Phase 4 integration and testing"

## Success Criteria

1. **Functional Execution Engine** - ActionExecutionEngine properly processes actions and reactions in correct sequence
2. **Event-Driven Resolution** - All action resolution emits events instead of direct effects
3. **Integrated Simulation Loop** - Main simulation uses new execution engine with proper turn sequencing
4. **Working Reaction System** - Reactions trigger properly based on events with resource costs and priorities
5. **Frontend Event Display** - EventLog component displays real-time combat events with filtering
6. **WASM Compatibility** - All new features compile to WASM and integrate with frontend
7. **Comprehensive Testing** - Full test coverage with integration tests validating end-to-end functionality

## Testing Strategy

### Unit Tests
- ActionExecutionEngine action processing
- Event emission from resolution system
- TurnContext damage/healing application
- ReactionManager reaction execution
- Frontend component rendering

### Integration Tests
- Full combat encounters with reactions
- Event processing and reaction triggering
- WASM compilation and frontend integration
- Turn sequencing and resource management

### Performance Tests
- Large combat encounters with many reactions
- Event processing overhead
- Memory usage optimization

## Next Steps

After Phase 4 completion, the system will have:
- Complete event-driven combat simulation
- Reactive ability system with proper sequencing
- Real-time event logging and visualization
- Solid foundation for advanced features

Phase 5 will focus on:
- Advanced action templates and scripting
- GUI for creating custom actions and reactions
- Performance optimization for large simulations
- Advanced statistics and analytics