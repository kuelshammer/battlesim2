import { FC } from "react"
import styles from './simulation.module.scss'

type EventLogProps = {
    events: string[]
    title?: string
}

const EventLog: FC<EventLogProps> = ({ events, title = "Combat Events" }) => {
    if (!events || events.length === 0) {
        return (
            <div className={styles.eventLog}>
                <h3>{title}</h3>
                <div className={styles.eventLogContent}>
                    <p>No events to display. Run a simulation to see combat events.</p>
                </div>
            </div>
        )
    }

    return (
        <div className={styles.eventLog}>
            <h3>{title}</h3>
            <div className={styles.eventLogContent}>
                {events.map((event, index) => (
                    <div key={index} className={styles.eventLogEntry}>
                        {event}
                    </div>
                ))}
            </div>
        </div>
    )
}

export default EventLog