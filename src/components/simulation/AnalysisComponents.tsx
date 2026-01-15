import { FC, memo, useMemo } from "react"
import { AggregateOutput } from "@/model/model"
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
        <div className={styles.validationNotice} data-testid="validation-notice">
            {warnings.map((w, i) => (
                <div key={i} className={styles.warningItem} data-testid="validation-warning">
                    <FontAwesomeIcon icon={faExclamationCircle} /> {w}
                </div>
            ))}
        </div>
    );
});

export const VitalsDashboard: FC<{ analysis: AggregateOutput | null, isPreliminary?: boolean }> = memo(({ analysis, isPreliminary }) => {
    if (!analysis || !analysis.vitals) return null;

    const { vitals } = analysis;
    const { lethalityIndex, attritionScore, doomHorizon, difficultyGrade, tpkRisk, deathsDoorIndex, pacingLabel, crisisParticipationRate, nearDeathSurvivors } = vitals;

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
        <div className={`${styles.vitalsDashboard} ${isPreliminary ? styles.isUpdating : ''}`} data-testid="vitals-dashboard">
            <div className={styles.vitalsGrid}>
                {/* 1. Lethality Section */}
                <div className={styles.vitalsCard} data-testid="vitals-lethality">
                    <div className={styles.cardHeader}>
                        <FontAwesomeIcon icon={faSkull} className={styles.iconLethality} />
                        <span>Lethality</span>
                    </div>
                    <div className={styles.cardValue} data-testid="lethality-value">
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
                        <div className={styles.tpkWarning} data-testid="tpk-rate">
                            <FontAwesomeIcon icon={faExclamationTriangle} /> {Math.round(tpkRisk * 100)}% TPK Risk
                        </div>
                    )}
                </div>

                {/* 2. Attrition Section */}
                <div className={styles.vitalsCard} data-testid="vitals-attrition">
                    <div className={styles.cardHeader}>
                        <FontAwesomeIcon icon={faGasPump} className={styles.iconAttrition} />
                        <span>Attrition</span>
                    </div>
                    <div className={styles.cardValue} data-testid="attrition-value">
                        {Math.round(attritionScore * 100)}%
                    </div>
                    <div className={styles.cardLabel}>{getAttritionText(attritionScore)}</div>
                    <div className={styles.subtext}>Of Daily Budget</div>
                </div>

                {/* 3. Thrilling Section */}
                <div className={styles.vitalsCard} data-testid="vitals-thrilling">
                    <div className={styles.cardHeader}>
                        <FontAwesomeIcon icon={faBolt} className={styles.iconThrilling} />
                        <span>Thrilling</span>
                    </div>
                    <div className={styles.cardValue} data-testid="thrilling-value">
                        {deathsDoorIndex.toFixed(1)}
                    </div>
                    <div className={styles.cardLabel}>{getThrillingText(deathsDoorIndex)}</div>
                    <div className={styles.subtext}>Rounds at Death's Door</div>
                </div>

                {/* 4. Experience Section */}
                <div className={styles.vitalsCard} data-testid="vitals-experience">
                    <div className={styles.cardHeader}>
                        <FontAwesomeIcon icon={faBrain} className={styles.iconExperience} />
                        <span>Experience</span>
                    </div>
                    <div className={styles.cardValue}>
                        {crisisParticipationRate ? Math.round(crisisParticipationRate * 100) : 0}%
                    </div>
                    <div className={styles.cardLabel}>Engagement</div>
                    <div className={styles.subtext}>{nearDeathSurvivors ? nearDeathSurvivors.toFixed(1) : 0} Near-Death</div>
                </div>

                {/* 5. Forecast Section */}
                <div className={styles.vitalsCard} data-testid="vitals-forecast">
                    <div className={styles.cardHeader}>
                        <FontAwesomeIcon icon={faCompass} className={styles.iconForecast} />
                        <span>Forecast ({pacingLabel})</span>
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

        const { encounterLabel, intensityTier, analysisSummary, isGoodDesign, pacing, vitals } = analysis;
        const safetyGrade = vitals?.safetyGrade || (analysis as { safetyGrade?: string }).safetyGrade;
        
        const getGradeColor = (grade: string) => {
            if (isShortRest) return "#2c5282"; // Blue for short rest
            if (grade?.startsWith('A')) return "#28a745"; // Green
            if (grade?.startsWith('B')) return "#fd7e14"; // Orange
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
        <div className={styles.encounterRating} style={{ backgroundColor: ratingInfo.color }} data-testid="encounter-rating">
            <div className={styles.ratingHeader}>
                {ratingInfo.statusIcon && <FontAwesomeIcon icon={ratingInfo.statusIcon} className={styles.statusIcon} data-testid="rating-status-icon" />}
                <span className={styles.ratingTitle} data-testid="rating-title">
                    {ratingInfo.title}
                </span>
                {!isShortRest && (
                    <div className={styles.intensityBolts} data-testid="intensity-bolts">
                        {Array.from({ length: 4 }).map((_, i) => (
                            <FontAwesomeIcon 
                                key={i} 
                                icon={faBolt} 
                                className={i < ratingInfo.stars ? styles.boltFilled : styles.boltEmpty} 
                            />
                        ))}
                    </div>
                )}
                {isPreliminary && <span className={styles.preliminaryNotice} data-testid="preliminary-notice"> (ESTIMATING...)</span>}
            </div>
            
            {!isShortRest && (
                <div className={styles.ratingSubline} data-testid="rating-tier">
                    <span>{ratingInfo.tier}</span>
                </div>
            )}

            <div className={styles.ratingDetails} data-testid="rating-summary">
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

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
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
        
        const greyCount = totalSegments - greenCount - redCount;

        return [
            ...Array(greenCount).fill('green'),
            ...Array(redCount).fill('red'),
            ...Array(greyCount).fill('grey')
        ];
    };

    return (
        <div className={`${styles.bestDecileDisplay} ${isPreliminary ? styles.isEstimating : ''}`} data-testid="median-performance-display">
            <h4>
                {isDaySummary ? 'TYPICAL DAY END STATE' : 'TYPICAL ENCOUNTER END STATE'}
                {isPreliminary && <small>(REFINE IN PROGRESS...)</small>}
            </h4>
            
            <div className={styles.bestDecileHeader}>
                <div className={styles.survivorsBadge} data-testid="survivors-count">
                    {medianDecile.medianSurvivors} / {medianDecile.partySize} Survivors
                </div>
                <div className={styles.winRateBadge} data-testid="win-rate">
                    {Math.round(medianDecile.winRate)}% Survival Rate
                </div>
            </div>

            <div className={styles.bestDecileCombatants} data-testid="median-combatants">
                {medianDecile.medianRunVisualization.map((c, i) => (
                    <div key={i} className={styles.bestDecileCombatant} data-testid={`median-combatant-${i}`}>
                        <div className={styles.combatantName} data-testid="creature-name">
                            {c.name} {c.isDead && <span className={styles.deathIndicator} data-testid="death-indicator">(KO)</span>}
                        </div>
                        <div className={styles.hpBar}>
                            <div className={styles.hpBarContainer} data-testid="hp-bar-container">
                                <div className={styles.hpBarVisual} data-testid="hp-bar-visual">
                                    {renderHpBar(c.currentHp, c.startHp, c.maxHp).map((color, idx) => (
                                        <span key={idx} className={
                                            color === 'green' ? styles.segmentGreen :
                                            color === 'red' ? styles.segmentRed :
                                            styles.segmentGrey
                                        }>█</span>
                                    ))}
                                </div>
                                <div className={styles.hpText} data-testid="hp-text">
                                    {c.currentHp} / {c.maxHp} HP
                                </div>
                            </div>
                        </div>
                    </div>
                ))}
            </div>

            <div className={styles.bestDecileMetrics}>
                <div className={styles.metric} data-testid="attrition-cost">
                    Cost: <strong>{Math.round(medianDecile.hpLostPercent)}%</strong> daily budget
                </div>
                <div className={styles.metric} data-testid="battle-duration">
                    Duration: <strong>{medianDecile.battleDurationRounds}</strong> rounds
                </div>
            </div>
        </div>
    );
});
