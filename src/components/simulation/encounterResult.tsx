import { FC, useState } from "react"
import { Combattant, EncounterResult as EncounterResultType, EncounterStats, FinalAction, Buff, DiceFormula } from "../../model/model"
import styles from './encounterResult.module.scss'
import { Round } from "../../model/model"
import { clone } from "../../model/utils"

type TeamPropType = {
    round: Round,
    team: Combattant[],
    stats?: Map<string, EncounterStats>,
    highlightedIds?: string[],
    onHighlight?: (targetIds: string[]) => void,
}

// Enhanced action label function that provides complete action descriptions
function getActionLabel(combattantAction: { action: FinalAction, targets: Map<string, number> }): string {
    const { action } = combattantAction

    switch (action.type) {
        case 'atk':
            return `Attack ${action.name}`
        case 'heal':
            return action.name
        case 'buff':
            return action.name
        case 'debuff':
            return action.name
        case 'template':
            return action.name
        default:
            return action.name
    }
}

// Enhanced target prefix function that provides appropriate "on" prefix based on action type
function getTargetPrefix(combattantAction: { action: FinalAction, targets: Map<string, number> }): string {
    const { action } = combattantAction

    switch (action.type) {
        case 'atk':
            return 'on'
        case 'heal':
            return 'on'
        case 'buff':
            return 'on'
        case 'debuff':
            return 'on'
        case 'template':
            return 'on'
        default:
            return 'on'
    }
}

