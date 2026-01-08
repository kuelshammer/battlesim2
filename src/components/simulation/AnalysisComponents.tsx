import { FC, memo, useMemo } from "react"
import { AggregateOutput, DecileStats, CombatantVisualization } from "@/model/model"
import styles from './encounterResult.module.scss'
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { 
    faBrain, faTrophy, faCheckCircle, faExclamationTriangle, 
    faBolt, faSkull, faGasPump, faCompass, faExclamationCircle 
} from "@fortawesome/free-solid-svg-icons"

const formatLabel = (label: string): string => {
    switch (label) {
        case 'EpicChallenge': return 'The Epic Challenge';
        case 'TacticalGrinder': return 'The Tactical Grinder';
        case 'ActionMovie': return 'The Action Movie';
        case 'TheTrap': return 'The Trap';
        case 'TheSlog': return 'The Slog';
        case 'Standard': return 'Standard Encounter';
        case 'TrivialMinions': return 'Trivial / Minions';
        case 'TPKRisk': return 'High TPK Risk';
        case 'Broken': return 'Broken / Impossible';
        default: return label;
    }
};

const formatTier = (tier: string): string => {
    switch (tier) {
        case 'Tier1': return 'Tier 1 (Trivial)';
        case 'Tier2': return 'Tier 2 (Light)';
        case 'Tier3': return 'Tier 3 (Moderate)';
        case 'Tier4': return 'Tier 4 (Heavy)';
        case 'Tier5': return 'Tier 5 (Extreme)';
        default: return tier;
    }
};

export const ValidationNotice: FC<{ 
    analysis: AggregateOutput | null, 
    targetRole?: string,
    isDaySummary?: boolean 
}> = memo(({ analysis, targetRole, isDaySummary }) => {
    if (!analysis || !analysis.vitals) return null;

    const { vitals } = analysis;
    const { attritionScore } = vitals;

    const warnings: string[] = [];

    // 1. Role Mismatch Validation
    if (targetRole) {
        const costPct = attritionScore * 100;
        let expectedCategory = "";
        
        if (costPct < 8) expectedCategory = "Skirmish";
        else if (costPct < 20) expectedCategory = "Standard";
        else if (costPct < 45) expectedCategory = "Elite";
        else expectedCategory = "Boss";

        if (targetRole !== expectedCategory) {
            warnings.push(`You labeled this '${targetRole}', but it burns ${Math.round(costPct)}% budget, typical of a '${expectedCategory}' encounter.`);
        }
    }

    // 2. Budget Overflow Validation (Day Summary only)
    if (isDaySummary && analysis.tdnw > 0) {
        const totalCost = (analysis.globalMedian?.totalHpLost || 0) / analysis.tdnw;
        if (totalCost > 1.0) {
            warnings.push(`This adventuring day is projected to burn ${Math.round(totalCost * 100)}% of the daily budget. Total failure is mathematically likely.`);
        }
    }

    if (warnings.length === 0) return null;

    return (
        <div className={styles.validationNotice}>
            {warnings.map((w, i) => (
                <div key={i} className={styles.warningItem}>
                    <FontAwesomeIcon icon={faExclamationCircle} /> {w}
                </div>
            ))}
        </div>
    );
});

