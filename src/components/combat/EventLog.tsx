import { FC, useState, useMemo } from 'react';
import { Event } from '../../model/model';
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
    faBan
} from '@fortawesome/free-solid-svg-icons';

type Props = {
    events: Event[];
    combatantNames: Record<string, string>; // Map ID to Name for display
};

type EventFilter = 'all' | 'combat' | 'spell' | 'status' | 'lifecycle';

const EventLog: FC<Props> = ({ events, combatantNames }) => {
    const [filter, setFilter] = useState<EventFilter>('all');

    const getName = (id: string) => combatantNames[id] || id;

    const filteredEvents = useMemo(() => {
        if (filter === 'all') return events;

        return events.filter(e => {
            switch (filter) {
                case 'combat':
                    return ['AttackHit', 'AttackMissed', 'DamageTaken', 'DamagePrevented', 'ActionStarted'].includes(e.type);
                case 'spell':
                    return ['SpellCast', 'SpellSaved', 'ConcentrationBroken'].includes(e.type);
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
        switch (event.type) {
            case 'ActionStarted':
                return (
                    <div key={index} className={`${styles.event} ${styles.action}`}>
                        <FontAwesomeIcon icon={faFistRaised} />
                        <span><strong>{getName(event.actor_id)}</strong> uses <strong>{event.action_id}</strong></span>
                    </div>
                );
            case 'AttackHit':
                return (
                    <div key={index} className={`${styles.event} ${styles.hit}`}>
                        <FontAwesomeIcon icon={faFistRaised} className={styles.iconHit} />
                        <span>Attack hits <strong>{getName(event.target_id)}</strong> for <strong>{event.damage.toFixed(1)}</strong> damage!</span>
                    </div>
                );
            case 'AttackMissed':
                return (
                    <div key={index} className={`${styles.event} ${styles.miss}`}>
                        <FontAwesomeIcon icon={faBan} className={styles.iconMiss} />
                        <span>Attack missed <strong>{getName(event.target_id)}</strong></span>
                    </div>
                );
            case 'DamageTaken':
                return (
                    <div key={index} className={`${styles.event} ${styles.damage}`}>
                        <FontAwesomeIcon icon={faHeart} className={styles.iconDamage} />
                        <span><strong>{getName(event.target_id)}</strong> takes <strong>{event.damage.toFixed(1)}</strong> {event.damage_type} damage</span>
                    </div>
                );
            case 'HealingApplied':
                return (
                    <div key={index} className={`${styles.event} ${styles.heal}`}>
                        <FontAwesomeIcon icon={faHeart} className={styles.iconHeal} />
                        <span><strong>{getName(event.target_id)}</strong> heals <strong>{event.amount.toFixed(1)}</strong> HP from {getName(event.source_id)}</span>
                    </div>
                );
            case 'UnitDied':
                return (
                    <div key={index} className={`${styles.event} ${styles.death}`}>
                        <FontAwesomeIcon icon={faSkull} />
                        <span><strong>{getName(event.unit_id)}</strong> has died!</span>
                    </div>
                );
            case 'RoundStarted':
                return (
                    <div key={index} className={`${styles.event} ${styles.round}`}>
                        <FontAwesomeIcon icon={faHourglassStart} />
                        <span>=== Round {event.round_number} Started ===</span>
                    </div>
                );
            case 'SpellCast':
                return (
                    <div key={index} className={`${styles.event} ${styles.spell}`}>
                        <FontAwesomeIcon icon={faMagic} />
                        <span><strong>{getName(event.caster_id)}</strong> casts <strong>{event.spell_id}</strong> (Lvl {event.spell_level})</span>
                    </div>
                );
            case 'BuffApplied':
                return (
                    <div key={index} className={`${styles.event} ${styles.buff}`}>
                        <FontAwesomeIcon icon={faShieldAlt} />
                        <span><strong>{getName(event.target_id)}</strong> gains <strong>{event.buff_id}</strong></span>
                    </div>
                );
            default:
                return (
                    <div key={index} className={styles.event}>
                        <span>{event.type}: {JSON.stringify(event)}</span>
                    </div>
                );
        }
    };

    return (
        <div className={styles.eventLogContainer}>
            <div className={styles.header}>
                <h3>Combat Log</h3>
                <div className={styles.filters}>
                    <button className={filter === 'all' ? styles.active : ''} onClick={() => setFilter('all')}>All</button>
                    <button className={filter === 'combat' ? styles.active : ''} onClick={() => setFilter('combat')}>Combat</button>
                    <button className={filter === 'spell' ? styles.active : ''} onClick={() => setFilter('spell')}>Magic</button>
                    <button className={filter === 'status' ? styles.active : ''} onClick={() => setFilter('status')}>Status</button>
                </div>
            </div>
            <div className={styles.logBody}>
                {filteredEvents.length === 0 ? (
                    <div className={styles.emptyState}>No events to display</div>
                ) : (
                    filteredEvents.map((e, i) => renderEvent(e, i))
                )}
            </div>
        </div>
    );
};

export default EventLog;
