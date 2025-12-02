# How to Run Simulations from Saved Adventuring Days

## Quick Start

### 1. Export Your Adventuring Day from the GUI

1. **Open Battlesim2** in your browser
2. **Load or create** your adventuring day in the GUI
3. **Open Developer Tools** (Press `F12`)
4. **Go to Console tab**
5. **Copy and paste** the following script:

```javascript
(function() {
    const saveName = localStorage.getItem('saveName') || 'default';
    const saveFilesJson = localStorage.getItem('saveFiles');
    
    if (!saveFilesJson) {
        console.error('No saved adventuring days found!');
        return;
    }
    
    const saveFiles = JSON.parse(saveFilesJson);
    const currentSave = saveFiles.find(save => save.name === saveName);
    
    if (!currentSave) {
        console.error(`Save "${saveName}" not found!`);
        return;
    }
    
    const exportData = {
        players: currentSave.players,
        encounters: currentSave.encounters
    };
    
    const blob = new Blob([JSON.stringify(exportData, null, 2)], {type: 'application/json'});
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'adventuring_day.json';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    
    console.log('✅ Exported:', saveName);
})();
```

6. **Press Enter** - The file `adventuring_day.json` will download automatically

### 2. Run the Simulation

Navigate to the `simulation-wasm` directory and run:

```bash
cd simulation-wasm
cargo run --example run_sim_from_file -- /path/to/adventuring_day.json > combat_log.txt
```

### 3. View the Results

Open `combat_log.txt` to see the detailed combat log with:
- Round-by-round breakdown
- Attack rolls with modifiers
- **Resistance calculations** (e.g., "Damage: 20 (Base) * 0.50 (Rage (Resisted)) = 10")
- Damage reduction
- Buff/debuff applications
- Concentration saves

## Example Output

```
=== SIMULATION LOGS ===

--- Encounter Start: Players vs Monsters ---

=== PRE-COMBAT SETUP ===
  > Wizard uses Mage Armour
      -> Casts Mage Armour on Wizard (AC: 15 → 18)

=== Round 1 ===
  > Turn: Barbarian (HP: 45 of 45)
    - Uses Action: Rage
      -> Casts Rage on Barbarian (Resistance to physical damage)
    - Uses Action: Greataxe Attack
      -> Attack vs Goblin: Rolled 15 + 5 (base bonus) = 20 vs AC 15
         Damage: 12 (Base) + 2 (Rage) = 14
         Goblin falls unconscious!
```

## Tips

- **Save your state** frequently in the GUI - it auto-saves to localStorage
- **Export before testing** new encounters to preserve your setup
- **Redirect output** to a file for easier reading: `... > log.txt`
- **Run multiple times** to see variance in dice rolls (change `iterations` parameter in the code)

## Troubleshooting

**"No saved adventuring days found!"**
- Make sure you've created and saved an adventuring day in the GUI first
- Check that localStorage is enabled in your browser

**"Failed to parse JSON"**
- The export might have failed - try running the script again
- Check the JSON file is valid (open in a text editor)

**Compilation errors**
- Run `cargo build` first to check for Rust errors
- Ensure all dependencies are installed (`cargo update`)
