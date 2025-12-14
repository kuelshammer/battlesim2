import { FC } from 'react'
import { ActionTag } from '@/model/enums'
import { ActionTagList } from '@/model/enums'
import Select from '@/utils/select'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { faTimes, faPlus } from '@fortawesome/free-solid-svg-icons'

type Props = {
    value: ActionTag[]
    onChange: (newValue: ActionTag[]) => void
}

const TagSelector: FC<Props> = ({ value, onChange }) => {
    const addTag = (tag: ActionTag) => {
        if (!value.includes(tag)) {
            onChange([...value, tag])
        }
    }

    const removeTag = (tagToRemove: ActionTag) => {
        onChange(value.filter(tag => tag !== tagToRemove))
    }

    // Filter available tags to exclude already selected ones
    const availableTags = ActionTagList.filter(tag => !value.includes(tag))

    return (
        <div style={{ display: 'flex', flexDirection: 'column', width: '100%', margin: '4px 0' }}>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px', marginBottom: '4px' }}>
                {value.map(tag => (
                    <span 
                        key={tag} 
                        style={{ 
                            background: '#445', 
                            color: '#eee',
                            borderRadius: '12px', 
                            padding: '2px 8px', 
                            fontSize: '0.85em',
                            display: 'flex',
                            alignItems: 'center',
                            gap: '4px'
                        }}
                    >
                        {tag}
                        <FontAwesomeIcon 
                            icon={faTimes} 
                            style={{ cursor: 'pointer', fontSize: '0.8em' }} 
                            onClick={() => removeTag(tag)}
                        />
                    </span>
                ))}
            </div>
            
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                 <span style={{ fontSize: '0.9em', fontWeight: 'bold' }}>Tags:</span>
                 <div style={{ width: '200px' }}>
                    <Select
                        value={availableTags[0]} // Placeholder, essentially
                        options={availableTags.map(t => ({ value: t, label: t }))}
                        onChange={addTag}
                    />
                 </div>
            </div>
        </div>
    )
}

export default TagSelector
