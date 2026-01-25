# Architecture: Battlesim2 Simulation Engine & UI

## Executive Summary

Battlesim2 is a high-fidelity D&D 5e combat simulator built with a Rust/WASM backend and a React/Next.js frontend. The system executes large-scale batch simulations (10,100 iterations) for statistical analysis with 1% granularity percentile visualization.

**Tech Stack:**
- **Backend**: Rust 2024, WASM (`wasm-bindgen`), `serde`, `proptest`
- **Frontend**: Next.js 15, React 19, TypeScript 5, SCSS Modules, `framer-motion`
- **Design Philosophy**: Deterministic, event-driven simulation with "Intentional Minimalism" UI

**Key Design Principles:**
1. **Single Source of Truth**: All state changes flow through `TurnContext` with consistent event emission
2. **Two-Pass Simulation**: Memory-efficient batch runs with selective deep-dive re-simulation
3. **Event-Driven Architecture**: 30+ event types enable comprehensive combat logging and replay
4. **Modular Resolution**: Action resolvers are decoupled from execution engine for extensibility

---

## System Architecture

### Overview Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FRONTEND (React/Next.js)                          │
│  ┌───────────────┐  ┌──────────────┐  ┌─────────────────────────────────┐  │
│  │ Creature Forms│  │ Simulation UI│  │    Visualization Components     │  │
│  │ (Player/Monster)│ │  (Timeline) │  │  (Skyline, Heatmap, Event Log) │  │
│  └───────────────┘  └──────────────┘  └─────────────────────────────────┘  │
│           │                  │                           │                  │
│           └──────────────────┼───────────────────────────┘                  │
│                              │                                              │
│                    ┌─────────▼─────────┐                                    │
│                    │  Custom Hooks     │                                    │
│                    │  - useSimulationWorker  │                              │
│                    │  - useSimulationSession  │                             │
│                    │  - useAutoSimulation      │                            │
│                    └─────────┬─────────┘                                    │
└──────────────────────────────┼──────────────────────────────────────────────┘
                               │ postMessage
                    ┌──────────▼──────────┐
                    │   WebWorker         │
                    │   (simulation.worker.controller.ts)                     │
                    └──────────┬──────────┘
                               │ wasm-bindgen
┌──────────────────────────────▼──────────────────────────────────────────────┐
│                        BACKEND (Rust/WASM)                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        WASM API Layer                               │   │
│  │  wasm_api.rs: run_simulation_wasm, auto_adjust_encounter_wasm      │   │
│  └────────────────────────────────┬────────────────────────────────────┘   │
│                                   │                                         │
│  ┌────────────────────────────────▼────────────────────────────────────┐   │
│  │                      Orchestration Layer                            │   │
│  │  runners.rs, gui.rs, balancer.rs                                   │   │
│  └────────────────────────────────┬────────────────────────────────────┘   │
│                                   │                                         │
│  ┌────────────────────────────────▼────────────────────────────────────┐   │
│  │                   Two-Pass Simulation System                        │   │
│  │  two_pass.rs, seed_selection.rs                                     │   │
│  │  - Phase 1: Survey (10,100 lean)                                    │   │
│  │  - Phase 2: Seed Selection (170 medians)                            │   │
│  │  - Phase 3: Re-simulation (full events)                             │   │
│  └────────────────────────────────┬────────────────────────────────────┘   │
│                                   │                                         │
│  ┌────────────────────────────────▼────────────────────────────────────┐   │
│  │                   Action Execution Engine                           │   │
│  │  execution/engine.rs, lean.rs                                       │   │
│  │  - Turn-by-turn execution loop                                      │   │
│  │  - Action selection with AI scoring                                 │   │
│  │  - Reaction management                                              │   │
│  └────────────────────────────────┬────────────────────────────────────┘   │
│                                   │                                         │
│  ┌────────────────────────────────▼────────────────────────────────────┐   │
│  │                    Action Resolver Layer                            │   │
│  │  action_resolver.rs, action_cache.rs                                │   │
│  │  resolvers/: attack, heal, buff, debuff, template                   │   │
│  └────────────────────────────────┬────────────────────────────────────┘   │
│                                   │                                         │
│  ┌────────────────────────────────▼────────────────────────────────────┐   │
│  │                    Core Data Structures                             │   │
│  │  model/: Creature, Action, Event types                              │   │
│  │  context.rs: TurnContext (Single Source of Truth)                   │   │
│  │  events.rs: EventBus, 30+ event types                               │   │
│  │  reactions.rs: Reaction system                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Backend Architecture (Rust/WASM)

