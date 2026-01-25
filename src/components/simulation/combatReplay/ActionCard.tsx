import { FC } from 'react'
import { motion } from 'framer-motion'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import {
  faBolt,
  faMagic,
  faShieldAlt,
  faClock,
  faHeart,
  faHand,
  faGavel
} from '@fortawesome/free-solid-svg-icons'
import type { ActionCardProps } from './combatReplayTypes'
import { getActionResultText } from './combatReplayUtils'

/**
 * Get action icon based on action type or sub-events
 */
const getActionIcon = (actionId: string, subEvents: import('@/model/model').Event[]) => {
  // Check action ID first
  if (actionId.toLowerCase().includes('attack')) return faBolt
  if (actionId.toLowerCase().includes('spell') || actionId.toLowerCase().includes('cast')) return faMagic
  if (actionId.toLowerCase().includes('dodge')) return faShieldAlt
  if (actionId.toLowerCase().includes('dash')) return faClock
  if (actionId.toLowerCase().includes('heal')) return faHeart
  if (actionId.toLowerCase().includes('help')) return faHand

  // Check sub-events
  const types = subEvents.map(e => e.type)
  if (types.includes('AttackHit') || types.includes('AttackMissed')) return faBolt
  if (types.includes('HealingApplied')) return faHeart
  if (types.includes('BuffApplied') || types.includes('ConditionAdded')) return faShieldAlt
  if (types.includes('SpellCast')) return faMagic

  return faGavel // Default action icon
}

/**
 * ActionCard - Individual action within a turn card
 */
export const ActionCard: FC<ActionCardProps> = ({
  action,
  actorId,
  actionIndex,
  isActive,
  onSeek
}) => {
  const icon = getActionIcon(action.actionId, action.subEvents)
  const resultText = getActionResultText(action.subEvents)

  return (
    <motion.button
      layout
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      whileHover={{ scale: 1.01, x: 2 }}
      whileTap={{ scale: 0.99 }}
      onClick={() => onSeek(actionIndex)}
      className={`w-full text-left px-3 py-2 rounded-lg border transition-all text-xs ${
        isActive
          ? 'bg-purple-600/20 border-purple-500/40'
          : 'bg-slate-800/30 border-slate-700/30 hover:bg-slate-700/40'
      }`}
    >
      <div className="flex items-center gap-2">
        <FontAwesomeIcon icon={icon} className={`text-xs ${isActive ? 'text-purple-400' : 'text-slate-500'}`} />
        <span className={`font-medium truncate ${isActive ? 'text-white' : 'text-slate-300'}`}>
          {action.actionId}
        </span>
        {resultText && (
          <span className={`text-[10px] font-mono truncate ml-auto ${isActive ? 'text-purple-300' : 'text-slate-500'}`}>
            {resultText}
          </span>
        )}
      </div>
    </motion.button>
  )
}
