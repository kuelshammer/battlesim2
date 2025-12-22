// src/components/simulation/ResourcePanel.tsx
import { FC } from 'react';
import { Combattant } from '@/model/model';
import styles from './simulation.module.scss';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faBolt, faDiceD6, faHandPaper, faHatWizard, faHeart, faPlus, faShieldAlt, faStar, faShoePrints } from '@fortawesome/free-solid-svg-icons';

type ResourcePanelProps = {
    combatant: Combattant;
};

const ResourcePanel: FC<ResourcePanelProps> = ({ combatant }) => {
    const { creature, finalState } = combatant;
    const { resources } = finalState;

    const currentHP = finalState.currentHp;
    const maxHP = creature.hp;
    const tempHP = finalState.tempHp || 0;

    // Helper to parse resource keys
    const parseResourceKey = (key: string) => {
        if (key === 'Action') return { type: 'Action', label: 'Action', icon: faBolt };
        if (key === 'BonusAction') return { type: 'BonusAction', label: 'Bonus', icon: faPlus };
        if (key === 'Reaction') return { type: 'Reaction', label: 'Reaction', icon: faHandPaper };
        if (key === 'Movement') return { type: 'Movement', label: 'Movement', icon: faShoePrints };

        if (key.startsWith('SpellSlot')) {
            const level = key.match(/\((\d+)\)/)?.[1] || '?';
            return { type: 'SpellSlot', label: `L${level}`, icon: faHatWizard, sort: parseInt(level) };
        }

        if (key.startsWith('ClassResource')) {
            const name = key.match(/\("(.+)"\)/)?.[1] || key.match(/\((.+)\)/)?.[1] || 'Resource';
            return { type: 'ClassResource', label: name, icon: faStar };
        }

        return { type: 'Other', label: key, icon: faDiceD6 };
    };

    // Group resources
    const groups: Record<string, any[]> = {
        'Main': [],
        'SpellSlot': [],
        'ClassResource': [],
        'Other': []
    };

    Object.entries(resources.current).forEach(([key, value]) => {
        const max = resources.max[key] || 0;
        const parsed = parseResourceKey(key);

        const item = {
            key,
            value,
            max,
            ...parsed
        };

        if (['Action', 'BonusAction', 'Reaction', 'Movement'].includes(parsed.type)) {
            groups['Main'].push(item);
        } else if (parsed.type === 'SpellSlot') {
            groups['SpellSlot'].push(item);
        } else if (parsed.type === 'ClassResource') {
            groups['ClassResource'].push(item);
        } else {
            groups['Other'].push(item);
        }
    });

    // Sort spell slots
    groups['SpellSlot'].sort((a, b) => (a.sort || 0) - (b.sort || 0));

    // Sort main resources order
    const mainOrder = ['Action', 'BonusAction', 'Reaction', 'Movement'];
    groups['Main'].sort((a, b) => mainOrder.indexOf(a.type) - mainOrder.indexOf(b.type));

    return (
        <div className={styles.resourcePanel}>
            <h4>{creature.name}'s Resources</h4>

            <div className={styles.resourceGroup}>
                <div className={styles.resourceItem}>
                    <FontAwesomeIcon icon={faHeart} title="Hit Points" />
                    <span className={styles.hpText}>
                        HP: {currentHP.toFixed(0)}/{maxHP.toFixed(0)}
                        {tempHP > 0 && <span className={styles.tempHP}>(+{tempHP.toFixed(0)})</span>}
                    </span>
                </div>
            </div>

            {groups['Main'].length > 0 && (
                <div className={styles.resourceGroup}>
                    {groups['Main'].map(res => (
                        <div key={res.key} className={`${styles.resourceItem} ${res.value <= 0 ? styles.depleted : ''}`}>
                            <FontAwesomeIcon icon={res.icon} title={res.label} /> {res.label}
                        </div>
                    ))}
                </div>
            )}

            {groups['SpellSlot'].length > 0 && (
                <div className={styles.resourceGroup}>
                    <div className={styles.groupHeader}><FontAwesomeIcon icon={faHatWizard} /> Spells</div>
                    <div className={styles.slotsContainer}>
                        {groups['SpellSlot'].map(res => (
                            <span key={res.key} className={`${styles.slotItem} ${res.value <= 0 ? styles.depleted : ''}`}>
                                {res.label}: {res.value}/{res.max}
                            </span>
                        ))}
                    </div>
                </div>
            )}

            {groups['ClassResource'].length > 0 && (
                <div className={styles.resourceGroup}>
                    {groups['ClassResource'].map(res => (
                        <div key={res.key} className={styles.resourceItem}>
                            <FontAwesomeIcon icon={res.icon} /> {res.label}: {res.value}/{res.max}
                        </div>
                    ))}
                </div>
            )}

            {groups['Other'].length > 0 && (
                <div className={styles.resourceGroup}>
                    {groups['Other'].map(res => (
                        <div key={res.key} className={styles.resourceItem}>
                            <FontAwesomeIcon icon={res.icon} /> {res.label}: {res.value}/{res.max}
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

export default ResourcePanel;