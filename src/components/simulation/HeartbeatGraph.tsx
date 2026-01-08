import React, { FC, useMemo } from 'react';
import { AggregateOutput } from '@/model/model';
import styles from './heartbeatGraph.module.scss';

interface HeartbeatGraphProps {
    encounters: AggregateOutput[];
    className?: string;
}

const HeartbeatGraph: FC<HeartbeatGraphProps> = ({ encounters, className }) => {
    const width = 600;
    const height = 120;
    const padding = { top: 20, right: 40, bottom: 20, left: 40 };

    const points = useMemo(() => {
        if (!encounters || encounters.length === 0) return [];

        const chartWidth = width - padding.left - padding.right;
        const chartHeight = height - padding.top - padding.bottom;

        // X steps
        const stepX = chartWidth / Math.max(1, encounters.length);

        return encounters.map((enc, i) => {
            const lethality = enc.vitals?.lethalityIndex || 0;
            // Map lethality 0..1 to height 0..chartHeight (inverted for SVG)
            const y = padding.top + chartHeight - (lethality * chartHeight);
            const x = padding.left + (i + 0.5) * stepX;
            return { x, y, lethality, label: `E${i + 1}` };
        });
    }, [encounters]);

    const pathData = useMemo(() => {
        if (points.length === 0) return "";
        
        // Start at left baseline
        let d = `M ${padding.left} ${height - padding.bottom}`;
        
        points.forEach((p, i) => {
            // Straight lines for "heartbeat" feel or curves?
            // Let's use simple lines for now
            d += ` L ${p.x} ${p.y}`;
        });

        // End at right baseline
        d += ` L ${width - padding.right} ${height - padding.bottom}`;
        
        return d;
    }, [points]);

    return (
        <div className={`${styles.heartbeatGraph} ${className || ''}`}>
            <svg viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="xMidYMid meet">
                {/* Background Zones */}
                <rect x={padding.left} y={padding.top} width={width - padding.left - padding.right} height={(height - padding.top - padding.bottom) * 0.5} fill="rgba(239, 68, 68, 0.05)" /> {/* Red Zone */}
                <rect x={padding.left} y={padding.top + (height - padding.top - padding.bottom) * 0.5} width={width - padding.left - padding.right} height={(height - padding.top - padding.bottom) * 0.3} fill="rgba(245, 158, 11, 0.05)" /> {/* Yellow Zone */}
                <rect x={padding.left} y={padding.top + (height - padding.top - padding.bottom) * 0.8} width={width - padding.left - padding.right} height={(height - padding.top - padding.bottom) * 0.2} fill="rgba(34, 197, 94, 0.05)" /> {/* Green Zone */}

                {/* Grid Lines */}
                <line x1={padding.left} y1={height - padding.bottom} x2={width - padding.right} y2={height - padding.bottom} stroke="rgba(255,255,255,0.1)" strokeWidth="1" />
                
                {/* Tension Line */}
                <path d={pathData} fill="none" stroke="#d4af37" strokeWidth="2" strokeLinejoin="round" className={styles.tensionPath} />

                {/* Nodes */}
                {points.map((p, i) => (
                    <g key={i} className={styles.node}>
                        <circle cx={p.x} cy={p.y} r="4" fill={p.lethality > 0.5 ? '#ef4444' : p.lethality > 0.2 ? '#f59e0b' : '#22c55e'} />
                        <text x={p.x} y={height - 5} textAnchor="middle" className={styles.axisLabel}>{p.label}</text>
                    </g>
                ))}

                {/* Y-Axis Labels */}
                <text x={padding.left - 5} y={padding.top + 5} textAnchor="end" className={styles.axisLabel}>Lethal</text>
                <text x={padding.left - 5} y={height - padding.bottom} textAnchor="end" className={styles.axisLabel}>Safe</text>
            </svg>
            <div className={styles.title}>Projected Tension Arc</div>
        </div>
    );
};

export default HeartbeatGraph;
