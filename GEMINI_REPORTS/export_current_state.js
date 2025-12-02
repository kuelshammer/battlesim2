#!/usr/bin/env node

/**
 * Extract the CURRENT auto-saved state (players + encounters) from localStorage
 * This is what the app is currently showing in the GUI
 * 
 * INSTRUCTIONS:
 * 1. Open Battlesim2 in browser
 * 2. Open Developer Tools (F12)
 * 3. Console tab
 * 4. Copy/paste this script
 * 5. Press Enter
 * 6. File "current_state.json" downloads
 */

(function () {
    // Get the current auto-saved players and encounters
    const playersJson = localStorage.getItem('players');
    const encountersJson = localStorage.getItem('encounters');

    if (!playersJson || !encountersJson) {
        console.error('❌ No auto-saved state found in localStorage!');
        console.log('Available keys:', Object.keys(localStorage));
        return;
    }

    const players = JSON.parse(playersJson);
    const encounters = JSON.parse(encountersJson);

    // Export data
    const exportData = {
        players: players,
        encounters: encounters
    };

    // Download as JSON file
    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'current_state.json';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    console.log('✅ Successfully exported current state!');
    console.log('📊 Players:', players.length);
    console.log('⚔️  Encounters:', encounters.length);
    console.log('💾 File downloaded: current_state.json');
    console.log('');
    console.log('🚀 To run simulation:');
    console.log('   cd simulation-wasm');
    console.log('   cargo run --example run_sim_from_file -- ~/Downloads/current_state.json > log.txt');
})();