### Directory Structure

```
simulation-wasm/src/
├── model/                    # Core data structures
│   ├── creature.rs          # Creature definition with stats, actions, resources
│   ├── action.rs            # Action enum (Atk, Heal, Buff, Debuff, Template)
│   ├── events.rs            # 30+ event types for combat logging
│   ├── timeline.rs          # TimelineEvent (Encounter, ShortRest)
│   └── enums.rs             # Support enums (TargetStrategy, DamageType, etc.)
├── execution/                # Simulation execution engine
│   ├── engine.rs            # ActionExecutionEngine (main combat loop)
│   └── lean.rs              # Lightweight execution for Phase 1
├── orchestration/            # Orchestration layer
│   ├── runners.rs           # Simulation runners (three-tier, two-pass)
│   ├── gui.rs               # GUI-specific orchestration
│   └── balancer.rs          # Encounter auto-balancing
├── api/                      # WASM bindings and DTOs
│   ├── wasm_api.rs          # JavaScript interop functions
│   └── dto.rs               # Data transfer objects
├── resolvers/                # Modular action resolvers
│   ├── attack.rs            # Attack resolution (to-hit, damage, crit)
│   ├── heal.rs              # Healing application
│   ├── buff.rs              # Buff application
│   ├── debuff.rs            # Debuff application
│   └── template.rs          # Template action resolution
├── analysis/                 # Analysis module (refactored from decile_analysis.rs)
│   ├── mod.rs               # Public API, re-exports
│   ├── types.rs             # Core types, GameBalance config
│   ├── narrative.rs         # "Director" logic (scoring, pacing, labels)
│   ├── statistics.rs        # "Mathematician" logic (percentiles)
│   └── visualization.rs     # "Presenter" logic (frontend mapping)
├── action_resolver.rs        # Action→Event conversion dispatcher
├── action_cache.rs           # Template resolution caching (1000 entries LRU)
├── context.rs                # TurnContext (Single Source of Truth)
├── events.rs                 # EventBus and event definitions
├── reactions.rs              # Reaction system (triggers, effects)
├── targeting.rs              # Target selection strategies
├── validation.rs             # Action requirements validation
├── enhanced_validation.rs    # Comprehensive validation
├── two_pass.rs               # Two-pass simulation orchestration
├── seed_selection.rs         # 1% bucket seed selection algorithm
├── auto_balancer.rs          # Encounter difficulty auto-adjustment
├── memory_guardrails.rs      # Memory safety for large runs
├── background_simulation.rs  # Background job queue
├── queue_manager.rs          # Queue management system
├── rng.rs                    # RNG management
├── dice.rs                   # Dice rolling
├── dice_reconstruction.rs    # Dice reconstruction utilities
├── sorting.rs                # Sorting utilities
├── cleanup.rs                # Cleanup utilities
├── config.rs                 # Configuration system
├── combat_stats.rs           # Combat statistics caching
├── cache.rs                  # General caching utilities
├── aggregation.rs            # Aggregation utilities
├── safe_aggregation.rs       # Safe aggregation functions
├── resources.rs              # Resource management
├── actions.rs                # Action utilities
├── utils.rs                  # General utilities
├── percentile_analysis.rs    # Percentile analysis
├── intensity_calculation.rs  # Intensity calculation
├── strategic_assessment.rs   # Strategic assessment
├── creature_adjustment.rs    # Creature adjustment for auto-balancing
├── encounter_balancer.rs     # Encounter tier classification
├── display_manager.rs        # Display mode management
├── progress_communication.rs # Progress communication protocol
├── progress_ui.rs            # Progress UI component functions
├── monitoring.rs             # Success metrics and monitoring
├── user_interaction.rs       # User interaction flows
├── error_handling.rs         # Enhanced error handling
├── recovery.rs               # Error recovery mechanisms
└── lib.rs                    # Module exports
```

