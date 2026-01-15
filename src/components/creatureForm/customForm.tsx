import { FC, useState } from "react"
import { Action, Creature } from "@/model/model"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faFolder, faPlus, faSave, faCog } from "@fortawesome/free-solid-svg-icons"
import styles from './customForm.module.scss'
import { clone } from "@/model/utils"
import ActionForm from "./actionForm"
import ResourceEditor from "./ResourceEditor"
import StrategyBuilder from "./StrategyBuilder"
import DecimalInput from "@/utils/DecimalInput"
import DiceFormulaInput from "@/utils/diceFormulaInput"
import { DiceFormula } from "@/model/model"
import Checkbox from "@/utils/checkbox"
import { v4 as uuid } from 'uuid'
import LoadCreatureForm, { saveCreature } from "./loadCreatureForm"
import SaveBonusModal from "./SaveBonusModal"
import ImportButton from "./ImportButton"

type PropType = {
    value: Creature,
    onChange: (newvalue: Creature) => void,
}

const CustomForm: FC<PropType> = ({ value, onChange }) => {
    const [isLoading, setIsLoading] = useState(false)
    const [isSaveModalOpen, setIsSaveModalOpen] = useState(false)

    function update(callback: (valueClone: Creature) => void) {
        const valueClone = clone(value)
        callback(valueClone)
        onChange(valueClone)
    }

    function handleImport(creature: Creature) {
        onChange({
            ...creature,
            id: value.id, // Keep the same ID
            mode: 'custom', // Force custom mode
            count: value.count // Preserve count
        });
    }

    function createAction() {
        update(v => {
            v.actions.push({
                id: uuid(),
                actionSlot: 0, // legacy field - default action
                cost: [], // empty cost array for new actions
                requirements: [], // empty requirements array
                tags: [], // empty tags array
                name: '',
                freq: 'at will',
                condition: 'default',
                targets: 1,
                type: 'atk',
                dpr: 0,
                toHit: 0,
                target: 'enemy with least HP',
            })
        })
    }

    function updateAction(index: number, newValue: Action) {
        update(v => { v.actions[index] = newValue })
    }

    function deleteAction(index: number) {
        update(v => { v.actions.splice(index, 1) })
    }

    const canSaveTemplate = !!localStorage && !!localStorage.getItem('useLocalStorage')

    // Check if any individual saves are overridden
    const hasDetailedSaves = value.strSaveBonus !== undefined ||
        value.dexSaveBonus !== undefined ||
        value.conSaveBonus !== undefined ||
        value.intSaveBonus !== undefined ||
        value.wisSaveBonus !== undefined ||
        value.chaSaveBonus !== undefined ||
        value.conSaveAdvantage ||
        value.saveAdvantage;

    return (
        <div className={styles.customForm}>
            <section>
                <div className={styles.nameContainer}>
                    <input
                        type='number'
                        min={1}
                        max={20}
                        value={value.count}
                        onChange={e => update(v => { v.count = Math.max(1, Math.min(20, Math.round(Number(e.target.value)))) })}
                        data-testid="count-input"
                        className={styles.countInput}
                        aria-label="Count"
                    />
                    {canSaveTemplate ? (
                        <>
                            <button onClick={() => saveCreature(value)}>
                                <FontAwesomeIcon icon={faSave} />
                                <span className={styles.btnText}>Save</span>
                            </button>
                            <button onClick={() => setIsLoading(true)}>
                                <FontAwesomeIcon icon={faFolder} />
                                <span className={styles.btnText}>Load</span>
                            </button>
                            <ImportButton 
                                onImport={handleImport}
                                className={styles.importBtn}
                            />
                        </>
                    ) : (
                        <ImportButton 
                            onImport={handleImport}
                            className={styles.importBtn}
                        />
                    )}
                </div>
            </section>
            <section>
                <h3>Hit Points</h3>
                <DecimalInput min={0} value={value.hp} onChange={hp => update(v => { v.hp = hp || 0 })} data-testid="hp-input" />
            </section>
            <section>
                <h3>Armor Class</h3>
                <DecimalInput min={0} value={value.ac} onChange={ac => update(v => { v.ac = ac || 0 })} data-testid="ac-input" />
            </section>
            <section className="tooltipContainer">
                <h3>Average Save Bonus</h3>
                <div className={styles.saveRow}>
                    <DecimalInput min={0} value={value.saveBonus} onChange={save => update(v => { v.saveBonus = save || 0 })} />
                    <button
                        onClick={() => setIsSaveModalOpen(true)}
                        className={styles.detailsBtn}
                        title="Configure individual saves and advantages"
                    >
                        <FontAwesomeIcon icon={faCog} />
                        <span>Details</span>
                        {hasDetailedSaves && <span className={styles.badge}>✓</span>}
                    </button>
                </div>
                <div className="tooltip">Average of all saves' bonuses. Click "Details" to set individual save bonuses and advantages (e.g., CON save advantage for Concentration).</div>
            </section>
            <section>
                <h3>Initiative Bonus</h3>
                <DiceFormulaInput value={value.initiativeBonus || 0} onChange={(init: DiceFormula | undefined) => update(v => { v.initiativeBonus = init || 0 })} />
            </section>
            <section>
                <h3>Initiative Advantage</h3>
                <Checkbox value={!!value.initiativeAdvantage} onToggle={() => update(v => { v.initiativeAdvantage = !v.initiativeAdvantage })} />
            </section>

            <ResourceEditor value={value} onChange={onChange} />

            <StrategyBuilder
                actions={value.actions}
                onReorder={(newActions) => update(v => { v.actions = newActions })}
            />

            <h3 className={styles.actionsHeader}>
                <span className={styles.label}>Actions (Detailed)</span>
                <button
                    onClick={createAction}
                    className={styles.createActionBtn}>
                    <FontAwesomeIcon icon={faPlus} />
                </button>
            </h3>
            <div className={styles.actions}>
                {value.actions.map((action, index) => (
                    <ActionForm
                        key={action.id}
                        value={action}
                        onChange={(a) => updateAction(index, a)}
                        onDelete={() => deleteAction(index)}
                        onMoveUp={(index <= 0) ? undefined : () => update(v => {
                            v.actions[index] = v.actions[index - 1]
                            v.actions[index - 1] = action
                        })}
                        onMoveDown={(index >= value.actions.length - 1) ? undefined : () => update(v => {
                            v.actions[index] = v.actions[index + 1]
                            v.actions[index + 1] = action
                        })}
                    />
                ))}
            </div>

            {isLoading ? (
                <LoadCreatureForm
                    onLoad={(creature) => { onChange(creature); setIsLoading(false) }}
                    onCancel={() => setIsLoading(false)} />
            ) : null}

            {isSaveModalOpen && (
                <SaveBonusModal
                    value={value}
                    onChange={onChange}
                    onClose={() => setIsSaveModalOpen(false)}
                />
            )}
        </div>
    )
}

export default CustomForm