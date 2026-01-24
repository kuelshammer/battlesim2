import type { FC } from 'react'
import type { Event } from '@/model/model'
import type { ReplayAction } from '@/model/replayTypes'

// Type guards for specific event types
export type AttackHitEvent = Extract<Event, { type: 'AttackHit' }>
export type AttackMissedEvent = Extract<Event, { type: 'AttackMissed' }>
export type DamageTakenEvent = Extract<Event, { type: 'DamageTaken' }>
export type HealingAppliedEvent = Extract<Event, { type: 'HealingApplied' }>
export type BuffAppliedEvent = Extract<Event, { type: 'BuffApplied' }>
export type ConditionAddedEvent = Extract<Event, { type: 'ConditionAdded' }>

// Type guard functions
export const isAttackHit = (event: Event): event is AttackHitEvent => {
  return event.type === 'AttackHit'
}

export const isAttackMissed = (event: Event): event is AttackMissedEvent => {
  return event.type === 'AttackMissed'
}

export const isDamageTaken = (event: Event): event is DamageTakenEvent => {
  return event.type === 'DamageTaken'
}

export const isHealingApplied = (event: Event): event is HealingAppliedEvent => {
  return event.type === 'HealingApplied'
}

export const isBuffApplied = (event: Event): event is BuffAppliedEvent => {
  return event.type === 'BuffApplied'
}

export const isConditionAdded = (event: Event): event is ConditionAddedEvent => {
  return event.type === 'ConditionAdded'
}

// Component Props Types
export interface CombatReplayModalProps {
  /** The replay data to visualize */
  replay: import('@/model/replayTypes').Replay | null
  /** Whether the modal is open */
  open: boolean
  /** Callback when modal open state changes */
  onOpenChange: (open: boolean) => void
}

export interface ActionCardProps {
  action: ReplayAction
  actorId: string
  actionIndex: number // Flattened index for seeking
  isActive: boolean
  onSeek: (index: number) => void
}

export interface TurnCardProps {
  id?: string // Optional id for scroll-into-view
  roundNumber: number
  turnIndex: number
  unitId: string
  actions: ReplayAction[]
  firstActionIndex: number // Flattened index for seeking to turn start
  isExpanded: boolean
  isActive: boolean
  onToggleExpand: () => void
  onSeek: (index: number) => void
}

export interface SyncLogPanelProps {
  /** Replay data */
  replay: import('@/model/replayTypes').Replay | null
  /** Flattened actions for seeking */
  actions: readonly import('@/hooks/useCombatPlayback').FlattenedAction[]
  /** Current action index */
  currentIndex: number
  /** Seek callback when user clicks an action */
  onSeek: (index: number) => void
  /** Whether playback is active (for auto-scroll) */
  isPlaying: boolean
}

export interface SubEventCardProps {
  event: Event
  index: number
}

export type InteractionType = 'attack' | 'heal' | 'buff' | 'self' | null

export type Faction = 'pc' | 'enemy' | 'neutral'

export interface FactionColors {
  border: string
  background: string
  text: string
  icon: string
}

export interface ActionStats {
  totalDamageDealt: number
  totalDamageTaken: number
  totalHealingDone: number
}
