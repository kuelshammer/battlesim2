import React from 'react';
import styles from './descentGraph.module.scss';
import { PacingData } from './pacingUtils';

interface DescentGraphProps {
    decileTimelines: number[][]; // 10 timelines, each an array of EHP %
    pacingData: PacingData;
}

const DescentGraph: React.FC<DescentGraphProps> = ({ decileTimelines, pacingData }) => {
    const { plannedTimeline, labels } = pacingData;
    const width = 400;
    const height = 200;
    const padding = 30;

    const steps = plannedTimeline.length;
    const xScale = (step: number) => padding + (step * (width - 2 * padding)) / (steps - 1);
    const yScale = (percent: number) => height - padding - (percent * (height - 2 * padding)) / 100;

    // Median is decile 4 (50th percentile)
    const medianTimeline = decileTimelines[4] || [];
    const p25Timeline = decileTimelines[2] || [];
    const p75Timeline = decileTimelines[7] || [];

    const getPathData = (timeline: number[]) => {
        if (timeline.length === 0) return '';
        return timeline
            .map((percent, i) => `${i === 0 ? 'M' : 'L'} ${xScale(i)} ${yScale(percent)}`)
            .join(' ');
    };

    const getAreaData = (topTimeline: number[], bottomTimeline: number[]) => {
        if (topTimeline.length === 0 || bottomTimeline.length === 0) return '';
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
                {/* Rest Background Highlighting */}
                {labels.map((label, i) => {
                    if (label === 'Rest' && i > 0) {
                        return (
                            <rect 
                                key={`rest-bg-${i}`}
                                x={xScale(i-1)} 
                                y={padding} 
                                width={xScale(i) - xScale(i-1)} 
                                height={height - 2 * padding}
                                className={styles.restBackground}
                            />
                        );
                    }
                    return null;
                })}

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

                {/* Shaded Risk Area (25th to 75th) */}
                <path d={getAreaData(p75Timeline, p25Timeline)} className={styles.riskArea} />

                {/* Plan Line (Dotted) */}
                <path d={getPathData(plannedTimeline)} className={styles.planLine} strokeDasharray="4 4" />

                {/* Median Line (Solid) */}
                <path d={getPathData(medianTimeline)} className={styles.medianLine} />

                {/* Data Points (Dots) */}
                {medianTimeline.map((p, i) => (
                    <circle key={`dot-${i}`} cx={xScale(i)} cy={yScale(p)} r="3" className={styles.medianDot} />
                ))}

                {/* X-Axis Labels */}
                {labels.map((label, i) => (
                    <text 
                        key={i} 
                        x={xScale(i)} 
                        y={height - padding + 15} 
                        className={styles.axisLabel} 
                        textAnchor="middle"
                    >
                        {label}
                    </text>
                ))}
            </svg>
            <div className={styles.legend}>
                <span className={styles.legendItem}><span className={styles.medianLineKey}></span> Reality</span>
                <span className={styles.legendItem}><span className={styles.planLineKey}></span> Plan</span>
                <span className={styles.legendItem}><span className={styles.riskAreaKey}></span> Risk (25-75th)</span>
            </div>
        </div>
    );
};

export default DescentGraph;
