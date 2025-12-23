import React from 'react';
import styles from './descentGraph.module.scss';
import { PacingData } from './pacingUtils';
import { DecileStats } from '@/model/model';

interface DescentGraphProps {
    pacingData: PacingData;
    deciles: DecileStats[];
}

const DescentGraph: React.FC<DescentGraphProps> = ({ pacingData, deciles }) => {
    const { plannedTimeline, labels, vitalityTimeline, powerTimeline } = pacingData;
    const width = 400;
    const height = 200;
    const padding = 30;

    const steps = plannedTimeline.length;
    const xScale = (step: number) => padding + (step * (width - 2 * padding)) / (steps - 1);
    const yScale = (percent: number) => height - padding - (percent * (height - 2 * padding)) / 100;

    // Vitality Deciles
    const v25 = deciles[2]?.vitalityTimeline || [];
    const v75 = deciles[7]?.vitalityTimeline || [];
    
    // Power Deciles
    const p25 = deciles[2]?.powerTimeline || [];
    const p75 = deciles[7]?.powerTimeline || [];

    const getPathData = (timeline: number[]) => {
        if (!timeline || timeline.length === 0) return '';
        return timeline
            .map((percent, i) => `${i === 0 ? 'M' : 'L'} ${xScale(i)} ${yScale(percent)}`)
            .join(' ');
    };

    const getAreaData = (topTimeline: number[], bottomTimeline: number[]) => {
        if (!topTimeline || topTimeline.length === 0 || !bottomTimeline || bottomTimeline.length === 0) return '';
        const topPath = topTimeline.map((percent, i) => `L ${xScale(i)} ${yScale(percent)}`);
        const bottomPath = [...bottomTimeline]
            .reverse()
            .map((percent, i) => `L ${xScale(bottomTimeline.length - 1 - i)} ${yScale(percent)}`);
        
        return `M ${xScale(0)} ${yScale(topTimeline[0])} ${topPath.join(' ')} ${bottomPath.join(' ')} Z`;
    };

    return (
        <div className={styles.graphContainer}>
            <div className={styles.graphTitle}>Resource Attrition (The Descent)</div>
            <svg viewBox={`0 0 ${width} ${height}`} className={styles.svg}>
                {/* Grid Lines */}
                {[0, 25, 50, 75, 100].map(p => (
                    <g key={p}>
                        <line 
                            x1={padding} y1={yScale(p)} x2={width - padding} y2={yScale(p)} 
                            className={styles.grid} 
                        />
                        <text x={padding - 5} y={yScale(p) + 4} className={styles.axisLabel} textAnchor="end">{p}%</text>
                    </g>
                ))}

                {/* Risk Areas */}
                <path d={getAreaData(v75, v25)} className={`${styles.riskArea} ${styles.vitalityRisk}`} />
                <path d={getAreaData(p75, p25)} className={`${styles.riskArea} ${styles.powerRisk}`} />

                {/* Plan Line (Dotted) - Still representing Total EHP for context */}
                <path d={getPathData(plannedTimeline)} className={styles.planLine} strokeDasharray="4 4" />

                {/* Metric Lines */}
                <path d={getPathData(vitalityTimeline)} className={`${styles.metricLine} ${styles.vitalityLine}`} />
                <path d={getPathData(powerTimeline)} className={`${styles.metricLine} ${styles.powerLine}`} />

                {/* Data Points */}
                {vitalityTimeline.map((p, i) => (
                    <circle key={`v-dot-${i}`} cx={xScale(i)} cy={yScale(p)} r="2" className={styles.vitalityDot} />
                ))}
                {powerTimeline.map((p, i) => (
                    <circle key={`p-dot-${i}`} cx={xScale(i)} cy={yScale(p)} r="2" className={styles.powerDot} />
                ))}

                {/* X-Axis Labels (Centered between points) */}
                {labels.slice(1).map((label, i) => (
                    <text 
                        key={`seg-label-${i}`} 
                        x={(xScale(i) + xScale(i+1)) / 2} 
                        y={height - padding + 15} 
                        className={styles.axisLabel} 
                        textAnchor="middle"
                    >
                        {label}
                    </text>
                ))}
            </svg>
            
            <div className={styles.legend}>
                <span className={styles.legendItem}><span className={styles.vitalityKey}></span> Vitality (Hull)</span>
                <span className={styles.legendItem}><span className={styles.powerKey}></span> Power (Ammo)</span>
                <span className={styles.legendItem}><span className={styles.planKey}></span> Planned Attrition</span>
            </div>
        </div>
    );
};

export default DescentGraph;