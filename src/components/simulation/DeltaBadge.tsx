import React from 'react';
import styles from './deltaBadge.module.scss';

interface DeltaBadgeProps {
    targetCost: number;
    actualCost: number;
    cumulativeDrift?: number;
}

const DeltaBadge: React.FC<DeltaBadgeProps> = ({ targetCost, actualCost, cumulativeDrift }) => {
    const delta = actualCost - targetCost;

    const getZone = (d: number) => {
        if (d < -10) return { label: 'Undertuned', colorClass: styles.majorUnder, icon: '🔵' };
        if (d < -5) return { label: 'Slightly Easy', colorClass: styles.minorUnder, icon: '💠' };
        if (d <= 5) return { label: 'On Target', colorClass: styles.perfect, icon: '🟢' };
        if (d <= 10) return { label: 'Minor Drift', colorClass: styles.minorOver, icon: '🟡' };
        return { label: 'Overtuned', colorClass: styles.majorOver, icon: '🔴' };
    };

    const zone = getZone(delta);
    const sign = delta >= 0 ? '+' : '';

    return (
        <div className={styles.deltaBadgeContainer} data-testid="delta-badge">
            <div className={`${styles.badge} ${zone.colorClass}`} data-testid="delta-zone">
                <span className={styles.icon} data-testid="delta-icon">{zone.icon}</span>
                <span className={styles.label} data-testid="delta-label">{zone.label}</span>
                <span className={styles.value} data-testid="delta-value">{sign}{Math.round(delta)}%</span>
            </div>
            {cumulativeDrift !== undefined && (
                <div className={styles.cumulativeDrift} data-testid="cumulative-drift">
                    Total Day Drift: {cumulativeDrift >= 0 ? '+' : ''}{Math.round(cumulativeDrift)}%
                </div>
            )}
        </div>
    );
};

export default DeltaBadge;
