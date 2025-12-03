// src/components/simulation/ResourcePanel.tsx
import { FC } from 'react';
import { Combattant } from '../../model/model';
import styles from './simulation.module.scss'; // Reusing simulation styles for now, or create new resourcePanel.module.scss
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faBolt, faDiceD6, faHandPaper, faHatWizard, faHeart, faPlus, faShieldAlt, faStar } from '@fortawesome/free-solid-svg-icons';

type ResourcePanelProps = {
    combatant: Combattant;
};

// Helper to format resource names
const formatResourceName = (name: string): string => {
    switch (name) {
        case 'Action': return 'Action';
        case 'BonusAction': return 'Bonus Action';
        case 'Reaction': return 'Reaction';
        case 'Movement': return 'Movement';
        case 'SpellSlot': return 'Spell Slot';
        case 'ClassResource': return 'Class Resource';
        case 'ItemCharge': return 'Item Charge';
        case 'HitDice': return 'Hit Dice';
        case 'HP': return 'Hit Points';
        case 'Custom': return 'Custom Resource';
        default: return name;
    }
};

const ResourcePanel: FC<ResourcePanelProps> = ({ combatant }) => {
    // This component will primarily display initial (or final) state values for resources.
    // The actual live resource tracking would come from event logs or direct context queries.

    const { creature, initialState, finalState } = combatant;

    // Simplified view of resources. In a real scenario, this data would come from the ResourceLedger.
    // For now, we'll try to infer some common resources.

    const currentHP = finalState.current_hp;
    const maxHP = creature.hp;
    const tempHP = finalState.temp_hp || 0;

    // Placeholder for other resources. The actual ResourceLedger data is not directly in Combattant.
    // We would ideally query the WASM for the full ResourceLedger of this combatant.
    // For this initial implementation, we can show basic HP and some "expected" slots.

    return (
        <div className={styles.resourcePanel}>
            <h4>{creature.name}'s Resources</h4>
            
            <div className={styles.resourceGroup}>
                <div className={styles.resourceItem}>
                    <FontAwesomeIcon icon={faHeart} title="Hit Points" />
                    HP: {currentHP.toFixed(0)}/{maxHP.toFixed(0)}
                    {tempHP > 0 && <span style={{ marginLeft: '0.5em', color: '#888' }}>(+{tempHP.toFixed(0)} Temp)</span>}
                </div>
            </div>

            {/* Placeholder for Action Economy - assuming 1 per type for simplicity */}
            <div className={styles.resourceGroup}>
                <div className={styles.resourceItem}>
                    <FontAwesomeIcon icon={faBolt} title="Action" /> Action
                </div>
                <div className={styles.resourceItem}>
                    <FontAwesomeIcon icon={faPlus} title="Bonus Action" /> Bonus
                </div>
                <div className={styles.resourceItem}>
                    <FontAwesomeIcon icon={faHandPaper} title="Reaction" /> Reaction
                </div>
            </div>

            {/* Spell Slots Placeholder */}
            {creature.class?.type && (
                <div className={styles.resourceGroup}>
                    <FontAwesomeIcon icon={faHatWizard} title="Spell Slots" /> Spell Slots:
                    {/* These numbers are hardcoded placeholders. 
                        In a full implementation, we'd pull these from creature.class.spell_slots or similar 
                        and track remaining uses via the ResourceLedger. */}
                    <span> L1: 4/4 </span>
                    <span> L2: 3/3 </span>
                    <span> L3: 2/2 </span>
                </div>
            )}

            {/* Class Resources Placeholder */}
            {creature.class?.type && (
                <div className={styles.resourceGroup}>
                    <FontAwesomeIcon icon={faStar} title="Class Resource" />
                    {creature.class.type === 'barbarian' && <span>Rage: 3/4</span>}
                    {creature.class.type === 'monk' && <span>Ki: 5/5</span>}
                    {creature.class.type === 'paladin' && <span>Lay on Hands: 25/25</span>}
                </div>
            )}
        </div>
    );
};

export default ResourcePanel;