import { FC, useLayoutEffect, useRef, useMemo } from "react"
import styles from './simulation.module.scss'
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faScroll, faFireFlameSimple } from "@fortawesome/free-solid-svg-icons"

type EventLogProps = {
    events: string[] // String events from backend
    title?: string
}

/**
 * Parse event text to apply semantic styling.
 * Patterns detected:
 * - Creature names (capitalized words at start, or before "attacks", "misses", "takes", etc.)
 * - Damage numbers (digits followed by "damage" or "HP")
 * - Healing amounts (digits in context of "healed")
 * - Keywords: "CRIT", "HIT", "miss"
 */
function parseEventText(text: string): React.ReactNode {
    // Split into parts and apply styling
    const parts: React.ReactNode[] = []
    let lastIndex = 0

    // Pattern: number + (damage|HP) -> highlight as damage
    const damageRegex = /(\d+(?:\.\d+)?)\s*(?:damage|HP)/gi
    // Pattern: number + healed -> highlight as heal
    const healRegex = /(\d+(?:\.\d+)?)\s*(?:healed|healing)/gi
    // Pattern: creature name at start (Capitalized word)
    const nameRegex = /^([A-Z][a-zA-Z]+(?:\s+[A-Z][a-zA-Z]+)?)\s+(attacks|misses|takes|is|casts|gains|falls|starts|skipped|uses)/
    // Pattern: "HIT", "miss", "CRIT"
    const keywordRegex = /\b(CRIT|HIT|miss)\b/g

    let match
    const matches: Array<{ start: number; end: number; type: string; content: string }> = []

    // Find all damage matches
    while ((match = damageRegex.exec(text)) !== null) {
        matches.push({ start: match.index, end: match.index + match[0].length, type: 'damage', content: match[0] })
    }
    damageRegex.lastIndex = 0

    // Find all heal matches
    while ((match = healRegex.exec(text)) !== null) {
        matches.push({ start: match.index, end: match.index + match[1].length, type: 'heal', content: match[1] })
    }
    healRegex.lastIndex = 0

    // Find creature name at start
    const nameMatch = text.match(nameRegex)
    if (nameMatch) {
        const name = nameMatch[1]
        matches.push({ start: 0, end: name.length, type: 'name', content: name })
    }

    // Find keywords (HIT, CRIT, miss)
    while ((match = keywordRegex.exec(text)) !== null) {
        const keyword = match[0]
        const type = keyword === 'miss' ? 'hit' : (keyword === 'CRIT' ? 'crit' : 'hit')
        matches.push({ start: match.index, end: match.index + keyword.length, type, content: keyword })
    }
    keywordRegex.lastIndex = 0

    // Sort matches by position
    matches.sort((a, b) => a.start - b.start)

    // Build parsed output
    matches.forEach((m, i) => {
        // Add text before this match
        if (m.start > lastIndex) {
            parts.push(text.slice(lastIndex, m.start))
        }

        // Add styled match
        const className = styles[m.type as keyof typeof styles] || ''
        parts.push(<span key={i} className={className} dangerouslySetInnerHTML={{ __html: m.content }} />)

        lastIndex = m.end
    })

    // Add remaining text
    if (lastIndex < text.length) {
        parts.push(text.slice(lastIndex))
    }

    return parts.length > 0 ? parts : text
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

    // Memoize parsed events to avoid re-parsing on every render
    const parsedEvents = useMemo(() => {
        return events.map(event => parseEventText(event))
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

                        {/* The parsed event text with semantic highlighting */}
                        <span className={styles.chronicleText}>{parsedEvents[index]}</span>
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