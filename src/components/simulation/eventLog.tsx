import { FC, useLayoutEffect, useRef } from "react"
import styles from './simulation.module.scss'
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faScroll, faFireFlameSimple } from "@fortawesome/free-solid-svg-icons"

type EventLogProps = {
    events: string[] // String events from backend
    title?: string
}

// The Grimoire Log - Avant-garde event display with arcane aesthetic
const EventLog: FC<EventLogProps> = ({ events, title = "Combat Chronicle" }) => {
    const eventCount = events.length
    const scrollRef = useRef<HTMLOListElement>(null)

    // Auto-scroll to bottom when new events arrive
    useLayoutEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight
        }
    }, [events])

    return (
        <div className={styles.grimoireLog}>
            {/* Asymmetric header with ornamental flourish */}
            <header className={styles.grimoireHeader}>
                <div className={styles.grimoireTitleGroup}>
                    <FontAwesomeIcon icon={faScroll} className={styles.quillIcon} aria-hidden="true" />
                    <div className={styles.titleColumn}>
                        <h2 className={styles.grimoireTitle}>{title}</h2>
                        <span className={styles.eventCounter}>{eventCount} entries inscribed</span>
                    </div>
                </div>
            </header>

            {/* Semantic ordered list for events - accessible to screen readers */}
            <ol
                ref={scrollRef}
                className={styles.grimoireContent}
                aria-label={`${title} - ${eventCount} events`}
            >
                {events.map((event, index) => (
                    <li
                        key={index}
                        className={styles.grimoireEntry}
                        // Cap animation delay at 600ms (first 20 items) for performance
                        style={{ animationDelay: `${Math.min(index * 30, 600)}ms` }}
                    >
                        {/* Ember icon for each entry - decorative */}
                        <FontAwesomeIcon icon={faFireFlameSimple} className={styles.ember} aria-hidden="true" />

                        {/* The event text */}
                        <span className={styles.chronicleText}>{event}</span>
                    </li>
                ))}

                {/* Empty state with atmospheric message */}
                {events.length === 0 && (
                    <li className={styles.emptyGrimoire}>
                        <FontAwesomeIcon icon={faScroll} className={styles.emptyScroll} aria-hidden="true" />
                        <p className={styles.emptyText}>The pages lie blank...</p>
                        <p className={styles.emptySubtext}>Await thy first tale of battle</p>
                    </li>
                )}
            </ol>

            {/* Decorative bottom flourish */}
            <div className={styles.grimoireFooter}>
                <span className={styles.adornment} aria-hidden="true">✧</span>
            </div>
        </div>
    )
}

export default EventLog