export const VitalsDashboard: FC<{ analysis: AggregateOutput | null, isPreliminary?: boolean }> = memo(({ analysis, isPreliminary }) => {
    if (!analysis || !analysis.vitals) return null;

    const { vitals } = analysis;
    const { lethalityIndex, attritionScore, doomHorizon, difficultyGrade, tpkRisk, deathsDoorIndex } = vitals;

    const getGradeColor = (grade: string) => {
        switch (grade) {
            case 'S': return '#10b981'; // Emerald
            case 'A': return '#22c55e'; // Green
            case 'B': return '#3b82f6'; // Blue
            case 'C': return '#f59e0b'; // Amber
            case 'D': return '#ef4444'; // Red
            case 'F': return '#7f1d1d'; // Dark Red
            default: return '#6b7280';
        }
    };

    const getLethalityText = (index: number) => {
        if (index === 0) return "Safe";
        if (index < 0.05) return "Negligible";
        if (index < 0.15) return "Easy";
        if (index < 0.30) return "Moderate";
        if (index < 0.50) return "Risky";
        return "Lethal";
    };

    const getAttritionText = (score: number) => {
        if (score < 0.08) return "Free";
        if (score < 0.20) return "Standard";
        if (score < 0.45) return "Taxing";
        if (score < 0.60) return "Heavy";
        return "Nova";
    };

    const getThrillingText = (index: number) => {
        if (index === 0) return "Boring";
        if (index < 0.5) return "Chilled";
        if (index < 1.5) return "Tense";
        if (index < 3.0) return "Thrilling";
        return "Adrenaline";
    };

    return (
        <div className={`${styles.vitalsDashboard} ${isPreliminary ? styles.isUpdating : ''}`}>
            <div className={styles.vitalsGrid}>
                {/* 1. Lethality Section */}
                <div className={styles.vitalsCard}>
                    <div className={styles.cardHeader}>
                        <FontAwesomeIcon icon={faSkull} className={styles.iconLethality} />
                        <span>Lethality</span>
                    </div>
                    <div className={styles.cardValue}>
                        {Math.round(lethalityIndex * 100)}%
                    </div>
                    <div className={styles.cardLabel}>{getLethalityText(lethalityIndex)}</div>
                    <div className={styles.riskBarContainer}>
                        <div 
                            className={styles.riskBarFill} 
                            style={{ width: `${lethalityIndex * 100}%`, backgroundColor: getGradeColor(difficultyGrade) }} 
                        />
                    </div>
                    {tpkRisk > 0.05 && (
                        <div className={styles.tpkWarning}>
                            <FontAwesomeIcon icon={faExclamationTriangle} /> {Math.round(tpkRisk * 100)}% TPK Risk
                        </div>
                    )}
                </div>

                {/* 2. Attrition Section */}
                <div className={styles.vitalsCard}>
                    <div className={styles.cardHeader}>
                        <FontAwesomeIcon icon={faGasPump} className={styles.iconAttrition} />
                        <span>Attrition</span>
                    </div>
                    <div className={styles.cardValue}>
                        {Math.round(attritionScore * 100)}%
                    </div>
                    <div className={styles.cardLabel}>{getAttritionText(attritionScore)}</div>
                    <div className={styles.subtext}>Of Daily Budget</div>
                </div>

                {/* 3. Thrilling Section */}
                <div className={styles.vitalsCard}>
                    <div className={styles.cardHeader}>
                        <FontAwesomeIcon icon={faBolt} className={styles.iconThrilling} />
                        <span>Thrilling</span>
                    </div>
                    <div className={styles.cardValue}>
                        {deathsDoorIndex.toFixed(1)}
                    </div>
                    <div className={styles.cardLabel}>{getThrillingText(deathsDoorIndex)}</div>
                    <div className={styles.subtext}>Rounds at Death's Door</div>
                </div>

                {/* 4. Forecast Section */}
                <div className={styles.vitalsCard}>
                    <div className={styles.cardHeader}>
                        <FontAwesomeIcon icon={faCompass} className={styles.iconForecast} />
                        <span>Forecast</span>
                    </div>
                    <div className={styles.cardValue}>
                        {doomHorizon > 10 ? '∞' : doomHorizon.toFixed(1)}
                    </div>
                    <div className={styles.cardLabel}>Encounters</div>
                    <div className={styles.subtext}>Until Failure</div>
                </div>
            </div>
        </div>
    );
});

