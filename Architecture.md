# Architecture: UI Cleanup and Toggle Management System

## Overview

This architecture implements a comprehensive UI cleanup strategy for Battlesim2, focusing on improving user experience through controlled visibility of complex UI elements. The solution introduces a toggle-based visibility system that maintains existing functionality while providing users with intuitive controls over what information they want to see.

## Current UI Analysis

### Existing Components

1. **Combat Log (EventLog.tsx)**: Always visible, displays all simulation events
2. **HP Bars (EncounterResult.tsx)**: Always visible, shows round-by-round HP bars for combatants
3. **Quintile Analysis (QuintileAnalysis.tsx)**: Always visible, displays all 5 quintile timelines

### User Requirements

1. **Combat Log**: Hide by default, add toggle to show/hide
2. **HP Bars**: Remove round-to-round HP bars, add toggle to show/hide
3. **Quintile Analysis**: Only show quintile 5 at startup, hide other quintiles initially

## System Design

### Core Architecture Principles

```
┌─────────────────────────────────────────────────────────────┐
│                   UI Toggle Management System              │
├─────────────────────────────────────────────────────────────┤
│ 1. Centralized State Management                           │
│    - React Context for toggle states                       │
│    - TypeScript interfaces for type safety                │
│    - Integration with existing Loading State System        │
├─────────────────────────────────────────────────────────────┤
│ 2. Component Visibility Control                           │
│    - Conditional rendering based on toggle states          │
│    - Smooth transitions and animations                    │
│    - Accessibility considerations                        │
├─────────────────────────────────────────────────────────────┤
│ 3. User Control Interface                                │
│    - Toggle buttons with clear labels                     │
│    - Visual indicators of current state                   │
│    - Keyboard accessibility                              │
└─────────────────────────────────────────────────────────────┘
```

### State Management Architecture

#### Centralized Toggle State

```typescript
// src/model/uiToggleState.ts
import { createContext, useContext, useState, ReactNode } from 'react'

interface UIToggleState {
  combatLogVisible: boolean
  hpBarsVisible: boolean
  quintileAnalysisVisible: {
    quintile1: boolean
    quintile2: boolean
    quintile3: boolean
    quintile4: boolean
    quintile5: boolean
  }
}

interface UIToggleContextType {
  state: UIToggleState
  toggleCombatLog: () => void
  toggleHpBars: () => void
  toggleQuintile: (quintile: number) => void
  setQuintileVisibility: (quintile: number, visible: boolean) => void
}

const UIToggleContext = createContext<UIToggleContextType | undefined>(undefined)

export const UIToggleProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [state, setState] = useState<UIToggleState>({
    combatLogVisible: false,
    hpBarsVisible: false,
    quintileAnalysisVisible: {
      quintile1: false,
      quintile2: false,
      quintile3: false,
      quintile4: false,
      quintile5: true // Only quintile 5 visible by default
    }
  })

  const toggleCombatLog = () => {
    setState(prev => ({ ...prev, combatLogVisible: !prev.combatLogVisible }))
  }

  const toggleHpBars = () => {
    setState(prev => ({ ...prev, hpBarsVisible: !prev.hpBarsVisible }))
  }

  const toggleQuintile = (quintile: number) => {
    setState(prev => ({
      ...prev,
      quintileAnalysisVisible: {
        ...prev.quintileAnalysisVisible,
        [`quintile${quintile}`]: !prev.quintileAnalysisVisible[`quintile${quintile}` as keyof typeof prev.quintileAnalysisVisible]
      }
    }))
  }

  const setQuintileVisibility = (quintile: number, visible: boolean) => {
    setState(prev => ({
      ...prev,
      quintileAnalysisVisible: {
        ...prev.quintileAnalysisVisible,
        [`quintile${quintile}`]: visible
      }
    }))
  }

  return (
    <UIToggleContext.Provider value={{ state, toggleCombatLog, toggleHpBars, toggleQuintile, setQuintileVisibility }}>
      {children}
    </UIToggleContext.Provider>
  )
}

export const useUIToggles = () => {
  const context = useContext(UIToggleContext)
  if (!context) {
    throw new Error('useUIToggles must be used within a UIToggleProvider')
  }
  return context
}
```

### Component Integration Strategy

#### 1. Combat Log Toggle

**Current Implementation**: Always visible in `EventLog.tsx`
**New Implementation**: Conditional rendering with toggle control

