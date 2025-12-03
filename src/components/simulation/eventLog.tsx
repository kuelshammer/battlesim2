import { FC, useState } from "react"
import styles from './simulation.module.scss'
import {
    SimulationEvent, ActionStartedEvent, AttackHitEvent, DamageTakenEvent, HealingAppliedEvent, UnitDiedEvent,
    BuffAppliedEvent, RoundStartedEvent, TurnStartedEvent, CustomEvent
} from "../../model/events" // Import structured event types
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import {
    faBolt, faDiceD20, faFistRaised, faHeart, faShieldAlt, faSkull,
    faMagic, faHourglassHalf, faCrosshairs, faUsers, faRedo, faBan,
    faHandSparkles, faSyringe, faMask, faStar, faWalking, faPlusCircle, faMinusCircle, faBrain
} from "@fortawesome/free-solid-svg-icons"
import { faQuestionCircle } from "@fortawesome/free-regular-svg-icons"


type EventLogProps = {
    events: SimulationEvent[] // Now expects structured events
    title?: string
}

type EventEntryProps = {
    event: SimulationEvent;
}

// Helper to get an icon based on event type
const getEventIcon = (eventType: string) => {
    switch (eventType) {
        case 'ActionStarted': return faFistRaised;
        case 'AttackHit': return faDiceD20;
        case 'AttackMissed': return faCrosshairs;
        case 'DamageTaken': return faHeart;
        case 'DamagePrevented': return faShieldAlt;
        case 'SpellCast': return faMagic;
        case 'SpellSaved': return faShieldAlt;
        case 'SpellFailed': return faBan;
        case 'ConcentrationBroken': return faBrain;
        case 'ConcentrationMaintained': return faStar;
        case 'BuffApplied': return faHandSparkles;
        case 'BuffExpired': return faHourglassHalf;
        case 'BuffRemoved': return faMinusCircle;
        case 'ConditionAdded': return faMask;
        case 'ConditionRemoved': return faPlusCircle;
        case 'HealingApplied': return faSyringe;
        case 'TempHPGranted': return faShieldAlt;
        case 'TempHPLost': return faShieldAlt;
        case 'UnitDied': return faSkull;
        case 'TurnStarted': return faRedo;
        case 'TurnEnded': return faRedo;
        case 'RoundStarted': return faRedo;
        case 'RoundEnded': return faRedo;
        case 'EncounterStarted': return faUsers;
        case 'EncounterEnded': return faUsers;
        case 'MovementStarted': return faWalking;
        case 'MovementInterrupted': return faWalking;
        case 'OpportunityAttack': return faBolt;
        case 'ResourceConsumed': return faMinusCircle;
        case 'ResourceRestored': return faPlusCircle;
        case 'ResourceDepleted': return faBan;
        case 'Custom': return faQuestionCircle;
        default: return faQuestionCircle;
    }
};

// Helper to get a CSS class based on event type
const getEventClass = (eventType: string): string => {
    switch (eventType) {
        case 'AttackHit':
        case 'DamageTaken': return styles.eventDamage;
        case 'HealingApplied':
        case 'TempHPGranted': return styles.eventHeal;
        case 'BuffApplied':
        case 'BuffRemoved':
        case 'ConcentrationMaintained': return styles.eventBuff;
        // Assuming DebuffApplied will be mapped to ConditionAdded/BuffApplied with negative effects
        case 'ConditionAdded': return styles.eventDebuff;
        case 'UnitDied': return styles.eventDeath;
        case 'EncounterEnded': return styles.eventEnd;
        default: return '';
    }
}

