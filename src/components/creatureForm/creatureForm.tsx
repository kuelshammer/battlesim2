import { FC, useEffect, useState } from "react"
import { Creature, CreatureSchema } from "@/model/model"
import creatureForm from './creatureForm.module.scss'
import { clone } from "@/model/utils"
import styles from './creatureForm.module.scss'
import PlayerForm from "./playerForm"
import MonsterForm from "./monsterForm"
import CustomForm from "./customForm"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faCheck, faTrash, faWrench } from "@fortawesome/free-solid-svg-icons"
import { v4 as uuid } from 'uuid'
import Modal from "../utils/modal"

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
    }
}

const CreatureForm:FC<PropType> = ({ initialMode, onSubmit, onCancel, initialValue, onDelete }) => {
    const [value, setValue] = useState<Creature>(initialValue || newCreature(initialMode || 'player'))
    const [isValid, setIsValid] = useState(false)

    useEffect(() => {
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

    function update(callback: (clonedValue: Creature) => void, condition?: boolean) {
        if (condition === false) return
        const clonedValue = clone(value)
        callback(clonedValue)
        setValue(clonedValue)
    }

    return (
        <Modal onCancel={onCancel} className={styles.creatureForm} title={`Edit ${value.name}`}>
            <div className={styles.modes} role="tablist" aria-label="Creature type">
                <button
                    role="tab"
                    aria-selected={value.mode === 'player'}
                    aria-controls="creature-form-panel"
                    className={(value.mode === 'player') ? styles.active : undefined}
                    onClick={() => update(c => { c.mode = 'player' })}
                >
                    Player Character
                </button>
                <button
                    role="tab"
                    aria-selected={value.mode === 'monster'}
                    aria-controls="creature-form-panel"
                    className={(value.mode === 'monster') ? styles.active : undefined}
                    onClick={() => update(c => { c.mode = 'monster' })}
                >
                    Monster
                </button>
                <button
                    role="tab"
                    aria-selected={value.mode === 'custom'}
                    aria-controls="creature-form-panel"
                    className={(value.mode === 'custom') ? styles.active : undefined}
                    onClick={() => update(c => { c.mode = 'custom' })}
                >
                    Custom
                </button>
            </div>

            <div className={styles.form} id="creature-form-panel" role="tabpanel">
                { (value.mode === "player") ? (
                    <PlayerForm
                        value={value}
                        onChange={(v) => setValue({ ...v, id: value.id })}
                    />
                ) : (value.mode === "monster") ? (
                    <MonsterForm
                        value={value}
                        onChange={(v) => setValue({ ...v, id: value.id })}
                    />
                ) : (
                    <CustomForm
                        value={value}
                        onChange={(v) => setValue({ ...v, id: value.id })}
                    />
                )}
            </div>

            <div className={styles.buttons}>
                <div className="tooltipContainer">
                    <button 
                        onClick={() => {
                            onSubmit(value)
                        }} 
                        disabled={!isValid}
                        style={{ width: '100%' }}
                        aria-label="Confirm changes"
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