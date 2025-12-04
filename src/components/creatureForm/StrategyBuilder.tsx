import { FC } from 'react';
import { Action } from '../../model/model';
import styles from './StrategyBuilder.module.scss';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faArrowUp, faArrowDown, faBolt, faHandPaper, faPlus, faClock } from '@fortawesome/free-solid-svg-icons';

type Props = {
    actions: Action[];
    onReorder: (newActions: Action[]) => void;
};

// Helper to safely extract action properties from any Action type
const getActionBase = (action: Action) => {
    // TemplateAction doesn't have a direct 'name' field - it uses templateOptions.templateName
    if (action.type === 'template') {
        return {
            id: action.id,
            name: action.templateOptions.templateName,
            actionSlot: action.actionSlot,
            cost: action.cost,
            requirements: action.requirements,
            tags: action.tags,
        };
    }

    return {
        id: action.id,
        name: action.name,
        actionSlot: action.actionSlot,
        cost: action.cost,
        requirements: action.requirements,
        tags: action.tags,
    };
};

const StrategyBuilder: FC<Props> = ({ actions, onReorder }) => {
    const moveUp = (index: number) => {
        if (index <= 0) return;
        const newActions = [...actions];
        [newActions[index - 1], newActions[index]] = [newActions[index], newActions[index - 1]];
        onReorder(newActions);
    };

    const moveDown = (index: number) => {
        if (index >= actions.length - 1) return;
        const newActions = [...actions];
        [newActions[index + 1], newActions[index]] = [newActions[index], newActions[index + 1]];
        onReorder(newActions);
    };

    const getCostIcon = (action: Action) => {
        const base = getActionBase(action);
        // Check costs
        const costs = base.cost;
        if (costs.some(c => c.resourceType === 'Action')) return <FontAwesomeIcon icon={faBolt} title="Action" className={styles.iconAction} />;
        if (costs.some(c => c.resourceType === 'BonusAction')) return <FontAwesomeIcon icon={faPlus} title="Bonus Action" className={styles.iconBonus} />;
        if (costs.some(c => c.resourceType === 'Reaction')) return <FontAwesomeIcon icon={faHandPaper} title="Reaction" className={styles.iconReaction} />;

        // Fallback to legacy actionSlot
        if (base.actionSlot === 0) return <FontAwesomeIcon icon={faBolt} title="Action" className={styles.iconAction} />;
        if (base.actionSlot === 1) return <FontAwesomeIcon icon={faPlus} title="Bonus Action" className={styles.iconBonus} />;
        if (base.actionSlot === 2) return <FontAwesomeIcon icon={faHandPaper} title="Reaction" className={styles.iconReaction} />;

        return <FontAwesomeIcon icon={faClock} title="Other" className={styles.iconOther} />;
    };

    return (
        <div className={styles.strategyBuilder}>
            <h4>Strategy & Priority</h4>
            <p className={styles.hint}>Actions are evaluated in order. The first valid and affordable action for each slot type will be used.</p>

            <div className={styles.actionList}>
                {actions.map((action, index) => {
                    const base = getActionBase(action);
                    return (
                        <div key={base.id} className={styles.actionItem}>
                            <div className={styles.priority}>{index + 1}</div>
                            <div className={styles.iconContainer}>
                                {getCostIcon(action)}
                            </div>
                            <div className={styles.details}>
                                <span className={styles.name}>{base.name}</span>
                                <span className={styles.summary}>
                                    {base.requirements.length > 0
                                        ? `${base.requirements.length} condition(s)`
                                        : 'Always'}
                                </span>
                            </div>
                            <div className={styles.controls}>
                                <button
                                    onClick={() => moveUp(index)}
                                    disabled={index === 0}
                                    className={styles.moveBtn}
                                >
                                    <FontAwesomeIcon icon={faArrowUp} />
                                </button>
                                <button
                                    onClick={() => moveDown(index)}
                                    disabled={index === actions.length - 1}
                                    className={styles.moveBtn}
                                >
                                    <FontAwesomeIcon icon={faArrowDown} />
                                </button>
                            </div>
                        </div>
                    );
                })}
                {actions.length === 0 && <div className={styles.empty}>No actions defined</div>}
            </div>
        </div>
    );
};

export default StrategyBuilder;
