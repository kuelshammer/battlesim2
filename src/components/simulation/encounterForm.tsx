import { FC, useState, memo } from "react"
import { Creature, Encounter } from "@/model/model"
import styles from './encounterForm.module.scss'
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faChevronDown, faChevronUp, faPen, faPlus, faTrash, faMagicWandSparkles } from "@fortawesome/free-solid-svg-icons"
import CreatureForm from "./../creatureForm/creatureForm"
import { clone } from "@/model/utils"
import Checkbox from "@/utils/checkbox"
import Select from "react-select"

type PropType = {
    mode: 'player' | 'monster',
    encounter: Encounter,
    onUpdate: (newValue: Encounter) => void,
    onDelete?: () => void,
    onAutoAdjust?: () => void,
    autoAdjustDisabled?: boolean,
    children?: ReactNode,
    onMoveUp?: () => void,
    onMoveDown?: () => void,
    onEditingChange?: (isEditing: boolean) => void,
}

const EncounterForm: FC<PropType> = memo(({ mode, encounter, onUpdate, onDelete, onAutoAdjust, autoAdjustDisabled, children, onMoveUp, onMoveDown, onEditingChange }) => {
    const [updating, setUpdating] = useState<number | null>(null)
    const [creating, setCreating] = useState(false)

    const handleSetUpdating = (index: number | null) => {
        setUpdating(index);
        onEditingChange?.(index !== null || creating);
    };

    const handleSetCreating = (isCreating: boolean) => {
        setCreating(isCreating);
        onEditingChange?.(updating !== null || isCreating);
    };

    function createCreature(creature: Creature) {
        const encounterClone = clone(encounter)
        // Generate a new UUID to prevent React key duplication
        const newCreature = { ...creature, id: crypto.randomUUID() }
        encounterClone.monsters.push(newCreature)
        onUpdate(encounterClone)
        handleSetCreating(false)
    }

    function updateCreature(index: number, newValue: Creature) {
        const encounterClone = clone(encounter)
        encounterClone.monsters[index] = newValue
        onUpdate(encounterClone)
        handleSetUpdating(null)
    }

    function deleteCreature(index: number) {
        const encounterClone = clone(encounter)
        encounterClone.monsters.splice(index, 1)
        onUpdate(encounterClone)
        handleSetUpdating(null)
    }

    function update(callback: (encounterClone: Encounter) => void) {
        const encounterClone = clone(encounter)
        callback(encounterClone)
        onUpdate(encounterClone)
    }

    return (
        <>
            <div className={styles.encounterForm}>
                <div className={styles.encounterActions}>
                    {!!onDelete && (
                        <button onClick={onDelete} aria-label="Delete encounter">
                            <FontAwesomeIcon icon={faTrash} />
                        </button>
                    )}
                    {(onMoveUp || onMoveDown) && (
                        <button onClick={onMoveUp} disabled={!onMoveUp} aria-label="Move encounter up">
                            <FontAwesomeIcon icon={faChevronUp} />
                        </button>
                    )}
                    {(onMoveUp || onMoveDown) && (
                        <button onClick={onMoveDown} disabled={!onMoveDown} aria-label="Move encounter down">
                            <FontAwesomeIcon icon={faChevronDown} />
                        </button>
                    )}
                </div>

                <h2 className={`${styles.header} ${(mode === "player") ? styles.player : styles.monster}`}>
                    {(mode === 'player') ? 'Player Characters' : 'Encounter'}
                </h2>

                <div className={styles.formBody}>
                    <div className={styles.creatures} data-testid="creature-list">
                        {encounter.monsters.map((creature, index) => (
                            <div key={creature.id} className={styles.creature} data-testid="creature-item">
                                <span className={styles.name} data-testid="creature-name">{creature.name}</span>
                                <span className={styles.inlineInput}>
                                    <label htmlFor={`count-${creature.id}`} className={styles.countLabel}>Count:</label>
                                    <input
                                        id={`count-${creature.id}`}
                                        type='number'
                                        min={1} max={20} step={1}
                                        value={creature.count}
                                        onChange={e => updateCreature(index, { ...creature, count: Math.max(0, Math.min(20, Math.round(Number(e.target.value)))) })}
                                        aria-label={`${creature.name} count`}
                                    />
                                </span>
                                {!children && <span className={styles.inlineInput}>
                                    <label htmlFor={`arrival-${creature.id}`} className={styles.countLabel}>Arrives on round:</label>
                                    <input
                                        id={`arrival-${creature.id}`}
                                        type='number'
                                        min={1} max={19} step={1}
                                        value={creature.arrival || 1}
                                        onChange={e => updateCreature(index, { ...creature, arrival: Math.max(0, Math.min(20, Math.round(Number(e.target.value)))) })}
                                        aria-label={`${creature.name} arrival round`}
                                    />
                                </span>}
                                <button onClick={() => handleSetUpdating(index)} aria-label={`Edit ${creature.name}`}>
                                    <FontAwesomeIcon icon={faPen} />
                                    <span>Edit</span>
                                </button>
                                <button onClick={() => deleteCreature(index)} aria-label={`Delete ${creature.name}`} data-testid="remove-creature-btn">
                                    <FontAwesomeIcon icon={faTrash} />
                                </button>
                            </div>
                        ))}
                    </div>
                    <div className={styles.encounterSettings}>
                        {children || (encounter.monsters.length ? (
                            <>
                                <Checkbox value={!!encounter.playersSurprised} onToggle={() => update(e => { e.playersSurprised = !e.playersSurprised })}>
                                    The players are surprised
                                </Checkbox>
                                <Checkbox value={!!encounter.monstersSurprised} onToggle={() => update(e => { e.monstersSurprised = !e.monstersSurprised })}>
                                    The enemies are surprised
                                </Checkbox>
                                <div className={styles.roleSelection}>
                                    <label>Encounter Role:</label>
                                    <Select
                                        className={styles.roleSelect}
                                        value={{ value: encounter.targetRole || 'Standard', label: encounter.targetRole || 'Standard' }}
                                        options={TargetRoleList.map(r => ({ value: r, label: r }))}
                                        onChange={(val: any) => update(e => { e.targetRole = val?.value || 'Standard' })}
                                    />
                                </div>
                            </>
                        ) : null)}
                    </div>
                </div>

                <div className={styles.formFooter}>
                    <button className={styles.addCreatureBtn} onClick={() => handleSetCreating(true)} data-testid="add-creature-btn">
                        <FontAwesomeIcon icon={faPlus} />
                        Add {(mode === 'player') ? 'Player Character' : 'Enemy'}
                    </button>
                    {mode === 'monster' && onAutoAdjust && (
                        <button 
                            className={styles.autoAdjustBtn} 
                            onClick={onAutoAdjust}
                            disabled={autoAdjustDisabled}
                            title="Optimize this encounter's difficulty automatically"
                        >
                            <FontAwesomeIcon icon={faMagicWandSparkles} />
                            Auto-Adjust
                        </button>
                    )}
                </div>
            </div>

            {(updating === null) ? null : (
                <CreatureForm
                    initialMode={mode}
                    initialValue={encounter.monsters[updating]}
                    onCancel={() => handleSetUpdating(null)}
                    onSubmit={(newValue) => updateCreature(updating, newValue)}
                    onDelete={() => deleteCreature(updating)}
                />
            )}

            {!creating ? null : (
                <CreatureForm
                    initialMode={mode}
                    onCancel={() => handleSetCreating(false)}
                    onSubmit={createCreature}
                />
            )}
        </>
    )
})

export default EncounterForm