// Component for a single event entry
const EventEntry: FC<EventEntryProps> = ({ event }) => {
    const [isExpanded, setIsExpanded] = useState(false);
    const toggleExpand = () => setIsExpanded(!isExpanded);

    let mainText = event.type;
    let detailText = JSON.stringify(event, null, 2); // Default detail is full event object

    // Custom formatting for common events
    switch (event.type) {
        case 'ActionStarted':
            const asEvent = event as ActionStartedEvent;
            mainText = `${asEvent.actor_id} starts ${asEvent.action_id}`;
            detailText = `Actor: ${asEvent.actor_id}\nAction: ${asEvent.action_id}`;
            break;
        case 'AttackHit':
            const ahEvent = event as AttackHitEvent;
            mainText = `${ahEvent.attacker_id} hits ${ahEvent.target_id} for ${ahEvent.damage.toFixed(1)} damage`;
            detailText = `Attacker: ${ahEvent.attacker_id}\nTarget: ${ahEvent.target_id}\nDamage: ${ahEvent.damage.toFixed(1)}`;
            break;
        case 'DamageTaken':
            const dtEvent = event as DamageTakenEvent;
            mainText = `${dtEvent.target_id} takes ${dtEvent.damage.toFixed(1)} ${dtEvent.damage_type} damage`;
            detailText = `Target: ${dtEvent.target_id}\nDamage: ${dtEvent.damage.toFixed(1)}\nType: ${dtEvent.damage_type}`;
            break;
        case 'HealingApplied':
            const haEvent = event as HealingAppliedEvent;
            mainText = `${haEvent.source_id} heals ${haEvent.target_id} for ${haEvent.amount.toFixed(1)} HP`;
            detailText = `Source: ${haEvent.source_id}\nTarget: ${haEvent.target_id}\nAmount: ${haEvent.amount.toFixed(1)}`;
            break;
        case 'UnitDied':
            const udEvent = event as UnitDiedEvent;
            mainText = `${udEvent.unit_id} dies!`;
            detailText = `Unit: ${udEvent.unit_id}${udEvent.killer_id ? `\nKiller: ${udEvent.killer_id}` : ''}`;
            break;
        case 'BuffApplied':
            const baEvent = event as BuffAppliedEvent;
            mainText = `${baEvent.source_id} applies ${baEvent.buff_id} to ${baEvent.target_id}`;
            detailText = `Source: ${baEvent.source_id}\nTarget: ${baEvent.target_id}\nBuff: ${baEvent.buff_id}`;
            break;
        case 'RoundStarted':
            const rsEvent = event as RoundStartedEvent;
            mainText = `--- Round ${rsEvent.round_number} Started ---`;
            detailText = `Round: ${rsEvent.round_number}`;
            break;
        case 'TurnStarted':
            const tsEvent = event as TurnStartedEvent;
            mainText = `${tsEvent.unit_id}'s turn (Round ${tsEvent.round_number})`;
            detailText = `Unit: ${tsEvent.unit_id}\nRound: ${tsEvent.round_number}`;
            break;
        case 'Custom':
            const cEvent = event as CustomEvent;
            mainText = `${cEvent.event_type} by ${cEvent.source_id}`;
            detailText = `Event Type: ${cEvent.event_type}\nSource: ${cEvent.source_id}\nData: ${JSON.stringify(cEvent.data, null, 2)}`;
            break;
        default:
            // For other events, default to the type and stringified details
            mainText = event.type;
            const sourceId = (event as any).source_id || (event as any).attacker_id || (event as any).caster_id;
            const targetId = (event as any).target_id || (event as any).unit_id;
            if (sourceId && targetId && sourceId !== targetId) {
                mainText += `: ${sourceId} -> ${targetId}`;
            } else if (sourceId) {
                mainText += `: ${sourceId}`;
            } else if (targetId) {
                mainText += `: ${targetId}`;
            }
            break;
    }


    return (
        <div className={`${styles.eventLogEntry} ${getEventClass(event.type)}`}>
            <span onClick={toggleExpand} style={{ cursor: 'pointer' }}>
                <FontAwesomeIcon icon={getEventIcon(event.type)} fixedWidth /> {mainText}
            </span>
            {isExpanded && detailText && (
                <pre className={styles.eventDetails}>{detailText}</pre>
            )}
        </div>
    );
};


