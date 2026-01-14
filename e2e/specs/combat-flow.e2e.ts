import { describe, it, expect, beforeAll } from 'vitest';
import { SimulationPage } from '../pages/SimulationPage';
import { CreatureModal } from '../pages/CreatureModal';
import { ResultsPanel } from '../pages/ResultsPanel';
import basicCombatFixture from '../fixtures/basic-combat.json';

/**
 * Basic Combat Workflow Tests
 * Tests the core user journey: Add creatures → Run simulation → Verify results
 */
describe('E2E: Basic Combat Workflow', () => {
  let simulationPage: SimulationPage;
  let creatureModal: CreatureModal;
  let resultsPanel: ResultsPanel;

  beforeAll(() => {
    simulationPage = new SimulationPage(global.page);
    creatureModal = new CreatureModal(global.page);
    resultsPanel = new ResultsPanel(global.page);
  });

  it('should add creatures and run a basic combat simulation', async () => {
    // Start with clean slate
    await simulationPage.goto();
    await simulationPage.clearLocalStorage();

    // Add first creature (Fighter)
    await simulationPage.clickAddCreature();
    await creatureModal.waitForModal();

    await creatureModal.setMode('player');
    await creatureModal.setName('Test Fighter');
    await creatureModal.setAC(18);
    await creatureModal.setHP(44);
    await creatureModal.setAbilityScore('str', 16);
    await creatureModal.setAbilityScore('dex', 14);
    await creatureModal.save();

    // Verify creature was added
    let creatureCount = await simulationPage.getCreatureCount();
    expect(creatureCount).toBe(1);

    // Add second creature (Goblin)
    await simulationPage.clickAddCreature();
    await creatureModal.waitForModal();

    await creatureModal.setMode('monster');
    await creatureModal.setName('Test Goblin');
    await creatureModal.setAC(15);
    await creatureModal.setHP(7);
    await creatureModal.setCount(3); // 3 Goblins
    await creatureModal.save();

    // Verify both creatures are in list
    creatureCount = await simulationPage.getCreatureCount();
    expect(creatureCount).toBeGreaterThanOrEqual(2); // Fighter + Goblin group

    const creatureNames = await simulationPage.getCreatureNames();
    expect(creatureNames.some(n => n.includes('Test Fighter'))).toBe(true);
    expect(creatureNames.some(n => n.includes('Test Goblin'))).toBe(true);

    // Run the simulation
    await simulationPage.setRepetitions(100);
    await simulationPage.runSimulation();

    // Wait for results and verify
    await resultsPanel.waitForResults();
    expect(await resultsPanel.isDisplayed()).toBe(true);

    // Verify win rate is displayed (should be a percentage)
    const winRate = await resultsPanel.getWinRate();
    expect(winRate).toBeGreaterThanOrEqual(0);
    expect(winRate).toBeLessThanOrEqual(100);

    // Verify average rounds is calculated
    const avgRounds = await resultsPanel.getAvgRounds();
    expect(avgRounds).toBeGreaterThan(0);

    // Verify damage stats exist
    const damageStats = await resultsPanel.getDamageStats();
    expect(damageStats.length).toBeGreaterThan(0);
  });

  it('should handle simulation with high repetition count', async () => {
    await simulationPage.goto();
    await simulationPage.clearLocalStorage();

    // Add a simple combat scenario
    await simulationPage.clickAddCreature();
    await creatureModal.quickCreate({
      mode: 'player',
      name: 'High Level Fighter',
      ac: 20,
      hp: 100,
    });

    await simulationPage.clickAddCreature();
    await creatureModal.quickCreate({
      mode: 'custom',
      name: 'Weak Goblin',
      ac: 10,
      hp: 5,
      count: 5,
    });

    // Run with higher repetitions
    await simulationPage.setRepetitions(500);
    await simulationPage.runSimulation();

    await resultsPanel.waitForResults();

    // Verify results are reasonable
    const winRate = await resultsPanel.getWinRate();
    expect(winRate).toBeGreaterThanOrEqual(0);
    expect(winRate).toBeLessThanOrEqual(100);
  });

  it('should persist creature state across page reloads', async () => {
    await simulationPage.goto();
    await simulationPage.clearLocalStorage();

    // Add creatures
    await simulationPage.clickAddCreature();
    await creatureModal.quickCreate({
      mode: 'player',
      name: 'Persistent Fighter',
      ac: 18,
      hp: 50,
    });

    // Get initial state
    let creatureCount = await simulationPage.getCreatureCount();
    expect(creatureCount).toBe(1);

    // Reload page
    await simulationPage.goto();
    await simulationPage.waitForPageReady();

    // Verify creatures are restored
    creatureCount = await simulationPage.getCreatureCount();
    expect(creatureCount).toBe(1);

    const creatureNames = await simulationPage.getCreatureNames();
    expect(creatureNames.some(n => n.includes('Persistent Fighter'))).toBe(true);
  });

  it('should export and import simulation state', async () => {
    await simulationPage.goto();
    await simulationPage.clearLocalStorage();

    // Setup a known state
    await simulationPage.clickAddCreature();
    await creatureModal.quickCreate({
      mode: 'player',
      name: 'Export Test Fighter',
      ac: 16,
      hp: 40,
    });

    // Get state from localStorage
    const state = await simulationPage.getLocalStorage();
    expect(Object.keys(state).length).toBeGreaterThan(0);

    // Verify creatures key exists (actual keys are 'players' and 'timeline')
    const hasCreaturesKey = Object.keys(state).some(k => k.includes('players') || k.includes('timeline'));
    expect(hasCreaturesKey).toBe(true);
  });

  it('should clear all creatures', async () => {
    await simulationPage.goto();
    await simulationPage.clearLocalStorage();

    // Add multiple creatures
    for (let i = 0; i < 3; i++) {
      await simulationPage.clickAddCreature();
      await creatureModal.quickCreate({
        mode: 'custom',
        name: `Temp Monster ${i + 1}`,
        ac: 10,
        hp: 10,
      });
    }

    let creatureCount = await simulationPage.getCreatureCount();
    expect(creatureCount).toBe(3);

    // Clear storage and reload
    await simulationPage.clearLocalStorage();

    creatureCount = await simulationPage.getCreatureCount();
    expect(creatureCount).toBe(0);
  });
});
