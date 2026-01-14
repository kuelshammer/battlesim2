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
    // Wait for mode switch to complete - the form re-renders after mode change
    await new Promise(r => setTimeout(r, 300));
  }

  /**
   * Fill in creature name
   */
  async setName(name: string): Promise<void> {
    await this.waitForVisible(this.selectors.nameInput);
    await this.page.evaluate(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        if (input) {
          const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')?.set;
          if (nativeInputValueSetter) {
            nativeInputValueSetter.call(input, val);
          } else {
            input.value = val;
          }
          // Trigger React's change detection
          input.dispatchEvent(new Event('input', { bubbles: true }));
          input.dispatchEvent(new Event('change', { bubbles: true }));
        }
      },
      this.selectors.nameInput,
      name
    );
    // Verify value is set to ensure React has picked up the event
    await this.page.waitForFunction(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        return input && input.value === val;
      },
      {},
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
        if (input) {
          input.value = val.toString();
          input.dispatchEvent(new Event('input', { bubbles: true }));
          input.dispatchEvent(new Event('change', { bubbles: true }));
        }
      },
      this.selectors.levelInput,
      level
    );
  }

  /**
   * Set AC (Armor Class)
   * Note: Uses direct value setting with event dispatching for reliability
   */
  async setAC(ac: number): Promise<void> {
    await this.waitForVisible(this.selectors.acInput);
    await this.page.evaluate(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        if (input) {
          const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')?.set;
          if (nativeInputValueSetter) {
            nativeInputValueSetter.call(input, val);
          } else {
            input.value = val;
          }
          // Trigger React's change detection with both input and change events
          input.dispatchEvent(new Event('input', { bubbles: true }));
          input.dispatchEvent(new Event('change', { bubbles: true }));
        }
      },
      this.selectors.acInput,
      ac.toString()
    );
    // Verify value is set to ensure React has picked up the event
    await this.page.waitForFunction(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        return input && input.value === val;
      },
      {},
      this.selectors.acInput,
      ac.toString()
    );
  }

  /**
   * Set HP (Hit Points)
   * Note: Uses direct value setting with event dispatching for reliability
   */
  async setHP(hp: number): Promise<void> {
    await this.waitForVisible(this.selectors.hpInput);
    await this.page.evaluate(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        if (input) {
          const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')?.set;
          if (nativeInputValueSetter) {
            nativeInputValueSetter.call(input, val);
          } else {
            input.value = val;
          }
          // Trigger React's change detection with both input and change events
          input.dispatchEvent(new Event('input', { bubbles: true }));
          input.dispatchEvent(new Event('change', { bubbles: true }));
        }
      },
      this.selectors.hpInput,
      hp.toString()
    );
    // Verify value is set to ensure React has picked up the event
    await this.page.waitForFunction(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        return input && input.value === val;
      },
      {},
      this.selectors.hpInput,
      hp.toString()
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
        if (input) {
          input.value = val.toString();
          input.dispatchEvent(new Event('input', { bubbles: true }));
          input.dispatchEvent(new Event('change', { bubbles: true }));
        }
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
        if (input) {
          input.value = val.toString();
          input.dispatchEvent(new Event('input', { bubbles: true }));
          input.dispatchEvent(new Event('change', { bubbles: true }));
        }
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
    // Wait for save button to be enabled (form is valid)
    await this.page.waitForFunction(
      () => {
        const btn = document.querySelector('[data-testid="save-creature-btn"]') as HTMLButtonElement;
        return btn && !btn.disabled;
      },
      { timeout: 5000 }
    );

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
    await this.page.evaluate(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        if (input) {
          input.value = val;
          input.dispatchEvent(new Event('input', { bubbles: true }));
          input.dispatchEvent(new Event('change', { bubbles: true }));
        }
      },
      this.selectors.monsterSearch,
      name
    );
    await new Promise(r => setTimeout(r, 500)); // Wait for debounce
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
   * Note: For player mode, we switch to custom mode to allow direct AC/HP entry
   */
  async quickCreate(params: {
    mode: 'player' | 'monster' | 'custom';
    name: string;
    ac: number;
    hp: number;
    count?: number;
  }): Promise<void> {
    await this.waitForModal();

    // For player mode with AC/HP specified, use custom mode instead
    // since player mode uses class templates and doesn't expose direct AC/HP inputs
    const effectiveMode = (params.mode === 'player') ? 'custom' : params.mode;

    await this.setMode(effectiveMode);
    await this.setName(params.name);

    // Set AC/HP for custom and monster modes
    await this.setAC(params.ac);
    await this.setHP(params.hp);

    if (params.count !== undefined) {
      await this.setCount(params.count);
    }

    // Give React time to propagate state updates from child components to parent
    await new Promise(r => setTimeout(r, 500));

    await this.save();
  }
}
