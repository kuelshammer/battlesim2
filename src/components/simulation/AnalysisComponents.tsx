import { FC, memo, useMemo } from "react"
import { AggregateOutput, DecileStats } from "@/model/model"
import styles from './encounterResult.module.scss'
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faBrain } from "@fortawesome/free-solid-svg-icons"

export const EncounterRating: FC<{ analysis: AggregateOutput | null, isPreliminary?: boolean, label?: string }> = memo(({ analysis, isPreliminary, label = "ENCOUNTER" }) => {
    const ratingInfo = useMemo(() => {
        if (!analysis || !analysis.deciles?.length) return null;

        // Use backend provided labels and grades
        const { encounterLabel, safetyGrade, intensityTier, analysisSummary } = analysis as any;
        
        // Map safety grade to color
        const getGradeColor = (grade: string) => {
            if (grade.startsWith('A')) return "#28a745";
            if (grade.startsWith('B')) return "#20c997";
            if (grade.startsWith('C')) return "#ffc107";
            if (grade.startsWith('D')) return "#fd7e14";
            return "#dc3545";
        };

        return {
            label: encounterLabel || "Standard",
            grade: safetyGrade || "A",
            tier: intensityTier || "Tier 1",
            summary: analysisSummary,
            color: getGradeColor(String(safetyGrade || 'A'))
        };
    }, [analysis]);

    if (!ratingInfo) return null;

    return (
        <div className={styles.encounterRating} style={{ backgroundColor: ratingInfo.color }}>
            <span className={styles.ratingText}>
                {label.toUpperCase()}: {String(ratingInfo.label).toUpperCase()} ({ratingInfo.grade})
                {isPreliminary && <span className={styles.preliminaryNotice}> (ESTIMATING...)</span>}
            </span>
            <div className={styles.ratingDetails}>
                <span>{ratingInfo.summary}</span>
            </div>
        </div>
    );
});

export const MedianPerformanceDisplay: FC<{ analysis: AggregateOutput | null, isPreliminary?: boolean }> = memo(({ analysis, isPreliminary }) => {
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
        const newDamage = Math.max(0, startHp - currentHp);
        const redCount = Math.floor((newDamage / maxHp) * totalSegments);
        const greyCount = Math.max(0, totalSegments - greenCount - redCount);
        
        const segments = [];
        for (let i = 0; i < greenCount; i++) segments.push(<span key={`g-${i}`} className={styles.segmentGreen}>█</span>);
        for (let i = 0; i < redCount; i++) segments.push(<span key={`r-${i}`} className={styles.segmentRed}>█</span>);
        for (let i = 0; i < greyCount; i++) segments.push(<span key={`gr-${i}`} className={styles.segmentGrey}>░</span>);
        
        while (segments.length < totalSegments) segments.push(<span key={`f-${segments.length}`} className={styles.segmentGrey}>░</span>);
        return segments.slice(0, totalSegments);
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