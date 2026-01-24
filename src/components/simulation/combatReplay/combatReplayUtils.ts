import type { Event } from '@/model/model'
import type { ReplayAction } from '@/model/replayTypes'
import { PlayerTemplates } from '@/data/data'
import type {
  InteractionType,
  Faction,
  FactionColors,
  ActionStats
} from './combatReplayTypes'
import {
  isDamageTaken,
  isHealingApplied,
  isAttackHit,
  isAttackMissed,
  isBuffApplied,
  isConditionAdded
} from './combatReplayTypes'

/**
 * Extract target ID from sub-events
 */
export const extractTargetId = (subEvents: Event[]): string | null => {
  if (!subEvents || subEvents.length === 0) return null

  // Priority order: AttackHit > AttackMissed > DamageTaken > HealingApplied > BuffApplied > ConditionAdded
  const targetEvent = subEvents.find(event => {
    return event.type === 'AttackHit' ||
           event.type === 'AttackMissed' ||
           event.type === 'DamageTaken' ||
           event.type === 'HealingApplied' ||
           event.type === 'BuffApplied' ||
           event.type === 'ConditionAdded'
  })

  if (targetEvent) {
    if ('target_id' in targetEvent) return targetEvent.target_id as string
  }

  return null
}

/**
 * Extract interaction type between actor and target
 */
export const getInteractionType = (subEvents: Event[]): InteractionType => {
  if (!subEvents || subEvents.length === 0) return null

  const types = subEvents.map(e => e.type)

  if (types.includes('AttackHit') || types.includes('AttackMissed')) return 'attack'
  if (types.includes('HealingApplied')) return 'heal'
  if (types.includes('BuffApplied') || types.includes('ConditionAdded')) return 'buff'

  return null
}

/**
 * Calculate damage dealt, taken, and healing for an action
 */
export const calculateActionStats = (_actorId: string, subEvents: Event[]): ActionStats => {
  let totalDamageDealt = 0
  let totalDamageTaken = 0
  let totalHealingDone = 0

  for (const event of subEvents) {
    if (isDamageTaken(event)) {
      if (event.target_id === _actorId) {
        totalDamageTaken += event.damage || 0
      } else {
        totalDamageDealt += event.damage || 0
      }
    }
    if (isHealingApplied(event)) {
      totalHealingDone += event.amount || 0
    }
  }

  return { totalDamageDealt, totalDamageTaken, totalHealingDone }
}

/**
 * Calculate stats for all actions in a turn
 */
export const calculateTurnStats = (actorId: string, actions: ReplayAction[]): ActionStats => {
  let totalDamageDealt = 0
  let totalDamageTaken = 0
  let totalHealingDone = 0

  for (const action of actions) {
    const stats = calculateActionStats(actorId, action.subEvents)
    totalDamageDealt += stats.totalDamageDealt
    totalDamageTaken += stats.totalDamageTaken
    totalHealingDone += stats.totalHealingDone
  }

  return { totalDamageDealt, totalDamageTaken, totalHealingDone }
}

/**
 * Determine faction (PC vs Enemy) based on unitId
 * Checks if unitId matches any PlayerTemplate key
 */
export const getUnitFaction = (unitId: string): Faction => {
  // Check if unitId matches any PlayerTemplate name
  const templateNames = Object.keys(PlayerTemplates)
  const isPc = templateNames.some(name =>
    name.toLowerCase() === unitId.toLowerCase() ||
    unitId.toLowerCase().includes(name.toLowerCase())
  )

  if (isPc) return 'pc'

  // Common enemy patterns
  const enemyPatterns = ['goblin', 'orc', 'dragon', 'zombie', 'skeleton', 'bandit', 'wolf', 'bear']
  const isEnemy = enemyPatterns.some(pattern => unitId.toLowerCase().includes(pattern))

  return isEnemy ? 'enemy' : 'neutral'
}

/**
 * Get color classes for faction
 */
export const getFactionColors = (faction: Faction, isActive: boolean): FactionColors => {
  if (isActive) {
    return {
      border: 'border-purple-500/50',
      background: 'from-purple-900/30 to-purple-800/10',
      text: 'text-purple-300',
      icon: 'text-purple-400'
    }
  }

  switch (faction) {
    case 'pc':
      return {
        border: 'border-cyan-500/30',
        background: 'from-cyan-900/20 to-cyan-800/10',
        text: 'text-cyan-300',
        icon: 'text-cyan-400'
      }
    case 'enemy':
      return {
        border: 'border-red-500/30',
        background: 'from-red-900/20 to-red-800/10',
        text: 'text-red-300',
        icon: 'text-red-400'
      }
    default:
      return {
        border: 'border-slate-600/30',
        background: 'from-slate-800/20 to-slate-700/10',
        text: 'text-slate-300',
        icon: 'text-slate-400'
      }
  }
}

/**
 * Get compact action result text
 */
export const getActionResultText = (subEvents: Event[]): string => {
  const types = subEvents.map(e => e.type)

  if (types.includes('AttackHit')) {
    const hit = subEvents.find(isAttackHit)
    const target = hit?.target_id || 'unknown'
    const damage = subEvents.find(isDamageTaken)
    const amount = damage?.damage || 0
    return `→ ${target} Hit ${amount}dmg`
  }

  if (types.includes('AttackMissed')) {
    const miss = subEvents.find(isAttackMissed)
    const target = miss?.target_id || 'unknown'
    return `→ ${target} Missed`
  }

  if (types.includes('HealingApplied')) {
    const heal = subEvents.find(isHealingApplied)
    const amount = heal?.amount || 0
    return `Heal ${amount}hp`
  }

  if (types.includes('BuffApplied')) {
    const buff = subEvents.find(isBuffApplied)
    return `Applied ${buff?.buff_id || 'buff'}`
  }

  if (types.includes('ConditionAdded')) {
    const cond = subEvents.find(isConditionAdded)
    return `Added ${cond?.condition || 'condition'}`
  }

  return ''
}
