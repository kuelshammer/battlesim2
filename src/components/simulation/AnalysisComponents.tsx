import { FC, memo, useMemo } from "react"
import { AggregateOutput, DecileStats, CombatantVisualization } from "@/model/model"
import styles from './encounterResult.module.scss'
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faBrain, faTrophy, faCheckCircle, faExclamationTriangle } from "@fortawesome/free-solid-svg-icons"

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

export const EncounterRating: FC<{ analysis: AggregateOutput | null, isPreliminary?: boolean, label?: string }> = memo(({ analysis, isPreliminary, label = "ENCOUNTER" }) => {
    const isDaySummary = label.toLowerCase().includes("day");

    const ratingInfo = useMemo(() => {
        if (!analysis || !analysis.deciles?.length) return null;

        const { encounterLabel, safetyGrade, intensityTier, analysisSummary, isGoodDesign } = analysis as any;
        
        const getGradeColor = (grade: string) => {
            if (grade.startsWith('A')) return "#28a745";
            if (grade.startsWith('B')) return "#20c997";
            if (grade.startsWith('C')) return "#ffc107";
            if (grade.startsWith('D')) return "#fd7e14";
            return "#dc3545";
        };

        let displayLabel = formatLabel(encounterLabel);
        let statusIcon = null;

        if (isDaySummary) {
            if (safetyGrade === 'B' && intensityTier === 'Tier5') {
                displayLabel = "🏆 PERFECT ADVENTURING DAY";
                statusIcon = faTrophy;
            } else if (isGoodDesign) {
                displayLabel = "✅ WELL BALANCED DAY";
                statusIcon = faCheckCircle;
            } else {
                displayLabel = "⚠️ IMBALANCED DAY";
                statusIcon = faExclamationTriangle;
            }
        }

        return {
            title: displayLabel,
            grade: safetyGrade || "A",
            tier: formatTier(intensityTier || "Tier1"),
            summary: analysisSummary,
            color: getGradeColor(String(safetyGrade || 'A')),
            statusIcon
        };
    }, [analysis, isDaySummary]);

    if (!ratingInfo) return null;

    return (
        <div className={styles.encounterRating} style={{ backgroundColor: ratingInfo.color }}>
            <div className={styles.ratingHeader}>
                {ratingInfo.statusIcon && <FontAwesomeIcon icon={ratingInfo.statusIcon} className={styles.statusIcon} />}
                <span className={styles.ratingTitle}>
                    {ratingInfo.title}
                </span>
                {isPreliminary && <span className={styles.preliminaryNotice}> (ESTIMATING...)</span>}
            </div>
            
            <div className={styles.ratingSubline}>
                <span>Grade {ratingInfo.grade}</span>
                <span className={styles.separator}>|</span>
                <span>{ratingInfo.tier}</span>
            </div>

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

    // IMPORTANT: In Day Summary mode, we only show players.
    // Otherwise, we show EVERYTHING (Players and Monsters).
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