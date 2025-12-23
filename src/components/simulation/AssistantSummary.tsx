import React from 'react';
import styles from './assistantSummary.module.scss';
import { PacingData } from './pacingUtils';

interface AssistantSummaryProps {
    pacingData: PacingData;
}

const AssistantSummary: React.FC<AssistantSummaryProps> = ({ pacingData }) => {
    const { actualSegments, plannedSegments, finalResources } = pacingData;
    
    // Calculate deltas only for combat encounters
    const deltas: number[] = [];
    actualSegments.forEach((actual, i) => {
        if (actual.type === 'combat') {
            const planned = plannedSegments[i];
            if (planned && planned.type === 'combat') {
                deltas.push(actual.percent - planned.percent);
            }
        }
    });

    const maxDelta = deltas.length > 0 ? Math.max(...deltas) : 0;
    const minDelta = deltas.length > 0 ? Math.min(...deltas) : 0;
    
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