```typescript
// src/components/combat/EventLog.tsx
import { useUIToggles } from '@/model/uiToggleState'

const EventLog: FC<Props> = ({ events, combatantNames, actionNames = {} }) => {
  const { state, toggleCombatLog } = useUIToggles()
  const [filter, setFilter] = useState<EventFilter>('all')

  if (!state.combatLogVisible) {
    return (
      <div className={styles.eventLogContainer}>
        <div className={styles.header}>
          <h3>Combat Log</h3>
          <button onClick={toggleCombatLog} className={styles.toggleButton}>
            {state.combatLogVisible ? 'Hide' : 'Show'} Combat Log
          </button>
        </div>
      </div>
    )
  }

  // Existing EventLog implementation...
}
```

#### 2. HP Bars Toggle

**Current Implementation**: Always visible in `EncounterResult.tsx`
**New Implementation**: Conditional rendering with toggle control

```typescript
// src/components/simulation/encounterResult.tsx
import { useUIToggles } from '@/model/uiToggleState'

const EncounterResult: FC<PropType> = memo(({ value }) => {
  const { state, toggleHpBars } = useUIToggles()

  if (!value.rounds.length) return <></>

  return (
    <div className={styles.encounterResult}>
      {state.hpBarsVisible ? (
        // Existing round-by-round HP bars implementation
        value.rounds.map((round, roundIndex) => (
          <div key={roundIndex} className={styles.round}>
            <h3>Round {roundIndex + 1}</h3>
            <div className={styles.lifebars}>
              {/* HP bars for each round */}
            </div>
          </div>
        ))
      ) : (
        // Simplified result view without round-by-round HP bars
        <div className={styles.round}>
          <h3>Result</h3>
          <div className={styles.lifebars}>
            {/* Only final result HP bars */}
          </div>
        </div>
      )}
      
      {/* Toggle button */}
      <button onClick={toggleHpBars} className={styles.toggleButton}>
        {state.hpBarsVisible ? 'Hide' : 'Show'} HP Bars
      </button>
    </div>
  )
})
```

#### 3. Quintile Analysis Toggle

**Current Implementation**: Always visible all quintiles in `QuintileAnalysis.tsx`
**New Implementation**: Conditional rendering with individual quintile toggles

```typescript
// src/components/simulation/quintileAnalysis.tsx
import { useUIToggles } from '@/model/uiToggleState'

const QuintileAnalysis: FC<PropType> = memo(({ analysis }) => {
  const { state, toggleQuintile, setQuintileVisibility } = useUIToggles()

  if (!analysis) {
    return (
      <div className={styles.quintileAnalysis}>
        <h3>Quintile Analysis</h3>
        <p>Run simulations to see quintile analysis...</p>
      </div>
    )
  }

  return (
    <div className={styles.quintileAnalysis}>
      <h3>5-Timeline Dashboard: {analysis.scenario_name}</h3>
      
      {/* Quintile toggle controls */}
      <div className={styles.quintileToggles}>
        {[1, 2, 3, 4, 5].map(quintile => (
          <button
            key={quintile}
            onClick={() => toggleQuintile(quintile)}
            className={state.quintileAnalysisVisible[`quintile${quintile}` as keyof typeof state.quintileAnalysisVisible] ? styles.active : ''}
          >
            Quintile {quintile}
          </button>
        ))}
      </div>

      <div className={styles.battleCards}>
        {analysis.quintiles.map((quintile) => (
          state.quintileAnalysisVisible[`quintile${quintile.quintile}` as keyof typeof state.quintileAnalysisVisible] && (
            <BattleCard key={quintile.quintile} quintile={quintile} />
          )
        ))}
      </div>
      
      <div className={styles.analysisSummary}>
        <p>Based on {analysis.total_runs} simulation runs</p>
      </div>
    </div>
  )
})
```

### Integration with Loading State Management

#### Enhanced Loading State

```typescript
// src/model/loadingState.ts
interface LoadingState {
  coreUI: 'loaded' | 'loading' | 'error'
  simulationEngine: 'idle' | 'loading' | 'ready' | 'error'
  simulationData: 'idle' | 'loading' | 'ready' | 'error'
  backgroundTasks: BackgroundTask[]
  uiToggles: UIToggleState  // Integrated with existing loading state
}

class LoadingManager {
  private state: LoadingState = {
    coreUI: 'loading',
    simulationEngine: 'idle',
    simulationData: 'idle',
    backgroundTasks: [],
    uiToggles: {
      combatLogVisible: false,
      hpBarsVisible: false,
      quintileAnalysisVisible: {
        quintile1: false,
        quintile2: false,
        quintile3: false,
        quintile4: false,
        quintile5: true
      }
    }
  }

  // Centralized state management methods...
}
```

