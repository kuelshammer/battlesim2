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

    const getEventClass = (event: string): string => {
        if (event.includes('Reaction') || event.includes('reaction')) {
            return styles.eventReaction
        }
        if (event.includes('Attack') || event.includes('Damage')) {
            return styles.eventAttack
        }
        if (event.includes('Heal')) {
            return styles.eventHeal
        }
        if (event.includes('Buff') || event.includes('Effect')) {
            return styles.eventEffect
        }
        return ''
    }

    const formatEvent = (event: string): string => {
        // Make events more readable by adding spacing around key elements
        return event
            .replace(/([A-Z][a-z]+)/g, ' $1')
            .replace(/\s+/g, ' ')
            .trim()
    }

    return (
        <div className={styles.eventLog}>
            <h3>{title}</h3>
            <div className={styles.eventLogContent}>
                {events.map((event, index) => (
                    <div
                        key={index}
                        className={`${styles.eventLogEntry} ${getEventClass(event)}`}
                    >
                        {formatEvent(event)}
                    </div>
                ))}
            </div>
        </div>
    )
}

export default EventLog