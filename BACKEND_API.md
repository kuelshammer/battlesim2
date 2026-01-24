# Backend API Reference (Rust/WASM)

**Purpose**: Function catalog for planning LLMs - understand what exists without reading implementations.

---

## Table of Contents

1. [WASM API Bindings](#wasm-api-bindings)
2. [Orchestration Layer](#orchestration-layer)
3. [Execution Engine](#execution-engine)
4. [Action Resolution](#action-resolution)
5. [Targeting System](#targeting-system)
6. [Context & State Management](#context--state-management)
7. [Event System](#event-system)
8. [Reaction System](#reaction-system)
9. [Caching Layer](#caching-layer)
10. [Analysis Module](#analysis-module)
11. [Type Definitions](#type-definitions)
12. [Utility Functions](#utility-functions)
13. [When to Modify What](#when-to-modify-what)

---

## WASM API Bindings

**File**: `simulation-wasm/src/api/wasm_api.rs`

### Core Simulation Functions

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `run_simulation_wasm()` | Main WASM entry point for simulation | `players: Vec<Creature>`, `timeline: Vec<TimelineEvent>`, `iterations: u32` | `SimulationResults` |
| `run_simulation_with_callback()` | Simulation with progress callback | Same as above + `callback: js_sys::Function` | `SimulationResults` |
| `run_event_driven_simulation()` | Event-driven simulation | `players: Vec<Creature>`, `timeline: Vec<TimelineEvent>`, `iterations: u32` | `FullAnalysisOutput` |
| `run_skyline_analysis_wasm()` | Compute skyline analysis | `results: Vec<SimulationResult>`, `party_size: usize`, `encounter_index: usize` | `SkylineAnalysis` |
| `run_decile_analysis_wasm()` | Compute decile statistics | `results: Vec<SimulationResult>`, `scenario_name: String`, `party_size: usize` | `DecileStats` |

### Auto-Balance Functions

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `auto_adjust_encounter_wasm()` | Encounter auto-balancing | `players: Vec<Creature>`, `monsters: Vec<Creature>`, `timeline: Vec<TimelineEvent>`, `encounter_index: usize` | `AutoAdjustmentResult` |
| `calculate_encounter_tier_wasm()` | Calculate encounter tier | `results: Vec<SimulationResult>` | `EncounterTier` |

### Cache Management Functions

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `clear_simulation_cache()` | Clear all caches | None | `() ` |
| `get_cache_stats()` | Get cache statistics | None | `CacheStats` (entries, bytes) |
| `clear_action_cache()` | Clear action template cache | None | `() ` |

### Memory Management Functions

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `init_memory_guardrails()` | Initialize memory guardrails | `max_iterations: u32` | `() ` |
| `should_force_lightweight_mode()` | Check if should use lightweight mode | `iterations: u32` | `bool` |

---

## Orchestration Layer

**Files**: `simulation-wasm/src/orchestration/runners.rs`, `gui.rs`, `balancer.rs`

### Simulation Runners

| Function | File | Purpose | Parameters | Returns |
|----------|------|---------|------------|---------|
| `run_simulation_with_rolling_stats()` | `runners.rs` | Two-pass with simple tiers | `players: Vec<Creature>`, `timeline: Vec<TimelineEvent>`, `iterations: u32`, `seed: Option<u64>` | `SimulationResults` |
| `run_simulation_with_three_tier()` | `runners.rs` | Two-pass with 1% buckets | `players: Vec<Creature>`, `timeline: Vec<TimelineEvent>`, `iterations: u32`, `seed: Option<u64>` | `SimulationResults` |
| `run_lightweight_survey()` | `runners.rs` | Phase 1 survey pass | `players: Vec<Creature>`, `timeline: Vec<TimelineEvent>`, `iterations: u32`, `seed: u64` | `Vec<SurveyResult>` |
| `run_deep_dive_simulation()` | `runners.rs` | Phase 3 re-simulation | `players: Vec<Creature>`, `timeline: Vec<TimelineEvent>`, `seeds: Vec<(u64, TierType)>` | `HashMap<u64, SimulationResult>` |

### GUI Orchestration

| Function | File | Purpose | Parameters | Returns |
|----------|------|---------|------------|---------|
| `run_simulation_for_gui()` | `gui.rs` | GUI-specific simulation | `players: Vec<Creature>`, `timeline: Vec<TimelineEvent>`, `iterations: u32` | `GuiSimulationOutput` |
| `build_gui_output()` | `gui.rs` | Build GUI output structure | `results: Vec<SimulationResult>`, `events: Vec<SimulationEvent>` | `FullAnalysisOutput` |

### Auto-Balancer

| Function | File | Purpose | Parameters | Returns |
|----------|------|---------|------------|---------|
| `auto_adjust_encounter()` | `balancer.rs` | Main auto-balance logic | `players: Vec<Creature>`, `monsters: Vec<Creature>`, `timeline: Vec<TimelineEvent>`, `encounter_index: usize` | `AutoAdjustmentResult` |
| `calculate_contextual_tier()` | `balancer.rs` | Calculate contextual tier | `isolated_tier: EncounterTier`, `resources_percent: f64` | `EncounterTier` |
| `adjust_monsters_to_tier()` | `balancer.rs` | Adjust monsters to target tier | `monsters: Vec<Creature>`, `target_tier: EncounterTier` | `Vec<Creature>` |

---

## Execution Engine

**Files**: `simulation-wasm/src/execution/engine.rs`, `lean.rs`

### Main Engine

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `ActionExecutionEngine::new()` | Create new engine | `players: Vec<Creature>`, `monsters: Vec<Creature>`, `log_events: bool` | `ActionExecutionEngine` |
| `execute_encounter()` | Run complete combat | `&mut self` | `SimulationResult` |
| `execute_combatant_turn()` | Execute single turn | `&mut self`, `combatant_id: String` | `() ` |
| `execute_action_with_reactions()` | Execute action + reactions | `&mut self`, `action: Action`, `actor_id: String` | `() ` |
| `process_reaction_phase()` | Process triggered reactions | `&mut self`, `trigger_event: &SimulationEvent` | `() ` |
| `select_actions_for_combatant()` | AI action selection | `&self`, `combatant_id: String` | `Vec<Action>` |
| `score_action()` | Score action for AI | `&self`, `action: &Action`, `combatant_id: String` | `f64` |

### Lean Execution (Phase 1)

| Function | File | Purpose | Parameters | Returns |
|----------|------|---------|------------|---------|
| `run_lean_simulation()` | `lean.rs` | Lightweight simulation | `players: Vec<Creature>`, `timeline: Vec<TimelineEvent>`, `seed: u64` | `LeanSimulationResult` |

---

## Action Resolution

**Files**: `simulation-wasm/src/action_resolver.rs`, `resolvers/*.rs`

### Main Resolver

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `resolve_action()` | Convert Action to events | `context: &mut TurnContext`, `action: &Action`, `actor_id: String` | `Vec<SimulationEvent>` |
| `resolve_attack()` | Resolve attack action | `context: &mut TurnContext`, `action: &AtkAction`, `actor_id: String` | `Vec<SimulationEvent>` |
| `resolve_heal()` | Resolve heal action | `context: &mut TurnContext`, `action: &HealAction`, `actor_id: String` | `Vec<SimulationEvent>` |
| `resolve_buff_application()` | Resolve buff action | `context: &mut TurnContext`, `action: &BuffAction`, `actor_id: String` | `Vec<SimulationEvent>` |
| `resolve_debuff()` | Resolve debuff action | `context: &mut TurnContext`, `action: &DebuffAction`, `actor_id: String` | `Vec<SimulationEvent>` |
| `resolve_template_action()` | Resolve template action | `context: &mut TurnContext`, `action: &TemplateAction`, `actor_id: String` | `Vec<SimulationEvent>` |

### Attack Resolver (`resolvers/attack.rs`)

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `resolve_single_attack()` | Resolve single attack | `context: &mut TurnContext`, `attack: &AtkAction`, `actor_id: String`, `target_id: String` | `Vec<SimulationEvent>` |
| `roll_to_hit()` | Roll d20 + bonus | `bonus: f64` | `HitRoll` |
| `roll_damage()` | Roll damage dice | `damage_dice: String`, `crit: bool` | `u32` |
| `apply_crit_damage()` | Apply critical damage | `base_damage: u32`, `damage_dice: String` | `u32` |

### Heal Resolver (`resolvers/heal.rs`)

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `resolve_heal_action()` | Resolve heal action | `context: &mut TurnContext`, `action: &HealAction`, `actor_id: String` | `Vec<SimulationEvent>` |

### Buff Resolver (`resolvers/buff.rs`)

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `resolve_buff_action()` | Resolve buff action | `context: &mut TurnContext`, `action: &BuffAction`, `actor_id: String` | `Vec<SimulationEvent>` |

### Debuff Resolver (`resolvers/debuff.rs`)

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `resolve_debuff_action()` | Resolve debuff action | `context: &mut TurnContext`, `action: &DebuffAction`, `actor_id: String` | `Vec<SimulationEvent>` |

### Template Resolver (`resolvers/template.rs`)

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `resolve_template_action()` | Resolve template action | `context: &mut TurnContext`, `action: &TemplateAction`, `actor_id: String` | `Vec<SimulationEvent>` |
| `resolve_template_to_action()` | Resolve template to action | `template: &str`, `overrides: HashMap<String, String>` | `Action` |

---

## Targeting System

**File**: `simulation-wasm/src/targeting.rs`

### Enemy Target Selection

| Strategy | Description |
|----------|-------------|
| `LeastHP` | Target enemy with lowest current HP percentage |
| `MostHP` | Target enemy with highest current HP |
| `HighestDPR` | Target enemy with highest damage per round |
| `LowestAC` | Target enemy with lowest armor class |
| `HighestSurvivability` | Target enemy with highest HP × AC |
| `Random` | Target random enemy |

### Ally Target Selection

| Strategy | Description |
|----------|-------------|
| `LeastHP` | Target ally with lowest current HP percentage |
| `Self` | Target self |

### Functions

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `select_enemy_target()` | Select enemy target | `context: &TurnContext`, `actor_id: String`, `strategy: TargetStrategy` | `String` (target_id) |
| `select_ally_target()` | Select ally target | `context: &TurnContext`, `actor_id: String`, `strategy: TargetStrategy` | `String` (target_id) |
| `select_multiple_targets()` | Select multiple targets | `context: &TurnContext`, `actor_id: String`, `strategy: TargetStrategy`, `count: usize` | `Vec<String>` |

---

## Context & State Management

**File**: `simulation-wasm/src/context.rs`

### TurnContext Creation

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `TurnContext::new()` | Create new context | `players: Vec<Creature>`, `monsters: Vec<Creature>` | `TurnContext` |
| `TurnContext::with_seed()` | Create context with seed | `players: Vec<Creature>`, `monsters: Vec<Creature>`, `seed: u64` | `TurnContext` |

### Damage & Healing

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `apply_damage()` | Apply damage to combatant | `&mut self`, `target_id: String`, `amount: u32`, `damage_type: DamageType` | `Vec<SimulationEvent>` |
| `apply_healing()` | Apply healing to combatant | `&mut self`, `target_id: String`, `amount: u32` | `Vec<SimulationEvent>` |
| `grant_temp_hp()` | Grant temporary HP | `&mut self`, `target_id: String`, `amount: u32` | `Vec<SimulationEvent>` |

### Buff Management

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `add_buff()` | Add buff to combatant | `&mut self`, `target_id: String`, `buff: Buff` | `Vec<SimulationEvent>` |
| `remove_buff()` | Remove buff from combatant | `&mut self`, `target_id: String`, `buff_id: String` | `Vec<SimulationEvent>` |
| `expire_buffs()` | Expire expired buffs | `&mut self`, `combatant_id: String` | `Vec<SimulationEvent>` |

### Resource Management

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `pay_costs()` | Pay action costs | `&mut self`, `actor_id: String`, `costs: &Vec<ActionCost>` | `Result<(), String>` |
| `can_afford()` | Check if can afford costs | `&self`, `actor_id: String`, `costs: &Vec<ActionCost>` | `bool` |
| `consume_resource()` | Consume resource | `&mut self`, `combatant_id: String`, `resource: String`, `amount: i32` | `Result<(), String>` |
| `restore_resource()` | Restore resource | `&mut self`, `combatant_id: String`, `resource: String`, `amount: i32` | `() ` |
| `reset_resources()` | Reset resources on rest | `&mut self`, `combatant_id: String`, `rest_type: RestType` | `() ` |

### Combatant Access

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `get_combatant()` | Get combatant | `&self`, `id: &str` | `Option<&CombattantState>` |
| `get_combatant_mut()` | Get mutable combatant | `&mut self`, `id: &str` | `Option<&mut CombattantState>` |
| `is_alive()` | Check if combatant alive | `&self`, `id: &str` | `bool` |
| `get_alive_combatants()` | Get all alive combatants | `&self`, `side: CombattantSide` | `Vec<String>` |

---

## Event System

**File**: `simulation-wasm/src/events.rs`

### EventBus Functions

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `EventBus::new()` | Create new event bus | None | `EventBus` |
| `emit()` | Emit event | `&mut self`, `event: SimulationEvent` | `() ` |
| `pending_events()` | Get pending events | `&self` | `&Vec<SimulationEvent>` |
| `clear_pending()` | Clear pending events | `&mut self` | `() ` |
| `add_listener()` | Add event listener | `&mut self`, `combatant_id: String` | `() ` |
| `get_history()` | Get event history | `&self` | `&Vec<SimulationEvent>` |

### Event Creation Helpers

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `damage_taken_event()` | Create damage event | `target_id: String`, `amount: u32` | `SimulationEvent` |
| `healing_applied_event()` | Create heal event | `target_id: String`, `amount: u32` | `SimulationEvent` |
| `unit_died_event()` | Create death event | `combatant_id: String` | `SimulationEvent` |

---

## Reaction System

**File**: `simulation-wasm/src/reactions.rs`

### ReactionManager Functions

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `ReactionManager::new()` | Create new manager | None | `ReactionManager` |
| `check_reactions()` | Check for triggered reactions | `&mut self`, `context: &TurnContext`, `trigger_event: &SimulationEvent` | `Vec<ReactionTemplate>` |
| `use_reaction()` | Mark reaction as used | `&mut self`, `combatant_id: String`, `reaction_id: String` | `Result<(), String>` |
| `reset_round_reactions()` | Reset per-round reactions | `&mut self` | `() ` |
| `reset_encounter_reactions()` | Reset per-encounter reactions | `&mut self` | `() ` |

### Reaction Triggers

| Trigger | Description |
|---------|-------------|
| `OnHit` | Trigger when this combatant hits |
| `OnBeingAttacked` | Trigger when this combatant is attacked |
| `OnMiss` | Trigger when this combatant misses |
| `OnBeingDamaged` | Trigger when this combatant takes damage |
| `OnEnemyDeath` | Trigger when an enemy dies |
| `OnAllyDeath` | Trigger when an ally dies |

---

## Caching Layer

**File**: `simulation-wasm/src/action_cache.rs`

### Cache Functions

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `get_or_create_template()` | Get or create cached template | `template_name: String`, `overrides: HashMap<String, String>` | `ResolvedAction` |
| `clear_cache()` | Clear template cache | None | `() ` |
| `get_cache_stats()` | Get cache statistics | None | `(usize, usize)` (entries, bytes) |

### Cached Templates

- `Bless`: +d4 to attack rolls and saves
- `Bane`: -d4 to attack rolls and saves
- `Haste`: Double speed, +2 AC, extra action
- `Shield`: +5 AC until next turn
- `Hunter's Mark`: Extra 1d6 damage
- `Hex`: -d4 to ability checks
- `Hypnotic Pattern`: Charm condition

---

## Analysis Module

**Files**: `simulation-wasm/src/analysis/statistics.rs`, `visualization.rs`, `narrative.rs`

### Statistics

| Function | File | Purpose | Parameters | Returns |
|----------|------|---------|------------|---------|
| `compute_decile_stats()` | `statistics.rs` | Compute per-decile stats | `results: Vec<SimulationResult>`, `scenario_name: String`, `party_size: usize` | `DecileStats` |
| `compute_skyline_analysis()` | `statistics.rs` | Compute skyline analysis | `results: Vec<SimulationResult>`, `party_size: usize` | `SkylineAnalysis` |
| `compute_vitals()` | `statistics.rs` | Compute vitals index | `skyline: &SkylineAnalysis` | `Vitals` |
| `compute_day_pacing()` | `statistics.rs` | Compute day pacing | `encounters: Vec<AggregateOutput>` | `DayPacing` |

### Visualization

| Function | File | Purpose | Parameters | Returns |
|----------|------|---------|------------|---------|
| `generate_spectrogram_data()` | `visualization.rs` | Generate spectrogram data | `events: Vec<SimulationEvent>` | `SpectrogramData` |
| `generate_heatmap_data()` | `visualization.rs` | Generate heatmap data | `results: Vec<SimulationResult>` | `HeatmapData` |

### Narrative

| Function | File | Purpose | Parameters | Returns |
|----------|------|---------|------------|---------|
| `generate_encounter_summary()` | `narrative.rs` | Generate encounter summary | `output: &AggregateOutput` | `String` |
| `generate_tpk_explanation()` | `narrative.rs` | Generate TPK explanation | `output: &AggregateOutput` | `String` |

---

## Type Definitions

### Key Types

| Type | File | Description |
|------|------|-------------|
| `Creature` | `model/creature.rs` | Creature definition with stats, actions, resources |
| `Combattant` | `model/creature.rs` | Runtime combatant instance |
| `Action` | `model/action.rs` | Action enum (Atk, Heal, Buff, Debuff, Template) |
| `AtkAction` | `model/action.rs` | Attack action with DPR, to-hit, target |
| `HealAction` | `model/action.rs` | Heal action with amount, target |
| `BuffAction` | `model/action.rs` | Buff action with buffs, target |
| `DebuffAction` | `model/action.rs` | Debuff action with debuffs, target |
| `TemplateAction` | `model/action.rs` | Template action with name, overrides |
| `ActionCost` | `model/action.rs` | Action cost (discrete or variable) |
| `ActionRequirement` | `model/action.rs` | Action requirement predicate |
| `SimulationEvent` | `model/events.rs` | Event enum (30+ variants) |
| `TimelineEvent` | `model/timeline.rs` | Timeline event (Encounter or ShortRest) |
| `Encounter` | `model/timeline.rs` | Combat encounter with monsters |
| `ShortRest` | `model/timeline.rs` | Short rest with duration |
| `SimulationResult` | `model/timeline.rs` | Single simulation result |
| `SkylineAnalysis` | `model/timeline.rs` | Skyline analysis data |
| `DecileStats` | `model/timeline.rs` | Per-decile statistics |
| `EncounterTier` | `model/enums.rs` | Encounter tier (Trivial, Safe, Challenging, Boss, Failed) |

---

## Utility Functions

### Dice Rolling (`simulation-wasm/src/dice.rs`)

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `roll_dice()` | Roll dice expression | `dice_str: String` | `u32` |
| `roll_dice_with_bonus()` | Roll dice with bonus | `dice_str: String`, `bonus: f64` | `u32` |
| `parse_dice_expression()` | Parse dice expression | `expr: String` | `DiceExpression` |

### RNG (`simulation-wasm/src/rng.rs`)

| Function | Purpose | Parameters | Returns |
|----------|---------|------------|---------|
| `SeededRng::new()` | Create seeded RNG | `seed: u64` | `SeededRng` |
| `roll()` | Roll random number | `&mut self` | `f64` |
| `roll_range()` | Roll in range | `&mut self`, `min: u32`, `max: u32` | `u32` |
| `shuffle()` | Shuffle slice | `&mut self`, `slice: &mut [T]` | `() ` |

---

## When to Modify What

### Adding a New Action Type

1. **Model** (`model/action.rs`):
   - Add variant to `Action` enum
   - Add corresponding action struct (e.g., `NewAction`)

2. **Resolver** (`action_resolver.rs` or `resolvers/new_action.rs`):
   - Create resolver function `resolve_new_action()`
   - Add to `resolve_action()` match arm

3. **Validation** (`validation.rs`):
   - Add validation for new action requirements

### Adding a New Event Type

1. **Model** (`model/events.rs`):
   - Add variant to `SimulationEvent` enum

2. **Context** (`context.rs`):
   - Add emit logic in relevant functions

### Adding a New Target Strategy

1. **Model** (`model/enums.rs`):
   - Add variant to `TargetStrategy` enum

2. **Targeting** (`targeting.rs`):
   - Implement selection logic in `select_enemy_target()` or `select_ally_target()`

### Modifying Simulation Flow

1. **Execution** (`execution/engine.rs`):
   - Modify `execute_encounter()` for main loop changes
   - Modify `execute_combatant_turn()` for turn logic changes

2. **Orchestration** (`orchestration/runners.rs`):
   - Modify `run_simulation_with_three_tier()` for two-pass changes

### Adding New Analysis

1. **Statistics** (`analysis/statistics.rs`):
   - Add analysis function
   - Wire into `compute_decile_stats()` or `compute_skyline_analysis()`

2. **Visualization** (`analysis/visualization.rs`):
   - Add data generation function

3. **WASM API** (`api/wasm_api.rs`):
   - Add WASM binding function

### Modifying Auto-Balance

1. **Balancer** (`orchestration/balancer.rs`):
   - Modify `auto_adjust_encounter()` for algorithm changes
   - Modify `calculate_contextual_tier()` for tier calculation

2. **Encounter Balancer** (`encounter_balancer.rs`):
   - Modify encounter tier thresholds

### Modifying Memory Management

1. **Memory Guardrails** (`memory_guardrails.rs`):
   - Adjust thresholds for lightweight mode
   - Add new memory checks

2. **Two-Pass** (`two_pass.rs`):
   - Modify seed selection algorithm
   - Adjust tier definitions

---

## References

- **ARCHITECTURE.md**: Comprehensive system architecture
- **FRONTEND_API.md**: React/TypeScript component catalog
- **DATA_FLOW.md**: Request lifecycles and state transitions
