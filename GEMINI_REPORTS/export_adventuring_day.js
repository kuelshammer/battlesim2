#!/usr/bin/env node

/**
 * Extract the current adventuring day from browser localStorage and save it as a JSON file
 * 
 * INSTRUCTIONS:
 * 1. Open your Battlesim2 app in the browser
 * 2. Open Developer Tools (F12)
 * 3. Go to the Console tab
 * 4. Copy and paste this entire script
 * 5. Press Enter
 * 6. A file named "adventuring_day.json" will be downloaded
 * 7. Run: cargo run --example run_sim_from_file -- adventuring_day.json
 */

(function () {
    // Get the current save name and save files from localStorage
    const saveName = localStorage.getItem('saveName') || 'default';
    const saveFilesJson = localStorage.getItem('saveFiles');

    if (!saveFilesJson) {
        console.error('No saved adventuring days found in localStorage!');
        return;
    }

    const saveFiles = JSON.parse(saveFilesJson);

    // Find the current save
    const currentSave = saveFiles.find(save => save.name === saveName);

    if (!currentSave) {
        console.error(`Save "${saveName}" not found!`);
        console.log('Available saves:', saveFiles.map(s => s.name));
        return;
    }

    // Extract players and encounters
    const exportData = {
        players: currentSave.players,
        encounters: currentSave.encounters
    };

    // Download as JSON file
    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'adventuring_day.json';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    console.log('✅ Successfully exported adventuring day:', saveName);
    console.log('📊 Players:', exportData.players.length);
    console.log('⚔️  Encounters:', exportData.encounters.length);
    console.log('💾 File downloaded: adventuring_day.json');
})();
