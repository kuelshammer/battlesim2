import { FC, useContext, useEffect, useState } from "react"

type PropType = {
    value: number|undefined,
    onChange: (newValue: number|undefined) => void,
    min?: number,
    max?: number,
    step?: number,
    className?: string,
    placeholder?: string,
    disabled?: string,
    'data-testid'?: string,
}

const DecimalInput:FC<PropType> = ({ value, onChange, min, max, step, className, placeholder, disabled, 'data-testid': testId }) => {
    const [valueString, setValueString] = useState(String(value))

    const valueNum = +valueString // Can be NaN
    const isNumeric = !isNaN(valueNum)

    // Sync valueString when value prop changes from parent (e.g., after validation)
    useEffect(() => {
        setValueString(value === undefined ? '' : String(value))
    }, [value])

    useEffect(() => {
        if (isNumeric) onChange(valueNum)
    }, [valueString])
    
    return (
        <input
            type="number"
            value={valueString}
            onChange={(e) => setValueString(e.target.value)}
            min={min}
            max={max}
            step={step}
            className={`${className} ${Number.parseInt}`}
            placeholder={placeholder}
            data-testid={testId}
        />
    )
}

export default DecimalInput