const EventLog: FC<EventLogProps> = ({ events, title = "Combat Events" }) => {
    const [filterType, setFilterType] = useState<string>('All');
    const [filterCombatant, setFilterCombatant] = useState<string>('');

    // Events are now already structured, no need for internal parsing
    const structuredEvents = events;

    // Get unique event types and combatant IDs for filters
    const uniqueEventTypes = Array.from(new Set(structuredEvents.map(e => e.type))).sort();
    const allCombatantIds = Array.from(new Set(structuredEvents.flatMap(e => {
        const ids: string[] = [];
        if ((e as any).actor_id) ids.push((e as any).actor_id);
        if ((e as any).attacker_id) ids.push((e as any).attacker_id);
        if ((e as any).caster_id) ids.push((e as any).caster_id);
        if ((e as any).source_id) ids.push((e as any).source_id);
        if ((e as any).target_id) ids.push((e as any).target_id);
        if ((e as any).unit_id) ids.push((e as any).unit_id);
        return ids;
    }))).sort();


    // Apply filters
    const filteredEvents = structuredEvents.filter(event => {
        const typeMatch = filterType === 'All' || event.type === filterType;
        if (!filterCombatant) return typeMatch;

        const combatantId = filterCombatant.toLowerCase();
        const involvedInEvent = ((event as any).actor_id && (event as any).actor_id.toLowerCase().includes(combatantId)) ||
                               ((event as any).attacker_id && (event as any).attacker_id.toLowerCase().includes(combatantId)) ||
                               ((event as any).caster_id && (event as any).caster_id.toLowerCase().includes(combatantId)) ||
                               ((event as any).source_id && (event as any).source_id.toLowerCase().includes(combatantId)) ||
                               ((event as any).target_id && (event as any).target_id.toLowerCase().includes(combatantId)) ||
                               ((event as any).unit_id && (event as any).unit_id.toLowerCase().includes(combatantId));
        return typeMatch && involvedInEvent;
    });


    if (filteredEvents.length === 0 && (filterType !== 'All' || filterCombatant !== '')) {
        return (
            <div className={styles.eventLog}>
                <h3>{title}</h3>
                <div className={styles.eventFilters}>
                    <label>
                        Type:
                        <select value={filterType} onChange={e => setFilterType(e.target.value)}>
                            <option value="All">All</option>
                            {uniqueEventTypes.map(type => <option key={type} value={type}>{type}</option>)}
                        </select>
                    </label>
                    <label>
                        Combatant:
                        <input
                            type="text"
                            value={filterCombatant}
                            onChange={e => setFilterCombatant(e.target.value)}
                            placeholder="Filter by ID"
                            list="combatant-ids"
                        />
                         <datalist id="combatant-ids">
                            {allCombatantIds.map(id => <option key={id} value={id} />)}
                        </datalist>
                    </label>
                </div>
                <div className={styles.eventLogContent}>
                    <p>No events match current filters.</p>
                </div>
            </div>
        )
    }


    return (
        <div className={styles.eventLog}>
            <h3>{title}</h3>
            <div className={styles.eventFilters}>
                <label>
                    Type:
                    <select value={filterType} onChange={e => setFilterType(e.target.value)}>
                        <option value="All">All</option>
                        {uniqueEventTypes.map(type => <option key={type} value={type}>{type}</option>)}
                    </select>
                </label>
                <label>
                    Combatant:
                    <input
                        type="text"
                        value={filterCombatant}
                        onChange={e => setFilterCombatant(e.target.value)}
                        placeholder="Filter by ID"
                        list="combatant-ids"
                    />
                    <datalist id="combatant-ids">
                        {allCombatantIds.map(id => <option key={id} value={id} />)}
                    </datalist>
                </label>
            </div>
            <div className={styles.eventLogContent}>
                {filteredEvents.map((event, index) => (
                    <EventEntry key={index} event={event} />
                ))}
            </div>
        </div>
    )
}

export default EventLog