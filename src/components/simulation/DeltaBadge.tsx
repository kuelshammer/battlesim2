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
        <div className={styles.deltaBadgeContainer}>
            <div className={`${styles.badge} ${zone.colorClass}`}>
                <span className={styles.icon}>{zone.icon}</span>
                <span className={styles.label}>{zone.label}</span>
                <span className={styles.value}>{sign}{Math.round(delta)}%</span>
            </div>
            {cumulativeDrift !== undefined && (
                <div className={styles.cumulativeDrift}>
                    Total Day Drift: {cumulativeDrift >= 0 ? '+' : ''}{Math.round(cumulativeDrift)}%
                </div>
            )}
        </div>
    );
};

export default DeltaBadge;
