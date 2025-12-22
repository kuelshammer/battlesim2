import React, { FC } from 'react';
import { Creature, AutoAdjustmentResult } from '@/model/model';
import Modal from '@/utils/modal';
import styles from './adjustmentPreview.module.scss';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faCheck, faTimes, faArrowRight } from '@fortawesome/free-solid-svg-icons';

type PropType = {
    originalMonsters: Creature[],
    adjustmentResult: AutoAdjustmentResult,
    onApply: () => void,
    onCancel: () => void,
};

const AdjustmentPreview: FC<PropType> = ({ originalMonsters, adjustmentResult, onApply, onCancel }) => {
    const { monsters: optimizedMonsters } = adjustmentResult;

    return (
        <Modal onCancel={onCancel} className={styles.previewModal}>
            <div className={styles.header}>
                <h2>Adjustment Preview</h2>
                <p>The encounter has been optimized for balance.</p>
            </div>

            <div className={styles.body}>
                <div className={styles.analysisSummary}>
                    <div className={styles.summaryItem}>
                        <span className={styles.label}>Projected Win Rate</span>
                        <span className={styles.value}>
                            {Math.round((adjustmentResult.analysis.globalMedian?.winRate ?? 0) * 100)}%
                        </span>
                    </div>
                    <div className={styles.summaryItem}>
                        <span className={styles.label}>Median Duration</span>
                        <span className={styles.value}>
                            {adjustmentResult.analysis.battleDurationRounds} Rounds
                        </span>
                    </div>
                </div>

                <div className={styles.monsterDiffs}>
                    {originalMonsters.map((original, idx) => {
                        const optimized = optimizedMonsters.find(m => m.name === original.name) || optimizedMonsters[idx];
                        if (!optimized) return null;

                        return (
                            <div key={original.id} className={styles.monsterCard}>
                                <h3>{original.name}</h3>
                                <div className={styles.statsGrid}>
                                    <div className={styles.statLabel}>HP</div>
                                    <div className={styles.statValue}>
                                        <span className={styles.oldValue}>{original.hp}</span>
                                        <FontAwesomeIcon icon={faArrowRight} className={styles.arrow} />
                                        <span className={original.hp < optimized.hp ? styles.buff : original.hp > optimized.hp ? styles.nerf : ''}>
                                            {Math.round(optimized.hp)}
                                        </span>
                                    </div>

                                    <div className={styles.statLabel}>AC</div>
                                    <div className={styles.statValue}>
                                        <span className={styles.oldValue}>{original.ac}</span>
                                        <FontAwesomeIcon icon={faArrowRight} className={styles.arrow} />
                                        <span className={original.ac < optimized.ac ? styles.buff : original.ac > optimized.ac ? styles.nerf : ''}>
                                            {optimized.ac}
                                        </span>
                                    </div>
                                    
                                    <div className={styles.statLabel}>Save Bonus</div>
                                    <div className={styles.statValue}>
                                        <span className={styles.oldValue}>{original.saveBonus}</span>
                                        <FontAwesomeIcon icon={faArrowRight} className={styles.arrow} />
                                        <span className={original.saveBonus < optimized.saveBonus ? styles.buff : original.saveBonus > optimized.saveBonus ? styles.nerf : ''}>
                                            {optimized.saveBonus}
                                        </span>
                                    </div>
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>

            <div className={styles.footer}>
                <button onClick={onCancel} className={styles.cancelBtn}>
                    <FontAwesomeIcon icon={faTimes} /> Cancel
                </button>
                <button onClick={onApply} className={styles.applyBtn}>
                    <FontAwesomeIcon icon={faCheck} /> Apply Changes
                </button>
            </div>
        </Modal>
    );
};

export default AdjustmentPreview;