### User Interface Design

#### Toggle Control Panel

```typescript
// src/components/utils/UiTogglePanel.tsx
import { useUIToggles } from '@/model/uiToggleState'

const UiTogglePanel: FC = () => {
  const { state, toggleCombatLog, toggleHpBars, toggleQuintile } = useUIToggles()

  return (
    <div className={styles.togglePanel}>
      <h4>Display Options</h4>
      
      <div className={styles.toggleGroup}>
        <label className={styles.toggleLabel}>
          <input
            type="checkbox"
            checked={state.combatLogVisible}
            onChange={toggleCombatLog}
          />
          Combat Log
        </label>
        
        <label className={styles.toggleLabel}>
          <input
            type="checkbox"
            checked={state.hpBarsVisible}
            onChange={toggleHpBars}
          />
          HP Bars
        </label>
      </div>

      <div className={styles.quintileToggles}>
        <h5>Quintile Analysis</h5>
        {[1, 2, 3, 4, 5].map(quintile => (
          <label key={quintile} className={styles.toggleLabel}>
            <input
              type="checkbox"
              checked={state.quintileAnalysisVisible[`quintile${quintile}` as keyof typeof state.quintileAnalysisVisible]}
              onChange={() => toggleQuintile(quintile)}
            />
            Quintile {quintile}
          </label>
        ))}
      </div>
    </div>
  )
}
```

### Implementation Phases

#### Phase 1: Core Infrastructure

1. **Create UI Toggle State Management**
   - Implement `uiToggleState.ts` with centralized state
   - Create `UIToggleProvider` and `useUIToggles` hooks
   - Integrate with existing Loading State System

2. **Add Toggle Control Components**
   - Create `UiTogglePanel.tsx` for centralized controls
   - Implement individual component toggles

#### Phase 2: Component Integration

1. **Combat Log Integration**
   - Modify `EventLog.tsx` for conditional rendering
   - Add toggle button and state management
   - Ensure smooth transitions

2. **HP Bars Integration**
   - Modify `EncounterResult.tsx` for conditional rendering
   - Remove round-by-round HP bars by default
   - Add simplified result view

3. **Quintile Analysis Integration**
   - Modify `QuintileAnalysis.tsx` for conditional rendering
   - Add individual quintile toggles
   - Ensure only quintile 5 is visible by default

#### Phase 3: User Experience Enhancements

1. **Styling and Animations**
   - Add smooth transitions for toggle changes
   - Implement visual feedback for active toggles
   - Ensure accessibility compliance

2. **Persistence**
   - Save toggle states to localStorage
   - Restore user preferences on reload
   - Implement default values

3. **Error Handling**
   - Add proper error boundaries
   - Handle edge cases in toggle state management
   - Ensure graceful degradation

### Technology Stack

- **React Context API**: Centralized state management
- **TypeScript**: Strong typing for toggle states
- **CSS Modules**: Scoped styling for toggle components
- **localStorage**: Persistence of user preferences
- **React Hooks**: Efficient state management

### Risk Assessment and Mitigation

#### Potential Risks

1. **State Management Complexity**: Increased complexity in state management
2. **Performance Impact**: Additional rendering cycles for conditional rendering
3. **User Experience Confusion**: Users may not understand toggle functionality
4. **Accessibility Issues**: Toggle controls may not be accessible

#### Mitigation Strategies

1. **Modular Design**: Keep toggle logic focused and independent
2. **Performance Optimization**: Use `React.memo` and `useMemo` for expensive operations
3. **User Testing**: Validate toggle functionality with real users
4. **Accessibility**: Implement proper ARIA attributes and keyboard navigation

### Success Metrics

1. **UI Cleanliness**: Reduced visual clutter by 60-70%
2. **User Control**: Users can customize UI to their preferences
3. **Performance**: No measurable performance degradation
4. **User Satisfaction**: Improved user experience ratings

This architecture provides a maintainable solution for UI cleanup that integrates seamlessly with the existing Loading State Management System while providing users with intuitive controls over their display preferences.