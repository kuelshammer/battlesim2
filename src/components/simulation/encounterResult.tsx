import { FC, useState, memo, useMemo } from "react"
import { Combattant, EncounterResult as EncounterResultType, EncounterStats, FinalAction, Buff, DiceFormula, AggregateOutput, FullAnalysisOutput, PlayerSlot } from "@/model/model"
import ResourcePanel from "./ResourcePanel"
import styles from './encounterResult.module.scss'
import { Round } from "@/model/model"
import { clone } from "@/model/utils"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faBrain } from "@fortawesome/free-solid-svg-icons"
import { useUIToggle } from "@/model/uiToggleState"
import { EncounterRating } from "./AnalysisComponents"
import DeltaBadge from "./DeltaBadge"
import PartyOverview from "./PartyOverview"
import PlayerGraphs from "./PlayerGraphs"
import { SkylineAnalysis } from "@/model/model"

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
    const actionName = action.name?.trim() || ''

    switch (action.type) {
        case 'atk':
            return actionName ? `Attack ${actionName}` : 'Attack'
        case 'heal':
            return actionName || 'Heal'
        case 'buff':
            return actionName || 'Buff'
        case 'debuff':
            return actionName || 'Debuff'
    }
}

function getTargetPrefix(combattantAction: { action: FinalAction, targets: Map<string, number> }): string {
    return 'on'
}


const TeamResults: FC<TeamPropType> = memo(({ round, team, stats, highlightedIds, onHighlight }) => {
    function getTarget(combattantAction: { action: FinalAction, targets: Map<string, number> }) {
        if (combattantAction.action.target === 'self') return 'itself'
        const allCombattants = [...round.team1, ...round.team2]
        const combattantMap = new Map(allCombattants.map(c => [c.id, c]))
        const creatureMap = new Map(allCombattants.map(c => [c.creature.id, c]))

        const targetNames = Array.from(combattantAction.targets.entries()).map(([targetId, count], index) => {
            let targetCombattant = combattantMap.get(targetId) || creatureMap.get(targetId)
            if (!targetCombattant) return `Target ${index + 1}`
            const creatureName = targetCombattant.creature.name
            return count === 1 ? creatureName : `${creatureName} x${count}`
        }).filter(nullable => !!nullable)

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
        if (buff.toHit != undefined) buffEffects.push(getNumberWithSign(buff.toHit) + ' to hit')
        return buffEffects.join(', ')
    }

    return (
        <div className={styles.team}>
            {team.map(combattant => (
                <div
                    key={combattant.id}
                    className={`${styles.lifebar} tooltipContainer`}>
                    <div className={`${styles.lifebarBackground} ${highlightedIds?.includes(combattant.id) ? styles.highlighted : ''}`}>
                        <div
                            className={styles.lifebarForeground}
                            style={{
                                width: `${100 * combattant.initialState.currentHp / (combattant.creature.hp + (combattant.initialState.tempHp || 0))}%`
                            }}
                        />
                        {combattant.initialState.tempHp ? (
                            <div
                                className={styles.lifebarTHP}
                                style={{
                                    width: `${100 * combattant.initialState.tempHp / (combattant.creature.hp + combattant.initialState.tempHp)}%`,
                                }}
                            />
                        ) : null}
                        <div className={styles.lifebarLabel}>
                            {combattant.initialState.currentHp}/{combattant.creature.hp}
                            {combattant.initialState.tempHp ? `+${combattant.initialState.tempHp}` : null}
                        </div>
                    </div>
                    <div className={styles.creatureName}>
                        {combattant.creature.name}
                        {combattant.finalState.concentratingOn ? (
                            <span className={styles.concentrationIcon} title={`Concentrating on ${combattant.finalState.concentratingOn}`}>
                                <FontAwesomeIcon icon={faBrain} />
                            </span>
                        ) : null}
                    </div>
                    <ResourcePanel combatant={combattant} />
                </div>
            ))}
        </div>
    )
})

type PropType = {
    value?: EncounterResultType,
    analysis?: AggregateOutput | null,
    fullAnalysis?: FullAnalysisOutput | null,
    playerNames?: Map<string, string>,
    isStale?: boolean,
    isPreliminary?: boolean,
    targetPercent?: number,
    actualPercent?: number,
    cumulativeDrift?: number,
    isShortRest?: boolean,
}

const EncounterResult: FC<PropType> = memo(({ value, analysis, fullAnalysis, playerNames, isStale, isPreliminary, targetPercent, actualPercent, cumulativeDrift, isShortRest }) => {
    const [hpBarsVisible, setHpBarsVisible] = useUIToggle('hp-bars')
    const [detailsExpanded, setDetailsExpanded] = useState(false)

    const lastRound = useMemo(() => {
        if (!value || !value.rounds.length) return null
        const cloned = clone(value.rounds[value.rounds.length - 1])
            ; ([...cloned.team1, ...cloned.team2]).forEach(combattant => {
                combattant.initialState = combattant.finalState
                combattant.actions = []
            })
        return cloned
    }, [value])

    if ((!value || !value.rounds.length) && !analysis) return <></>

    const hasRounds = value && value.rounds.length > 0

    return (
        <div className={`${styles.encounterResult} ${isStale ? styles.stale : ''}`}>
            {isStale && <div className={styles.staleBadge}>Out of Date</div>}
            
            {(targetPercent !== undefined && actualPercent !== undefined) && (
                <DeltaBadge 
                    targetCost={targetPercent} 
                    actualCost={actualPercent} 
                    cumulativeDrift={cumulativeDrift}
                />
            )}

            <EncounterRating analysis={analysis || null} isPreliminary={isPreliminary} isShortRest={isShortRest} />

            <div className={styles.detailsSection}>
                {fullAnalysis?.partySlots && fullAnalysis.partySlots.length > 0 && analysis?.skyline && (
                    <>
                        {/* Party Overview - Top Section */}
                        <PartyOverview
                            skyline={analysis.skyline}
                            partySlots={fullAnalysis.partySlots}
                            playerNames={playerNames}
                        />

                        {/* Individual Player Graphs - Below Party Overview */}
                        <PlayerGraphs
                            skyline={analysis.skyline}
                            partySlots={fullAnalysis.partySlots}
                            playerNames={playerNames}
                        />
                    </>
                )}

                <button
                    className={styles.detailsToggle}
                    onClick={() => setDetailsExpanded(!detailsExpanded)}
                >
                    {detailsExpanded ? '🔽' : '▶️'} {detailsExpanded ? 'Hide' : 'Show'} Detailed Analysis
                </button>

                {detailsExpanded && hasRounds && value && lastRound && (
                    <div className={styles.detailsContent}>
                        <div className={styles.round}>
                            <h3>Final State</h3>
                            <div className={styles.lifebars}>
                                <TeamResults round={lastRound} team={lastRound.team1} stats={value.stats} />
                                <hr />
                                <TeamResults round={lastRound} team={lastRound.team2} stats={value.stats} />
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    )
})

export default EncounterResult