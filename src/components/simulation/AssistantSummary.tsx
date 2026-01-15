import React from 'react';
import styles from './assistantSummary.module.scss';
import { PacingData } from './pacingUtils';

interface AssistantSummaryProps {
    pacingData: PacingData;
}

const AssistantSummary: React.FC<AssistantSummaryProps> = ({ pacingData }) => {
    const { actualSegments, plannedSegments, finalResources, finalVitality, finalPower } = pacingData;
    
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
    
    const getPartyState = (vitality: number, power: number) => {
        if (vitality >= 50 && power >= 50) return { label: 'Fresh', desc: 'The party is ready for anything.', class: styles.green };
        if (vitality >= 50 && power < 50) return { label: 'Exhausted', desc: 'The party is healthy, but they are out of resources. Expect a slog.', class: styles.yellow };
        if (vitality < 50 && power >= 50) return { label: 'Glass Cannon', desc: 'High firepower, but one bad round could mean TPK.', class: styles.yellow };
        return { label: 'Doomed', desc: 'The party needs a Long Rest immediately.', class: styles.red };
    };

    const state = getPartyState(finalVitality, finalPower);

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
        <div className={styles.summaryContainer} data-testid="assistant-summary">
            <div className={`${styles.assessment} ${summary.statusClass}`} data-testid="assistant-assessment">
                <div className={styles.title}>Assistant Assessment</div>
                <div className={styles.message} data-testid="assistant-message">{summary.message}</div>
            </div>
            
            <div className={`${styles.partyState} ${state.class}`} data-testid="party-state">
                <div className={styles.title}>Party State: {state.label}</div>
                <div className={styles.message}>{state.desc}</div>
                <div className={styles.metrics} data-testid="party-metrics">
                    <span data-testid="final-vitality">❤️ Vitality: {Math.round(finalVitality)}%</span>
                    <span data-testid="final-power">⚡ Power: {Math.round(finalPower)}%</span>
                </div>
            </div>
        </div>
    );
};

export default AssistantSummary;