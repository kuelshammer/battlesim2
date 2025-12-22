import React from 'react';
import styles from './assistantSummary.module.scss';

interface AssistantSummaryProps {
    actualCosts: number[];
    targetWeights: number[];
    finalResources: number; // EHP % at the end
}

const AssistantSummary: React.FC<AssistantSummaryProps> = ({ actualCosts, targetWeights, finalResources }) => {
    const totalWeight = targetWeights.reduce((a, b) => a + b, 0);
    const deltas = actualCosts.map((cost, i) => {
        const target = (targetWeights[i] / (totalWeight || 1)) * 100;
        return cost - target;
    });

    const maxDelta = Math.max(...deltas);
    const minDelta = Math.min(...deltas);
    
    const getSummary = () => {
        if (finalResources <= 0) {
            return {
                message: `⛔ Impossible Day. Party is projected to run out of resources before completing the day.`,
                statusClass: styles.red
            };
        }

        if (maxDelta > 10) {
            return {
                message: `🔴 Overtuned / Budget Hog. Some encounters are significantly harder than planned.`,
                statusClass: styles.red
            };
        }

        if (maxDelta > 5 || minDelta < -5) {
            return {
                message: `⚠️ Minor Pacing Drift. The day is slightly ${maxDelta > 5 ? 'harder' : 'easier'} than planned.`,
                statusClass: styles.yellow
            };
        }

        return {
            message: `✅ Balanced. The party is expected to reach the finale with ${Math.round(finalResources)}% resources.`,
            statusClass: styles.green
        };
    };

    const summary = getSummary();

    return (
        <div className={`${styles.summaryContainer} ${summary.statusClass}`}>
            <div className={styles.title}>Assistant Assessment</div>
            <div className={styles.message}>{summary.message}</div>
        </div>
    );
};

export default AssistantSummary;
