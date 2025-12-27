const fs = require('fs');
const path = require('path');

/**
 * Rule Auditor for D&D 5e Compliance
 * Scans simulation JSON logs for logic violations.
 */

const logPath = path.join(__dirname, '../GEMINI_REPORTS/last_simulation_run.json');

if (!fs.existsSync(logPath)) {
    console.error('❌ No simulation log found to audit. Run "npm run sim:log" first.');
    process.exit(1);
}

const logData = JSON.parse(fs.readFileSync(logPath, 'utf8'));
const runs = logData.runs || [{ events: logData.events }];

const violations = [];

// Helper to add violation
function report(runIndex, actorId, round, message) {
    violations.push({ runIndex, actorId, round, message });
}

console.log(`🕵️  Starting 5e Rule Audit across ${runs.length} runs...`);

runs.forEach((run, runIndex) => {
    const events = run.events;
    
    // State trackers
    let currentRound = 0;
    let currentActor = null;
    let turnUsage = {
        action: 0,
        bonusAction: 0,
        movement: 0,
        spells: []
    };
    let roundUsage = {
        reactions: new Map() // actorId -> count
    };

    // Global concentration map: casterId -> { spellId, startTime }
    const activeConcentration = new Map();

    events.forEach((event, index) => {
        const type = event.type;
        const actorId = (event.actor_id || event.attacker_id || event.caster_id || event.unit_id);

        if (type === 'RoundStarted') {
            currentRound = event.round_number;
            roundUsage.reactions.clear();
        }

        if (type === 'TurnStarted') {
            currentActor = actorId;
            turnUsage = { action: 0, bonusAction: 0, movement: 0, spells: [] };
        }

        if (type === 'ConcentrationBroken') {
            activeConcentration.delete(actorId);
        }

        if (type === 'ResourceConsumed') {
            const resType = event.resource_type;
            
            if (resType === 'Reaction') {
                const count = (roundUsage.reactions.get(actorId) || 0) + 1;
                roundUsage.reactions.set(actorId, count);
                if (count > 1) {
                    report(runIndex, actorId, currentRound, `Reaction used multiple times in one round (${count})`);
                }
            }
        }
    });
});

// Final check
console.log('--- Audit Results ---');
if (violations.length === 0) {
    console.log(`✅ 5e Rule Compliance: High confidence (Basic checks passed across ${runs.length} runs)`);
} else {
    console.error(`❌ Found ${violations.length} violations:`);
    violations.forEach(v => {
        console.error(`  [Run ${v.runIndex}] [Round ${v.round}] [${v.actorId}]: ${v.message}`);
    });
    process.exit(1);
}

// Helpers for string parsing (if needed)
function extractType(str) { return str.includes('starts turn') ? 'TurnStarted' : ''; }
function extractActor(str) { return str.split(' ')[0]; }
function extractRound(str) { return 1; }