const TeamResults: FC<TeamPropType> = ({ round, team, stats, highlightedIds, onHighlight }) => {
    function getTarget(combattantAction: { action: FinalAction, targets: Map<string, number> }) {
        if (combattantAction.action.target === 'self') return 'itself'

        const allCombattants = [...round.team1, ...round.team2]

        // Create lookup maps for efficient searching
        const combattantMap = new Map(allCombattants.map(c => [c.id, c]))
        const creatureMap = new Map(allCombattants.map(c => [c.creature.id, c]))

  
        const targetNames = Array.from(combattantAction.targets.entries()).map(([targetId, count], index) => {
            // Try to find by combatant ID first
            let targetCombattant = combattantMap.get(targetId)

            // If not found, try by creature ID as fallback
            if (!targetCombattant) {
                targetCombattant = creatureMap.get(targetId)
            }

            if (!targetCombattant) {
                // Enhanced fallback: try partial ID matching and display useful info
                const similarCombattants = allCombattants.filter(c =>
                    c.id.includes(targetId) || targetId.includes(c.id) ||
                    c.creature.id.includes(targetId) || targetId.includes(c.creature.id)
                )

                if (similarCombattants.length === 1) {
                    targetCombattant = similarCombattants[0]
                } else if (similarCombattants.length > 1) {
                    // Multiple matches - use first one for now, but add debug info
                    console.warn(`Multiple matching targets found for ID ${targetId}, using first match`)
                    targetCombattant = similarCombattants[0]
                }
            }

            if (!targetCombattant) {
                // Debug: show what IDs we're looking for vs what's available
                console.warn(`Target ID ${targetId} not found. Available combatant IDs:`,
                    allCombattants.map(c => c.id))
                console.warn(`Available creature IDs:`,
                    allCombattants.map(c => c.creature.id))

                // Last resort: provide informative fallback
                const actionType = combattantAction.action.type
                const isEnemyAction = actionType === 'atk' || actionType === 'debuff'

                // Try to infer target from action context
                if (isEnemyAction && round.team1.length > 0 && round.team2.length > 0) {
                    // Find the source combatant that performed this action
                    const sourceCombattant = allCombattants.find(c =>
                        c.actions.some(a => a.action.id === combattantAction.action.id)
                    )
                    if (sourceCombattant) {
                        // Determine target team based on source team
                        const isSourceFromTeam1 = round.team1.some(t => t.id === sourceCombattant.id)
                        const targetTeam = isSourceFromTeam1 ? round.team2 : round.team1
                        if (targetTeam.length > 0) {
                            return `${targetTeam[0].creature.name}${count > 1 ? ` x${count}` : ''} (inferred)`
                        }
                    }
                }

                return `Target ${index + 1} (ID: ${targetId.substring(0, 8)}...)`
            }

            const creatureName = targetCombattant.creature.name
            if (count === 1) return creatureName

            return creatureName + ' x' + count
        })
            .filter(nullable => !!nullable)

        return targetNames.join(' and ')
    }

    function getNumberWithSign(n: DiceFormula) {
        let result = String(n)
        if (!result.startsWith('-')) result = '+' + result
        return ' ' + result
    }

    function getBuffEffect(buff: Buff) {
        const buffEffects: string[] = []

        if (buff.ac != undefined) buffEffects.push(getNumberWithSign(buff.ac) + ' AC')
        if (buff.condition != undefined) buffEffects.push(' ' + buff.condition)
        if (buff.damageMultiplier != undefined) buffEffects.push(' x' + buff.damageMultiplier + ' damage')
        if (buff.damageTakenMultiplier != undefined) buffEffects.push(' x' + buff.damageTakenMultiplier + ' damage taken')
        if (buff.toHit != undefined) buffEffects.push(getNumberWithSign(buff.toHit) + ' to hit')
        if (buff.save != undefined) buffEffects.push(getNumberWithSign(buff.save) + ' to save')
        if (buff.damage != undefined) buffEffects.push(getNumberWithSign(buff.damage) + ' extra damage')
        if (buff.damageReduction != undefined) buffEffects.push(getNumberWithSign(buff.damageReduction) + ' reduced damage')

        return buffEffects.join(', ')
    }

    return (
        <div className={styles.team}>
            {team.map(combattant => (
                <div
                    key={combattant.id}
                    onMouseEnter={() => onHighlight?.(combattant.actions.flatMap(action => Array.from(action.targets.keys())))}
                    onMouseLeave={() => onHighlight?.([])}
                    className={`${styles.lifebar} tooltipContainer`}>
                    <div className={`${styles.lifebarBackground} ${highlightedIds?.includes(combattant.id) ? styles.highlighted : ''}`}>
                        <div
                            className={styles.lifebarForeground}
                            style={{
                                width: `${100 * combattant.initialState.currentHP / (combattant.creature.hp + (combattant.initialState.tempHP || 0))}%`
                            }}
                        />
                        {combattant.initialState.tempHP ? (
                            <div
                                className={styles.lifebarTHP}
                                style={{
                                    width: `${100 * combattant.initialState.tempHP / (combattant.creature.hp + combattant.initialState.tempHP)}%`,
                                }}
                            />
                        ) : null}
                        <div className={styles.lifebarLabel}>
                            {combattant.initialState.currentHP.toFixed(1)}/{combattant.creature.hp}
                            {combattant.initialState.tempHP ? `+${combattant.initialState.tempHP.toFixed(1)}` : null}
                        </div>
                    </div>
                    <div className={styles.creatureName}>
                        {combattant.creature.name}
                    </div>

                    {(!stats && (combattant.actions.length === 0) && (combattant.finalState.buffs.size)) ? null : (
                        <div className="tooltip">
                            <ul>
                                {stats ? (() => {
                                    const creatureStats = stats.get(combattant.id)
                                    if (!creatureStats) return <>No Stats</>

                                    return (
                                        <>
                                            {creatureStats.damageDealt ? <li><b>Damage Dealt:</b> {Math.round(creatureStats.damageDealt)} dmg</li> : null}
                                            {creatureStats.damageTaken ? <li><b>Damage Taken:</b> {Math.round(creatureStats.damageTaken)} dmg</li> : null}
                                            {creatureStats.healGiven ? <li><b>Healed allies for:</b> {Math.round(creatureStats.healGiven)} hp</li> : null}
                                            {creatureStats.healReceived ? <li><b>Was healed for:</b> {Math.round(creatureStats.healReceived)} hp</li> : null}
                                            {creatureStats.timesUnconscious ? <li><b>Went unconscious:</b> {Math.round(creatureStats.timesUnconscious)} times</li> : null}
                                            {creatureStats.charactersBuffed ? <li><b>Buffed:</b> {Math.round(creatureStats.charactersBuffed)} allies</li> : null}
                                            {creatureStats.buffsReceived ? <li><b>Was buffed:</b> {Math.round(creatureStats.buffsReceived)} times</li> : null}
                                            {creatureStats.charactersDebuffed ? <li><b>Debuffed:</b> {Math.round(creatureStats.charactersDebuffed)} enemies</li> : null}
                                            {creatureStats.debuffsReceived ? <li><b>Was debuffed:</b> {Math.round(creatureStats.debuffsReceived)} times</li> : null}
                                        </>
                                    )
                                })() : (() => {
                                    const li = combattant.actions
                                        .filter(({ targets }) => !!targets.size)
                                        .map((action, index) => (
                                            <li
                                                key={index}
                                                onMouseEnter={() => onHighlight?.(Array.from(action.targets.keys()))}
                                                onMouseLeave={() => onHighlight?.(combattant.actions.flatMap(a => Array.from(a.targets.keys())))}>
                                                <b>{getActionLabel({ action, targets: action.targets })}</b> {getTargetPrefix(action)} {getTarget({ action, targets: action.targets })}
                                            </li>
                                        ))

                                    //todo effects that disappear in the same round are not shown, which can be misleading
                                    const buffCount = combattant.finalState.buffs.size
                                    const bi = Array.from(combattant.finalState.buffs)
                                        .filter(([_, buff]) => ((buff.magnitude === undefined) || (buff.magnitude > 0.1)))
                                        .map(([buffId, buff], index) => (
                                            (buffCount <= 3) ?
                                                <li key={buffId}>
                                                    <b>{buff.displayName}</b>{getBuffEffect(buff)} {(buff.magnitude !== undefined && buff.magnitude !== 1) ? (
                                                        `(${Math.round(buff.magnitude * 100)}%)`
                                                    ) : ''}
                                                </li> :
                                                <>
                                                    <b>{buff.displayName}</b>{(index < buffCount - 1) ? ', ' : null}
                                                </>
                                        ))

                                    return (
                                        <>
                                            {li.length ? li : <b>No Actions</b>}
                                            {bi.length ? <>
                                                <br /><u>Active Effects</u><br />
                                                {bi}
                                            </> : null}
                                        </>
                                    )
                                })()}
                            </ul>
                        </div>
                    )}
                </div>
            ))}
        </div>
    )
}

