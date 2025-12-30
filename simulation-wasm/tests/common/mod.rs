use simulation_wasm::model::{Creature, TimelineStep};
use std::fs;
use std::path::PathBuf;

/// Load a scenario from the tests/scenarios directory
pub fn load_scenario(filename: &str) -> (Vec<Creature>, Vec<TimelineStep>) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/scenarios");
    if !filename.contains('/') && !filename.ends_with(".json") {
         // Fallback for names without extension if needed, but usually we pass full filename
    }
    path.push(filename);

    if !path.exists() {
        // Try basic/ if not found in root scenarios
        let mut basic_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        basic_path.push("tests/scenarios/basic");
        basic_path.push(filename);
        if basic_path.exists() {
            path = basic_path;
        }
    }

    let content = fs::read_to_string(&path).expect(&format!("Failed to read scenario file: {:?}", path));
    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    let players: Vec<Creature> =
        serde_json::from_value(data["players"].clone()).expect("Failed to parse players");

    let timeline: Vec<TimelineStep> = if let Some(t) = data.get("timeline") {
        serde_json::from_value(t.clone()).expect("Failed to parse timeline")
    } else if let Some(e) = data.get("encounters") {
        let encounters: Vec<simulation_wasm::model::Encounter> =
            serde_json::from_value(e.clone()).expect("Failed to parse encounters");
        encounters.into_iter().map(TimelineStep::Combat).collect()
    } else {
        Vec::new()
    };

    (players, timeline)
}
