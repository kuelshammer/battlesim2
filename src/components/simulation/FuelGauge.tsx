import React from 'react';
import styles from './fuelGauge.module.scss';
import { PacingSegment } from './pacingUtils';

interface FuelGaugeProps {
    plannedSegments: PacingSegment[];
    actualSegments: PacingSegment[];
}

const FuelGauge: React.FC<FuelGaugeProps> = ({ plannedSegments, actualSegments }) => {
    // We use two shades of Orange/Red for alternating combat segments
    const combatColors = ['#f28e2c', '#e15759']; 
    const restColor = '#59a14f'; // Green for filling the tank

    const renderBar = (segments: PacingSegment[]) => {
        let combatCounter = 0;
        return (
            <div className={styles.bar}>
                {segments.map((seg, i) => {
                    const isCombat = seg.type === 'combat';
                    const color = isCombat ? combatColors[combatCounter++ % 2] : restColor;
                    
                    if (seg.type === 'shortRest') {
                        return (
                            <div 
                                key={seg.id || i} 
                                className={styles.restDivider} 
                                style={{ backgroundColor: color }}
                                title="Short Rest (Refill)"
                            />
                        );
                    }

                    return (
                        <div
                            key={seg.id || i}
                            className={styles.segment}
                            style={{ width: `${seg.percent}%`, backgroundColor: color }}
                            title={`${seg.label}: ${Math.round(seg.percent)}%`}
                        >
                            {seg.percent > 8 && (
                                <span className={styles.segmentLabel}>
                                    {seg.label}: {Math.round(seg.percent)}%
                                </span>
                            )}
                        </div>
                    );
                })}
            </div>
        );
    };

    const totalActual = actualSegments.reduce((sum, s) => sum + s.percent, 0);

    return (
        <div className={styles.fuelGaugeContainer}>
            <div className={styles.gaugeRow}>
                <div className={styles.rowLabel}>The Plan</div>
                {renderBar(plannedSegments)}
            </div>

            <div className={styles.gaugeRow}>
                <div className={styles.rowLabel}>The Reality</div>
                <div className={styles.barContainer}>
                    {renderBar(actualSegments)}
                    {totalActual > 100.1 && (
                        <div className={styles.emptyMarker}>
                            ⚠️ Tank Overdrawn! ({Math.round(totalActual)}%)
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

export default FuelGauge;