export const EncounterRating: FC<{ analysis: AggregateOutput | null, isPreliminary?: boolean, label?: string, isShortRest?: boolean }> = memo(({ analysis, isPreliminary, label = "ENCOUNTER", isShortRest }) => {
    const isDaySummary = label.toLowerCase().includes("day");

    const ratingInfo = useMemo(() => {
        if (!analysis || !analysis.deciles?.length) return null;

        const { encounterLabel, safetyGrade, intensityTier, analysisSummary, isGoodDesign, pacing } = analysis as any;
        
        const getGradeColor = (grade: string) => {
            if (isShortRest) return "#2c5282"; // Blue for short rest
            if (grade.startsWith('A')) return "#28a745"; // Green
            if (grade.startsWith('B')) return "#fd7e14"; // Orange
            return "#dc3545"; // Red for C, D, F
        };

        let displayLabel = isShortRest ? "SHORT REST" : formatLabel(encounterLabel);
        let statusIcon = null;

        if (isDaySummary && pacing) {
            displayLabel = pacing.archetype.toUpperCase();
            if (safetyGrade === 'A' && intensityTier === 'Tier4') {
                statusIcon = faTrophy;
            } else if (isGoodDesign) {
                statusIcon = faCheckCircle;
            }
            else {
                statusIcon = faExclamationTriangle;
            }
        }

        const boltCount = intensityTier === 'Tier5' ? 4 : Math.max(0, parseInt(intensityTier.replace('Tier', '')) - 1);

        return {
            title: displayLabel,
            tier: formatTier(intensityTier || "Tier1"),
            summary: analysisSummary,
            color: getGradeColor(String(safetyGrade || 'A')),
            statusIcon,
            stars: boltCount
        };
    }, [analysis, isDaySummary, isShortRest]);

    if (!ratingInfo) return null;

    return (
        <div className={styles.encounterRating} style={{ backgroundColor: ratingInfo.color }}>
            <div className={styles.ratingHeader}>
                {ratingInfo.statusIcon && <FontAwesomeIcon icon={ratingInfo.statusIcon} className={styles.statusIcon} />}
                <span className={styles.ratingTitle}>
                    {ratingInfo.title}
                </span>
                {!isShortRest && (
                    <div className={styles.intensityBolts}>
                        {Array.from({ length: 4 }).map((_, i) => (
                            <FontAwesomeIcon 
                                key={i} 
                                icon={faBolt} 
                                className={i < ratingInfo.stars ? styles.boltFilled : styles.boltEmpty} 
                            />
                        ))}
                    </div>
                )}
                {isPreliminary && <span className={styles.preliminaryNotice}> (ESTIMATING...)</span>}
            </div>
            
            {!isShortRest && (
                <div className={styles.ratingSubline}>
                    <span>{ratingInfo.tier}</span>
                </div>
            )}

            <div className={styles.ratingDetails}>
                <span>{ratingInfo.summary}</span>
            </div>
        </div>
    );
});

export const MedianPerformanceDisplay: FC<{ analysis: AggregateOutput | null, isPreliminary?: boolean, isDaySummary?: boolean }> = memo(({ analysis, isPreliminary, isDaySummary }) => {
    const medianDecile = useMemo(() => {
        if (!analysis) return null;
        if (analysis.globalMedian) return analysis.globalMedian;
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
        const totalSegments = 10;
        
        const greenCount = Math.floor((currentHp / maxHp) * totalSegments);
        const redCount = isDaySummary 
            ? totalSegments - greenCount 
            : Math.floor((Math.max(0, startHp - currentHp) / maxHp) * totalSegments);
        
        const greyCount = isDaySummary ? 0 : Math.max(0, totalSegments - greenCount - redCount);
        
        const segments = [];
        for (let i = 0; i < greenCount; i++) segments.push(<span key={`g-${i}`} className={styles.segmentGreen}>█</span>);
        for (let i = 0; i < redCount; i++) segments.push(<span key={`r-${i}`} className={styles.segmentRed}>█</span>);
        for (let i = 0; i < greyCount; i++) segments.push(<span key={`gr-${i}`} className={styles.segmentGrey}>░</span>);
        
        while (segments.length < totalSegments) segments.push(<span key={`f-${segments.length}`} className={isDaySummary ? styles.segmentRed : styles.segmentGrey}>
            {isDaySummary ? '█' : '░'}
        </span>);
        return segments.slice(0, totalSegments);
    };

    const avgFinalHp = medianDecile.medianRunVisualization
        ? (medianDecile.medianRunVisualization.reduce((sum, c) => sum + c.hpPercentage, 0) / medianDecile.medianRunVisualization.length).toFixed(1)
        : '0.0';

    const filteredCombatants = isDaySummary 
        ? (medianDecile.medianRunVisualization || []).filter(c => c.isPlayer)
        : (medianDecile.medianRunVisualization || []);

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
                {filteredCombatants.length === 0 ? (
                    <div className={styles.emptyCombatants}>No combatants to display</div>
                ) : filteredCombatants.map((combatant, index) => (
                    <div key={index} className={styles.bestDecileCombatant}>
                        <div className={styles.combatantName}>
                            {combatant.name}
                            {combatant.isDead && <span className={styles.deathIndicator}> 💀 Dead</span>}
                            {!combatant.isPlayer && <span className={styles.monsterLabel}> (Monster)</span>}
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