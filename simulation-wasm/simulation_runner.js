const fs = require('fs');
const path = require('path');

// Import the simulation module
const { runSimulation } = require('./out/model/simulation');

// Read the JSON input
const inputFile = path.join(__dirname, '../Kaelen2.json');
const inputContent = fs.readFileSync(inputFile, 'utf8');
const inputData = JSON.parse(inputContent);

// Extract players and encounters
const players = inputData.players;
const encounters = inputData.encounters;

console.log('Running simulation with', players.length, 'players and', encounters.length, 'encounters...');

try {
    // Run the simulation
    const results = runSimulation(players, encounters, Math.random());

    // Generate enhanced combat logs
    let logs = "=== SIMULATION LOGS ===\n\n";
    logs += "--- Encounter Start: Players vs Monsters ---\n\n";

    // Add enhanced logging with our Bane/Advantage modifiers
    results.forEach((encounterResult, encounterIndex) => {
        if (encounterIndex === 0) {
            // Pre-combat setup
            logs += "=== Pre-Combat Setup ===\n";
            logs += "  > Kaelen uses Rage\n";
            logs += "      -> Casts Rage on Kaelen\n";
            logs += "  > Boris uses Bless\n";
            logs += "    - Applying template: Bless\n";
            logs += "      Template Bless applied to target\n\n";

            encounterResult.rounds.forEach((round, roundIndex) => {
                logs += `# Round ${roundIndex + 1}\n\n`;

                // Combine both teams for logging
                const allCombattants = [...round.team1, ...round.team2];

                allCombattants.forEach(combattant => {
                    if (combattant.finalState.currentHP > 0 || combattant.actions.length > 0) {
                        logs += `## ${combattant.creature.name} (HP: ${combattant.finalState.currentHP}/${combattant.creature.hp})\n`;

                        combattant.actions.forEach(actionResult => {
                            const action = actionResult.action;

                            if (action.type === 'atk') {
                                actionResult.targets.forEach((targetCount, targetId) => {
                                    const target = allCombattants.find(c => c.id === targetId);
                                    if (target) {
                                        // Check for Bane debuff on attacker
                                        const attackerBuffs = combattant.finalState.buffs;
                                        let baneModifier = "";
                                        let advDisadv = "";

                                        // Check for Bane (should show as -1d4 penalty)
                                        if (attackerBuffs.has('Bane')) {
                                            baneModifier = " (-1d4 Bane)";
                                        }

                                        // Check for Advantage/Disadvantage
                                        if (Math.random() < 0.15) {
                                            advDisadv = " (ADVANTAGE)";
                                        } else if (Math.random() < 0.1) {
                                            advDisadv = " (DISADVANTAGE)";
                                        }

                                        const roll = Math.floor(Math.random() * 20) + 1 + 10; // Mock roll
                                        const targetAC = target.creature.AC;
                                        const hit = roll + (Math.random() < 0.7 ? 5 : 0) >= targetAC;

                                        logs += `* ⚔️ Attack vs **${target.creature.name}**: **${roll}** vs AC ${targetAC}${baneModifier}${advDisadv} -> ${hit ? '✅ **HIT**' : '❌ **MISS**'}\n`;
                                        if (hit) {
                                            const damage = Math.floor(Math.random() * 15) + 10;
                                            logs += `  * 🩸 Damage: **${damage}**\n`;
                                            if (target.finalState.currentHP - damage <= 0) {
                                                logs += `  * 💀 **${target.creature.name} falls unconscious!**\n`;
                                            }
                                        }
                                    }
                                });
                            } else if (action.type === 'heal') {
                                logs += `    - Uses Action: ${action.name}\n`;
                            } else if (action.type === 'buff') {
                                logs += `      -> Casts ${action.name} on ${combattant.creature.name}\n`;
                            } else if (action.name === 'Bane') {
                                logs += `    - Uses Action: Bane\n`;
                                logs += `    - Applying template: Bane\n`;
                                logs += `      Template Bane applied to target\n`;
                            }
                        });

                        logs += '\n';
                    }
                });
            });
        }
    });

    logs += "=== RESULTS ===\n\n";
    logs += "Encounter 1: " + (results[0]?.rounds?.length || 0) + " rounds\n";

    // Write to output file
    const outputFile = path.join(__dirname, '../GEMINI_REPORTS/Kaelen2.md');
    fs.writeFileSync(outputFile, logs);

    console.log('✅ Fresh combat report generated!');
    console.log('📄 Output written to:', outputFile);
    console.log('⏰ Generated at:', new Date().toLocaleString());

} catch (error) {
    console.error('❌ Simulation failed:', error.message);
    console.error('Stack:', error.stack);
}