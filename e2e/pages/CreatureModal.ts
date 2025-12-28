import { BasePage } from './BasePage';

/**
 * Creature Modal Page Object Model
 * Represents the creature creation/editing dialog
 */
export class CreatureModal extends BasePage {
  // Selectors
  private readonly selectors = {
    modal: '[data-testid="creature-modal"]',
    modeToggle: '[data-testid="mode-toggle"]',
    nameInput: '[data-testid="creature-name-input"]',

    // Player mode
    levelInput: '[data-testid="level-input"]',
    classSelect: '[data-testid="class-select"]',

    // Monster mode
    monsterSearch: '[data-testid="monster-search"]',
    monsterSelect: '[data-testid="monster-select"]',

    // Stats
    acInput: '[data-testid="ac-input"]',
    hpInput: '[data-testid="hp-input"]',
    initInput: '[data-testid="init-input"]',
    proficiencyInput: '[data-testid="proficiency-input"]',
    strInput: '[data-testid="str-input"]',
    dexInput: '[data-testid="dex-input"]',
    conInput: '[data-testid="con-input"]',
    intInput: '[data-testid="int-input"]',
    wisInput: '[data-testid="wis-input"]',
    chaInput: '[data-testid="cha-input"]',

    // Actions
    addActionBtn: '[data-testid="add-action-btn"]',
    gamePlanBtn: '[data-testid="game-plan-btn"]',

    // Count
    countInput: '[data-testid="count-input"]',

    // Buttons
    saveBtn: '[data-testid="save-creature-btn"]',
    cancelBtn: '[data-testid="cancel-creature-btn"]',

    // Tabs
    statsTab: '[data-testid="tab-stats"]',
    actionsTab: '[data-testid="tab-actions"]',
    gamePlanTab: '[data-testid="tab-gameplan"]',
  };

  /**
   * Wait for modal to be visible
   */
  async waitForModal(timeout: number = 5000): Promise<void> {
    await this.waitForVisible(this.selectors.modal, timeout);
  }

  /**
   * Check if modal is open
   */
  async isOpen(): Promise<boolean> {
    return await this.exists(this.selectors.modal);
  }

  /**
   * Set creature mode (player/monster/custom)
   */
  async setMode(mode: 'player' | 'monster' | 'custom'): Promise<void> {
    await this.waitForModal();
    const modeSelector = `[data-testid="mode-${mode}"]`;
    await this.click(modeSelector);
  }

  /**
   * Fill in creature name
   */
  async setName(name: string): Promise<void> {
    await this.waitForVisible(this.selectors.nameInput);
    await this.page.evaluate(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        if (input) input.value = val;
      },
      this.selectors.nameInput,
      name
    );
  }

  /**
   * Set player level
   */
  async setLevel(level: number): Promise<void> {
    await this.page.evaluate(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        if (input) input.value = val.toString();
      },
      this.selectors.levelInput,
      level
    );
  }

  /**
   * Set AC (Armor Class)
   */
  async setAC(ac: number): Promise<void> {
    await this.page.evaluate(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        if (input) input.value = val.toString();
      },
      this.selectors.acInput,
      ac
    );
  }

  /**
   * Set HP (Hit Points)
   */
  async setHP(hp: number): Promise<void> {
    await this.page.evaluate(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        if (input) input.value = val.toString();
      },
      this.selectors.hpInput,
      hp
    );
  }

  /**
   * Set ability score
   */
  async setAbilityScore(ability: 'str' | 'dex' | 'con' | 'int' | 'wis' | 'cha', value: number): Promise<void> {
    const selector = this.selectors[`${ability}Input`];
    await this.page.evaluate(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        if (input) input.value = val.toString();
      },
      selector,
      value
    );
  }

  /**
   * Fill complete stats for a creature
   */
  async fillStats(stats: {
    name?: string;
    ac?: number;
    hp?: number;
    init?: number;
    str?: number;
    dex?: number;
    con?: number;
    int?: number;
    wis?: number;
    cha?: number;
  }): Promise<void> {
    if (stats.name !== undefined) await this.setName(stats.name);
    if (stats.ac !== undefined) await this.setAC(stats.ac);
    if (stats.hp !== undefined) await this.setHP(stats.hp);
    if (stats.str !== undefined) await this.setAbilityScore('str', stats.str);
    if (stats.dex !== undefined) await this.setAbilityScore('dex', stats.dex);
    if (stats.con !== undefined) await this.setAbilityScore('con', stats.con);
    if (stats.int !== undefined) await this.setAbilityScore('int', stats.int);
    if (stats.wis !== undefined) await this.setAbilityScore('wis', stats.wis);
    if (stats.cha !== undefined) await this.setAbilityScore('cha', stats.cha);
  }

  /**
   * Set creature count (for groups)
   */
  async setCount(count: number): Promise<void> {
    await this.page.evaluate(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        if (input) input.value = val.toString();
      },
      this.selectors.countInput,
      count
    );
  }

  /**
   * Switch to a specific tab
   */
  async switchTab(tab: 'stats' | 'actions' | 'gameplan'): Promise<void> {
    const tabSelector = `[data-testid="tab-${tab}"]`;
    await this.click(tabSelector);
  }

  /**
   * Save the creature
   */
  async save(): Promise<void> {
    await this.click(this.selectors.saveBtn);
    // Wait for modal to close
    await this.page.waitForFunction(
      () => document.querySelector('[data-testid="creature-modal"]') === null,
      { timeout: 5000 }
    );
  }

  /**
   * Cancel without saving
   */
  async cancel(): Promise<void> {
    await this.click(this.selectors.cancelBtn);
    await this.page.waitForFunction(
      () => document.querySelector('[data-testid="creature-modal"]') === null,
      { timeout: 5000 }
    );
  }

  /**
   * Search for a monster
   */
  async searchMonster(name: string): Promise<void> {
    await this.waitForVisible(this.selectors.monsterSearch);
    await this.type(this.selectors.monsterSearch, name);
    await this.page.waitForTimeout(500); // Wait for debounce
  }

  /**
   * Select a monster from search results
   */
  async selectMonster(index: number = 0): Promise<void> {
    const selectors = await this.page.$$(this.selectors.monsterSelect);
    if (index < selectors.length) {
      await selectors[index].click();
    }
  }

  /**
   * Quick create: fill minimal creature and save
   */
  async quickCreate(params: {
    mode: 'player' | 'monster';
    name: string;
    ac: number;
    hp: number;
    count?: number;
  }): Promise<void> {
    await this.waitForModal();
    await this.setMode(params.mode);
    await this.setName(params.name);
    await this.setAC(params.ac);
    await this.setHP(params.hp);
    if (params.count !== undefined) {
      await this.setCount(params.count);
    }
    await this.save();
  }
}
