const fs = require('fs');
const path = require('path');

/**
 * This script runs a combat simulation using the WASM backend and outputs
 * a full, raw JSON event log to GEMINI_REPORTS/ for AI self-checking.
 */

async function run() {
    const inputPath = process.argv[2] || 'XYZ123.json';
    const outputPath = path.join(__dirname, '../GEMINI_REPORTS/last_simulation_run.json');

    console.log(`Reading scenario from: ${inputPath}`);
    
    if (!fs.existsSync(inputPath)) {
        console.error(`Error: File not found ${inputPath}`);
        process.exit(1);
    }

    const inputData = JSON.parse(fs.readFileSync(inputPath, 'utf8'));
    const players = inputData.players || [];
    const timeline = inputData.timeline || inputData.encounters || [];

    // The wasm pkg is at simulation-wasm/pkg
    // We need to load it. Since it's wasm-bindgen for web, 
    // it might be tricky to run directly in Node.
    // However, we have simulation-wasm/src/bin/sim_cli.rs which is a native binary.
    
    console.log('Using native sim_cli for reliable JSON output...');
    
    const { execSync } = require('child_process');
    try {
        // Run sim_cli batch-log to get multiple runs for auditing
        const command = `cd simulation-wasm && cargo run --quiet --bin sim_cli -- batch-log ../${inputPath} --count 10`;
        const result = execSync(command, { encoding: 'utf8', maxBuffer: 10 * 1024 * 1024 });
        
        fs.writeFileSync(outputPath, result);
        
        console.log('✅ Computer-readable batch log generated!');
        console.log('📄 File path:', outputPath);
        
        // Brief summary
        const summary = JSON.parse(result);
        console.log('📊 Runs recorded:', summary.runs.length);
        console.log('📊 Total events recorded:', summary.runs.reduce((acc, run) => acc + run.events.length, 0));
        
    } catch (e) {
        console.error('❌ Simulation failed:', e.message);
        process.exit(1);
    }
}

run();
