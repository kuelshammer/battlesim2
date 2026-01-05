import { v4 as uuid } from 'uuid'
import type { Buff } from "../model/model"

export type MagicItemTemplateName = keyof typeof MagicItemTemplates;

// Magic item template - base structure without runtime fields
type MagicItemTemplate = {
    name: string;
    description?: string;
    buffs: Buff[];
}

// For type safety
function createTemplate(item: MagicItemTemplate): MagicItemTemplate {
    return item
}

export const MagicItemTemplates: Record<string, MagicItemTemplate> = {
    // Placeholder - templates will be added in subsequent tasks:
    // - Cloak of Protection
    // - Ring of Protection
    // - Cloak of Displacement
    // - Bracers of Defense
}

// Helper function to get magic item buffs by name
export function getMagicItemBuffs(itemName: MagicItemTemplateName): Buff[] {
    const template = MagicItemTemplates[itemName]
    return template?.buffs ?? []
}
