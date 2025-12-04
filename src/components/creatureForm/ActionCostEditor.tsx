import { FC } from 'react'
import { ActionCost } from '../../model/model'
import { ResourceTypeList } from '../../model/enums'
import Select from '../utils/select'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { faTrash, faPlus, faExchange } from '@fortawesome/free-solid-svg-icons' // faExchangeAlt? faRetweet? faRandom? Using faExchange for swapping types
import { faRandom } from '@fortawesome/free-solid-svg-icons'
import styles from './actionForm.module.scss'

type Props = {
    value: ActionCost[]
    onChange: (newValue: ActionCost[]) => void
}

const ActionCostEditor: FC<Props> = ({ value, onChange }) => {
    const addCost = () => {
        onChange([...value, { type: 'Discrete', resourceType: 'Action', amount: 1 }])
    }

    const removeCost = (index: number) => {
        const newValue = [...value]
        newValue.splice(index, 1)
        onChange(newValue)
    }

    const updateCost = (index: number, newCost: ActionCost) => {
        const newValue = [...value]
        newValue[index] = newCost
        onChange(newValue)
    }

    const toggleType = (index: number, cost: ActionCost) => {
        if (cost.type === 'Discrete') {
            updateCost(index, { type: 'Variable', resourceType: cost.resourceType, min: cost.amount, max: cost.amount })
        } else {
            updateCost(index, { type: 'Discrete', resourceType: cost.resourceType, amount: cost.min })
        }
    }

    return (
        <div style={{ display: 'flex', flexDirection: 'column', width: '100%', margin: '4px 0' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                <span style={{ fontSize: '0.9em', fontWeight: 'bold' }}>Costs:</span>
                <button onClick={addCost} title="Add Cost">
                    <FontAwesomeIcon icon={faPlus} />
                </button>
            </div>
            {value.map((cost, index) => (
                <div key={index} className={styles.modifier} style={{ marginTop: '4px' }}>
                    <Select
                        value={cost.resourceType}
                        options={ResourceTypeList.map(r => ({ value: r, label: r }))}
                        onChange={rt => updateCost(index, { ...cost, resourceType: rt })}
                    />

                    {['SpellSlot', 'ClassResource', 'ItemCharge', 'HitDice', 'Custom'].includes(cost.resourceType) && (
                        <input
                            type="text"
                            value={cost.resourceVal || ''}
                            onChange={e => updateCost(index, { ...cost, resourceVal: e.target.value })}
                            placeholder={
                                cost.resourceType === 'SpellSlot' ? 'Level' :
                                    cost.resourceType === 'ClassResource' ? 'Name' :
                                        cost.resourceType === 'ItemCharge' ? 'Item' :
                                            cost.resourceType === 'HitDice' ? 'Die' :
                                                'Value'
                            }
                            title="Resource Value (e.g. Spell Level, Resource Name)"
                            style={{ width: '80px' }}
                        />
                    )}

                    {cost.type === 'Discrete' ? (
                        <>
                            <input
                                type="number"
                                value={cost.amount}
                                onChange={e => updateCost(index, { ...cost, amount: Number(e.target.value) })}
                                title="Amount"
                                style={{ width: '45px' }}
                            />
                        </>
                    ) : (
                        <>
                            <input
                                type="number"
                                value={cost.min}
                                onChange={e => updateCost(index, { ...cost, min: Number(e.target.value) })}
                                placeholder="Min"
                                title="Min Amount"
                                style={{ width: '45px' }}
                            />
                            -
                            <input
                                type="number"
                                value={cost.max}
                                onChange={e => updateCost(index, { ...cost, max: Number(e.target.value) })}
                                placeholder="Max"
                                title="Max Amount"
                                style={{ width: '45px' }}
                            />
                        </>
                    )}

                    <button onClick={() => toggleType(index, cost)} title={cost.type === 'Discrete' ? "Switch to Variable" : "Switch to Fixed"}>
                        <FontAwesomeIcon icon={faRandom} color={cost.type === 'Variable' ? '#4CAF50' : undefined} />
                    </button>

                    <button onClick={() => removeCost(index)}>
                        <FontAwesomeIcon icon={faTrash} />
                    </button>
                </div>
            ))}
        </div>
    )
}

export default ActionCostEditor
