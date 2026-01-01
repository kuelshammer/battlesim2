import { FC, useState, memo, useMemo } from "react"
import { Combattant, EncounterResult as EncounterResultType, EncounterStats, FinalAction, Buff, DiceFormula, AggregateOutput } from "@/model/model"
import ResourcePanel from "./ResourcePanel"
import styles from './encounterResult.module.scss'
import { Round } from "@/model/model"
import { clone } from "@/model/utils"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faBrain } from "@fortawesome/free-solid-svg-icons"
import { useUIToggle } from "@/model/uiToggleState"
import { EncounterRating, MedianPerformanceDisplay } from "./AnalysisComponents"
import DeltaBadge from "./DeltaBadge"
import { SkylineAnalysis, CharacterBucketData, valueToColor, DEFAULT_SKYLINE_COLORS } from "@/model/skylineTypes"

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

// Skyline stats display for players (HD = Heat & Drain)
type SkylinePlayerStatsProps = {
    skyline: SkylineAnalysis | null | undefined;
    players: Combattant[];
}

const SkylinePlayerStats: FC<SkylinePlayerStatsProps> = memo(({ skyline, players }) => {
    if (!skyline || !skyline.buckets.length) return null;

    // Get median bucket (50th percentile = index 49)
    const medianBucket = skyline.buckets[49];

    return (
        <div className={styles.skylineStats}>
            <h4>Skyline Analysis (Median Run)</h4>
            <div className={styles.skylineStatsGrid}>
                {players.map(player => {
                    const charData = medianBucket.characters.find(c => c.id === player.creature.id);
                    if (!charData) return null;

                    const hpColor = valueToColor(charData.hpPercent, DEFAULT_SKYLINE_COLORS);
                    const resourceColor = valueToColor(charData.resourcePercent, DEFAULT_SKYLINE_COLORS);

                    return (
                        <div key={player.creature.id} className={styles.skylineStatCard}>
                            <div className={styles.skylineStatName}>{player.creature.name}</div>
                            <div className={styles.skylineStatRow}>
                                <span className={styles.skylineStatLabel}>HP:</span>
                                <span
                                    className={styles.skylineStatValue}
                                    style={{ backgroundColor: hpColor, color: charData.hpPercent < 50 ? '#fff' : '#000' }}
                                >
                                    {charData.hpPercent.toFixed(0)}%
                                </span>
                            </div>
                            <div className={styles.skylineStatRow}>
                                <span className={styles.skylineStatLabel}>Resources:</span>
                                <span
                                    className={styles.skylineStatValue}
                                    style={{ backgroundColor: resourceColor, color: charData.resourcePercent < 50 ? '#fff' : '#000' }}
                                >
                                    {charData.resourcePercent.toFixed(0)}%
                                </span>
                            </div>
                            <div className={styles.skylineStatRow}>
                                <span className={styles.skylineStatLabel}>Hit Dice:</span>
                                <span className={styles.skylineStatText}>
                                    {charData.resourceBreakdown.hitDice}/{charData.resourceBreakdown.hitDiceMax}
                                </span>
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
});

type PropType = {
    value?: EncounterResultType,
    analysis?: AggregateOutput | null,
    isStale?: boolean,
    isPreliminary?: boolean,
    targetPercent?: number,
    actualPercent?: number,
    cumulativeDrift?: number,
}

const EncounterResult: FC<PropType> = memo(({ value, analysis, isStale, isPreliminary, targetPercent, actualPercent, cumulativeDrift }) => {
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

            <EncounterRating analysis={analysis || null} isPreliminary={isPreliminary} />
            <MedianPerformanceDisplay analysis={analysis || null} isPreliminary={isPreliminary} />

            <div className={styles.detailsSection}>
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
                        <SkylinePlayerStats skyline={analysis?.skyline || null} players={lastRound.team1} />
                    </div>
                )}
            </div>
        </div>
    )
})

export default EncounterResult