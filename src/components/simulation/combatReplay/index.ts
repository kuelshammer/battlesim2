// Type exports
export type {
  CombatReplayModalProps,
  ActionCardProps,
  TurnCardProps,
  SyncLogPanelProps,
  SubEventCardProps,
  InteractionType,
  Faction,
  FactionColors,
  ActionStats
} from './combatReplayTypes'

export type {
  AttackHitEvent,
  AttackMissedEvent,
  DamageTakenEvent,
  HealingAppliedEvent,
  BuffAppliedEvent,
  ConditionAddedEvent
} from './combatReplayTypes'

// Utility exports
export {
  extractTargetId,
  getInteractionType,
  calculateActionStats,
  calculateTurnStats,
  getUnitFaction,
  getFactionColors,
  getActionResultText
} from './combatReplayUtils'

export {
  isAttackHit,
  isAttackMissed,
  isDamageTaken,
  isHealingApplied,
  isBuffApplied,
  isConditionAdded
} from './combatReplayTypes'

// Component exports
export { SubEventCard } from './SubEventCard'
export { ActionCard } from './ActionCard'
export { TurnCard } from './TurnCard'
export { SyncLogPanel } from './SyncLogPanel'
