import { FC, useState, memo, useMemo } from "react"
import { Combattant, EncounterResult as EncounterResultType, AggregateOutput, FullAnalysisOutput } from "@/model/model"
import ResourcePanel from "./ResourcePanel"
import ActionEconomyDisplay from "./ActionEconomyDisplay"
import styles from './encounterResult.module.scss'
import { Round } from "@/model/model"
import { clone } from "@/model/utils"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faBrain } from "@fortawesome/free-solid-svg-icons"
import { EncounterRating, VitalsDashboard, ValidationNotice } from "./AnalysisComponents"
import DeltaBadge from "./DeltaBadge"
import PartyOverview from "./PartyOverview"
import PlayerGraphs from "./PlayerGraphs"

type TeamPropType = {
    round: Round,
    team: Combattant[],
    highlightedIds?: string[],
}

const TeamResults: FC<TeamPropType> = memo(({ team, highlightedIds }) => {
    return (
        <div className={styles.team} data-testid="team-results">
            {team.map(combattant => (
                <div
                    key={combattant.id}
                    className={`${styles.lifebar} tooltipContainer`}
                    data-testid={`lifebar-${combattant.id}`}>
                    <div className={`${styles.lifebarBackground} ${highlightedIds?.includes(combattant.id) ? styles.highlighted : ''}`} data-testid="lifebar-background">
                        <div
                            className={styles.lifebarForeground}
                            style={{
                                width: `${100 * combattant.initialState.currentHp / (combattant.creature.hp + (combattant.initialState.tempHp || 0))}%`
                            }}
                            data-testid="lifebar-foreground"
                        />
                        {combattant.initialState.tempHp ? (
                            <div
                                className={styles.lifebarTHP}
                                style={{
                                    width: `${100 * combattant.initialState.tempHp / (combattant.creature.hp + combattant.initialState.tempHp)}%`,
                                }}
                                data-testid="lifebar-thp"
                            />
                        ) : null}
                        <div className={styles.lifebarLabel} data-testid="lifebar-label">
                            {combattant.initialState.currentHp}/{combattant.creature.hp}
                            {combattant.initialState.tempHp ? `+${combattant.initialState.tempHp}` : null}
                        </div>
                    </div>
                    <div className={styles.creatureName} data-testid="creature-name">
                        {combattant.creature.name}
                        {combattant.finalState.concentratingOn ? (
                            <span className={styles.concentrationIcon} title={`Concentrating on ${combattant.finalState.concentratingOn}`} data-testid="concentration-icon">
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
    targetRole?: string,
}

const EncounterResult: FC<PropType> = memo(({ value, analysis, fullAnalysis, playerNames, isStale, isPreliminary, targetPercent, actualPercent, cumulativeDrift, isShortRest, targetRole }) => {
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
        <div className={`${styles.encounterResult} ${isStale ? styles.stale : ''}`} data-testid="results-panel">
            {isStale && <div className={styles.staleBadge}>Out of Date</div>}
            
            {(targetPercent !== undefined && actualPercent !== undefined) && (
                <DeltaBadge 
                    targetCost={targetPercent} 
                    actualCost={actualPercent} 
                    cumulativeDrift={cumulativeDrift}
                />
            )}

            {fullAnalysis?.partySlots && fullAnalysis.partySlots.length > 0 && analysis?.skyline && (
                <div className={styles.prioritizedSkyline}>
                    {/* Party Overview - Prioritized */}
                    <PartyOverview
                        skyline={analysis.skyline}
                        partySlots={fullAnalysis.partySlots}
                        playerNames={playerNames}
                        className="encounter-party-overview"
                    />
                </div>
            )}

            {analysis && <VitalsDashboard analysis={analysis} isPreliminary={isPreliminary} />}

            {analysis && (
                <ValidationNotice 
                    analysis={analysis} 
                    targetRole={targetRole} 
                />
            )}

            <EncounterRating analysis={analysis || null} isPreliminary={isPreliminary} isShortRest={isShortRest} />

            <div className={styles.detailsSection}>
                {fullAnalysis?.partySlots && fullAnalysis.partySlots.length > 0 && analysis?.skyline && (
                    <>
                        {/* Individual Player Graphs - Moved here */}
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
                    data-testid="detail-toggle"
                >
                    {detailsExpanded ? '🔽' : '▶️'} {detailsExpanded ? 'Hide' : 'Show'} Detailed Analysis
                </button>

                {detailsExpanded && hasRounds && value && lastRound && (
                    <div className={styles.detailsContent} data-testid="details-content">
                        <div className={styles.round}>
                            <h3>Final State</h3>
                            <ActionEconomyDisplay round={lastRound} />
                            <div className={styles.lifebars} data-testid="final-state-lifebars">
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