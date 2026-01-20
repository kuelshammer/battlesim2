// src/components/simulation/ResourcePanel.tsx
import { FC, memo } from 'react';
import { Combattant } from '@/model/model';
import styles from './resourcePanel.module.scss';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { IconDefinition } from '@fortawesome/fontawesome-svg-core';
import { 
    faBolt, faDiceD6, faHandPaper, faHatWizard, 
    faHeart, faPlus, faStar, faShoePrints,
    faMagic, faShieldHalved
} from '@fortawesome/free-solid-svg-icons';
import { motion } from 'framer-motion';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

type ResourcePanelProps = {
    combatant: Combattant;
};

const ResourcePanel: FC<ResourcePanelProps> = memo(({ combatant }) => {
    const { creature, finalState } = combatant;
    const { resources } = finalState;

    const currentHP = finalState.currentHp;
    const maxHP = creature.hp;
    const tempHP = finalState.tempHp || 0;

    const parseResourceKey = (key: string) => {
        if (key === 'Action') return { type: 'Action', label: 'Action', icon: faBolt, color: '#FFD700' };
        if (key === 'BonusAction') return { type: 'BonusAction', label: 'Bonus', icon: faPlus, color: '#FF8C00' };
        if (key === 'Reaction') return { type: 'Reaction', label: 'Reaction', icon: faHandPaper, color: '#BA55D3' };
        if (key === 'Movement') return { type: 'Movement', label: 'Movement', icon: faShoePrints, color: '#90EE90' };

        if (key.startsWith('SpellSlot')) {
            const level = key.match(/\((\d+)\)/)?.[1] || '?';
            return { type: 'SpellSlot', label: `L${level}`, icon: faHatWizard, sort: parseInt(level), color: '#4facfe' };
        }

        if (key.startsWith('ClassResource')) {
            const name = key.match(/\("(.+)"\)/)?.[1] || key.match(/\((.+)\)/)?.[1] || 'Resource';
            return { type: 'ClassResource', label: name, icon: faStar, color: '#f093fb' };
        }

        return { type: 'Other', label: key, icon: faDiceD6, color: '#aaa' };
    };

    const groups: Record<string, Array<{ key: string; value: number; max: number; type: string; label: string; icon: IconDefinition; color: string; sort?: number }>> = {
        'Main': [],
        'SpellSlot': [],
        'ClassResource': [],
        'Other': []
    };

    Object.entries(resources.current).forEach(([key, value]) => {
        const max = resources.max[key] || 0;
        const parsed = parseResourceKey(key);
        const item = { key, value, max, ...parsed };

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

    groups['SpellSlot'].sort((a, b) => (a.sort || 0) - (b.sort || 0));
    const mainOrder = ['Action', 'BonusAction', 'Reaction', 'Movement'];
    groups['Main'].sort((a, b) => mainOrder.indexOf(a.type) - mainOrder.indexOf(b.type));

    const containerVariants = {
        hidden: { opacity: 0, scale: 0.95, y: 10 },
        visible: { 
            opacity: 1, 
            scale: 1, 
            y: 0,
            transition: { 
                staggerChildren: 0.05,
                delayChildren: 0.1
            }
        }
    };

    const itemVariants = {
        hidden: { opacity: 0, x: -10 },
        visible: { opacity: 1, x: 0 }
    };

    return (
        <motion.div 
            className={styles.arcaneVellum}
            variants={containerVariants}
            initial="hidden"
            animate="visible"
            data-testid="resource-panel"
        >
            <div className={styles.vellumHeader}>
                <span className={styles.sigil}><FontAwesomeIcon icon={faMagic} /></span>
                <h4 data-testid="creature-name">{creature.name}</h4>
            </div>

            <div className={styles.statsGrid} data-testid="stats-grid">
                <motion.div variants={itemVariants} className={styles.statPill} data-testid="hp-stat-pill">
                    <FontAwesomeIcon icon={faHeart} className={styles.hpIcon} />
                    <span className={styles.statValue} data-testid="hp-value">{currentHP}/{maxHP}</span>
                    {tempHP > 0 && <span className={styles.tempHP} data-testid="thp-value">+{tempHP}</span>}
                </motion.div>
                {finalState.arcaneWardHp !== undefined && finalState.arcaneWardHp > 0 && (
                    <motion.div variants={itemVariants} className={cn(styles.statPill, styles.wardPill)} data-testid="ward-stat-pill">
                        <FontAwesomeIcon icon={faShieldHalved} />
                        <span className={styles.statValue} data-testid="ward-value">{finalState.arcaneWardHp}</span>
                    </motion.div>
                )}
            </div>

            <div className={styles.resourceGroups} data-testid="resource-groups">
                {groups['Main'].length > 0 && (
                    <div className={styles.mainGroup} data-testid="main-resources">
                        {groups['Main'].map(res => (
                            <motion.div
                                key={res.key}
                                variants={itemVariants}
                                className={cn(styles.mainIcon, res.value <= 0 && styles.depleted)}
                                style={{ '--glow-color': res.color } as React.CSSProperties}
                                title={`${res.label}: ${res.value}/${res.max}`}
                                data-testid={`resource-${res.type.toLowerCase()}`}
                            >
                                <FontAwesomeIcon icon={res.icon} />
                            </motion.div>
                        ))}
                    </div>
                )}

                {groups['SpellSlot'].length > 0 && (
                    <div className={styles.spellGroup} data-testid="spell-slots">
                        <div className={styles.divider}><span>Weave Sockets</span></div>
                        <div className={styles.slotsGrid}>
                            {groups['SpellSlot'].map(res => (
                                <motion.div 
                                    key={res.key} 
                                    variants={itemVariants}
                                    className={cn(styles.slotNode, res.value <= 0 && styles.empty)}
                                    title={`${res.label}: ${res.value}/${res.max}`}
                                    data-testid={`spell-slot-${res.sort}`}
                                >
                                    <span className={styles.levelLabel}>{res.label}</span>
                                    <div className={styles.nodeTrack}>
                                        <div 
                                            className={styles.nodeFill} 
                                            style={{ width: `${(res.value / res.max) * 100}%` }} 
                                        />
                                    </div>
                                </motion.div>
                            ))}
                        </div>
                    </div>
                )}

                {groups['ClassResource'].length > 0 && (
                    <div className={styles.classGroup} data-testid="class-resources">
                        <div className={styles.divider}><span>Essence</span></div>
                        {groups['ClassResource'].map(res => (
                            <motion.div 
                                key={res.key} 
                                variants={itemVariants}
                                className={styles.essenceItem}
                                data-testid={`class-resource-${res.label.toLowerCase().replace(/\s+/g, '-')}`}
                            >
                                <FontAwesomeIcon icon={res.icon} style={{ color: res.color }} />
                                <span className={styles.essenceName}>{res.label}</span>
                                <span className={styles.essenceCount}>{res.value}/{res.max}</span>
                            </motion.div>
                        ))}
                    </div>
                )}
            </div>
        </motion.div>
    );
});

export default ResourcePanel;
