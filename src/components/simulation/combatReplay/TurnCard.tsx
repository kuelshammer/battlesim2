import { FC, useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import {
  faChevronDown,
  faFire,
  faShieldAlt,
  faHeart
} from '@fortawesome/free-solid-svg-icons'
import type { TurnCardProps } from './combatReplayTypes'
import { getUnitFaction, getFactionColors, calculateTurnStats } from './combatReplayUtils'
import { ActionCard } from './ActionCard'

/**
 * TurnCard - Collapsible card for a single turn
 */
export const TurnCard: FC<TurnCardProps> = ({
  id,
  roundNumber,
  turnIndex,
  unitId,
  actions,
  firstActionIndex,
  isExpanded,
  isActive,
  onToggleExpand,
  onSeek
}) => {
  const faction = getUnitFaction(unitId)
  const colors = getFactionColors(faction, isActive)
  const stats = calculateTurnStats(unitId, actions)

  return (
    <motion.div
      id={id}
      layout
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className={`relative overflow-hidden rounded-xl border-l-4 ${colors.border} bg-gradient-to-br ${colors.background} backdrop-blur-sm`}
    >
      {/* Pulse animation for active turn */}
      {isActive && (
        <motion.div
          className="absolute inset-0 pointer-events-none"
          animate={{
            boxShadow: [
              '0 0 0 0px rgba(168, 85, 247, 0.4)',
              '0 0 0 8px rgba(168, 85, 247, 0)'
            ]
          }}
          transition={{ duration: 2, repeat: Infinity }}
        />
      )}

      {/* Header - Always visible */}
      <motion.button
        layout
        whileHover={{ scale: isExpanded ? 1 : 1.01, x: isExpanded ? 0 : 2 }}
        whileTap={{ scale: 0.99 }}
        onClick={() => isExpanded ? onToggleExpand() : onSeek(firstActionIndex)}
        className="w-full px-4 py-3 flex items-center justify-between"
      >
        {/* Left: Round/Turn badge + Unit name */}
        <div className="flex items-center gap-3 min-w-0 flex-1">
          {/* Round/Turn badge */}
          <span className={`flex-shrink-0 px-2 py-1 rounded text-[10px] font-mono font-semibold ${
            isActive
              ? 'bg-purple-500 text-white'
              : 'bg-slate-700 text-slate-400'
          }`}>
            R{roundNumber} • T{turnIndex + 1}
          </span>

          {/* Unit name with faction color */}
          <span className={`font-medium truncate ${colors.text}`}>
            {unitId}
          </span>

          {/* Faction badge */}
          <span className={`flex-shrink-0 px-1.5 py-0.5 rounded text-[9px] font-semibold uppercase ${
            faction === 'pc' ? 'bg-cyan-500/20 text-cyan-400' :
            faction === 'enemy' ? 'bg-red-500/20 text-red-400' :
            'bg-slate-500/20 text-slate-400'
          }`}>
            {faction === 'pc' ? 'PC' : faction === 'enemy' ? 'NPC' : '?'}
          </span>
        </div>

        {/* Right: Expand/collapse indicator */}
        <motion.div
          animate={{ rotate: isExpanded ? 180 : 0 }}
          transition={{ duration: 0.2 }}
          className="flex-shrink-0 ml-2"
        >
          <FontAwesomeIcon icon={faChevronDown} className={`text-xs ${isActive ? 'text-purple-400' : 'text-slate-500'}`} />
        </motion.div>
      </motion.button>

      {/* Expanded content */}
      <AnimatePresence>
        {isExpanded && (
          <motion.div
            layout
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ type: 'spring', stiffness: 300, damping: 30 }}
          >
            {/* Action list */}
            <div className="px-3 pb-2 space-y-1">
              {actions.map((action, idx) => (
                <ActionCard
                  key={idx}
                  action={action}
                  actorId={unitId}
                  actionIndex={firstActionIndex + idx}
                  isActive={false}
                  onSeek={onSeek}
                />
              ))}
            </div>

            {/* Footer stats */}
            <div className="px-3 pb-3 pt-2 border-t border-slate-700/30">
              <div className="flex items-center justify-center gap-4 text-[10px]">
                {stats.totalDamageDealt > 0 && (
                  <div className="flex items-center gap-1">
                    <FontAwesomeIcon icon={faFire} className="text-red-400" />
                    <span className="text-slate-300">{stats.totalDamageDealt}</span>
                  </div>
                )}
                {stats.totalDamageTaken > 0 && (
                  <div className="flex items-center gap-1">
                    <FontAwesomeIcon icon={faShieldAlt} className="text-amber-400" />
                    <span className="text-slate-300">{stats.totalDamageTaken}</span>
                  </div>
                )}
                {stats.totalHealingDone > 0 && (
                  <div className="flex items-center gap-1">
                    <FontAwesomeIcon icon={faHeart} className="text-emerald-400" />
                    <span className="text-slate-300">{stats.totalHealingDone}</span>
                  </div>
                )}
                {stats.totalDamageDealt === 0 && stats.totalDamageTaken === 0 && stats.totalHealingDone === 0 && (
                  <span className="text-slate-500 italic">No damage/healing</span>
                )}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  )
}
