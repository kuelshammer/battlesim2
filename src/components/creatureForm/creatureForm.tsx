import { FC, useEffect, useState, useRef } from "react"
import { Creature, CreatureSchema } from "@/model/model"
import { clone } from "@/model/utils"
import styles from './creatureForm.module.scss'
import PlayerForm from "./playerForm"
import MonsterForm from "./monsterForm"
import CustomForm from "./customForm"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faCheck, faTrash, faWrench } from "@fortawesome/free-solid-svg-icons"
import { v4 as uuid } from 'uuid'
import Modal from "../utils/modal"
import DecimalInput from "@/utils/DecimalInput"
import * as Dialog from "@radix-ui/react-dialog"

type PropType = {
    onSubmit: (value: Creature) => void,
    onCancel: () => void,

    initialMode?: 'player'|'monster',
    initialValue?: Creature,
    onDelete?: () => void,
}

function newCreature(mode: 'player'|'monster'): Creature {
    return {
        id: uuid(),
        mode,
        name: (mode === 'player') ? 'Player Character' : 'Monster',
        ac: 10,
        saveBonus: 2,
        count: 1,
        hp: 10,
        actions: [],
        magicItems: [],
        initialBuffs: [],
    }
}

const CreatureForm:FC<PropType> = ({ initialMode, onSubmit, onCancel, initialValue, onDelete }) => {
    const [value, setValue] = useState<Creature>(initialValue || newCreature(initialMode || 'player'))
    const [isValid, setIsValid] = useState(false)
    const valueRef = useRef(value)
    
    // Track if fields were manually edited to avoid template overwrites
    const isNameDirty = useRef(initialValue !== undefined && initialValue.name !== 'Player Character' && initialValue.name !== 'Monster')

    useEffect(() => {
        valueRef.current = value
        if (!CreatureSchema.safeParse(value).success) {
            setIsValid(false)
            return
        }

        if (value.mode === 'player') {
            setIsValid(!!value.class)
        } else if (value.mode === 'monster') {
            setIsValid(!!value.cr)
        } else {
            setIsValid(true)
        }
    }, [value])

    function update(callback: (clonedValue: Creature) => void) {
        setValue(prev => {
            const clonedValue = clone(prev)
            callback(clonedValue)
            return clonedValue
        })
    }

    const handleChildChange = (newValue: Creature) => {
        setValue(prev => {
            const merged = { ...newValue };
            // If user manually edited name, preserve it unless child is specifically MonsterForm and just selected a monster
            if (isNameDirty.current && prev.mode === newValue.mode) {
                merged.name = prev.name;
            }
            return merged;
        });
    }

    return (
        <Modal onCancel={onCancel} className={styles.creatureForm} title={`Edit ${value.name}`} data-testid="creature-modal">
            <Dialog.Title className="sr-only">Edit Creature: {value.name}</Dialog.Title>
            <Dialog.Description className="sr-only">Form to edit creature statistics, actions and game plan.</Dialog.Description>
            
            <div className={styles.modes} role="tablist" aria-label="Creature type" data-testid="mode-toggle">
                <button
                    role="tab"
                    aria-selected={value.mode === 'player'}
                    aria-controls="creature-form-panel"
                    className={(value.mode === 'player') ? styles.active : undefined}
                    onClick={() => update(c => { c.mode = 'player' })}
                    data-testid="mode-player"
                >
                    Player Character
                </button>
                <button
                    role="tab"
                    aria-selected={value.mode === 'monster'}
                    aria-controls="creature-form-panel"
                    className={(value.mode === 'monster') ? styles.active : undefined}
                    onClick={() => update(c => { c.mode = 'monster' })}
                    data-testid="mode-monster"
                >
                    Monster
                </button>
                <button
                    role="tab"
                    aria-selected={value.mode === 'custom'}
                    aria-controls="creature-form-panel"
                    className={(value.mode === 'custom') ? styles.active : undefined}
                    onClick={() => update(c => { c.mode = 'custom' })}
                    data-testid="mode-custom"
                >
                    Custom
                </button>
            </div>

            <div className={styles.form} id="creature-form-panel" role="tabpanel">
                <section className={styles.commonFields}>
                    <div className={styles.row}>
                        <div className={styles.field}>
                            <h3>Name</h3>
                            <input
                                type='text'
                                value={value.name}
                                onChange={e => {
                                    isNameDirty.current = true;
                                    update(v => { v.name = e.target.value });
                                }}
                                placeholder="Creature Name"
                                data-testid="creature-name-input"
                            />
                        </div>
                        <div className={styles.field} style={{ width: '80px' }}>
                            <h3>Count</h3>
                            <input
                                type='number'
                                min={1} max={20}
                                value={value.count}
                                onChange={e => update(v => { v.count = Math.max(1, Math.min(20, Math.round(Number(e.target.value)))) })}
                                data-testid="count-input"
                            />
                        </div>
                    </div>
                    <div className={styles.row}>
                        <div className={styles.field}>
                            <h3>Hit Points</h3>
                            <DecimalInput min={0} value={value.hp} onChange={hp => update(v => { v.hp = hp || 0 })} data-testid="hp-input" />
                        </div>
                        <div className={styles.field}>
                            <h3>Armor Class</h3>
                            <DecimalInput min={0} value={value.ac} onChange={ac => update(v => { v.ac = ac || 0 })} data-testid="ac-input" />
                        </div>
                    </div>
                </section>

                { (value.mode === "player") ? (
                    <PlayerForm
                        value={value}
                        onChange={handleChildChange}
                    />
                ) : (value.mode === "monster") ? (
                    <MonsterForm
                        value={value}
                        onChange={(v) => {
                            isNameDirty.current = false; // Reset dirty flag when selecting a new monster
                            setValue(v);
                        }}
                    />
                ) : (
                    <CustomForm
                        value={value}
                        onChange={handleChildChange}
                    />
                )}
            </div>

            <div className={styles.buttons}>
                <button
                    onClick={onCancel}
                    aria-label="Cancel"
                    data-testid="cancel-creature-btn"
                >
                    Cancel
                </button>
                <div className="tooltipContainer">
                    <button
                        onClick={() => {
                            onSubmit(valueRef.current)
                        }}
                        disabled={!isValid}
                        style={{ width: '100%' }}
                        aria-label="Confirm changes"
                        data-testid="save-creature-btn"
                    >
                        <FontAwesomeIcon icon={faCheck} />
                        OK
                    </button>
                    
                    <span className="tooltip" role="tooltip">
                        { isValid 
                            ? "Save this creature for the current encounter"
                            : "Please fix the errors in the form before saving"
                        }
                    </span>
                </div>
                { (value.mode === 'custom') ? null : (
                    <div className="tooltipContainer">
                        <button 
                            onClick={() => setValue({...value, mode: 'custom'})} 
                            disabled={!isValid} 
                            aria-label="Advanced customization"
                        >
                            <FontAwesomeIcon icon={faWrench} />
                            Customize
                        </button>
                        <span className="tooltip" role="tooltip">
                            Go to the advanced editing mode
                        </span>
                    </div>
                )}
                { !onDelete ? null : (
                    <div className="tooltipContainer">
                        <button 
                            onClick={onDelete} 
                            aria-label="Delete creature"
                        >
                            <FontAwesomeIcon icon={faTrash} />
                            Delete
                        </button>
                        <span className="tooltip" role="tooltip">
                            Remove this creature from the current encounter
                        </span>
                    </div>
                )}
            </div>
        </Modal>
    )
}

export default CreatureForm