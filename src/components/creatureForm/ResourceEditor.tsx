// src/components/creatureForm/ResourceEditor.tsx
import { FC } from 'react';
import { Creature } from '../../model/model';
import styles from './resourceEditor.module.scss'; // Creating a new style module
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faPlus, faTrash } from '@fortawesome/free-solid-svg-icons';
import DecimalInput from '../utils/DecimalInput';
import { clone } from '../../model/utils';

type Props = {
    value: Creature;
    onChange: (newValue: Creature) => void;
};

const ResourceEditor: FC<Props> = ({ value, onChange }) => {
    function updateCreature(callback: (creatureClone: Creature) => void) {
        const creatureClone = clone(value);
        callback(creatureClone);
        onChange(creatureClone);
    }

    // --- Spell Slots ---
    const addSpellSlot = () => {
        updateCreature(c => {
            if (!c.spellSlots) c.spellSlots = {};
            const nextLevel = (Object.keys(c.spellSlots).length + 1);
            c.spellSlots[`level_${nextLevel}`] = 1; // Default to level 1, 1 slot
        });
    };

    const updateSpellSlot = (level: string, newCount: number) => {
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
            c.classResources[`New Resource ${Object.keys(c.classResources).length + 1}`] = 1; // Default name, 1 count
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

    return (
        <div className={styles.resourceEditor}>
            <section>
                <h3 className={styles.sectionHeader}>
                    Spell Slots
                    <button onClick={addSpellSlot} title="Add Spell Slot">
                        <FontAwesomeIcon icon={faPlus} />
                    </button>
                </h3>
                {value.spellSlots && Object.entries(value.spellSlots).map(([level, count]) => (
                    <div key={level} className={styles.resourceEntry}>
                        <label>Level:</label>
                        {/* Assuming level is always 'level_X' or similar string, can be parsed to int for display */}
                        <input
                            type="text"
                            value={level}
                            // Allow editing level string, but warn about consistency if not 'level_X'
                            onChange={e => {
                                // For simplicity, we don't allow changing the key directly here without a more complex map management.
                                // Instead, we might focus on 'count' and just display 'level'.
                                // If actual level editing is needed, consider an array or more complex state.
                            }}
                            disabled // Disable editing the key for now
                            className={styles.resourceName}
                        />
                        <label>Count:</label>
                        <DecimalInput
                            value={count}
                            onChange={newCount => updateSpellSlot(level, newCount || 0)}
                            min={0}
                        />
                        <button onClick={() => deleteSpellSlot(level)} title="Delete Spell Slot">
                            <FontAwesomeIcon icon={faTrash} />
                        </button>
                    </div>
                ))}
                {(!value.spellSlots || Object.keys(value.spellSlots).length === 0) && <p>No spell slots configured.</p>}
            </section>

            <section>
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
                        />
                        <label>Count:</label>
                        <DecimalInput
                            value={count}
                            onChange={newCount => updateClassResourceCount(name, newCount || 0)}
                            min={0}
                        />
                        <button onClick={() => deleteClassResource(name)} title="Delete Class Resource">
                            <FontAwesomeIcon icon={faTrash} />
                        </button>
                    </div>
                ))}
                {(!value.classResources || Object.keys(value.classResources).length === 0) && <p>No class resources configured.</p>}
            </section>
        </div>
    );
};

export default ResourceEditor;