---

### Core Data Structures

#### Creature vs Combattant

**Creature** (`model/creature.rs`): Static definition
```rust
pub struct Creature {
    pub id: String,
    pub name: String,
    pub count: f64,              // Unit count (1-10+), e.g., 3.5 for goblins
    pub hp: u32,
    pub ac: u32,
    pub save_bonus: f64,         // Average save bonus
    pub str_save_bonus: Option<f64>,
    pub dex_save_bonus: Option<f64>,
    pub con_save_bonus: Option<f64>,
    pub int_save_bonus: Option<f64>,
    pub wis_save_bonus: Option<f64>,
    pub cha_save_bonus: Option<f64>,
    pub actions: Vec<Action>,
    pub triggers: Vec<ActionTrigger>,
    pub spell_slots: Option<HashMap<String, i32>>,
    pub class_resources: Option<HashMap<String, i32>>,
    pub hit_dice: Option<String>,
    pub con_modifier: Option<f64>,
    pub magic_items: Vec<String>,
    pub max_arcane_ward_hp: Option<u32>,
    pub initial_buffs: Vec<Buff>,
}
```

**Combattant** (`model/creature.rs`): Runtime instance
```rust
pub struct Combattant {
    pub id: String,
    pub side: CombattantSide,    // Hero or Monster
    pub creature_index: usize,
    pub final_state: CombattantState,
    pub actions: Vec<Action>,    // Actual actions taken (not from Creature definition)
}
```

#### Action Enum (`model/action.rs`)

```rust
pub enum Action {
    Atk(AtkAction),              // Attack with dpr, toHit, target
    Heal(HealAction),            // Healing ability
    Buff(BuffAction),            // Buffing allies
    Debuff(DebuffAction),        // Debuffing enemies
    Template(TemplateAction),    // Resolved template spell
}
```

**ActionCost Types:**
- **Discrete**: Fixed resource consumption (1 action, 1 bonus action)
- **Variable**: Resource with min/max (movement 30ft/60ft)

**Action Requirements:**
- `ResourceAvailable`: Spell slot, ki points, etc.
- `CombatState`: Bloodied, not bloodied, round number
- `StatusEffect`: Has condition, doesn't have condition
- `Custom`: Arbitrary predicate string

---

### TurnContext - Single Source of Truth

The `TurnContext` (`context.rs`) maintains all combat state and ensures consistent event emission:

```rust
pub struct TurnContext {
    // Event Tracking
    pub event_bus: EventBus,
    pub round_number: u32,
    pub current_turn_owner: Option<String>,
    pub log_enabled: bool,

    // Combat State
    pub combatants: HashMap<String, CombattantState>,
    pub active_effects: HashMap<String, ActiveEffect>,

    // Performance
    pub combat_stats_cache: CombatStatsCache,

    // Roll Manipulation
    pub roll_modifications: RollModificationQueue,

    // Interrupt System
    pub action_interrupted: bool,
}
```

**Key Methods:**
- `apply_damage()`: Unified damage application (Arcane Ward → Temp HP → HP)
- `apply_healing()`: Unified healing with temp HP/normal HP handling
- `pay_costs()`: Resource consumption with event emission
- `can_afford()`: Resource availability check
- `add_buff()`, `remove_buff()`: Buff lifecycle management
- `get_combatant()`, `get_combatant_mut()`: Combatant access

