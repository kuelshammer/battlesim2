import React, { FC, useMemo } from 'react';
import { AggregateOutput, DecileStats } from '@/model/model';
import styles from './pacingDashboard.module.scss';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faBolt, faCheckCircle, faExclamationTriangle, faArrowTrendDown, faArrowTrendUp } from '@fortawesome/free-solid-svg-icons';

interface BudgetStatus {
    color: string;
    zone: string;
    label: string;
    icon: any;
    delta: number;
}

export const getBudgetStatus = (targetPercent: number, actualPercent: number): BudgetStatus => {
    const delta = actualPercent - targetPercent;
    
    if (delta < -10.0) return { color: '#2196f3', zone: 'Major Under', label: 'Undertuned', icon: faArrowTrendDown, delta };
    if (delta < -5.0) return { color: '#00bcd4', zone: 'Minor Under', label: 'Slightly Easy', icon: faArrowTrendDown, delta };
    if (delta <= 5.0) return { color: '#4caf50', zone: 'Perfect', label: 'On Target', icon: faCheckCircle, delta };
    if (delta <= 10.0) return { color: '#ffeb3b', zone: 'Minor Over', label: 'Minor Drift', icon: faArrowTrendUp, delta };
    return { color: '#f44336', zone: 'Major Over', label: 'Overtuned', icon: faExclamationTriangle, delta };
};

export const DeltaBadge: FC<{ targetPercent: number, actualPercent: number, cumulativeDrift?: number }> = ({ targetPercent, actualPercent, cumulativeDrift }) => {
    const status = useMemo(() => getBudgetStatus(targetPercent, actualPercent), [targetPercent, actualPercent]);
    
    const displayActual = Math.round(actualPercent);
    const displayTarget = Math.round(targetPercent);
    const displayDelta = Math.round(status.delta);
    const prefix = displayDelta > 0 ? '+' : '';

    return (
        <div className={styles.deltaBadge} style={{ borderLeftColor: status.color }}>
            <div className={styles.badgeHeader}>
                <span className={styles.energyLabel}>
                    <FontAwesomeIcon icon={faBolt} /> Energy Drain: {displayActual}%
                </span>
                <span className={styles.targetLabel}>
                    (Target: {displayTarget}%)
                </span>
            </div>
            <div className={styles.badgeStatus} style={{ color: status.color }}>
                <FontAwesomeIcon icon={status.icon} />
                <span className={styles.statusText}>{status.label} ({prefix}{displayDelta}%)</span>
            </div>
            {cumulativeDrift !== undefined && (
                <div className={styles.cumulativeDrift}>
                    Total Day Drift: {cumulativeDrift > 0 ? '+' : ''}{Math.round(cumulativeDrift)}%
                </div>
            )}
        </div>
    );
};

export const FuelGauge: FC<{ analysis: AggregateOutput, weights: number[] }> = ({ analysis, weights }) => {
    const { globalMedian } = analysis;
    
    const resourceTimeline = globalMedian?.resourceTimeline || [];
    
    const totalWeight = weights.reduce((sum, w) => sum + w, 0);
    const planSegments = useMemo(() => {
        return weights.map((w, i) => ({
            label: `Enc ${i + 1}`,
            width: (w / totalWeight) * 100,
            color: `hsla(${200 + i * 40}, 40%, 40%, 0.5)`
        }));
    }, [weights, totalWeight]);

    // Reality segments
    const realitySegments = useMemo(() => {
        if (resourceTimeline.length < 2) return [];
        const segments = [];
        for (let i = 0; i < resourceTimeline.length - 1; i++) {
            const start = resourceTimeline[i];
            const end = resourceTimeline[i + 1];
            segments.push({
                label: `Enc ${i + 1}`,
                width: Math.max(0, start - end),
                color: `hsl(${200 + i * 40}, 70%, 50%)`
            });
        }
        return segments;
    }, [resourceTimeline]);

    const totalRealityWidth = realitySegments.reduce((sum, s) => sum + s.width, 0);

    return (
        <div className={styles.fuelGauge}>
            <h3><FontAwesomeIcon icon={faBolt} /> Daily Fuel Gauge</h3>
            
            <div className={styles.gaugeContainer}>
                <div className={styles.gaugeRow}>
                    <div className={styles.rowLabel}>PLAN</div>
                    <div className={styles.bar}>
                        {planSegments.map((s, i) => (
                            <div 
                                key={i} 
                                className={styles.segment} 
                                style={{ width: `${s.width}%`, backgroundColor: s.color }}
                            >
                                {s.width > 5 ? s.label : ''}
                            </div>
                        ))}
                    </div>
                </div>

                <div className={styles.gaugeRow}>
                    <div className={styles.rowLabel}>REALITY</div>
                    <div className={styles.bar}>
                        {realitySegments.map((s, i) => (
                            <div 
                                key={i} 
                                className={styles.segment} 
                                style={{ width: `${s.width}%`, backgroundColor: s.color }}
                                title={`${s.label}: ${Math.round(s.width)}% drain`}
                            >
                                {s.width > 5 ? s.label : ''}
                            </div>
                        ))}
                        {totalRealityWidth < 99 && <div className={styles.emptySpace} />}
                        {totalRealityWidth > 100 && (
                            <div className={styles.overBudget} title="Party is projected to run out of resources!">
                                EMPTY!
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
};
