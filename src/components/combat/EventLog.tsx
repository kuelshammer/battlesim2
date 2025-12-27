import { FC, useState, useMemo } from 'react';
import { Event } from '@/model/model';
import styles from './EventLog.module.scss';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
    faFistRaised,
    faSkull,
    faMagic,
    faShieldAlt,
    faHeart,
    faHourglassStart,
    faHourglassEnd,
    faExclamationTriangle,
    faRunning,
    faBan,
    faEye,
    faEyeSlash,
    faChevronDown,
    faChevronRight
} from '@fortawesome/free-solid-svg-icons';
import { useUIToggle } from '@/model/uiToggleState';
import { LogFormatter } from '@/model/logFormatter';

type Props = {
    events: Event[];
    combatantNames: Record<string, string>; // Map ID to Name for display
    actionNames?: Record<string, string>; // Map ActionID to Name
    isModal?: boolean; // New prop to handle modal context
};

type EventFilter = 'all' | 'combat' | 'spell' | 'status' | 'lifecycle';

const EventLog: FC<Props> = ({ events, combatantNames, actionNames = {}, isModal = false }) => {
    const [filter, setFilter] = useState<EventFilter>('all');
    const [combatLogVisible, setCombatLogVisible] = useUIToggle('combat-log');
    const [expandedEvents, setExpandedEvents] = useState<Set<number>>(new Set());

    const toggleEvent = (index: number) => {
        const newExpanded = new Set(expandedEvents);
        if (newExpanded.has(index)) {
            newExpanded.delete(index);
        } else {
            newExpanded.add(index);
        }
        setExpandedEvents(newExpanded);
    };

    const getActionName = (id: string) => actionNames[id] || id;

    const filteredEvents = useMemo(() => {
        if (filter === 'all') return events;

        return events.filter(e => {
            switch (filter) {
                case 'combat':
                    return ['AttackHit', 'AttackMissed', 'DamageTaken', 'DamagePrevented', 'ActionStarted', 'ActionSkipped'].includes(e.type);
                case 'spell':
                    return ['SpellCast', 'SpellSaved', 'ConcentrationBroken', 'SpellFailed'].includes(e.type);
                case 'status':
                    return ['BuffApplied', 'BuffExpired', 'ConditionAdded', 'ConditionRemoved', 'HealingApplied', 'TempHPGranted'].includes(e.type);
                case 'lifecycle':
                    return ['UnitDied', 'TurnStarted', 'TurnEnded', 'RoundStarted', 'RoundEnded', 'EncounterStarted', 'EncounterEnded'].includes(e.type);
                default:
                    return true;
            }
        });
    }, [events, filter]);

    const renderEvent = (event: Event, index: number) => {
        const isExpanded = expandedEvents.has(index);
        const hasDetails = ['AttackHit', 'AttackMissed', 'ActionStarted'].includes(event.type); // Events with enriched data

        let icon = faExclamationTriangle;
        let eventClass = '';

        switch (event.type) {
            case 'ActionStarted':
                icon = faFistRaised;
                eventClass = styles.action;
                break;
            case 'ActionSkipped':
                icon = faExclamationTriangle;
                eventClass = styles.skipped;
                break;
            case 'AttackHit':
                icon = faFistRaised;
                eventClass = styles.hit;
                break;
            case 'AttackMissed':
                icon = faBan;
                eventClass = styles.miss;
                break;
            case 'DamageTaken':
                icon = faHeart;
                eventClass = styles.damage;
                break;
            case 'HealingApplied':
                icon = faHeart;
                eventClass = styles.heal;
                break;
            case 'UnitDied':
                icon = faSkull;
                eventClass = styles.death;
                break;
            case 'RoundStarted':
                icon = faHourglassStart;
                eventClass = styles.round;
                break;
            case 'TurnStarted':
                icon = faRunning;
                break;
            case 'SpellCast':
                icon = faMagic;
                eventClass = styles.spell;
                break;
            case 'BuffApplied':
                icon = faShieldAlt;
                eventClass = styles.buff;
                break;
            case 'ConditionAdded':
                icon = faExclamationTriangle;
                eventClass = styles.buff;
                break;
            case 'ConditionRemoved':
                icon = faShieldAlt;
                eventClass = styles.buff;
                break;
            case 'EncounterEnded':
                icon = faHourglassEnd;
                eventClass = styles.round;
                break;
            case 'TurnEnded':
            case 'RoundEnded':
                return null;
        }

        const summary = LogFormatter.toSummary(event, combatantNames, actionNames);
        const details = hasDetails ? LogFormatter.toDetails(event, combatantNames, actionNames) : null;

        return (
            <div
                key={index}
                className={`${styles.event} ${eventClass} ${hasDetails ? styles.clickable : ''}`}
                onClick={hasDetails ? () => toggleEvent(index) : undefined}
            >
                <div className={styles.eventContent}>
                    <div className={styles.eventSummary}>
                        <FontAwesomeIcon icon={icon} className={event.type === 'AttackHit' ? styles.iconHit : event.type === 'AttackMissed' ? styles.iconMiss : ''} />
                        <span>{summary}</span>
                        {hasDetails && (
                            <FontAwesomeIcon
                                icon={isExpanded ? faChevronDown : faChevronRight}
                                style={{ marginLeft: 'auto', fontSize: '0.7rem', opacity: 0.5 }}
                            />
                        )}
                    </div>
                    {isExpanded && details && (
                        <div className={styles.eventDetails}>
                            {details}
                        </div>
                    )}
                </div>
            </div>
        );
    };

    if (!combatLogVisible && !isModal) {
        return (
            <div className={styles.eventLogContainer}>
                <div className={styles.header}>
                    <h3>Combat Log</h3>
                    <div className={styles.toggleContainer}>
                        <button
                            onClick={() => setCombatLogVisible(true)}
                            className={styles.toggleButton}
                            aria-label="Show combat log"
                            title="Show combat log"
                        >
                            <FontAwesomeIcon icon={faEye} />
                            <span>Show Log</span>
                        </button>
                    </div>
                </div>
                <div className={styles.emptyState}>
                    <FontAwesomeIcon icon={faEyeSlash} className={styles.emptyIcon} />
                    <p>Combat log is hidden</p>
                    <p className={styles.emptyHint}>Click "Show Log" to view combat events</p>
                </div>
            </div>
        );
    }

    return (
        <div className={`${styles.eventLogContainer} ${isModal ? styles.modalMode : ''}`}>
            {!isModal && (
                <div className={styles.header}>
                    <h3>Combat Log</h3>
                    <div className={styles.controls}>
                        <div className={styles.filters}>
                            <button className={filter === 'all' ? styles.active : ''} onClick={() => setFilter('all')}>All</button>
                            <button className={filter === 'combat' ? styles.active : ''} onClick={() => setFilter('combat')}>Combat</button>
                            <button className={filter === 'spell' ? styles.active : ''} onClick={() => setFilter('spell')}>Magic</button>
                            <button className={filter === 'status' ? styles.active : ''} onClick={() => setFilter('status')}>Status</button>
                        </div>
                        <div className={styles.toggleContainer}>
                            <button
                                onClick={() => setCombatLogVisible(false)}
                                className={styles.toggleButton}
                                aria-label="Hide combat log"
                                title="Hide combat log"
                            >
                                <FontAwesomeIcon icon={faEyeSlash} />
                                <span>Hide Log</span>
                            </button>
                        </div>
                    </div>
                </div>
            )}
            <div className={styles.logBody}>
                {filteredEvents.length === 0 ? (
                    <div className={styles.emptyState}>
                        <p>No events to display</p>
                        {filter !== 'all' && (
                            <p className={styles.emptyHint}>Try changing the filter or running a simulation</p>
                        )}
                    </div>
                ) : (
                    filteredEvents.map((e, i) => renderEvent(e, i))
                )}
            </div>
        </div>
    );
};

export default EventLog;
