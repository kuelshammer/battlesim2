// src/components/creatureForm/ResourceEditor.tsx
import { FC, useState, useEffect } from 'react';
import { Creature } from '../../model/model';
import styles from './resourceEditor.module.scss';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faPlus, faTrash, faMagicWandSparkles } from '@fortawesome/free-solid-svg-icons';
import { clone } from '../../model/utils';
import { calculateSpellSlots, detectCasterLevel, CasterType, CASTER_TYPE_LABELS } from '../../model/spellSlots';

type Props = {
    value: Creature;
    onChange: (newValue: Creature) => void;
};

const ResourceEditor: FC<Props> = ({ value, onChange }) => {
    // State for caster level feature
    const [casterLevel, setCasterLevel] = useState<number | ''>('');
    const [casterType, setCasterType] = useState<CasterType>('full');
    const [showCasterHelper, setShowCasterHelper] = useState(false);

    // Detect caster level from existing spell slots on mount
    useEffect(() => {
        const detected = detectCasterLevel(value.spellSlots);
        if (detected) {
            setCasterLevel(detected.level);
            setCasterType(detected.type);
        }
    }, []);

    function updateCreature(callback: (creatureClone: Creature) => void) {
        const creatureClone = clone(value);
        callback(creatureClone);
        onChange(creatureClone);
    }

    // --- Caster Level Helper ---
    const applyCasterLevel = () => {
        if (typeof casterLevel !== 'number' || casterLevel < 1) return;

        const slots = calculateSpellSlots(casterLevel, casterType);
        updateCreature(c => {
            c.spellSlots = Object.keys(slots).length > 0 ? slots : undefined;
        });
    };

    // --- Spell Slots ---
    const addSpellSlot = () => {
        updateCreature(c => {
            if (!c.spellSlots) c.spellSlots = {};
            const existingLevels = Object.keys(c.spellSlots).map(k => parseInt(k.replace('level_', '')) || 0);
            const nextLevel = existingLevels.length > 0 ? Math.max(...existingLevels) + 1 : 1;
            c.spellSlots[`level_${nextLevel}`] = 2;
        });
    };

    const updateSpellSlotLevel = (oldLevel: string, newLevel: string) => {
        if (oldLevel === newLevel) return;
        updateCreature(c => {
            if (c.spellSlots) {
                const count = c.spellSlots[oldLevel];
                delete c.spellSlots[oldLevel];
                c.spellSlots[newLevel] = count;
            }
        });
    };

    const updateSpellSlotCount = (level: string, newCount: number) => {
        updateCreature(c => {
            if (c.spellSlots) c.spellSlots[level] = newCount;
        });
    };

    const deleteSpellSlot = (level: string) => {
        updateCreature(c => {
            if (c.spellSlots) delete c.spellSlots[level];
        });
    };

    // --- Class Resources ---
    const addClassResource = () => {
        updateCreature(c => {
            if (!c.classResources) c.classResources = {};
            const existingCount = Object.keys(c.classResources).length;
            c.classResources[`Resource ${existingCount + 1}`] = 1;
        });
    };

    const updateClassResourceName = (oldName: string, newName: string) => {
        if (oldName === newName) return;
        updateCreature(c => {
            if (c.classResources) {
                const count = c.classResources[oldName];
                delete c.classResources[oldName];
                c.classResources[newName] = count;
            }
        });
    };

    const updateClassResourceCount = (name: string, newCount: number) => {
        updateCreature(c => {
            if (c.classResources) c.classResources[name] = newCount;
        });
    };

    const deleteClassResource = (name: string) => {
        updateCreature(c => {
            if (c.classResources) delete c.classResources[name];
        });
    };

    // Sort spell slots by level for display
    const sortedSpellSlots = value.spellSlots
        ? Object.entries(value.spellSlots).sort((a, b) => {
            const levelA = parseInt(a[0].replace('level_', '')) || 0;
            const levelB = parseInt(b[0].replace('level_', '')) || 0;
            return levelA - levelB;
        })
        : [];

    return (
        <div className={styles.resourceEditor}>
            <div className={styles.section}>
                <h3 className={styles.sectionHeader}>
                    Spell Slots
                    <div className={styles.headerButtons}>
                        <button
                            onClick={() => setShowCasterHelper(!showCasterHelper)}
                            title="Set slots from Caster Level"
                            className={showCasterHelper ? styles.activeBtn : ''}
                        >
                            <FontAwesomeIcon icon={faMagicWandSparkles} />
                        </button>
                        <button onClick={addSpellSlot} title="Add Spell Slot">
                            <FontAwesomeIcon icon={faPlus} />
                        </button>
                    </div>
                </h3>

                {showCasterHelper && (
                    <div className={styles.casterHelper}>
                        <div className={styles.casterRow}>
                            <label>Caster Level:</label>
                            <input
                                type="number"
                                min={1}
                                max={20}
                                value={casterLevel}
                                onChange={e => setCasterLevel(e.target.value === '' ? '' : parseInt(e.target.value))}
                                placeholder="1-20"
                                style={{ width: '60px' }}
                            />
                        </div>
                        <div className={styles.casterRow}>
                            <label>Caster Type:</label>
                            <select
                                value={casterType}
                                onChange={e => setCasterType(e.target.value as CasterType)}
                            >
                                {Object.entries(CASTER_TYPE_LABELS).map(([type, label]) => (
                                    <option key={type} value={type}>{label}</option>
                                ))}
                            </select>
                        </div>
                        <button
                            onClick={applyCasterLevel}
                            disabled={typeof casterLevel !== 'number' || casterLevel < 1}
                            className={styles.applyBtn}
                        >
                            Apply Spell Slots
                        </button>
                    </div>
                )}

                {sortedSpellSlots.map(([level, count]) => (
                    <div key={level} className={styles.resourceEntry}>
                        <label>Level:</label>
                        <input
                            type="text"
                            value={level.replace('level_', '')}
                            onChange={e => {
                                const newLevel = `level_${e.target.value}`;
                                updateSpellSlotLevel(level, newLevel);
                            }}
                            className={styles.resourceName}
                            style={{ width: '50px' }}
                        />
                        <label>Slots:</label>
                        <input
                            type="number"
                            value={count}
                            onChange={e => updateSpellSlotCount(level, parseInt(e.target.value) || 0)}
                            min={0}
                            style={{ width: '50px' }}
                        />
                        <button onClick={() => deleteSpellSlot(level)} title="Delete Spell Slot">
                            <FontAwesomeIcon icon={faTrash} />
                        </button>
                    </div>
                ))}
                {sortedSpellSlots.length === 0 && <p>No spell slots configured.</p>}
            </div>

            <div className={styles.section}>
                <h3 className={styles.sectionHeader}>
                    Class Resources
                    <button onClick={addClassResource} title="Add Class Resource">
                        <FontAwesomeIcon icon={faPlus} />
                    </button>
                </h3>
                {value.classResources && Object.entries(value.classResources).map(([name, count]) => (
                    <div key={name} className={styles.resourceEntry}>
                        <label>Name:</label>
                        <input
                            type="text"
                            value={name}
                            onChange={e => updateClassResourceName(name, e.target.value)}
                            className={styles.resourceName}
                            style={{ width: '120px' }}
                        />
                        <label>Uses:</label>
                        <input
                            type="number"
                            value={count}
                            onChange={e => updateClassResourceCount(name, parseInt(e.target.value) || 0)}
                            min={0}
                            style={{ width: '50px' }}
                        />
                        <button onClick={() => deleteClassResource(name)} title="Delete Class Resource">
                            <FontAwesomeIcon icon={faTrash} />
                        </button>
                    </div>
                ))}
                {(!value.classResources || Object.keys(value.classResources).length === 0) && <p>No class resources configured.</p>}
            </div>
        </div>
    );
};

export default ResourceEditor;