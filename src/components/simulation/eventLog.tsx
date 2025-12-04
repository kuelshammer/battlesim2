import { FC, useState } from "react"
import styles from './simulation.module.scss'
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faBolt } from "@fortawesome/free-solid-svg-icons"

type EventLogProps = {
    events: string[] // String events from backend
    title?: string
}

// Simplified EventLog that just displays string events
const EventLog: FC<EventLogProps> = ({ events, title = "Combat Events" }) => {
    return (
        <div className={styles.eventLog}>
            <h3>{title}</h3>
            <div className={styles.eventLogContent}>
                {events.map((event, index) => (
                    <div key={index} className={styles.eventLogEntry}>
                        <FontAwesomeIcon icon={faBolt} fixedWidth /> {event}
                    </div>
                ))}
                {events.length === 0 && <p>No events recorded.</p>}
            </div>
        </div>
    )
}

export default EventLog