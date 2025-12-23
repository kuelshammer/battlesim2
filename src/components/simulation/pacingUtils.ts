import { TimelineEvent, FullAnalysisOutput } from "@/model/model";

export interface PacingSegment {
    type: 'combat' | 'shortRest';
    percent: number;
    label: string;
    id: string;
}

export interface PacingData {
    actualSegments: PacingSegment[];
    plannedSegments: PacingSegment[];
    grandTotalBudget: number;
    initialEhp: number;
    totalRecovery: number;
    totalWeight: number;
    finalResources: number;
    actualCosts: number[]; // Combat-only costs
    cumulativeDrifts: number[]; // Combat-only drifts
    plannedTimeline: number[]; // Points for the line chart
    labels: string[]; // Labels for the X-axis
}

export function calculatePacingData(
    timeline: TimelineEvent[],
    analysis: FullAnalysisOutput | null,
    encounterWeights: number[]
): PacingData | null {
    if (!analysis?.overall?.globalMedian?.resourceTimeline) return null;

    const resTimeline = analysis.overall.globalMedian.resourceTimeline;
    const tdnw = analysis.overall.tdnw;
    
    // Initial EHP is what the party started with at step 0
    const initialEhp = (resTimeline[0] / 100) * tdnw;

    let totalRecovery = 0;
    const stepChanges: { type: 'combat' | 'shortRest', val: number, id: string }[] = [];

    // Map each timeline event to its resource change
    timeline.forEach((item, i) => {
        const startEhp = (resTimeline[i] / 100) * tdnw;
        const endEhp = (resTimeline[i + 1] / 100) * tdnw;
        const change = endEhp - startEhp;

        if (change > 0) {
            totalRecovery += change;
        }

        stepChanges.push({
            type: item.type === 'combat' ? 'combat' : 'shortRest',
            val: change,
            id: item.id || `step-${i}`
        });
    });

    const grandTotalBudget = initialEhp + totalRecovery;

    // Map actual segments
    let actualCombatCount = 0;
    const actualCosts: number[] = [];
    const actualSegments: PacingSegment[] = stepChanges.map((change) => {
        const absVal = Math.abs(change.val);
        const percentOfBudget = grandTotalBudget > 0 ? (absVal / grandTotalBudget) * 100 : 0;
        
        if (change.type === 'combat') {
            actualCombatCount++;
            actualCosts.push(percentOfBudget);
            return {
                type: 'combat',
                percent: percentOfBudget,
                label: `Enc ${actualCombatCount}`,
                id: change.id
            };
        } else {
            return {
                type: 'shortRest',
                percent: 0, 
                label: 'Rest',
                id: change.id
            };
        }
    });

    // Map planned segments (using weights)
    const totalWeight = encounterWeights.reduce((a, b) => a + b, 0);
    let plannedCombatCount = 0;
    const plannedSegments: PacingSegment[] = timeline.map((item) => {
        if (item.type === 'combat') {
            const weight = encounterWeights[plannedCombatCount];
            plannedCombatCount++;
            return {
                type: 'combat' as const,
                percent: totalWeight > 0 ? (weight / totalWeight) * 100 : 0,
                label: `Enc ${plannedCombatCount}`,
                id: item.id || `plan-${plannedCombatCount}`
            };
        } else {
            return {
                type: 'shortRest' as const,
                percent: 0,
                label: 'Rest',
                id: item.id || `rest-${plannedCombatCount}`
            };
        }
    });

    // Calculate drifts (combat-only)
    let currentDrift = 0;
    const cumulativeDrifts: number[] = [];
    const combatOnlyPlanned = plannedSegments.filter(s => s.type === 'combat');
    actualCosts.forEach((cost, i) => {
        const target = combatOnlyPlanned[i]?.percent || 0;
        currentDrift += (cost - target);
        cumulativeDrifts.push(currentDrift);
    });

    // Calculate Planned Timeline points
    // Y-axis is % of TDNW (Max HP)
    const plannedTimeline = [resTimeline[0]];
    const labels = ['Start'];
    let currentPlannedEhp = initialEhp;
    let combatIdx = 0;

    timeline.forEach((item, i) => {
        if (item.type === 'combat') {
            const weight = encounterWeights[combatIdx];
            const weightPercent = totalWeight > 0 ? (weight / totalWeight) : 0;
            const plannedDrain = weightPercent * grandTotalBudget;
            currentPlannedEhp -= plannedDrain;
            combatIdx++;
            labels.push(`E${combatIdx}`);
        } else {
            // Apply recovery to plan
            const recovery = stepChanges[i].val; // Actual recovery used as target
            currentPlannedEhp += recovery;
            labels.push('Rest');
        }
        plannedTimeline.push(tdnw > 0 ? (currentPlannedEhp / tdnw) * 100 : 0);
    });

    return {
        actualSegments,
        plannedSegments,
        grandTotalBudget,
        initialEhp,
        totalRecovery,
        totalWeight,
        finalResources: resTimeline[resTimeline.length - 1],
        actualCosts,
        cumulativeDrifts,
        plannedTimeline,
        labels
    };
}
