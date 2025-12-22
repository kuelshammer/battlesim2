# Spec: Fix Simulation Error after Import

The simulation fails after importing a creature because the actions created by the 5e.tools parser lack the mandatory 'id' field required by the Rust/WASM simulation engine.
