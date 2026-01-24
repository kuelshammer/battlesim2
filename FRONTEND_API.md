# Frontend API Reference (React/TypeScript)

**Purpose**: Component and hook catalog for planning LLMs - understand what exists without reading implementations.

---

## Table of Contents

1. [Custom Hooks](#custom-hooks)
2. [Simulation Components](#simulation-components)
3. [Creature Form Components](#creature-form-components)
4. [Combat Event Components](#combat-event-components)
5. [Skyline Visualization Components](#skyline-visualization-components)
6. [Analysis Components](#analysis-components)
7. [UI Components (Radix UI)](#ui-components-radix-ui)
8. [Model Layer](#model-layer)
9. [Worker Communication](#worker-communication)
10. [Data Import](#data-import)
11. [When to Modify What](#when-to-modify-what)

---

## Custom Hooks

### Global Hooks

#### `useCombatPlayback` (`hooks/useCombatPlayback.ts`)

**Purpose**: Manage combat replay state

**State**:
```typescript
{
    isPlaying: boolean
    currentRound: number
    currentTurn: number
    speed: number
    events: SimulationEvent[]
    onEventClick?: (event: SimulationEvent) => void
}
```

**Actions**:
- `play()`: Start playback
- `pause()`: Pause playback
- `next()`: Next event
- `prev()`: Previous event
- `seek(round, turn)`: Seek to position
- `setSpeed(speed)`: Set playback speed

---

### Simulation Hooks

#### `useSimulationWorker` (`model/useSimulationWorker.ts`)

**Purpose**: Manage WebWorker communication and simulation state

**State**:
```typescript
{
    isRunning: boolean
    progress: number
    kFactor: number
    maxK: number
    results: SimulationResult[] | null
    analysis: FullAnalysisOutput | null
    events: SimulationEvent[] | null
    error: string | null
    optimizedResult: AutoAdjustmentResult | null
    genId: number
    isCancelled: boolean
}
```

**Actions**:
- `runSimulation(players, timeline, maxK, seed)`: Start simulation
- `autoAdjustEncounter(players, monsters, timeline, encounterIndex)`: Auto-balance
- `cancel()`: Cancel current simulation

**File**: `src/model/useSimulationWorker.ts`

---

#### `useSimulationSession` (`components/simulation/hooks/useSimulationSession.ts`)

**Purpose**: Manage timeline and player state

**State**:
```typescript
{
    players: Creature[]
    timeline: TimelineEvent[]
    hasChanges: boolean
}
```

**Actions**:
- `setPlayers(players)`: Set players
- `setTimeline(timeline)`: Set timeline
- `createCombat(monsters, playersSurprised)`: Create encounter
- `createShortRest(duration)`: Create short rest
- `updateTimelineItem(index, item)`: Update timeline item
- `deleteTimelineItem(index)`: Delete timeline item
- `swapTimelineItems(index1, index2)`: Swap timeline items
- `saveState()`: Save to localStorage
- `loadState()`: Load from localStorage

**File**: `src/components/simulation/hooks/useSimulationSession.ts`

---

#### `useAutoSimulation` (`components/simulation/hooks/useAutoSimulation.ts`)

**Purpose**: Auto-trigger simulations on edits (debounced 500ms)

**State**:
```typescript
{
    simulationResults: EncounterResultType[]
    simulationEvents: SimulationEvent[]
    needsResimulation: boolean
    isStale: boolean
    highPrecision: boolean
    isHighPrecisionLoaded: boolean
    isEditing: boolean
    canSave: boolean
}
```

**Actions**:
- `triggerResimulation()`: Trigger re-simulation
- `setHighPrecision(value)`: Set precision mode (3 vs 51 iterations)
- `setIsEditing(value)`: Set editing state
- `saveScenario()`: Save scenario to localStorage

**File**: `src/components/simulation/hooks/useAutoSimulation.ts`

---

#### `useSkylineAnalysis` (`model/useSkylineAnalysis.ts`)

**Purpose**: Compute skyline analysis for visualizations

**Analysis Types**:
- **Percentile Buckets**: HP/resource percentages per run
- **Vitals Index**: Lethality, TPK risk, volatility, doom horizon
- **Day Pacing**: Rhythm, attrition, recovery scores

**State**:
```typescript
{
    skyline: SkylineAnalysis | null
    vitals: Vitals | null
    dayPacing: DayPacing | null
    loading: boolean
    error: string | null
}
```

**Actions**:
- `analyze(results, partySize, encounterIndex)`: Run analysis
- `clear()`: Clear analysis

**File**: `src/model/useSkylineAnalysis.ts`

---

## Simulation Components

### Main Components

#### `Simulation` (`components/simulation/simulation.tsx`)

**Purpose**: Main container orchestrating all simulation UI

**Props**:
```typescript
{
    scenarioName?: string
}
```

**Structure**:
```tsx
<UIToggleProvider>
  <semiPersistentContext.Provider>
    <SimulationHeader />
    <BackendStatusPanel />
    <PlayerFormSection />
    <CrosshairProvider>
      {timeline.map(item => <TimelineItem />)}
      <AddTimelineButtons />
      <OverallSummary />
      <CrosshairTooltip />
    </CrosshairProvider>
    <SimulationModals />
    <OnboardingTour />
    <PerformanceDashboard />
  </semiPersistentContext.Provider>
</UIToggleProvider>
```

**File**: `src/components/simulation/simulation.tsx`

---

#### `SimulationHeader` (`components/simulation/simulationHeader.tsx`)

**Purpose**: Header controls for simulation

**Props**:
```typescript
{
    highPrecision: boolean
    onTogglePrecision: () => void
    onRun: () => void
    onStop: () => void
    isRunning: boolean
}
```

**Features**:
- Run/Stop buttons
- Precision toggle (Low/High)
- Scenario name display
- Backend status indicator

**File**: `src/components/simulation/simulationHeader.tsx`

---

#### `PlayerFormSection` (`components/simulation/playerFormSection.tsx`)

**Purpose**: Player forms section

**Props**:
```typescript
{
    players: Creature[]
    onPlayersChange: (players: Creature[]) => void
    canAdd: boolean
}
```

**Features**:
- Add/remove players
- Player form cards
- Total level display

**File**: `src/components/simulation/playerFormSection.tsx`

---

#### `TimelineItem` (`components/simulation/timelineItem.tsx`)

**Purpose**: Single timeline event editor

**Props**:
```typescript
{
    item: TimelineEvent
    index: number
    onUpdate: (index: number, item: TimelineEvent) => void
    onDelete: (index: number) => void
    onSwap: (index1: number, index2: number) => void
    canMoveUp: boolean
    canMoveDown: boolean
}
```

**Features**:
- Encounter/ShortRest editing
- Move up/down buttons
- Delete button
- Drag-and-drop reordering

**File**: `src/components/simulation/timelineItem.tsx`

---

#### `AddTimelineButtons` (`components/simulation/addTimelineButtons.tsx`)

**Purpose**: Add combat/rest buttons

**Props**:
```typescript
{
    onAddCombat: () => void
    onAddRest: () => void
}
```

**File**: `src/components/simulation/addTimelineButtons.tsx`

---

#### `OverallSummary` (`components/simulation/overallSummary.tsx`)

**Purpose**: Overall statistics summary

**Props**:
```typescript
{
    results: SimulationResult[]
    analysis: FullAnalysisOutput
    events: SimulationEvent[]
}
```

**Features**:
- Win rate display
- TPK rate display
- Average deaths per encounter
- Day pacing visualization

**File**: `src/components/simulation/overallSummary.tsx`

---

#### `BackendStatusPanel` (`components/simulation/backendStatusPanel.tsx`)

**Purpose**: Backend status display

**Props**:
```typescript
{
    status: 'idle' | 'running' | 'complete' | 'error'
    progress?: number
    error?: string
    cacheStats?: { entries: number, bytes: number }
}
```

**File**: `src/components/simulation/backendStatusPanel.tsx`

---

### Modals

#### `SimulationModals` (`components/simulation/simulationModals/index.tsx`)

**Purpose**: Container for all simulation modals

**Modals**:
- `CombatReplayModal`: Combat event replay
- `EncounterSetupModal`: Encounter configuration
- `ImportModal`: Import from 5etools
- `PerformanceDashboard`: Performance metrics

**File**: `src/components/simulation/simulationModals/index.tsx`

---

#### `CombatReplayModal` (`components/combat/combatReplayModal.tsx`)

**Purpose**: Replay combat with detailed events

**Props**:
```typescript
{
    events: SimulationEvent[]
    isOpen: boolean
    onClose: () => void
}
```

**Features**:
- Event-by-event playback
- Play/pause/skip controls
- Event filtering
- Turn/round navigation

**File**: `src/components/combat/combatReplayModal.tsx`

---

## Creature Form Components

### Main Forms

#### `CreatureForm` (`components/creatureForm/creatureForm.tsx`)

**Purpose**: Main creature form container

**Props**:
```typescript
{
    creature: Creature
    onChange: (creature: Creature) => void
    onDelete?: () => void
    type: 'player' | 'monster' | 'custom'
}
```

**Features**:
- Name, HP, AC editing
- Save bonus editing
- Action list management
- Resource management
- Buff/debuff editing

**File**: `src/components/creatureForm/creatureForm.tsx`

---

#### `PlayerForm` (`components/creatureForm/playerForm.tsx`)

**Purpose**: Player-specific fields

**Props**:
```typescript
{
    player: Creature
    onChange: (player: Creature) => void
}
```

**Features**:
- Level input
- Class selection (Fighter, Wizard, etc.)
- Spell slots per level
- Class resources (Ki, Sorcery Points, etc.)
- Hit dice
- Con modifier
- Magic items

**File**: `src/components/creatureForm/playerForm.tsx`

---

#### `MonsterForm` (`components/creatureForm/monsterForm.tsx`)

**Purpose**: Monster-specific fields

**Props**:
```typescript
{
    monster: Creature
    onChange: (monster: Creature) => void
}
```

**Features**:
- Count input (supports decimals like 3.5 for goblins)
- CR display
- Type display
- Import from 5etools button

**File**: `src/components/creatureForm/monsterForm.tsx`

---

#### `CustomForm` (`components/creatureForm/customForm.tsx`)

**Purpose**: Custom creature fields

**Props**:
```typescript
{
    creature: Creature
    onChange: (creature: Creature) => void
}
```

**File**: `src/components/creatureForm/customForm.tsx`

---

### Action Forms

#### `ActionForm` (`components/creatureForm/actionForm.tsx`)

**Purpose**: Action definition form

**Props**:
```typescript
{
    action: Action
    onChange: (action: Action) => void
    onDelete?: () => void
    creature: Creature
}
```

**Features**:
- Action type selection (Attack, Heal, Buff, Debuff, Template)
- Target selection
- Cost editing
- Requirement editing
- Effect configuration

**File**: `src/components/creatureForm/actionForm.tsx`

---

#### `ActionCostEditor` (`components/creatureForm/actionCostEditor.tsx`)

**Purpose**: Action cost editing

**Props**:
```typescript
{
    costs: ActionCost[]
    onChange: (costs: ActionCost[]) => void
}
```

**Features**:
- Discrete cost (action, bonus action)
- Variable cost (movement, spell slot)
- Resource selection

**File**: `src/components/creatureForm/actionCostEditor.tsx`

---

#### `ActionRequirementEditor` (`components/creatureForm/actionRequirementEditor.tsx`)

**Purpose**: Action requirement editing

**Props**:
```typescript
{
    requirements: ActionRequirement[]
    onChange: (requirements: ActionRequirement[]) => void
}
```

**Requirement Types**:
- `ResourceAvailable`: Spell slot, ki points, etc.
- `CombatState`: Bloodied, round number, etc.
- `StatusEffect`: Has condition
- `Custom`: Arbitrary predicate

**File**: `src/components/creatureForm/actionRequirementEditor.tsx`

---

### Buff/Resource Editors

#### `BuffEditor` (`components/creatureForm/buffEditor.tsx`)

**Purpose**: Buff editing

**Props**:
```typescript
{
    buffs: Buff[]
    onChange: (buffs: Buff[]) => void
}
```

**Features**:
- Add/remove buffs
- Stat bonuses (AC, saves, etc.)
- Duration editing
- Condition editing

**File**: `src/components/creatureForm/buffEditor.tsx`

---

#### `ResourceEditor` (`components/creatureForm/resourceEditor.tsx`)

**Purpose**: Resource editing

**Props**:
```typescript
{
    resources: HashMap<string, number>
    onChange: (resources: HashMap<string, number>) => void
    type: 'spell_slots' | 'class_resources'
}
```

**File**: `src/components/creatureForm/resourceEditor.tsx`

---

### Import Components

#### `ImportModal` (`components/creatureForm/importModal.tsx`)

**Purpose**: Import creature from 5etools

**Props**:
```typescript
{
    isOpen: boolean
    onClose: () => void
    onImport: (creature: Creature) => void
}
```

**Features**:
- Search 5etools database
- Preview creature stats
- Auto-convert to Creature format

**File**: `src/components/creatureForm/importModal.tsx`

---

#### `ImportButton` (`components/creatureForm/importButton.tsx`)

**Purpose**: Trigger import modal

**Props**:
```typescript
{
    onImport: (creature: Creature) => void
    type: 'monster' | 'player'
}
```

**File**: `src/components/creatureForm/importButton.tsx`

---

#### `SaveBonusModal` (`components/creatureForm/saveBonusModal.tsx`)

**Purpose**: Save bonus editing modal

**Props**:
```typescript
{
    isOpen: boolean
    onClose: () => void
    creature: Creature
    onChange: (creature: Creature) => void
}
```

**File**: `src/components/creatureForm/saveBonusModal.tsx`

---

## Combat Event Components

#### `EventLog` (`components/combat/eventLog.tsx`)

**Purpose**: Full combat log display

**Props**:
```typescript
{
    events: SimulationEvent[]
    onEventClick?: (event: SimulationEvent) => void
    filter?: EventFilter
}
```

**Features**:
- Chronological event list
- Event type filtering
- Event highlighting
- Copy to clipboard

**File**: `src/components/combat/eventLog.tsx`

---

#### `DescentGraph` (`components/combat/descentGraph.tsx`)

**Purpose**: HP over time visualization

**Props**:
```typescript
{
    events: SimulationEvent[]
    combatants: string[]
}
```

**File**: `src/components/combat/descentGraph.tsx`

---

#### `HeartbeatGraph` (`components/combat/heartbeatGraph.tsx`)

**Purpose**: Damage rhythm visualization

**Props**:
```typescript
{
    events: SimulationEvent[]
}
```

**File**: `src/components/combat/heartbeatGraph.tsx`

---

## Skyline Visualization Components

#### `SkylineCanvas` (`components/simulation/skyline/skylineCanvas.tsx`)

**Purpose**: Unified canvas rendering for all skyline visualizations

**Props**:
```typescript
{
    data: SkylineData
    mode: 'hp' | 'resource' | 'heatmap' | 'spectrogram'
    width: number
    height: number
    options?: CanvasOptions
}
```

**Modes**:
- `hp`: HP per run across iterations
- `resource`: Resource usage over time
- `heatmap`: Heatmap visualization
- `spectrogram`: Spectrogram analysis

**File**: `src/components/simulation/skyline/skylineCanvas.tsx`

---

#### `HPSkyline` (`components/simulation/skyline/hpSkyline.tsx`)

**Purpose**: HP per run visualization

**Props**:
```typescript
{
    results: SimulationResult[]
    partySize: number
}
```

**File**: `src/components/simulation/skyline/hpSkyline.tsx`

---

#### `ResourceSkyline` (`components/simulation/skyline/resourceSkyline.tsx`)

**Purpose**: Resource usage visualization

**Props**:
```typescript
{
    results: SimulationResult[]
    partySize: number
    resourceType: string
}
```

**File**: `src/components/simulation/skyline/resourceSkyline.tsx`

---

#### `SkylineHeatmap` (`components/simulation/skyline/skylineHeatmap.tsx`)

**Purpose**: Heatmap visualization

**Props**:
```typescript
{
    data: HeatmapData
    options?: HeatmapOptions
}
```

**File**: `src/components/simulation/skyline/skylineHeatmap.tsx`

---

#### `SkylineSpectrogram` (`components/simulation/skyline/skylineSpectrogram.tsx`)

**Purpose**: Spectrogram analysis visualization

**Props**:
```typescript
{
    events: SimulationEvent[]
    options?: SpectrogramOptions
}
```

**File**: `src/components/simulation/skyline/skylineSpectrogram.tsx`

---

#### `DecileAnalysis` (`components/simulation/skyline/decileAnalysis.tsx`)

**Purpose**: Per-decile statistics visualization

**Props**:
```typescript
{
    deciles: DecileStats[]
}
```

**File**: `src/components/simulation/skyline/decileAnalysis.tsx`

---

#### `PlayerGraphs` (`components/simulation/skyline/playerGraphs.tsx`)

**Purpose**: Player-specific graphs

**Props**:
```typescript
{
    results: SimulationResult[]
    playerId: string
}
```

**File**: `src/components/simulation/skyline/playerGraphs.tsx`

---

## Analysis Components

#### `AssistantSummary` (`components/simulation/analysisComponents/assistantSummary.tsx`)

**Purpose**: AI-generated encounter summary

**Props**:
```typescript
{
    output: AggregateOutput
}
```

**File**: `src/components/simulation/analysisComponents/assistantSummary.tsx`

---

#### `EncounterResult` (`components/simulation/analysisComponents/encounterResult.tsx`)

**Purpose**: Single encounter result display

**Props**:
```typescript
{
    result: EncounterResultType
    index: number
    onReplay?: () => void
}
```

**File**: `src/components/simulation/analysisComponents/encounterResult.tsx`

---

#### `EventLog` (simulation) (`components/simulation/analysisComponents/eventLog.tsx`)

**Purpose**: Combat event log for simulation results

**Props**:
```typescript
{
    events: SimulationEvent[]
    onEventClick?: (event: SimulationEvent) => void
}
```

**File**: `src/components/simulation/analysisComponents/eventLog.tsx`

---

## UI Components (Radix UI)

### Modals

#### `Modal` (`components/utils/modal.tsx`)

**Purpose**: Reusable modal wrapper (Radix UI Dialog)

**Props**:
```typescript
{
    isOpen: boolean
    onClose: () => void
    title: string
    children: React.ReactNode
}
```

**File**: `src/components/utils/modal.tsx`

---

### Inputs

#### `Select` (`components/utils/select.tsx`)

**Purpose**: Dropdown select (Radix UI Select)

**Props**:
```typescript
{
    value: string
    onChange: (value: string) => void
    options: { value: string, label: string }[]
    placeholder?: string
}
```

**File**: `src/components/utils/select.tsx`

---

#### `Toggle` (`components/utils/toggle.tsx`)

**Purpose**: Toggle switch

**Props**:
```typescript
{
    checked: boolean
    onChange: (checked: boolean) => void
    label?: string
}
```

**File**: `src/components/utils/toggle.tsx`

---

#### `RangeInput` (`components/utils/rangeInput.tsx`)

**Purpose**: Range slider input

**Props**:
```typescript
{
    value: number
    onChange: (value: number) => void
    min: number
    max: number
    step?: number
}
```

**File**: `src/components/utils/rangeInput.tsx`

---

### Loading States

#### `LoadingSpinner` (`components/utils/loadingSpinner.tsx`)

**Purpose**: Animated loading spinner

**Props**:
```typescript
{
    size?: 'sm' | 'md' | 'lg'
}
```

**File**: `src/components/utils/loadingSpinner.tsx`

---

#### `LoadingOverlay` (`components/utils/loadingOverlay.tsx`)

**Purpose**: Full-screen loading overlay

**Props**:
```typescript
{
    isLoading: boolean
    message?: string
}
```

**File**: `src/components/utils/loadingOverlay.tsx`

---

#### `LoadingSkeleton` (`components/utils/loadingSkeleton.tsx`)

**Purpose**: Skeleton placeholder

**Props**:
```typescript
{
    width?: string
    height?: string
    variant?: 'text' | 'rect' | 'circle'
}
```

**File**: `src/components/utils/loadingSkeleton.tsx`

---

## Model Layer

### Types (`model/model.ts`)

**Key Types**:
```typescript
// Creature definition
type Creature = {
    id: string
    name: string
    count: number
    hp: number
    ac: number
    save_bonus: number
    str_save_bonus?: number
    dex_save_bonus?: number
    con_save_bonus?: number
    int_save_bonus?: number
    wis_save_bonus?: number
    cha_save_bonus?: number
    actions: Action[]
    triggers: ActionTrigger[]
    spell_slots?: Record<string, number>
    class_resources?: Record<string, number>
    hit_dice?: string
    con_modifier?: number
    magic_items: string[]
    max_arcane_ward_hp?: number
    initial_buffs: Buff[]
}

// Runtime combatant
type Combattant = {
    id: string
    side: 'Hero' | 'Monster'
    creature_index: number
    final_state: CombattantState
    actions: Action[]
}

// Action enum
type Action =
    | { type: 'Atk', data: AtkAction }
    | { type: 'Heal', data: HealAction }
    | { type: 'Buff', data: BuffAction }
    | { type: 'Debuff', data: DebuffAction }
    | { type: 'Template', data: TemplateAction }

// Timeline events
type TimelineEvent =
    | { type: 'Encounter', data: Encounter }
    | { type: 'ShortRest', data: ShortRest }

// Simulation events (30+ types)
type SimulationEvent =
    | { type: 'ActionStarted', ... }
    | { type: 'AttackHit', ... }
    | { type: 'DamageTaken', ... }
    | { type: 'UnitDied', ... }
    // ... 30+ variants
```

**File**: `src/model/model.ts`

---

### Schemas (`model/schemas.ts`)

**Purpose**: Zod validation schemas for runtime validation

**Schemas**:
- `CreatureSchema`: Creature validation
- `ActionSchema`: Action validation
- `TimelineEventSchema`: Timeline event validation
- `SimulationEventSchema`: Simulation event validation

**File**: `src/model/schemas.ts`

---

### Store (`model/store.ts`)

**Purpose**: Zustand state store (minimal use)

**State**:
```typescript
{
    // Add global state here if needed
}
```

**File**: `src/model/store.ts`

---

## Worker Communication

### Worker Types (`worker/simulation.worker.ts`)

**Worker Message**:
```typescript
type WorkerMessage =
    | { type: 'START_SIMULATION', players: Creature[], timeline: TimelineEvent[], maxK?: number, seed?: number, genId: number }
    | { type: 'AUTO_ADJUST_ENCOUNTER', players: Creature[], monsters: Creature[], timeline: TimelineEvent[], encounterIndex: number, genId: number }
    | { type: 'CANCEL_SIMULATION', genId: number }
```

**Worker Response**:
```typescript
type WorkerResponse =
    | { type: 'SIMULATION_UPDATE', genId: number, kFactor: number, results?: SimulationResult[], analysis?: FullAnalysisOutput }
    | { type: 'SIMULATION_COMPLETE', genId: number, results: SimulationResult[], analysis: FullAnalysisOutput, events: SimulationEvent[] }
    | { type: 'AUTO_ADJUST_COMPLETE', genId: number, result: AutoAdjustmentResult }
    | { type: 'SIMULATION_CANCELLED', genId: number }
    | { type: 'SIMULATION_ERROR', genId: number, error: string }
```

**File**: `src/worker/simulation.worker.ts`

---

### Worker Controller (`worker/simulation.worker.controller.ts`)

**Purpose**: Manages WebWorker lifecycle and message handling

**Class**: `SimulationWorkerController`

**Methods**:
- `initialize()`: Initialize WASM module
- `runSimulation(params)`: Run simulation
- `autoAdjustEncounter(params)`: Auto-balance encounter
- `cancel(genId)`: Cancel simulation
- `onMessage(callback)`: Register message handler
- `terminate()`: Terminate worker

**File**: `src/worker/simulation.worker.controller.ts`

---

## Data Import

### 5etools Mapper

**Purpose**: Map 5etools JSON to Creature format

**File**: Not specified in exploration (likely in `components/creatureForm/` or `model/`)

---

### Static Data

**Purpose**: Pre-defined creatures, monsters, spells

**File**: Not specified in exploration (likely in `public/` or `model/`)

---

## When to Modify What

### Adding a New Component

1. **Determine component category**:
   - Simulation: `components/simulation/`
   - Creature Form: `components/creatureForm/`
   - Combat: `components/combat/`
   - Utils: `components/utils/`

2. **Create component file**:
   - Use TypeScript
   - Use Radix UI primitives for UI elements
   - Follow naming convention: `camelCase.tsx`

3. **Add to index** (if directory has index.tsx):
   - Export component

### Adding a New Hook

1. **Determine hook scope**:
   - Global: `hooks/`
   - Simulation: `components/simulation/hooks/`

2. **Create hook file**:
   - Use `use` prefix
   - Follow naming convention: `useHookName.ts`

### Modifying Worker Communication

1. **Add message type** (`worker/simulation.worker.ts`):
   - Add to `WorkerMessage` union
   - Add to `WorkerResponse` union

2. **Update worker controller** (`worker/simulation.worker.controller.ts`):
   - Add message handler

3. **Update useSimulationWorker** (`model/useSimulationWorker.ts`):
   - Add action method

### Adding a New Visualization

1. **Create component** (`components/simulation/skyline/`):
   - Extend `SkylineCanvas` if possible
   - Use canvas rendering for performance

2. **Add analysis hook** (`model/useSkylineAnalysis.ts`):
   - Add analysis function

### Modifying State Management

1. **Session state** (`components/simulation/hooks/useSimulationSession.ts`):
   - Timeline, player state

2. **Worker state** (`model/useSimulationWorker.ts`):
   - Simulation state, results

3. **Auto-simulation** (`components/simulation/hooks/useAutoSimulation.ts`):
   - Auto-trigger logic

### Modifying Data Import

1. **5etools mapper** (likely in `components/creatureForm/`):
   - Add mapping for new creature types

2. **Static data** (likely in `public/` or `model/`):
   - Add pre-defined creatures

---

## References

- **ARCHITECTURE.md**: Comprehensive system architecture
- **BACKEND_API.md**: Rust/WASM function catalog
- **DATA_FLOW.md**: Request lifecycles and state transitions
- **AGENTS.md**: Protocols and guidelines for LLM agents