---

### Event System (`events.rs`)

**Event Types (30+):**

| Category | Events |
|----------|--------|
| **Combat** | `ActionStarted`, `ActionSkipped`, `AttackHit`, `AttackMissed`, `DamageTaken`, `DamagePrevented`, `HealingApplied`, `TempHPGranted` |
| **Spell** | `CastSpell`, `SpellCast`, `SpellSaved`, `SpellFailed`, `ConcentrationBroken`, `ConcentrationMaintained` |
| **Status** | `BuffApplied`, `BuffExpired`, `BuffRemoved`, `ConditionAdded`, `ConditionRemoved` |
| **Lifecycle** | `TurnStarted`, `TurnEnded`, `RoundStarted`, `RoundEnded`, `EncounterStarted`, `EncounterEnded`, `UnitDied` |
| **Roll** | `SaveAttempted`, `SaveResult`, `AbilityCheckMade` |
| **Resource** | `ResourceConsumed`, `ResourceRestored`, `ResourceDepleted` |

**EventBus:**
- Pending events queue for reaction triggering
- Event history (max 1000 when logging enabled)
- Event listeners per combatant for reactions

---

### Reaction System (`reactions.rs`)

Reactions allow combatants to respond to specific events:

```rust
pub struct ReactionManager {
    available_reactions: HashMap<String, Vec<ReactionTemplate>>,
    used_reactions: HashMap<String, HashSet<String>>,     // Per round
    encounter_used: HashMap<String, HashSet<String>>,     // Per encounter
    current_round: u32,
}
```

**Reaction Components:**
- **TriggerCondition**: OnHit, OnBeingAttacked, OnMiss, OnBeingDamaged, OnEnemyDeath, etc.
- **TriggerEffect**: DealDamage, GrantImmediateAction, InterruptAction, AddToRoll, ForceReroll, etc.
- **TriggerRequirement**: HasTempHP, etc.

**Priority System:** Higher priority reactions execute first, sorted by combatant ID for determinism.

---

### Two-Pass Simulation System

Memory-efficient simulation for 10,100 iterations:

#### Phase 1: Lightweight Survey Pass
- **Iterations**: 10,100
- **Memory**: ~323 KB (32 bytes per run: seed + score + deaths)
- **Time**: ~10 seconds
- **No event collection**

#### Phase 2: Seed Selection with 1% Bucket Granularity

**Three-Tier Classification:**

| Tier | Seeds | Description |
|------|-------|-------------|
| **A** | 11 | Global decile medians (P5, P15, P25, P35, P45, P50, P55, P65, P75, P85, P95) |
| **B** | 100 | Per-1% bucket medians (P0-1, P1-2, ..., P99-100) |
| **C** | 59 | Per-encounter extremes (P0, P50, P100 for each encounter) |

**Total**: ~170 interesting seeds

#### Phase 3: Deep Dive Re-simulation

| Tier | Events | Memory |
|------|--------|--------|
| **A** (11 seeds) | Full events | 2.2 MB |
| **B** (100 seeds) | Lean round summaries | 2 MB |
| **C** (59 seeds) | None (reuse Phase 1) | ~2 KB |

**Total Phase 3 Memory**: ~4.2 MB (vs ~15-20 MB for all full event logs)

---

### Action Execution Engine (`execution/engine.rs`)

```rust
pub struct ActionExecutionEngine {
    pub context: TurnContext,
    pub reaction_manager: ReactionManager,
    pub action_resolver: ActionResolver,
    pub initiative_order: Vec<String>,
}
```

**Key Methods:**
- `execute_encounter()`: Main combat loop (max 50 rounds, 200 turns)
- `execute_combatant_turn()`: Single combatant's actions
- `execute_action_with_reactions()`: Process action + trigger reactions
- `process_reaction_phase()`: Execute triggered reactions
- `select_actions_for_combatant()`: AI action selection with scoring
- `score_action()`: Action priority scoring based on combat situation

