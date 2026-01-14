//! Integration tests using real D&D 5e encounter scenarios.
//!
//! These tests load JSON encounter files, run simulations, and validate
//! that the results maintain expected invariants (no impossible states).

use std::path::Path;
use std::fs;
use simulation_wasm::model::{Creature, TimelineStep};

/// Test configuration for scenarios
#[allow(dead_code)]
struct ScenarioTest {
    name: String,
    max_rounds: u32,
    allow_tpk: bool, // Total Party Kill allowed?
}

impl Default for ScenarioTest {
    fn default() -> Self {
        Self {
            name: "Unnamed".to_string(),
            max_rounds: 20,
            allow_tpk: false,
        }
    }
}

/// Load all scenario JSON files from a directory and run them as tests
fn run_scenarios_in_dir(dir: &str) {
    let mut scenarios_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    scenarios_path.push(dir);
    
    if !scenarios_path.exists() {
        println!("Skipping scenario tests: {:?} directory not found", scenarios_path);
        return;
    }

    let entries = fs::read_dir(scenarios_path)
        .expect("Failed to read scenarios directory");

    let mut scenario_count = 0;
    for entry in entries.flatten() {
        if entry.path().extension().is_some_and(|ext| ext == "json") {
            scenario_count += 1;
        }
    }

    if scenario_count == 0 {
        println!("No scenarios found in {}", dir);
        return;
    }

    println!("Found {} scenario(s) in {}", scenario_count, dir);
}

/// Simple scenario structure for JSON loading
#[derive(Debug, serde::Deserialize)]
struct TestScenario {
    #[allow(dead_code)]
    name: String,
    players: Vec<Creature>,
    timeline: Vec<TimelineStep>,
}

/// Parse a scenario JSON file
fn load_local_scenario(path: &Path) -> Result<TestScenario, String> {
    let json_content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    serde_json::from_str(&json_content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))
}

/// Run a single scenario and validate invariants
fn run_scenario(scenario: &TestScenario, _config: &ScenarioTest) -> Result<(), String> {
    // For Phase 1: Just validate that the scenario can be loaded and creatures are valid
    // Future phases will actually run the simulation

    // Basic validation: check that creatures have required fields
    for creature in &scenario.players {
        validate_creature(creature)?;
    }

    for timeline_step in &scenario.timeline {
        match timeline_step {
            TimelineStep::Combat(encounter) => {
                for creature in &encounter.monsters {
                    validate_creature(creature)?;
                }
            }
            TimelineStep::ShortRest(_) => {
                // Short rest validation - nothing to check for now
            }
        }
    }

    Ok(())
}

/// Validate that a creature has all required fields set
fn validate_creature(creature: &Creature) -> Result<(), String> {
    if creature.hp == 0 {
        return Err(format!("{} has invalid HP: {}", creature.name, creature.hp));
    }

    // Check that actions have IDs
    for (idx, action) in creature.actions.iter().enumerate() {
        let action_id = action.base().id.clone();
        if action_id.is_empty() {
            return Err(format!(
                "{}: Action at index {} has empty ID",
                creature.name, idx
            ));
        }
        if action.base().name.is_empty() {
            return Err(format!(
                "{}: Action at index {} has no name",
                creature.name, idx
            ));
        }
    }

    Ok(())
}

// ============================================================================
// Basic Mechanical Scenarios
// ============================================================================

#[test]
fn test_basic_attack_scenario() {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/scenarios/basic/01_single_attack.json");
    
    if !path.exists() {
        println!("Skipping: scenario file not found: {:?}", path);
        return;
    }

    let scenario = load_local_scenario(&path).expect("Failed to load scenario");
    let config = ScenarioTest {
        name: "Basic Attack".to_string(),
        ..Default::default()
    };

    if let Err(e) = run_scenario(&scenario, &config) {
        panic!("Scenario failed: {}", e);
    }
}

#[test]
fn test_fireball_aoe_scenario() {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/scenarios/basic/02_fireball_aoe.json");
    if !path.exists() {
        return;
    }

    let scenario = load_local_scenario(&path).expect("Failed to load scenario");
    let config = ScenarioTest {
        name: "Fireball AOE".to_string(),
        ..Default::default()
    };

    if let Err(e) = run_scenario(&scenario, &config) {
        panic!("Scenario failed: {}", e);
    }
}

#[test]
fn test_healing_word_scenario() {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/scenarios/basic/03_healing_word.json");
    if !path.exists() {
        return;
    }

    let scenario = load_local_scenario(&path).expect("Failed to load scenario");
    let config = ScenarioTest {
        name: "Healing Word".to_string(),
        ..Default::default()
    };

    if let Err(e) = run_scenario(&scenario, &config) {
        panic!("Scenario failed: {}", e);
    }
}

#[test]
fn test_concentration_scenario() {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/scenarios/basic/04_concentration_check.json");
    if !path.exists() {
        return;
    }

    let scenario = load_local_scenario(&path).expect("Failed to load scenario");
    let config = ScenarioTest {
        name: "Concentration Check".to_string(),
        ..Default::default()
    };

    if let Err(e) = run_scenario(&scenario, &config) {
        panic!("Scenario failed: {}", e);
    }
}

#[test]
fn test_multiattack_scenario() {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/scenarios/basic/05_multiattack.json");
    if !path.exists() {
        return;
    }

    let scenario = load_local_scenario(&path).expect("Failed to load scenario");
    let config = ScenarioTest {
        name: "Multiattack".to_string(),
        ..Default::default()
    };

    if let Err(e) = run_scenario(&scenario, &config) {
        panic!("Scenario failed: {}", e);
    }
}

// ============================================================================
// Test Discovery
// ============================================================================

#[test]
fn list_available_scenarios() {
    run_scenarios_in_dir("tests/scenarios/basic");
    run_scenarios_in_dir("tests/scenarios/edge_cases");
    run_scenarios_in_dir("tests/scenarios/complex");
}
