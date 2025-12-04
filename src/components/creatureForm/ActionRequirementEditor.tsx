import { FC } from 'react'
import { ActionRequirement } from '../../model/model'
import { ResourceTypeList } from '../../model/enums'
import Select from '../utils/select'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { faTrash, faPlus } from '@fortawesome/free-solid-svg-icons'
import styles from './actionForm.module.scss'

type Props = {
    value: ActionRequirement[]
    onChange: (newValue: ActionRequirement[]) => void
}

const RequirementTypes = ['ResourceAvailable', 'CombatState', 'StatusEffect', 'Custom'] as const
const CombatConditions = ['EnemyInRange', 'IsSurprised'] as const

const ActionRequirementEditor: FC<Props> = ({ value, onChange }) => {
    const addReq = () => {
        onChange([...value, { type: 'ResourceAvailable', resourceType: 'Action', amount: 1 }])
    }

    const removeReq = (index: number) => {
        const newValue = [...value]
        newValue.splice(index, 1)
        onChange(newValue)
    }

    const updateReq = (index: number, newReq: ActionRequirement) => {
        const newValue = [...value]
        newValue[index] = newReq
        onChange(newValue)
    }

    const changeType = (index: number, type: ActionRequirement['type']) => {
        const current = value[index]
        if (current.type === type) return

        let newReq: ActionRequirement
        switch (type) {
            case 'ResourceAvailable':
                newReq = { type, resourceType: 'Action', amount: 1 }
                break
            case 'CombatState':
                newReq = { type, condition: 'EnemyInRange', value: 5 }
                break
            case 'StatusEffect':
                newReq = { type, effect: '' }
                break
            case 'Custom':
                newReq = { type, description: '' }
                break
        }
        updateReq(index, newReq)
    }

    return (
        <div style={{ display: 'flex', flexDirection: 'column', width: '100%', margin: '4px 0' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                <span style={{ fontSize: '0.9em', fontWeight: 'bold' }}>Reqs:</span>
                <button onClick={addReq} title="Add Requirement">
                    <FontAwesomeIcon icon={faPlus} />
                </button>
            </div>
            {value.map((req, index) => (
                <div key={index} className={styles.modifier} style={{ marginTop: '4px' }}>
                    {/* Requirement Type Selector */}
                    <Select
                        value={req.type}
                        options={RequirementTypes.map(t => ({ value: t, label: t }))}
                        onChange={t => changeType(index, t)}
                    />

                    {/* Type specific fields */}
                    {req.type === 'ResourceAvailable' && (
                        <>
                            <Select
                                value={req.resourceType}
                                options={ResourceTypeList.map(r => ({ value: r, label: r }))}
                                onChange={rt => updateReq(index, { ...req, resourceType: rt })}
                            />
                            {['SpellSlot', 'ClassResource', 'ItemCharge', 'HitDice', 'Custom'].includes(req.resourceType) && (
                                <input
                                    type="text"
                                    value={req.resourceVal || ''}
                                    onChange={e => updateReq(index, { ...req, resourceVal: e.target.value })}
                                    placeholder={
                                        req.resourceType === 'SpellSlot' ? 'Level' :
                                            req.resourceType === 'ClassResource' ? 'Name' :
                                                req.resourceType === 'ItemCharge' ? 'Item' :
                                                    req.resourceType === 'HitDice' ? 'Die' :
                                                        'Value'
                                    }
                                    title="Resource Value (e.g. Spell Level, Resource Name)"
                                    style={{ width: '80px' }}
                                />
                            )}
                            <input
                                type="number"
                                value={req.amount}
                                onChange={e => updateReq(index, { ...req, amount: Number(e.target.value) })}
                                style={{ width: '45px' }}
                            />
                        </>
                    )}

                    {req.type === 'CombatState' && (
                        <>
                            <Select
                                value={req.condition}
                                options={CombatConditions.map(c => ({ value: c, label: c }))}
                                onChange={c => updateReq(index, { ...req, condition: c })}
                            />
                            <input
                                type="number"
                                value={req.value || 0}
                                onChange={e => updateReq(index, { ...req, value: Number(e.target.value) })}
                                placeholder="Val"
                                style={{ width: '45px' }}
                            />
                        </>
                    )}

                    {req.type === 'StatusEffect' && (
                        <input
                            type="text"
                            value={req.effect}
                            onChange={e => updateReq(index, { ...req, effect: e.target.value })}
                            placeholder="Effect Name"
                            style={{ minWidth: '100px' }}
                        />
                    )}

                    {req.type === 'Custom' && (
                        <input
                            type="text"
                            value={req.description}
                            onChange={e => updateReq(index, { ...req, description: e.target.value })}
                            placeholder="Description"
                            style={{ minWidth: '100px' }}
                        />
                    )}

                    <button onClick={() => removeReq(index)}>
                        <FontAwesomeIcon icon={faTrash} />
                    </button>
                </div>
            ))}
        </div>
    )
}

export default ActionRequirementEditor
