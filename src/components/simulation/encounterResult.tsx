import { FC, useState, memo, useMemo } from "react"
import { Combattant, EncounterResult as EncounterResultType, EncounterStats, FinalAction, Buff, DiceFormula, AggregateOutput } from "@/model/model"
import ResourcePanel from "./ResourcePanel"
import styles from './encounterResult.module.scss'
import { Round } from "@/model/model"
import { clone } from "@/model/utils"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faBrain } from "@fortawesome/free-solid-svg-icons"
import { useUIToggle } from "@/model/uiToggleState"

// Encounter Rating Component
const EncounterRating: FC<{ analysis: AggregateOutput | null, isPreliminary?: boolean }> = memo(({ analysis, isPreliminary }) => {
    const getEncounterRating = useMemo(() => {
        if (!analysis || !analysis.deciles?.length) return null;

        // Calculate overall win rate and average HP loss
        const totalRuns = analysis.totalRuns;
        let totalWins = 0;
        let totalHpLossPercent = 0;

        analysis.deciles.forEach(decile => {
            // Each decile represents 10% of runs
            const decileRuns = totalRuns / analysis.deciles.length;
            totalWins += ((decile.winRate || 0) / 100) * decileRuns;
            totalHpLossPercent += (decile.hpLostPercent || 0);
        });

        const overallWinRate = (totalWins / totalRuns) * 100;
        const avgHpLossPercent = totalHpLossPercent / analysis.deciles.length;

        // Rating logic
        if (overallWinRate < 20 || avgHpLossPercent > 80) return { rating: "Deadly", color: "#dc3545", icon: "🔴" };
        if (overallWinRate < 40 || avgHpLossPercent > 60) return { rating: "Hard", color: "#fd7e14", icon: "🟠" };
        if (overallWinRate < 60 || avgHpLossPercent > 40) return { rating: "Medium", color: "#ffc107", icon: "🟡" };
        if (overallWinRate < 80 || avgHpLossPercent > 20) return { rating: "Easy", color: "#28a745", icon: "🟢" };
        return { rating: "Trivial", color: "#20c997", icon: "🟢" };
    }, [analysis]);

    if (!getEncounterRating) return null;

    const { rating, color, icon } = getEncounterRating;

    return (
        <div className={styles.encounterRating} style={{ backgroundColor: color }}>
            <span className={styles.ratingIcon}>{icon}</span>
            <span className={styles.ratingText}>
                {rating.toUpperCase()} ENCOUNTER
                {isPreliminary && <span className={styles.preliminaryNotice}> (ESTIMATING...)</span>}
            </span>
            <div className={styles.ratingDetails}>
                {analysis && (
                    <>
                        <span>Win Rate: {((analysis.deciles.reduce((sum, q) => sum + (q.winRate || 0), 0) / analysis.deciles.length)).toFixed(1)}%</span>
                        <span>Avg HP Lost: {(analysis.deciles.reduce((sum, q) => sum + (q.hpLostPercent || 0), 0) / analysis.deciles.length).toFixed(1)}%</span>
                    </>
                )}
            </div>
        </div>
    );
});

