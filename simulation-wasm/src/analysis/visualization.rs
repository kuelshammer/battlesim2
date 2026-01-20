use super::types::CombatantVisualization;
use crate::model::SimulationResult;

/// Extract combatant visualization data from simulation result
pub fn extract_combatant_visualization_partial(
    result: &SimulationResult,
    encounter_idx: Option<usize>,
) -> (Vec<CombatantVisualization>, usize) {
    let mut combatants = Vec::new();
    let mut battle_duration = 0;

    let slice = if let Some(idx) = encounter_idx {
        if idx < result.encounters.len() {
            &result.encounters[idx..=idx]
        } else {
            &[]
        }
    } else {
        &result.encounters[..]
    };

    for encounter in slice {
        battle_duration += encounter.rounds.len();
    }

    if let Some(final_encounter) = slice.last() {
        if let (Some(first_round), Some(last_round)) = (
            final_encounter.rounds.first(),
            final_encounter.rounds.last(),
        ) {
            let start_hps: std::collections::HashMap<String, u32> = first_round
                .team1
                .iter()
                .chain(first_round.team2.iter())
                .map(|c| (c.id.clone(), c.initial_state.current_hp))
                .collect();

            for combatant in &last_round.team1 {
                let hp_percentage = if combatant.creature.hp > 0 {
                    (combatant.final_state.current_hp as f64 / combatant.creature.hp as f64) * 100.0
                } else {
                    0.0
                };

                combatants.push(CombatantVisualization {
                    name: combatant.creature.name.clone(),
                    max_hp: combatant.creature.hp,
                    start_hp: *start_hps
                        .get(&combatant.id)
                        .unwrap_or(&combatant.creature.hp),
                    current_hp: combatant.final_state.current_hp,
                    is_dead: combatant.final_state.current_hp == 0,
                    is_player: true,
                    hp_percentage,
                });
            }

            for combatant in &last_round.team2 {
                let hp_percentage = if combatant.creature.hp > 0 {
                    (combatant.final_state.current_hp as f64 / combatant.creature.hp as f64) * 100.0
                } else {
                    0.0
                };

                combatants.push(CombatantVisualization {
                    name: combatant.creature.name.clone(),
                    max_hp: combatant.creature.hp,
                    start_hp: *start_hps
                        .get(&combatant.id)
                        .unwrap_or(&combatant.creature.hp),
                    current_hp: combatant.final_state.current_hp,
                    is_dead: combatant.final_state.current_hp == 0,
                    is_player: false,
                    hp_percentage,
                });
            }
        }
    }

    (combatants, battle_duration)
}

/// Slice events for a specific encounter
pub fn slice_events_for_encounter(
    events: &[crate::events::Event],
    encounter_idx: usize,
) -> Vec<crate::events::Event> {
    let mut sliced = Vec::new();
    let mut current_encounter = 0;
    let mut recording = false;

    for event in events {
        if let crate::events::Event::EncounterStarted { .. } = event {
            if recording {
                break;
            }
            if current_encounter == encounter_idx {
                recording = true;
            }
            current_encounter += 1;
        }

        if recording {
            sliced.push(event.clone());
        }

        if let crate::events::Event::EncounterEnded { .. } = event {
            if recording {
                break;
            }
        }
    }
    sliced
}
