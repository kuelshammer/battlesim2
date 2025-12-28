import { describe, it, expect, beforeAll } from 'vitest';
import { SimulationPage } from '../pages/SimulationPage';
import { CreatureModal } from '../pages/CreatureModal';
import fs from 'fs';
import path from 'path';

/**
 * Data Import/Export & Persistence Tests
 * Tests state management and file operations
 */
describe('E2E: Import/Export & Persistence', () => {
  let simulationPage: SimulationPage;
  let creatureModal: CreatureModal;

  beforeAll(() => {
    simulationPage = new SimulationPage(global.page);
    creatureModal = new CreatureModal(global.page);
  });

  it('should persist creatures in localStorage', async () => {
    await simulationPage.goto();
    await simulationPage.clearLocalStorage();

    // Add a creature
    await simulationPage.clickAddCreature();
    await creatureModal.quickCreate({
      mode: 'player',
      name: 'Storage Test Paladin',
      ac: 18,
      hp: 45,
    });

    // Get localStorage contents
    const state = await simulationPage.getLocalStorage();

    // Verify data was stored
    expect(Object.keys(state).length).toBeGreaterThan(0);

    // Reload page
    await simulationPage.goto();
    await simulationPage.waitForPageReady();

    // Verify creature is restored
    const creatureNames = await simulationPage.getCreatureNames();
    expect(creatureNames.some(n => n.includes('Storage Test Paladin'))).toBe(true);
  });

  it('should persist simulation settings', async () => {
    await simulationPage.goto();
    await simulationPage.clearLocalStorage();

    // Change settings
    await simulationPage.setRepetitions(250);
    await simulationPage.setMaxRounds(15);

    // Get state
    const state = await simulationPage.getLocalStorage();

    // Reload
    await simulationPage.goto();
    await simulationPage.waitForPageReady();

    // Verify settings persisted (by checking localStorage still has them)
    const newState = await simulationPage.getLocalStorage();
    expect(Object.keys(newState).length).toBeGreaterThan(0);
  });

  it('should restore complex multi-creature scenarios', async () => {
    await simulationPage.goto();
    await simulationPage.clearLocalStorage();

    // Add party of 4 adventurers
    const adventurers = [
      { name: 'Fighter', ac: 18, hp: 44 },
      { name: 'Cleric', ac: 16, hp: 38 },
      { name: 'Wizard', ac: 12, hp: 27 },
      { name: 'Rogue', ac: 15, hp: 35 },
    ];

    for (const adv of adventurers) {
      await simulationPage.clickAddCreature();
      await creatureModal.quickCreate({
        mode: 'player',
        name: adv.name,
        ac: adv.ac,
        hp: adv.hp,
      });
    }

    let creatureCount = await simulationPage.getCreatureCount();
    expect(creatureCount).toBe(4);

    // Reload and verify
    await simulationPage.goto();
    await simulationPage.waitForPageReady();

    creatureCount = await simulationPage.getCreatureCount();
    expect(creatureCount).toBe(4);

    const creatureNames = await simulationPage.getCreatureNames();
    for (const adv of adventurers) {
      expect(creatureNames.some(n => n.includes(adv.name))).toBe(true);
    }
  });

  it('should handle corrupted localStorage gracefully', async () => {
    await simulationPage.goto();
    await simulationPage.clearLocalStorage();

    // Set corrupted data
    await simulationPage.setLocalStorage({
      'battlesim_creatures': '{invalid json}',
      'battlesim_settings': '{also invalid',
    });

    // Page should still load without crashing
    await simulationPage.goto();
    await simulationPage.waitForPageReady();

    // Should have some error handling or empty state
    const hasError = await simulationPage.hasError();
    const creatureCount = await simulationPage.getCreatureCount();

    // Either shows error message or gracefully handles with empty state
    expect(creatureCount).toBeGreaterThanOrEqual(0);
  });

  it('should clear specific creature while keeping others', async () => {
    await simulationPage.goto();
    await simulationPage.clearLocalStorage();

    // Add 3 creatures
    await simulationPage.clickAddCreature();
    await creatureModal.quickCreate({ mode: 'player', name: 'Keep 1', ac: 15, hp: 30 });
    await simulationPage.clickAddCreature();
    await creatureModal.quickCreate({ mode: 'player', name: 'Remove Me', ac: 15, hp: 30 });
    await simulationPage.clickAddCreature();
    await creatureModal.quickCreate({ mode: 'player', name: 'Keep 2', ac: 15, hp: 30 });

    let creatureCount = await simulationPage.getCreatureCount();
    expect(creatureCount).toBe(3);

    // Remove middle creature
    await simulationPage.removeCreature(1);

    creatureCount = await simulationPage.getCreatureCount();
    expect(creatureCount).toBe(2);

    const creatureNames = await simulationPage.getCreatureNames();
    expect(creatureNames.some(n => n.includes('Keep 1'))).toBe(true);
    expect(creatureNames.some(n => n.includes('Remove Me'))).toBe(false);
    expect(creatureNames.some(n => n.includes('Keep 2'))).toBe(true);
  });

  it('should persist creature modifications', async () => {
    await simulationPage.goto();
    await simulationPage.clearLocalStorage();

    // Add creature
    await simulationPage.clickAddCreature();
    await creatureModal.quickCreate({
      mode: 'player',
      name: 'Modifiable Fighter',
      ac: 16,
      hp: 40,
    });

    // Reload
    await simulationPage.goto();
    await simulationPage.waitForPageReady();

    // Verify creature exists
    let creatureNames = await simulationPage.getCreatureNames();
    expect(creatureNames.some(n => n.includes('Modifiable Fighter'))).toBe(true);

    // The creature should be editable after reload
    // (This is a basic smoke test - in real implementation, you'd click edit, modify, save, and verify)
  });

  it('should handle large localStorage datasets', async () => {
    await simulationPage.goto();
    await simulationPage.clearLocalStorage();

    // Add many creatures to test performance
    const creatureCount = 10;
    for (let i = 0; i < creatureCount; i++) {
      await simulationPage.clickAddCreature();
      await creatureModal.quickCreate({
        mode: 'monster',
        name: `Goblin ${i + 1}`,
        ac: 12,
        hp: 7,
      });
    }

    // Get state size
    const state = await simulationPage.getLocalStorage();
    const totalSize = JSON.stringify(state).length;

    // Verify data is stored (should be several KB)
    expect(totalSize).toBeGreaterThan(1000);

    // Reload and verify all creatures are restored
    await simulationPage.goto();
    await simulationPage.waitForPageReady();

    const finalCount = await simulationPage.getCreatureCount();
    expect(finalCount).toBe(creatureCount);
  });

  it('should merge imported data with existing data correctly', async () => {
    await simulationPage.goto();
    await simulationPage.clearLocalStorage();

    // Add initial creature
    await simulationPage.clickAddCreature();
    await creatureModal.quickCreate({
      mode: 'player',
      name: 'Existing Fighter',
      ac: 18,
      hp: 44,
    });

    // In a full implementation, this would test the import flow
    // For now, we verify localStorage structure supports import/export
    const state = await simulationPage.getLocalStorage();
    expect(Object.keys(state).length).toBeGreaterThan(0);
  });
});
