# TODO.md

*Last modified: 20251204-08:13:00

## Phase 0: Housekeeping & Documentation Hygiene
*These steps are low-effort but high-value for project clarity. They remove noise and ensure the roadmap is based on reality.*

- [ ] **Purge Transient Logs**: Delete the `GEMINI_REPORTS/` directory entirely. Any still-relevant critical bugs from these files should be merged into `BUG_ANALYSIS.md` or `ISSUES.md` first.
- [ ] **Update Status Documentation**: Rewrite the Executive Summary in `architecture_redesign_review.md` and `REWORK_CRITIC.md`.
    - [ ] Update to reflect that Phase 2 (GUI Editors) and Phase 3 (EventLog) are **Complete**.
    - [ ] Mark backend AI and D&D Fidelity as the new primary blockers.
- [ ] **Safety Refactor**: Replace `static mut LAST_SIMULATION_EVENTS` in `lib.rs`.
    - [ ] Use `std::sync::Mutex<Option<Vec<String>>>` or `std::cell::RefCell` (if single-threaded WASM is guaranteed) to ensure memory safety best practices.

## Phase 1: Core Simulation Integrity (The "Source of Truth")
*We cannot build a smart AI or a pretty UI on top of broken physics. These steps fix the "plumbing" of the engine to ensure state changes are consistent.*

- [ ] **Unify HP Modification (Critical)**: Refactor `action_resolver.rs`.
    - [ ] Locate all instances of direct HP modification (e.g., `target.hp -= damage`).
    - [ ] Replace them with calls to a central `TurnContext::apply_damage()` and `TurnContext::apply_healing()` method.
    - [ ] Ensure these methods **always** emit the corresponding events (`DamageTaken`, `Healed`, `UnitDied`).
- [ ] **Fix Buff Cleanup Logic**: Implement fixes from `BUFF_CLEANUP_ANALYSIS.md`.
    - [ ] Create `remove_all_buffs_from_source(source_id)` function.
    - [ ] Call this function immediately inside the `UnitDied` event handler.
    - [ ] Standardize HP thresholds (handle `< 0`, `<= 0`, and `0` consistently).

## Phase 2: D&D 5e Rule Fidelity
*Moving from "Mock Simulation" to "Actual Simulator".*

- [ ] **Implement Dice Mechanics**: Remove the placeholder `base_roll + 10.0` logic in `action_resolver.rs`.
    - [ ] Import/create a Random Number Generator (RNG) accessible to WASM.
    - [ ] Implement `roll_d20()`.
- [ ] **Attack Roll Resolution**:
    - [ ] Update `roll_attack` to use `roll_d20() + attack_bonus`.
    - [ ] Implement `Critical Hit` (Natural 20) and `Critical Miss` (Natural 1) logic.
- [ ] **Damage Calculation**:
    - [ ] Update `calculate_damage` to roll actual damage dice (e.g., `2d6 + str`) defined in the `Action` struct.
    - [ ] Implement "Double Dice" rule for Critical Hits.
- [ ] **Update Class Templates**: Bring remaining classes up to 5e 2014 standards (Spell Slots, Class Resources, Feature parity).
    - [ ] Artificer
    - [ ] Barbarian (Rage, etc.)
    - [ ] Bard (Spell Slots, Bardic Inspiration)
    - [ ] Cleric (Spell Slots, Channel Divinity)
    - [ ] Druid (Spell Slots, Wild Shape)
    - [ ] Monk (Ki Points)
    - [ ] Paladin (Spell Slots, Lay on Hands)
    - [ ] Rogue (Feature review)
    - [ ] Sorcerer (Spell Slots, Sorcery Points)
    - [ ] Warlock (Pact Magic, Arcanum)

## Phase 3: Artificial Intelligence & Targeting
*Enabling the simulation to run autonomously. This depends on Phase 1 & 2 working correctly so the AI makes valid decisions.*

- [ ] **Smart Targeting Integration**: Refactor `execution.rs`.
    - [ ] Replace `get_random_target` with the existing (but unused) logic in `targeting.rs`.
    - [ ] Wire up `select_enemy_target` (lowest HP? closest?) and `select_ally_target` (for heals/buffs).
- [ ] **Resource-Aware Action Selection**: Upgrade `score_action` in `execution.rs`.
    - [ ] Check `ResourceLedger` availability *before* scoring an action.
    - [ ] Prioritize actions that use "per encounter" or "recharge" resources over basic attacks (At-Will).
- [ ] **Template Resolution**:
    - [ ] Flesh out the stub `resolve_template()` in `action_resolver.rs` to actually instantiate actions based on templates.

## Phase 4: Frontend "Power User" Features
*Now that the backend supports complex resources and strategies, we build the UI to control them.*

- [ ] **Resource Panel UI**: Create `src/components/simulation/ResourcePanel.tsx`.
    - [ ] Visualize `ResourceLedger` data (Spell slots, superiority dice, legendary actions).
    - [ ] Update in real-time based on `EventLog` or state sync.
- [ ] **Strategy Builder UI**: Create `src/components/creatureForm/StrategyBuilder.tsx`.
    - [ ] Allow users to define simple "Gambits" (e.g., "Use *Heal* if Ally HP < 50%").
    - [ ] Save these preferences to the `Combatant` model.
- [ ] **AI Configuration**: Add controls to the Simulation Dashboard.
    - [ ] Toggle between "Random", "Aggressive", and "Conservative" AI profiles (passing this config to the WASM backend).

## Phase 5: Final Polish & Optimization
- [ ] **WASM Offloading**: Move the heavy simulation loop to a Web Worker to prevent UI freezing during large batch simulations.
- [ ] **Legacy Code Removal**: Once Phase 3 is verified, remove the legacy `simulation.rs` path and the deprecated `action_slot` field from `model.rs`.

---

### Clarifying Questions
Before beginning Phase 4 (Frontend), I would need to know:
1.  **Design Specs:** Are there existing wireframes or design preferences for the *Resource Panel* or *Strategy Builder*?
2.  **Web Workers:** Is the infrastructure for Web Workers already set up in the Next.js config, or does that need to be scaffolded from scratch?