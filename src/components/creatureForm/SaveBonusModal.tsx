// src/components/creatureForm/SaveBonusModal.tsx
import { FC, useState } from 'react';
import { Creature } from '../../model/model';
import { clone } from '../../model/utils';
import styles from './saveBonusModal.module.scss';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faTimes, faCheck } from '@fortawesome/free-solid-svg-icons';
import Checkbox from '../utils/checkbox';

type Props = {
    value: Creature;
    onChange: (newValue: Creature) => void;
    onClose: () => void;
};

const SAVES = [
    { key: 'strSaveBonus', label: 'Strength', short: 'STR' },
    { key: 'dexSaveBonus', label: 'Dexterity', short: 'DEX' },
    { key: 'conSaveBonus', label: 'Constitution', short: 'CON' },
    { key: 'intSaveBonus', label: 'Intelligence', short: 'INT' },
    { key: 'wisSaveBonus', label: 'Wisdom', short: 'WIS' },
    { key: 'chaSaveBonus', label: 'Charisma', short: 'CHA' },
] as const;

type SaveKey = typeof SAVES[number]['key'];

const SaveBonusModal: FC<Props> = ({ value, onChange, onClose }) => {
    const [localValue, setLocalValue] = useState<Creature>(clone(value));

    const getSaveValue = (key: SaveKey): number => {
        const specific = localValue[key];
        return specific ?? localValue.saveBonus;
    };

    const updateSave = (key: SaveKey, newValue: number | undefined) => {
        const updated = clone(localValue);
        if (newValue === undefined || newValue === updated.saveBonus) {
            // Clear the override if it matches the average
            delete (updated as any)[key];
        } else {
            (updated as any)[key] = newValue;
        }
        setLocalValue(updated);
    };

    const handleSave = () => {
        onChange(localValue);
        onClose();
    };

    const hasAnyOverride = SAVES.some(s => localValue[s.key] !== undefined);

    return (
        <div className={styles.modalOverlay} onClick={onClose}>
            <div className={styles.modal} onClick={e => e.stopPropagation()}>
                <div className={styles.header}>
                    <h2>Saving Throw Bonuses</h2>
                    <button onClick={onClose} className={styles.closeBtn}>
                        <FontAwesomeIcon icon={faTimes} />
                    </button>
                </div>

                <div className={styles.content}>
                    <div className={styles.averageSection}>
                        <label>Average Save Bonus (Used when no specific value set):</label>
                        <input
                            type="number"
                            value={localValue.saveBonus}
                            onChange={e => {
                                const updated = clone(localValue);
                                updated.saveBonus = parseInt(e.target.value) || 0;
                                setLocalValue(updated);
                            }}
                            className={styles.averageInput}
                        />
                    </div>

                    <div className={styles.divider} />

                    <h3>Individual Saves (Optional Overrides)</h3>
                    <p className={styles.hint}>
                        Leave blank to use the average. Set a specific value to override.
                    </p>

                    <div className={styles.savesGrid}>
                        {SAVES.map(save => {
                            const specificValue = localValue[save.key];
                            const isOverridden = specificValue !== undefined;

                            return (
                                <div key={save.key} className={styles.saveRow}>
                                    <span className={styles.saveLabel}>
                                        <strong>{save.short}</strong> ({save.label})
                                    </span>
                                    <input
                                        type="number"
                                        value={specificValue ?? ''}
                                        placeholder={String(localValue.saveBonus)}
                                        onChange={e => {
                                            const val = e.target.value === '' ? undefined : parseInt(e.target.value);
                                            updateSave(save.key, val);
                                        }}
                                        className={`${styles.saveInput} ${isOverridden ? styles.overridden : ''}`}
                                    />
                                    {save.key === 'conSaveBonus' && (
                                        <label className={styles.advantageLabel}>
                                            <Checkbox
                                                value={!!localValue.conSaveAdvantage}
                                                onToggle={() => {
                                                    const updated = clone(localValue);
                                                    updated.conSaveAdvantage = !updated.conSaveAdvantage || undefined;
                                                    setLocalValue(updated);
                                                }}
                                            />
                                            <span>Advantage (Concentration)</span>
                                        </label>
                                    )}
                                </div>
                            );
                        })}
                    </div>

                    <div className={styles.divider} />

                    <div className={styles.globalAdvantage}>
                        <label>
                            <Checkbox
                                value={!!localValue.saveAdvantage}
                                onToggle={() => {
                                    const updated = clone(localValue);
                                    updated.saveAdvantage = !updated.saveAdvantage || undefined;
                                    setLocalValue(updated);
                                }}
                            />
                            <span>Advantage on ALL Saving Throws</span>
                            <span className={styles.hint}>(e.g., Paladin Aura of Protection effect)</span>
                        </label>
                    </div>
                </div>

                <div className={styles.footer}>
                    <button onClick={onClose} className={styles.cancelBtn}>Cancel</button>
                    <button onClick={handleSave} className={styles.saveBtn}>
                        <FontAwesomeIcon icon={faCheck} /> Save
                    </button>
                </div>
            </div>
        </div>
    );
};

export default SaveBonusModal;
