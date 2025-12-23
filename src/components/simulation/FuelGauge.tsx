import React from 'react';
import styles from './fuelGauge.module.scss';
import { PacingData, PacingSegment } from './pacingUtils';

interface FuelGaugeProps {
    pacingData: PacingData;
}

const FuelGauge: React.FC<FuelGaugeProps> = ({ pacingData }) => {
    const { vitalitySegments, powerSegments, plannedSegments } = pacingData;

    const renderBar = (segments: PacingSegment[], baseColors: [string, string], restColor: string) => {
        let combatCounter = 0;
        return (
            <div className={styles.bar}>
                {segments.map((seg, i) => {
                    const isCombat = seg.type === 'combat';
                    const color = isCombat ? baseColors[combatCounter++ % 2] : restColor;
                    
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
                            {seg.percent > 10 && (
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

    const vitColors: [string, string] = ['#e15759', '#c0392b'];
    const powColors: [string, string] = ['#4e79a7', '#2980b9'];
    const planColors: [string, string] = ['#7f8c8d', '#95a5a6'];
    const restColor = '#59a14f';

    return (
        <div className={styles.fuelGaugeContainer}>
            <div className={styles.gaugeRow}>
                <div className={styles.rowLabel}>Daily Budget Plan (Total EHP)</div>
                {renderBar(plannedSegments, planColors, restColor)}
            </div>

            <div className={styles.gaugeRow}>
                <div className={styles.rowLabel}>❤️ Vitality Attrition</div>
                {renderBar(vitalitySegments, vitColors, restColor)}
            </div>

            <div className={styles.gaugeRow}>
                <div className={styles.rowLabel}>⚡ Power Attrition</div>
                {renderBar(powerSegments, powColors, restColor)}
            </div>
        </div>
    );
};

export default FuelGauge;