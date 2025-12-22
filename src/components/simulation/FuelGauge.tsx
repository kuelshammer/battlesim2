import React from 'react';
import styles from './fuelGauge.module.scss';

interface FuelGaugeProps {
    plannedWeights: number[];
    actualCosts: number[];
}

const FuelGauge: React.FC<FuelGaugeProps> = ({ plannedWeights, actualCosts }) => {
    const totalWeight = plannedWeights.reduce((a, b) => a + b, 0);
    const totalActual = actualCosts.reduce((a, b) => a + b, 0);

    const colors = ['#4e79a7', '#f28e2c', '#e15759', '#76b7b2', '#59a14f', '#edc949', '#af7aa1', '#ff9da7', '#9c755f', '#bab0ab'];

    const getPlannedSegments = () => {
        if (totalWeight === 0) return [];
        return plannedWeights.map((w, i) => ({
            width: (w / totalWeight) * 100,
            color: colors[i % colors.length],
            label: `Enc ${i + 1}`
        }));
    };

    const getActualSegments = () => {
        return actualCosts.map((c, i) => ({
            width: c, // width in percentage of the bar
            color: colors[i % colors.length],
            label: `Enc ${i + 1}`
        }));
    };

    return (
        <div className={styles.fuelGaugeContainer}>
            <div className={styles.gaugeRow}>
                <div className={styles.rowLabel}>The Plan</div>
                <div className={styles.bar}>
                    {getPlannedSegments().map((seg, i) => (
                        <div
                            key={i}
                            className={styles.segment}
                            style={{ width: `${seg.width}%`, backgroundColor: seg.color }}
                            title={`${seg.label}: ${Math.round(seg.width)}%`}
                        />
                    ))}
                </div>
            </div>

            <div className={styles.gaugeRow}>
                <div className={styles.rowLabel}>The Reality</div>
                <div className={styles.barContainer}>
                    <div className={styles.bar}>
                        {getActualSegments().map((seg, i) => (
                            <div
                                key={i}
                                className={styles.segment}
                                style={{ width: `${seg.width}%`, backgroundColor: seg.color }}
                                title={`${seg.label}: ${Math.round(seg.width)}%`}
                            />
                        ))}
                    </div>
                    {totalActual > 100 && (
                        <div className={styles.emptyMarker}>
                            ⚠️ Tank Empty! ({Math.round(totalActual)}%)
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

export default FuelGauge;
