import { v4 as uuid } from 'uuid'
import type { Buff } from "../model/model"

export type MagicItemTemplateName = keyof typeof MagicItemTemplates;

// Magic item template - base structure without runtime fields
type MagicItemTemplate = {
    name: string;
    description?: string;
    requiresAttunement: boolean;
    buffs: Buff[];
}

// For type safety
function createTemplate(item: MagicItemTemplate): MagicItemTemplate {
    return item
}

export const MagicItemTemplates = {
    'Cloak of Protection': createTemplate({
        name: 'Cloak of Protection',
        description: 'Wondrous item, uncommon (requires attunement)',
        requiresAttunement: true,
        buffs: [{
            displayName: 'Cloak of Protection',
            duration: 'entire encounter',
            ac: 1,
            save: 1,
        }],
    }),

    'Ring of Protection': createTemplate({
        name: 'Ring of Protection',
        description: 'Ring, uncommon (requires attunement)',
        requiresAttunement: true,
        buffs: [{
            displayName: 'Ring of Protection',
            duration: 'entire encounter',
            ac: 1,
            save: 1,
        }],
    }),
}

// Helper function to get magic item buffs by name
export function getMagicItemBuffs(itemName: MagicItemTemplateName): Buff[] {
    const template = MagicItemTemplates[itemName]
    return template?.buffs ?? []
}