**Action Scoring (AI Prioritization):**
- Attacks: `dpr * targets * 10`
- Healing: `amount * injured_allies * 15`
- Buffs: Higher score in rounds 1-2 (50), then 20
- Debuffs: Score based on strong enemies count
- Templates: High priority, 100 in rounds 1-2, then 40

---

### Action Resolver (`action_resolver.rs`)

Converts high-level actions into events using modular resolvers:

**Template Resolution Cache (`action_cache.rs`):**
- Resolves template spells (Bless, Bane, Haste, Shield, Hunter's Mark, Hex, Hypnotic Pattern)
- Caches resolved actions by template name + overrides
- Max 1000 entries before clearing
- Cache stats: entry count + estimated bytes

**Resolvers:**
- `resolvers/attack.rs`: Attack resolution with d20 rolls, crits, target selection
- `resolvers/heal.rs`: Healing application
- `resolvers/buff.rs`: Buff application
- `resolvers/debuff.rs`: Debuff application
- `resolvers/template.rs`: Template action resolution

**Target Selection:**
- Enemy: "LeastHP", "MostHP", "HighestDPR", "LowestAC", "HighestSurvivability", "Random"
- Ally: "LeastHP", "Self"
- Multi-target with strategy (all enemies, all allies)

---

### Caching Layer (`action_cache.rs`)

Template cache for spell actions:
- **Thread-local** `RefCell<HashMap<TemplateCacheKey, ResolvedAction>>`
- **Cache key**: template name + sorted overrides (SaveDC, Amount, Target)
- **LRU eviction**: Max 1000 entries
- **Cache stats**: `get_cache_stats_wasm()` returns entry count + estimated bytes

---

### Auto-Balancing System (`auto_balancer.rs`, `encounter_balancer.rs`)

Auto-adjusts encounter difficulty based on party stats:

**Encounter Tier Classification:**
```rust
pub enum EncounterTier {
    Trivial = -1,  // CR < 1/8 party level, <10% drain
    Safe = 0,      // P99 <= 0, P50 = 0, P1 <= 1, 10-30% drain
    Challenging = 1, // P99 <= 1, P50 <= 1, P1 <= 2, 30-50% drain
    Boss = 2,      // P99 <= 2, P50 1-3, P1 <= 4, 50-80% drain
    Failed = 3,    // TPK occurred or out of acceptable bounds
}
```

**Contextual Difficulty Adjustment:**
```rust
pub struct ContextualEncounterMetrics {
    pub position_in_day: usize,
    pub resources_remaining_percent: f64,
    pub isolated_tier: EncounterTier,
    pub contextual_tier: EncounterTier,
}
```

Adjusts difficulty based on remaining resources:
- 85-100%: No adjustment
- 70-84%: +1 tier
- 40-69%: +2 tiers
- <40%: +3 tiers (everything becomes Failed)

---

## Frontend Architecture (React/TypeScript)

### Directory Structure

```
src/
├── model/                        # TypeScript types, validation, store
│   ├── model.ts                  # Core types (Creature, Action, Event)
│   ├── schemas.ts                # Zod validation schemas
│   ├── store.ts                  # Zustand state store (minimal use)
│   ├── useSimulationWorker.ts    # WebWorker orchestration
│   ├── useSkylineAnalysis.ts     # Skyline computation
│   ├── useSimulationSession.ts   # Timeline/player state
│   └── useAutoSimulation.ts      # Auto-trigger simulations
├── components/
│   ├── combat/                   # Combat event components
│   │   ├── eventLog.tsx          # Full combat log display
│   │   ├── combatReplayModal.tsx # Replay combat with detailed events
│   │   ├── descentGraph.tsx      # HP over time visualization
│   │   └── heartbeatGraph.tsx    # Damage rhythm visualization
│   ├── creatureForm/             # Creature editing components
│   │   ├── creatureForm.tsx      # Main form container
│   │   ├── playerForm.tsx        # Player-specific fields
│   │   ├── monsterForm.tsx       # Monster-specific fields
│   │   ├── customForm.tsx        # Custom creature fields
│   │   ├── actionForm.tsx        # Action definition
│   │   ├── actionCostEditor.tsx  # Action cost editing
│   │   ├── ActionRequirementEditor.tsx # Action requirement editing
│   │   ├── buffEditor.tsx        # Buff editing
│   │   ├── resourceEditor.tsx    # Resource editing
│   │   ├── importModal.tsx       # Import from 5etools
│   │   ├── importButton.tsx      # Import trigger button
│   │   ├── saveBonusModal.tsx    # Save bonus editing
│   │   ├── strategyBuilder.tsx   # Strategy building UI
│   │   ├── tagSelector.tsx       # Tag selection UI
│   │   └── loadCreatureForm.tsx  # Creature form loading
│   ├── simulation/               # Main simulation UI (70+ components)
│   │   ├── simulation.tsx        # Main container
│   │   ├── components/           # Sub-components directory
│   │   │   ├── simulationHeader.tsx  # Header controls
│   │   │   ├── playerFormSection.tsx # Player forms section
│   │   │   ├── timelineItem.tsx      # Single timeline event editor
│   │   │   ├── addTimelineButtons.tsx # Add combat/rest buttons
│   │   │   ├── overallSummary.tsx    # Overall statistics summary
│   │   │   ├── backendStatusPanel.tsx # Backend status display
│   │   │   └── simulationModals/     # Modal dialogs
│   │   ├── analysisComponents/   # Analysis visualizations
│   │   │   ├── assistantSummary.tsx
│   │   │   ├── encounterResult.tsx
│   │   │   └── eventLog.tsx
│   │   ├── skyline/              # Skyline visualizations
│   │   │   ├── hpSkyline.tsx
│   │   │   ├── resourceSkyline.tsx
│   │   │   ├── skylineHeatmap.tsx
│   │   │   ├── skylineSpectrogram.tsx
│   │   │   ├── skylineCanvas.tsx # Unified canvas rendering
│   │   │   ├── decileAnalysis.tsx
│   │   │   └── playerGraphs.tsx
│   │   ├── accessibility/        # Accessibility components
│   │   │   ├── accessibilityToggle.tsx
│   │   │   └── accessibilityContext.tsx
│   │   ├── hooks/                # Simulation-specific hooks
│   │   │   ├── useAutoSimulation.ts
│   │   │   └── useSimulationSession.ts
│   │   ├── actionEconomyDisplay.tsx   # Action economy visualization
│   │   ├── balancerBandOverlay.tsx    # Balancer overlay
│   │   ├── partyOverview.tsx          # Party status overview
│   │   ├── resourcePanel.tsx          # Resource display
│   │   ├── adventuringDayForm.tsx     # Day configuration
│   │   ├── battleCard.tsx             # Battle card display
│   │   ├── deltaBadge.tsx             # Delta display badge
│   │   ├── fuelGauge.tsx              # Fuel/resource gauge
│   │   ├── crosshairContext.tsx       # Crosshair state
│   │   ├── crosshairLine.tsx          # Crosshair visualization
│   │   ├── onboardingTour.tsx         # New user tour
│   │   ├── deathBar.tsx               # Death visualization
│   │   ├── adjustmentPreview.tsx      # Adjustment preview
│   │   ├── encounterForm.tsx          # Encounter form
│   │   ├── descentGraph.tsx           # HP descent graph
│   │   ├── heartbeatGraph.tsx         # Damage rhythm graph
│   │   ├── eventLog.tsx               # Event log
│   │   ├── assistantSummary.tsx       # AI summary
│   │   ├── encounterResult.tsx        # Encounter result
│   │   ├── decileAnalysis.tsx         # Decile analysis (root level)
│   │   └── combatReplayModal.tsx      # Combat replay modal
│   └── utils/                    # Reusable components
│       ├── loadingSpinner.tsx
│       ├── loadingOverlay.tsx
│       ├── loadingSkeleton.tsx
│       ├── modal.tsx
│       ├── select.tsx
│       ├── toggle.tsx
│       ├── rangeInput.tsx
│       ├── progressUI.tsx         # Progress display UI
│       ├── progressVisualizer.tsx # Progress visualization
│       ├── checkbox.tsx
│       ├── decimalInput.tsx
│       ├── diceFormulaInput.tsx
│       ├── rgpd.tsx
│       ├── footer.tsx
│       ├── logo.tsx
│       ├── sortTable.tsx
│       ├── uiTogglePanel.tsx
│       └── loadingExample.tsx
├── hooks/                        # Global custom hooks
│   └── useCombatPlayback.ts      # Combat replay state
├── pages/                        # Next.js pages
│   └── index.tsx                 # Main page
└── worker/                       # WebWorker implementation
    ├── simulation.worker.ts      # Worker entry point
    └── simulation.worker.controller.ts # Worker controller
```

---

### State Management

**Custom Hooks (Primary):**
- `useSimulationWorker`: WebWorker orchestration, simulation state
- `useSimulationSession`: Timeline and player state management
- `useAutoSimulation`: Auto-trigger on edits (debounced 500ms)
- `useSkylineAnalysis`: Skyline computation for visualizations
- `useCombatPlayback`: Combat replay state

**Context Providers:**
- `UIToggleProvider`: UI state (modals, tooltips)
- `semiPersistentContext`: Semi-persistent state (survives hot reload)

**Minimal Zustand Store (`model/store.ts`):**
- Used sparingly for specific global state

---

### WebWorker Communication

**Worker Message Types:**
```typescript
// Request
WorkerMessage: {
    type: 'START_SIMULATION' | 'AUTO_ADJUST_ENCOUNTER' | 'CANCEL_SIMULATION'
    players: Creature[]
    timeline: TimelineEvent[]
    monsters: Creature[]
    encounterIndex: number
    genId: number
    maxK?: number
    seed?: number
}

// Response
WorkerResponse: {
    type: 'SIMULATION_UPDATE' | 'AUTO_ADJUST_COMPLETE' | 'SIMULATION_CANCELLED' | 'SIMULATION_ERROR'
    genId: number
    results?: SimulationResult[]
    analysis?: FullAnalysisOutput
    events?: SimulationEvent[]
    kFactor?: number
    isFinal?: boolean
    result?: AutoAdjustmentResult
    error?: string
}
```

---

### Key Components

#### Simulation (`components/simulation/simulation.tsx`)
Main container orchestrating all simulation UI:
- `SimulationHeader`: Run controls, precision toggle
- `BackendStatusPanel`: Backend status display
- `PlayerFormSection`: Player forms section
- `TimelineItem`: Single timeline event editor
- `AddTimelineButtons`: Add combat/rest buttons
- `OverallSummary`: Overall statistics summary
- `SimulationModals`: Modal dialogs
- `OnboardingTour`: New user tour
- `PerformanceDashboard`: Debugging dashboard

#### Skyline Visualizations
- `SkylineCanvas`: Unified canvas rendering for all skyline visualizations
- `HPSkyline`: HP per run across iterations
- `ResourceSkyline`: Resource usage over time
- `SkylineHeatmap`: Heatmap visualization
- `SkylineSpectrogram`: Spectrogram analysis
- `DecileAnalysis`: Per-decile statistics
- `PlayerGraphs`: Player-specific graphs

#### Creature Forms
- `CreatureForm`: Main form container
- `PlayerForm`: Player-specific fields
- `MonsterForm`: Monster-specific fields
- `ActionForm`: Action definition
- `ActionCostEditor`: Action cost editing
- `ActionRequirementEditor`: Action requirement editing
- `BuffEditor`: Buff editing
- `ResourceEditor`: Resource editing
- `ImportModal`: Import from 5etools
- `SaveBonusModal`: Save bonus editing

---

## Data Flow & Integration

### Simulation Request Flow

1. **User edits players/monsters** → `useAutoSimulation` detects changes
2. **Debounced trigger (500ms)** → `useSimulationWorker.runSimulation()`
3. **WebWorker initializes WASM** → loads `simulation_wasm_bg.wasm`
4. **Worker calls `run_simulation_wasm()`** with players/timeline
5. **WASM runs two-pass simulation** → returns results to worker
6. **Worker streams updates** via `postMessage()`
7. **Frontend updates UI** with results/analysis

See **DATA_FLOW.md** for detailed flow diagrams.

---

## Design Patterns & Conventions

### Backend Patterns
- **Event-Driven**: All state changes emit events to EventBus
- **Resolver Pattern**: Actions delegated to modular resolvers
- **Strategy Pattern**: Different resolution strategies per action type
- **Caching**: Template resolution and combat stats caching
- **Two-Pass**: Lightweight survey + deep dive analysis

### Frontend Patterns
- **Custom Hooks**: Encapsulate worker and state logic
- **Context Providers**: UI toggles and semi-persistent state
- **Component Composition**: 70+ reusable simulation components
- **Debouncing**: Change detection with 500ms delay
- **WebWorker Isolation**: Heavy computations off-main-thread

---

## Key Integration Points

| Integration | Files |
|-------------|-------|
| **WASM → Frontend** | `wasm_api.rs` ↔ `simulation.worker.controller.ts` |
| **Action Resolution** | `action_resolver.rs` ↔ `resolvers/*.rs` |
| **Event Generation** | `context.rs` ↔ `events.rs` |
| **Reaction System** | `reactions.rs` ↔ `events.rs` |
| **Simulation → UI** | `useSimulationWorker.ts` ↔ `simulation.tsx` |
| **Auto-Balance** | `auto_balancer.rs` ↔ `balancer.rs` |
| **Skyline Analysis** | `analysis/statistics.rs` ↔ `useSkylineAnalysis.ts` |

---

## Performance Considerations

### Backend
- **Two-Pass Simulation**: Reduces memory from ~15-20 MB to ~4.2 MB
- **Template Cache**: Avoids repeated resolution (1000 entries LRU)
- **Combat Stats Cache**: Caches combatant statistics for performance
- **Memory Guardrails**: Auto-switches to lightweight mode when iterations > 10,000

### Frontend
- **WebWorker Isolation**: Heavy computations off-main-thread
- **Debouncing**: 500ms delay prevents excessive simulations
- **Canvas Rendering**: Optimized with `requestAnimationFrame`
- **Virtual Scrolling**: For long lists (React Virtuoso)

---

## Testing Strategy

### Backend
- **Property-Based Testing**: `proptest` for core simulation logic
- **Unit Tests**: Per-module tests
- **Integration Tests**: End-to-end simulation tests

### Frontend
- **Vitest**: Unit tests for hooks and components
- **E2E Tests**: `SkylineE2E.ts` for skyline components
- **jsdom**: DOM simulation for component tests

---

## Development Workflow

1. **Backend Changes**: Edit `simulation-wasm/src/`, run `wasm-pack build --dev`
2. **Frontend Changes**: Edit `src/`, run `npm run dev`
3. **Testing**: `npm test` for frontend, `cargo test` for backend
4. **Build**: `npm run build` for production

---

## References

- **BACKEND_API.md**: Function catalog for Rust/WASM backend
- **FRONTEND_API.md**: Component catalog for React/TypeScript frontend
- **DATA_FLOW.md**: Request lifecycles and state transitions
- **AGENTS.md**: Protocols and guidelines for LLM agents
