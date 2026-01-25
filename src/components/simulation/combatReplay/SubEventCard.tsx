import { FC } from 'react'
import { motion } from 'framer-motion'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import {
  faBolt,
  faSkull,
  faHeart,
  faMagic,
  faShieldAlt,
  faClock
} from '@fortawesome/free-solid-svg-icons'
import type { Event } from '@/model/model'
import type { SubEventCardProps } from './combatReplayTypes'

/**
 * Helper component to render individual sub-events with appropriate icons and colors
 */
export const SubEventCard: FC<SubEventCardProps> = ({ event, index }) => {
  const getEventColor = () => {
    switch (event.type) {
      case 'AttackHit': return 'from-red-600/20 to-orange-600/20 border-red-500/30'
      case 'AttackMissed': return 'from-slate-600/20 to-slate-600/20 border-slate-500/30'
      case 'DamageTaken': return 'from-red-700/30 to-rose-700/30 border-red-400/40'
      case 'HealingApplied': return 'from-emerald-600/20 to-green-600/20 border-emerald-500/30'
      case 'TempHPGranted': return 'from-cyan-600/20 to-blue-600/20 border-cyan-500/30'
      case 'BuffApplied': return 'from-amber-600/20 to-yellow-600/20 border-amber-500/30'
      case 'ConditionAdded': return 'from-purple-600/20 to-violet-600/20 border-purple-500/30'
      case 'UnitDied': return 'from-gray-700/30 to-slate-700/30 border-gray-500/40'
      default: return 'from-slate-700/20 to-slate-800/20 border-slate-600/30'
    }
  }

  const getEventIcon = () => {
    switch (event.type) {
      case 'AttackHit':
      case 'AttackMissed':
        return faBolt
      case 'DamageTaken':
        return faSkull
      case 'HealingApplied':
      case 'TempHPGranted':
        return faHeart
      case 'BuffApplied':
        return faMagic
      case 'ConditionAdded':
        return faShieldAlt
      case 'UnitDied':
        return faSkull
      default:
        return faClock
    }
  }

  const getEventLabel = () => {
    switch (event.type) {
      case 'AttackHit':
        return `${event.attacker_id} hit ${event.target_id}`
      case 'AttackMissed':
        return `${event.attacker_id} missed ${event.target_id}`
      case 'DamageTaken':
        return `${event.target_id} took ${event.damage} ${event.damage_type || 'damage'}`
      case 'HealingApplied':
        return `${event.target_id} healed for ${event.amount}`
      case 'TempHPGranted':
        return `${event.target_id} gained ${event.amount} temp HP`
      case 'BuffApplied':
        return `${event.buff_id} applied to ${event.target_id}`
      case 'ConditionAdded':
        return `${event.condition} added to ${event.target_id}`
      case 'UnitDied':
        return `${event.unit_id} was slain`
      default:
        return event.type
    }
  }

  return (
    <motion.div
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ delay: index * 0.05 }}
      className={`relative overflow-hidden rounded-lg border bg-gradient-to-r ${getEventColor()} backdrop-blur-sm`}
    >
      <div className="flex items-start gap-3 p-3">
        <div className="mt-0.5 flex-shrink-0 w-8 h-8 rounded-full bg-slate-900/50 flex items-center justify-center">
          <FontAwesomeIcon
            icon={getEventIcon()}
            className="text-xs"
            style={{ color: event.type === 'AttackHit' || event.type === 'DamageTaken' ? '#f87171' : '#94a3b8' }}
          />
        </div>
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium text-slate-200 truncate">{getEventLabel()}</p>
          {event.type === 'AttackHit' && event.damage_roll && (
            <p className="text-xs text-slate-400 font-mono mt-0.5">
              {event.damage_roll.formula} = {event.damage_roll.total}
            </p>
          )}
          {event.type === 'AttackHit' && event.attack_roll && (
            <p className="text-xs text-slate-400 font-mono">
              vs AC {event.target_ac}
            </p>
          )}
        </div>
        <span className="text-[10px] text-slate-500 font-mono self-start">#{index + 1}</span>
      </div>
    </motion.div>
  )
}
