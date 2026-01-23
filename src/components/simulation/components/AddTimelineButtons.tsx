import React, { memo } from "react"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faPlus, faBed } from "@fortawesome/free-solid-svg-icons"
import styles from '../simulation.module.scss'

interface AddTimelineButtonsProps {
    createCombat: () => void
    createShortRest: () => void
}

export const AddTimelineButtons = memo<AddTimelineButtonsProps>(({
    createCombat,
    createShortRest
}) => {
    return (
        <div className={styles.addButtons} data-testid="add-timeline-buttons">
            <button
                onClick={createCombat}
                className={styles.addEncounterBtn}
                data-testid="add-combat-btn"
            >
                <FontAwesomeIcon icon={faPlus} />
                Add Combat
            </button>
            <button
                onClick={createShortRest}
                className={`${styles.addEncounterBtn} ${styles.restBtn}`}
                data-testid="add-rest-btn"
            >
                <FontAwesomeIcon icon={faBed} />
                Add Short Rest
            </button>
        </div>
    )
})

AddTimelineButtons.displayName = 'AddTimelineButtons'