// Median Performance Display Component
const MedianPerformanceDisplay: FC<{ analysis: AggregateOutput | null, isPreliminary?: boolean }> = memo(({ analysis, isPreliminary }) => {
    const medianDecile = useMemo(() => {
        if (!analysis) return null;
        
        // Prefer explicit globalMedian if provided (2511 methodology)
        if (analysis.globalMedian) return analysis.globalMedian;
        
        // Fallback to standard median decile
        if (!analysis.deciles?.length) return null;
        const medianIndex = analysis.deciles.length === 10 ? 4 : Math.floor(analysis.deciles.length / 2);
        return analysis.deciles[medianIndex];
    }, [analysis]);

    if (!medianDecile) return null;

    const getHpBarColor = (hpPercentage: number, isDead: boolean): string => {
        if (isDead) return styles.dead;
        if (hpPercentage <= 20) return styles.danger;
        if (hpPercentage <= 50) return styles.bloodied;
        return styles.healthy;
    };

    const renderHpBar = (currentHp: number, startHp: number, maxHp: number) => {
        if (!maxHp || maxHp <= 0) return [];
        
        // Calculate segments (10 total)
        const totalSegments = 10;
        
        // Green: Remaining HP
        const greenCount = Math.floor((currentHp / maxHp) * totalSegments);
        
        // Red: Lost in this battle (Start HP - Current HP)
        const newDamage = Math.max(0, startHp - currentHp);
        const redCount = Math.floor((newDamage / maxHp) * totalSegments);
        
        // Grey: Previously lost HP (Max HP - Start HP)
        const greyCount = Math.max(0, totalSegments - greenCount - redCount);
        
        const segments = [];
        // Green segments (Remaining)
        for (let i = 0; i < greenCount; i++) {
            segments.push(<span key={`g-${i}`} className={styles.segmentGreen}>█</span>);
        }
        // Red segments (Newly lost)
        for (let i = 0; i < redCount; i++) {
            segments.push(<span key={`r-${i}`} className={styles.segmentRed}>█</span>);
        }
        // Grey segments (Previously lost)
        for (let i = 0; i < greyCount; i++) {
            segments.push(<span key={`gr-${i}`} className={styles.segmentGrey}>░</span>);
        }
        
        // Ensure we always have exactly 10 segments due to rounding
        while (segments.length < totalSegments) {
            segments.push(<span key={`f-${segments.length}`} className={styles.segmentGrey}>░</span>);
        }
        if (segments.length > totalSegments) {
            segments.splice(totalSegments);
        }

        return segments;
    };

    const avgFinalHp = medianDecile.medianRunVisualization
        ? (medianDecile.medianRunVisualization.reduce((sum, c) => sum + c.hpPercentage, 0) / medianDecile.medianRunVisualization.length).toFixed(1)
        : '0.0';

    return (
        <div className={`${styles.bestDecileDisplay} ${isPreliminary ? styles.isEstimating : ''}`}>
            <h4>📊 {medianDecile.label === "Global Median" ? "True Global Median" : "Median Performance"} {isPreliminary && <small>(Updating...)</small>}</h4>
            <div className={styles.bestDecileHeader}>
                <span className={styles.survivorsBadge}>
                    ✅ {medianDecile.medianSurvivors}/{medianDecile.partySize} Survivors
                </span>
                <span className={styles.winRateBadge}>
                    {(medianDecile.winRate || 0).toFixed(1)}% Win Rate
                </span>
            </div>

            <div className={styles.bestDecileCombatants}>
                {medianDecile.medianRunVisualization?.map((combatant, index) => (
                    <div key={index} className={styles.bestDecileCombatant}>
                        <div className={styles.combatantName}>
                            {combatant.name}
                            {combatant.isDead && <span className={styles.deathIndicator}> 💀 Dead</span>}
                        </div>
                        <div className={styles.hpBar}>
                            <span className={getHpBarColor(combatant.hpPercentage, combatant.isDead)}>
                                <div className={styles.hpBarContainer}>
                                    <div className={styles.hpBarVisual}>
                                        [{renderHpBar(combatant.currentHp, combatant.startHp, combatant.maxHp)}]
                                    </div>
                                    <span className={styles.hpText}>
                                        {combatant.currentHp}/{combatant.maxHp} HP ({combatant.hpPercentage.toFixed(0)}%)
                                    </span>
                                </div>
                            </span>
                        </div>
                    </div>
                ))}
            </div>

            <div className={styles.bestDecileMetrics}>
                <div className={styles.metric}>
                    <strong>Average Final HP:</strong> {avgFinalHp}%
                </div>
                <div className={styles.metric}>
                    <strong>Combat Duration:</strong> {medianDecile.battleDurationRounds} rounds
                </div>
            </div>
        </div>
    );
});

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

    // Handle empty or whitespace-only action names
    const actionName = action.name?.trim() || ''

    switch (action.type) {
        case 'atk':
            if (actionName) {
                return `Attack ${actionName}`
            }
            // Fallback: infer from damage properties if available
            if ('dpr' in action && action.dpr) {
                return `Attack ${action.dpr} damage`
            }
            return 'Attack'
        case 'heal':
            if (actionName) {
                return actionName
            }
            // Fallback: infer from healing properties if available
            if ('amount' in action && action.amount) {
                return `Heal ${action.amount} HP`
            }
            return 'Heal'
        case 'buff':
            if (actionName) {
                return actionName
            }
            // Fallback: infer from buff properties if available
            if ('buff' in action && action.buff && action.buff.displayName) {
                return `Buff ${action.buff.displayName}`
            }
            return 'Buff'
        case 'debuff':
            if (actionName) {
                return actionName
            }
            // Fallback: infer from debuff properties if available
            if ('buff' in action && action.buff && action.buff.displayName) {
                return `Debuff ${action.buff.displayName}`
            }
            return 'Debuff'
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
    }
}