type PropType = {
    value: EncounterResultType,
}

const EncounterResult: FC<PropType> = ({ value }) => {
    if (!value.rounds.length) return <></>
    const lastRound = clone(value.rounds[value.rounds.length - 1])
    const [highlightedIds, setHighlightedIds] = useState<string[]>([])
    const [highlightedRound, setHighlightedRound] = useState(0)

        ; ([...lastRound.team1, ...lastRound.team2]).forEach(combattant => {
            combattant.initialState = combattant.finalState
            combattant.actions = []
        })

    if (value.rounds.length === 1 && (!value.rounds[0].team1.length || !value.rounds[0].team2.length)) return <></>

    return (
        <div className={styles.encounterResult}>
            {value.rounds.map((round, roundIndex) => (
                <div key={roundIndex} className={styles.round}>
                    <h3>Round {roundIndex + 1}</h3>

                    <div className={styles.lifebars}>
                        <TeamResults
                            round={round}
                            team={round.team1}
                            highlightedIds={highlightedRound === roundIndex ? highlightedIds : undefined}
                            onHighlight={targetIds => { setHighlightedIds(targetIds); setHighlightedRound(roundIndex) }} />
                        <hr />
                        <TeamResults
                            round={round}
                            team={round.team2}
                            highlightedIds={highlightedRound === roundIndex ? highlightedIds : undefined}
                            onHighlight={targetIds => { setHighlightedIds(targetIds); setHighlightedRound(roundIndex) }} />
                    </div>
                </div>
            ))}
            <div className={styles.round}>
                <h3>Result</h3>

                <div className={styles.lifebars}>
                    <TeamResults round={lastRound} team={lastRound.team1} stats={value.stats} />
                    <hr />
                    <TeamResults round={lastRound} team={lastRound.team2} stats={value.stats} />
                </div>
            </div>
        </div>
    )
}

export default EncounterResult