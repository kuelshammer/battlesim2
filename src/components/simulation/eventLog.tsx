import { FC, useState } from "react"
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

    return (
        <div className={styles.grimoireLog}>
            {/* Asymmetric header with ornamental flourish */}
            <header className={styles.grimoireHeader}>
                <div className={styles.grimoireTitleGroup}>
                    <FontAwesomeIcon icon={faScroll} className={styles.quillIcon} />
                    <div className={styles.titleColumn}>
                        <h2 className={styles.grimoireTitle}>{title}</h2>
                        <span className={styles.eventCounter}>{eventCount} entries inscribed</span>
                    </div>
                </div>
            </header>

            {/* Parchment-style content area with atmospheric depth */}
            <div className={styles.grimoireContent}>
                {events.map((event, index) => (
                    <div key={index} className={styles.grimoireEntry} style={{ animationDelay: `${index * 30}ms` }}>
                        {/* Runed number column - asymmetric */}
                        <span className={styles.runedNumber}>{String(index + 1).padStart(2, '0')}</span>

                        {/* Ember icon for each entry */}
                        <FontAwesomeIcon icon={faFireFlameSimple} className={styles.ember} />

                        {/* The event text */}
                        <span className={styles.chronicleText}>{event}</span>
                    </div>
                ))}

                {/* Empty state with atmospheric message */}
                {events.length === 0 && (
                    <div className={styles.emptyGrimoire}>
                        <FontAwesomeIcon icon={faScroll} className={styles.emptyScroll} />
                        <p className={styles.emptyText}>The pages lie blank...</p>
                        <p className={styles.emptySubtext}>Await thy first tale of battle</p>
                    </div>
                )}
            </div>

            {/* Decorative bottom flourish */}
            <div className={styles.grimoireFooter}>
                <span className={styles.adornment}>✧</span>
            </div>
        </div>
    )
}

export default EventLog