const TeamResults: FC<TeamPropType> = memo(({ round, team, stats, highlightedIds, onHighlight }) => {
    function getTarget(combattantAction: { action: FinalAction, targets: Map<string, number> }) {
        if (combattantAction.action.target === 'self') return 'itself'

        const allCombattants = [...round.team1, ...round.team2]

        // Debug: Log target IDs and available combatant/creature IDs
        // console.log('getTarget() called with target IDs:', Array.from(combattantAction.targets.keys()));
        // console.log('Available combatant IDs:', allCombattants.map(c => c.id));
        // console.log('Available creature IDs:', allCombattants.map(c => c.creature.id));

        // Create lookup maps for efficient searching
        const combattantMap = new Map(allCombattants.map(c => [c.id, c]))
        const creatureMap = new Map(allCombattants.map(c => [c.creature.id, c]))

        const targetNames = Array.from(combattantAction.targets.entries()).map(([targetId, count], index) => {
            // Try to find by combatant ID first (this is what Rust stores)
            let targetCombattant = combattantMap.get(targetId)

            // If not found, try by creature ID as fallback
            if (!targetCombattant) {
                targetCombattant = creatureMap.get(targetId)
            }

            if (!targetCombattant) {
                // Enhanced fallback: try partial ID matching for UUID-style IDs
                const similarCombattants = allCombattants.filter(c => {
                    // For UUIDs, try matching the first 8 characters
                    if (targetId.length >= 8 && c.id.length >= 8) {
                        return targetId.substring(0, 8) === c.id.substring(0, 8) ||
                            targetId.substring(0, 8) === c.creature.id.substring(0, 8)
                    }
                    // Fallback to contains matching
                    return c.id.includes(targetId) || targetId.includes(c.id) ||
                        c.creature.id.includes(targetId) || targetId.includes(c.creature.id)
                })

                if (similarCombattants.length === 1) {
                    targetCombattant = similarCombattants[0]
                } else if (similarCombattants.length > 1) {
                    // Multiple matches - try to find the best one by checking action context
                    const actionType = combattantAction.action.type
                    const isEnemyAction = actionType === 'atk' || actionType === 'debuff'

                    // Find the source combatant that performed this action
                    const sourceCombattant = allCombattants.find(c =>
                        c.actions.some(a => a.action.id === combattantAction.action.id)
                    )

                    if (sourceCombattant) {
                        // Determine target team based on source team
                        const isSourceFromTeam1 = round.team1.some(t => t.id === sourceCombattant.id)
                        const expectedTargetTeam = isSourceFromTeam1 ? round.team2 : round.team1

                        // Filter matches to only those on the expected target team
                        const teamMatches = similarCombattants.filter(c =>
                            expectedTargetTeam.some(t => t.id === c.id)
                        )

                        if (teamMatches.length === 1) {
                            targetCombattant = teamMatches[0]
                        } else {
                            console.warn(`Multiple matching targets found for ID ${targetId} on expected team, using first match`)
                            targetCombattant = teamMatches[0] || similarCombattants[0]
                        }
                    } else {
                        console.warn(`Multiple matching targets found for ID ${targetId}, using first match`)
                        targetCombattant = similarCombattants[0]
                    }
                }
            }

            if (!targetCombattant) {
                // Debug: show what IDs we're looking for vs what's available
                console.warn(`Target ID ${targetId} not found. Available combatant IDs:`,
                    allCombattants.map(c => c.id))
                console.warn(`Available creature IDs:`,
                    allCombattants.map(c => c.creature.id))

                // Last resort: provide informative fallback based on action context
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
                                                <b>{getActionLabel(action)}</b> {getTargetPrefix(action)} {getTarget(action)}
                                            </li>
                                        ))

                                    // Handle buffs (support both Map and Object)
                                    const buffs = combattant.finalState.buffs
                                    const buffEntries = (buffs instanceof Map)
                                        ? Array.from(buffs.entries())
                                        : Object.entries(buffs || {})

                                    const buffCount = buffEntries.length
                                    const bi = buffEntries
                                        .filter(([_, buff]: [string, any]) => ((buff.magnitude === undefined) || (buff.magnitude > 0.1)))
                                        .map(([buffId, buff]: [string, any], index) => (
                                            (buffCount <= 3) ?
                                                <li key={buffId}>
                                                    <b>{buff.displayName}</b>{getBuffEffect(buff)} {(buff.magnitude !== undefined && buff.magnitude !== 1) ? (
                                                        `(${Math.round(buff.magnitude * 100)}%)`
                                                    ) : ''}
                                                </li> :
                                                <span key={buffId}>
                                                    <b>{buff.displayName}</b>{(index < buffCount - 1) ? ', ' : null}
                                                </span>
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
})

type PropType = {
    value?: EncounterResultType,
    analysis?: AggregateOutput | null,
    isStale?: boolean,
    isPreliminary?: boolean,
}

const EncounterResult: FC<PropType> = memo(({ value, analysis, isStale, isPreliminary }) => {
    const [hpBarsVisible, setHpBarsVisible] = useUIToggle('hp-bars')
    const [detailsExpanded, setDetailsExpanded] = useState(false)

    // Memoize expensive clone operation
    const lastRound = useMemo(() => {
        if (!value || !value.rounds.length) return null
        const cloned = clone(value.rounds[value.rounds.length - 1])
            ; ([...cloned.team1, ...cloned.team2]).forEach(combattant => {
                combattant.initialState = combattant.finalState
                combattant.actions = []
            })
        return cloned
    }, [value])

    const [highlightedIds, setHighlightedIds] = useState<string[]>([])
    const [highlightedRound, setHighlightedRound] = useState(0)

    // If we have neither value nor analysis, there's nothing to show
    if ((!value || !value.rounds.length) && !analysis) return <></>

    const hasRounds = value && value.rounds.length > 0 && (value.rounds.length > 1 || (value.rounds[0].team1.length && value.rounds[0].team2.length))

    return (
        <div className={`${styles.encounterResult} ${isStale ? styles.stale : ''}`}>
            {isStale && <div className={styles.staleBadge}>Out of Date</div>}{/* Stale state indicator */}
            {/* Encounter Rating - Always visible */}
            <EncounterRating analysis={analysis || null} isPreliminary={isPreliminary} />

            {/* Median Performance Display - Main focus */}
            <MedianPerformanceDisplay analysis={analysis || null} isPreliminary={isPreliminary} />

            {/* Collapsible Details Section */}
            <div className={styles.detailsSection}>
                <button
                    className={styles.detailsToggle}
                    onClick={() => setDetailsExpanded(!detailsExpanded)}
                >
                    {detailsExpanded ? '🔽' : '▶️'} {detailsExpanded ? 'Hide' : 'Show'} Detailed Analysis
                </button>

                {detailsExpanded && hasRounds && value && lastRound && (
                    <div className={styles.detailsContent}>
                        {/* HP Bars Toggle Control */}
                        <div className={styles.toggleControl}>
                            <label className={styles.toggleLabel}>
                                <input
                                    type="checkbox"
                                    checked={hpBarsVisible}
                                    onChange={(e) => setHpBarsVisible(e.target.checked)}
                                    className={styles.toggleInput}
                                />
                                <span className={styles.toggleSwitch}></span>
                                <span className={styles.toggleText}>
                                    Show Round-by-Round HP Bars
                                </span>
                            </label>
                        </div>

                        {hpBarsVisible ? (
                            // Show round-by-round HP bars when toggle is enabled
                            value.rounds.map((round, roundIndex) => (
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
                            ))
                        ) : (
                            // Show simplified result view when HP bars are hidden
                            <div className={styles.round}>
                                <h3>Encounter Result</h3>
                                <div className={styles.resultSummary}>
                                    <div className={styles.summaryItem}>
                                        <strong>Duration:</strong> {value.rounds.length} rounds
                                    </div>
                                    <div className={styles.summaryItem}>
<strong>Winner:</strong> {
                                        (() => {
                                            const lastRound = value.rounds[value.rounds.length - 1]
                                            const team1Alive = lastRound.team1.filter(c => c.finalState.currentHp > 0).length
                                            const team2Alive = lastRound.team2.filter(c => c.finalState.currentHp > 0).length
                                            const maxRounds = 50; // Match the Rust simulation max rounds

                                            // Check if encounter is complete based on team HP sums
                                            const team1HasHp = lastRound.team1.some(c => c.finalState.currentHp > 0)
                                            const team2HasHp = lastRound.team2.some(c => c.finalState.currentHp > 0)

                                            if (team1HasHp && !team2HasHp) return "Players"
                                            if (team2HasHp && !team1HasHp) return "Monsters"
                                            if (!team1HasHp && !team2HasHp) return "Draw"
                                            if (value.rounds.length >= maxRounds) return "Draw"
                                            return "In Progress" // Don't declare draw until max rounds reached
                                        })()
                                    }
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* Always show final result */}
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