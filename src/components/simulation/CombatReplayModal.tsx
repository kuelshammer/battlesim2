import { FC, useState, useRef, useEffect, useMemo } from 'react'
import * as Dialog from '@radix-ui/react-dialog'
import { motion, AnimatePresence } from 'framer-motion'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import {
  faPlay,
  faPause,
  faStepForward,
  faStepBackward,
  faTimes,
  faClock,
  faHistory,
  faShieldAlt,
  faSkull,
  faHeart,
  faMagic,
  faBolt,
  faUser,
  faCrosshairs,
  faGavel,
  faExchangeAlt,
  faUserFriends,
  faList,
  faHand,
  faChevronDown,
  faFire
} from '@fortawesome/free-solid-svg-icons'
import { useCombatPlayback, type FlattenedAction } from '@/hooks/useCombatPlayback'
import type { Replay, ReplayAction } from '@/model/replayTypes'
import type { Event } from '@/model/model'
import { PlayerTemplates } from '@/data/data'

interface CombatReplayModalProps {
  /** The replay data to visualize */
  replay: Replay | null
  /** Whether the modal is open */
  open: boolean
  /** Callback when modal open state changes */
  onOpenChange: (open: boolean) => void
}

/**
 * Extract target ID from sub-events
 */
const extractTargetId = (subEvents: Event[]): string | null => {
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
const getInteractionType = (subEvents: Event[]): 'attack' | 'heal' | 'buff' | 'self' | null => {
  if (!subEvents || subEvents.length === 0) return null

  const types = subEvents.map(e => e.type)

  if (types.includes('AttackHit') || types.includes('AttackMissed')) return 'attack'
  if (types.includes('HealingApplied')) return 'heal'
  if (types.includes('BuffApplied') || types.includes('ConditionAdded')) return 'buff'

  return null
}

/**
 * Helper component to render individual sub-events with appropriate icons and colors
 */
const SubEventCard: FC<{ event: Event; index: number }> = ({ event, index }) => {
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

/**
 * Get action icon based on action type or sub-events
 */
const getActionIcon = (actionId: string, subEvents: Event[]) => {
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
 * Calculate damage dealt, taken, and healing for an action
 */
const calculateActionStats = (_actorId: string, subEvents: Event[]) => {
  let damageDealt = 0
  let damageTaken = 0
  let healingDone = 0

  for (const event of subEvents) {
    if (event.type === 'DamageTaken') {
      const evt = event as any
      if (evt.target_id === _actorId) {
        damageTaken += evt.damage || 0
      } else {
        damageDealt += evt.damage || 0
      }
    }
    if (event.type === 'HealingApplied') {
      const evt = event as any
      if (evt.target_id === _actorId) {
        healingDone += evt.amount || 0
      } else {
        healingDone += evt.amount || 0
      }
    }
  }

  return { damageDealt, damageTaken, healingDone }
}

/**
 * Calculate stats for all actions in a turn
 */
const calculateTurnStats = (actorId: string, actions: ReplayAction[]) => {
  let totalDamageDealt = 0
  let totalDamageTaken = 0
  let totalHealingDone = 0

  for (const action of actions) {
    const stats = calculateActionStats(actorId, action.subEvents)
    totalDamageDealt += stats.damageDealt
    totalDamageTaken += stats.damageTaken
    totalHealingDone += stats.healingDone
  }

  return { totalDamageDealt, totalDamageTaken, totalHealingDone }
}

/**
 * Determine faction (PC vs Enemy) based on unitId
 * Checks if unitId matches any PlayerTemplate key
 */
const getUnitFaction = (unitId: string): 'pc' | 'enemy' | 'neutral' => {
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
const getFactionColors = (faction: 'pc' | 'enemy' | 'neutral', isActive: boolean) => {
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
const getActionResultText = (subEvents: Event[]): string => {
  const types = subEvents.map(e => e.type)

  if (types.includes('AttackHit')) {
    const hit = subEvents.find(e => e.type === 'AttackHit') as any
    const target = hit?.target_id || 'unknown'
    const damage = subEvents.find(e => e.type === 'DamageTaken') as any
    const amount = damage?.damage || 0
    return `→ ${target} Hit ${amount}dmg`
  }

  if (types.includes('AttackMissed')) {
    const miss = subEvents.find(e => e.type === 'AttackMissed') as any
    const target = miss?.target_id || 'unknown'
    return `→ ${target} Missed`
  }

  if (types.includes('HealingApplied')) {
    const heal = subEvents.find(e => e.type === 'HealingApplied') as any
    const amount = heal?.amount || 0
    return `Heal ${amount}hp`
  }

  if (types.includes('BuffApplied')) {
    const buff = subEvents.find(e => e.type === 'BuffApplied') as any
    return `Applied ${buff?.buff_id || 'buff'}`
  }

  if (types.includes('ConditionAdded')) {
    const cond = subEvents.find(e => e.type === 'ConditionAdded') as any
    return `Added ${cond?.condition || 'condition'}`
  }

  return ''
}

/**
 * ActionCard - Individual action within a turn card
 */
interface ActionCardProps {
  action: ReplayAction
  actorId: string
  actionIndex: number // Flattened index for seeking
  isActive: boolean
  onSeek: (index: number) => void
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const ActionCard: FC<ActionCardProps> = ({ action, actorId, actionIndex, isActive, onSeek }) => {
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

/**
 * TurnCard - Collapsible card for a single turn
 */
interface TurnCardProps {
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

const TurnCard: FC<TurnCardProps> = ({
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

/**
 * SyncLogPanel - Card-based turn-by-turn combat log
 *
 * Displays turns as collapsible cards with:
 * - Round/Turn information
 * - Unit identification with faction coloring
 * - Action list (when expanded)
 * - Damage/healing summary
 * - Click-to-seek functionality
 * - Auto-expand for active turn during playback
 */
interface SyncLogPanelProps {
  /** Replay data */
  replay: Replay | null
  /** Flattened actions for seeking */
  actions: readonly FlattenedAction[]
  /** Current action index */
  currentIndex: number
  /** Seek callback when user clicks an action */
  onSeek: (index: number) => void
  /** Whether playback is active (for auto-scroll) */
  isPlaying: boolean
}

const SyncLogPanel: FC<SyncLogPanelProps> = ({
  replay,
  actions,
  currentIndex,
  onSeek,
  isPlaying
}) => {
  const listRef = useRef<HTMLDivElement>(null)

  // Expansion state: Set of turn keys (roundNumber-turnIndex) that are expanded
  const [expandedTurns, setExpandedTurns] = useState<Set<string>>(new Set())

  // Auto-expand current turn during playback
  useEffect(() => {
    if (actions.length > 0 && currentIndex >= 0 && currentIndex < actions.length) {
      const currentAction = actions[currentIndex]
      const turnKey = `${currentAction.roundNumber}-${currentAction.turnIndex}`
      setExpandedTurns(prev => new Set([...prev, turnKey]))
    }
  }, [currentIndex, actions])

  // Auto-scroll to current turn during playback
  useEffect(() => {
    if (isPlaying && actions.length > 0 && currentIndex >= 0) {
      const currentAction = actions[currentIndex]
      const turnKey = `turn-${currentAction.roundNumber}-${currentAction.turnIndex}`
      const element = document.getElementById(turnKey)
      if (element) {
        element.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
      }
    }
  }, [currentIndex, isPlaying, actions])

  const toggleTurn = (turnKey: string) => {
    setExpandedTurns(prev => {
      const next = new Set(prev)
      if (next.has(turnKey)) {
        next.delete(turnKey)
      } else {
        next.add(turnKey)
      }
      return next
    })
  }

  // Build a lookup map: actionIndex → FlattenedAction
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const actionMap = useMemo(() => {
    const map = new Map<number, FlattenedAction>()
    actions.forEach(a => map.set(a.index, a))
    return map
  }, [actions])

  // Get current action info for highlighting
  const currentAction = actions.length > 0 && currentIndex >= 0 && currentIndex < actions.length
    ? actions[currentIndex]
    : null

  if (!replay) {
    return (
      <div className="h-full flex flex-col bg-slate-900/40 border-r border-slate-800/50">
        <div className="flex items-center gap-2 px-4 py-3 border-b border-slate-800/50 bg-slate-900/60">
          <FontAwesomeIcon icon={faList} className="text-purple-400 text-xs" />
          <h3 className="text-sm font-medium text-slate-300">Sync Log</h3>
        </div>
        <div className="flex-1 flex items-center justify-center">
          <p className="text-slate-500 text-sm">No replay data</p>
        </div>
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col bg-slate-900/40 border-r border-slate-800/50">
      {/* Header */}
      <div className="flex items-center gap-2 px-4 py-3 border-b border-slate-800/50 bg-slate-900/60">
        <FontAwesomeIcon icon={faList} className="text-purple-400 text-xs" />
        <h3 className="text-sm font-medium text-slate-300">Combat Log</h3>
        <span className="text-xs text-slate-500">({replay.rounds.length} rounds)</span>
      </div>

      {/* Scrollable Turn Cards */}
      <div ref={listRef} className="flex-1 overflow-auto overflow-x-hidden">
        <div className="p-3 space-y-2">
          {replay.rounds.map((round) => (
            <div key={round.roundNumber}>
              {round.turns.map((turn, turnIdx) => {
                const turnKey = `${round.roundNumber}-${turnIdx}`
                const isExpanded = expandedTurns.has(turnKey)

                // Check if this turn contains the current action
                const isActive = currentAction &&
                  currentAction.roundNumber === round.roundNumber &&
                  currentAction.turnIndex === turnIdx

                // Find the first action index for this turn
                const firstAction = actions.find(a =>
                  a.roundNumber === round.roundNumber && a.turnIndex === turnIdx
                )
                const firstActionIndex = firstAction?.index ?? 0

                return (
                  <TurnCard
                    key={turnKey}
                    id={`turn-${round.roundNumber}-${turnIdx}`}
                    roundNumber={round.roundNumber}
                    turnIndex={turnIdx}
                    unitId={turn.unitId}
                    actions={turn.actions}
                    firstActionIndex={firstActionIndex}
                    isExpanded={isExpanded}
                    isActive={isActive ?? false}
                    onToggleExpand={() => toggleTurn(turnKey)}
                    onSeek={onSeek}
                  />
                )
              })}
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

/**
 * Combat Replay Modal - "Chronomancer's Table"
 *
 * A modal component for visualizing and navigating through combat replays.
 * Features a focus stage in the center and timeline scrubber in the header.
 */
export const CombatReplayModal: FC<CombatReplayModalProps> = ({
  replay,
  open,
  onOpenChange
}) => {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
const [showGlobalEvents, setShowGlobalEvents] = useState(false)

  const playback = useCombatPlayback(replay, {
    autoAdvanceInterval: 800
  })

  const {
    currentAction,
    currentRoundIndex,
    isPlaying,
    totalActions,
    totalRounds,
    progress,
    nextAction,
    prevAction,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  play,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  pause,
    togglePlay,
    seekToStart,
    seekToEnd,
    seekToRound
  } = playback

  // Format action info for display
  const actionInfo = currentAction
    ? `R${currentAction.roundNumber} • ${currentAction.unitId}`
    : 'No action selected'

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <AnimatePresence>
        {open && (
          <>
            {/* Backdrop */}
            <Dialog.Overlay asChild>
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 bg-black/90 backdrop-blur-md z-50"
              />
            </Dialog.Overlay>

            {/* Modal Content */}
            <Dialog.Content asChild>
              <motion.div
                initial={{ opacity: 0, scale: 0.95, y: 20 }}
                animate={{ opacity: 1, scale: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.95, y: 20 }}
                transition={{ type: 'spring', damping: 25, stiffness: 300 }}
                className="fixed inset-4 md:inset-8 lg:inset-12 flex flex-col bg-gradient-to-br from-slate-950 via-slate-900 to-zinc-950 border border-slate-700/50 rounded-2xl shadow-2xl z-50 overflow-hidden"
              >
                {/* Header - Timeline Scrubber */}
                <div className="flex flex-col border-b border-slate-800/50 bg-slate-900/60 backdrop-blur-xl">
                  {/* Title Bar */}
                  <div className="flex items-center justify-between px-6 py-4">
                    <div className="flex items-center gap-3">
                      <motion.div
                        animate={{ rotate: isPlaying ? 360 : 0 }}
                        transition={{ duration: 2, repeat: Infinity, ease: 'linear' }}
                        className="w-12 h-12 rounded-xl bg-gradient-to-br from-purple-600/30 via-purple-500/20 to-blue-600/30 border border-purple-500/40 flex items-center justify-center shadow-lg shadow-purple-500/10"
                      >
                        <FontAwesomeIcon icon={faClock} className="text-purple-400 text-base" />
                      </motion.div>
                      <div>
                        <Dialog.Title className="text-lg font-semibold text-white tracking-wide">
                          Chronomancer's Table
                        </Dialog.Title>
                        <Dialog.Description className="sr-only">
                          Navigate through combat replay timeline with playback controls, view detailed action events and sub-events
                        </Dialog.Description>
                        <p className="text-xs text-slate-400">
                          {totalActions} actions across {totalRounds} rounds
                        </p>
                      </div>
                    </div>

                    <Dialog.Close asChild>
                      <motion.button
                        whileHover={{ scale: 1.1, rotate: 90 }}
                        whileTap={{ scale: 0.9 }}
                        className="w-10 h-10 rounded-xl bg-slate-800/50 hover:bg-red-600/20 border border-slate-700/50 hover:border-red-500/30 text-slate-400 hover:text-red-400 transition-all flex items-center justify-center"
                      >
                        <FontAwesomeIcon icon={faTimes} />
                      </motion.button>
                    </Dialog.Close>
                  </div>

                  {/* Playback Controls */}
                  <div className="px-6 pb-4">
                    {/* Timeline Progress */}
                    <div className="mb-4">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-xs text-slate-400 font-mono">
                          {currentAction?.index ?? 0} / {totalActions}
                        </span>
                        <span className="text-xs text-purple-400 font-mono">
                          {actionInfo}
                        </span>
                      </div>
                      <div
                        className="h-2.5 bg-slate-800/80 rounded-full overflow-hidden cursor-pointer group relative"
                        onClick={(e) => {
                          const rect = e.currentTarget.getBoundingClientRect()
                          const percent = (e.clientX - rect.left) / rect.width
                          playback.seek(Math.floor(percent * (totalActions - 1)))
                        }}
                      >
                        <motion.div
                          className="h-full bg-gradient-to-r from-purple-600 via-purple-500 to-blue-500 rounded-full relative"
                          initial={{ width: 0 }}
                          animate={{ width: `${progress * 100}%` }}
                          transition={{ duration: 0.15 }}
                        >
                          <motion.div
                            className="absolute right-0 top-1/2 -translate-y-1/2 w-3 h-3 bg-white rounded-full shadow-lg"
                            initial={{ scale: 0 }}
                            animate={{ scale: progress > 0 ? 1 : 0 }}
                            transition={{ type: 'spring', stiffness: 500 }}
                          />
                        </motion.div>
                      </div>
                    </div>

                    {/* Control Buttons */}
                    <div className="flex items-center justify-center gap-2">
                      <motion.button
                        whileHover={{ scale: 1.1 }}
                        whileTap={{ scale: 0.95 }}
                        onClick={seekToStart}
                        className="px-3 py-2 rounded-lg bg-slate-800/50 hover:bg-slate-700/50 border border-slate-700/50 text-slate-400 hover:text-white transition-all text-xs"
                        title="Go to start"
                      >
                        <FontAwesomeIcon icon={faHistory} className="fa-flip-horizontal" />
                      </motion.button>

                      <motion.button
                        whileHover={{ scale: 1.1 }}
                        whileTap={{ scale: 0.95 }}
                        onClick={prevAction}
                        className="px-4 py-2 rounded-lg bg-slate-800/50 hover:bg-slate-700/50 border border-slate-700/50 text-slate-400 hover:text-white transition-all"
                        title="Previous action"
                      >
                        <FontAwesomeIcon icon={faStepBackward} />
                      </motion.button>

                      <motion.button
                        whileHover={{ scale: 1.05 }}
                        whileTap={{ scale: 0.95 }}
                        onClick={togglePlay}
                        className={`px-8 py-3 rounded-xl border transition-all ${
                          isPlaying
                            ? 'bg-amber-600/20 border-amber-500/50 text-amber-400 hover:bg-amber-600/30'
                            : 'bg-emerald-600/20 border-emerald-500/50 text-emerald-400 hover:bg-emerald-600/30'
                        }`}
                        title={isPlaying ? 'Pause' : 'Play'}
                      >
                        <FontAwesomeIcon icon={isPlaying ? faPause : faPlay} />
                      </motion.button>

                      <motion.button
                        whileHover={{ scale: 1.1 }}
                        whileTap={{ scale: 0.95 }}
                        onClick={nextAction}
                        className="px-4 py-2 rounded-lg bg-slate-800/50 hover:bg-slate-700/50 border border-slate-700/50 text-slate-400 hover:text-white transition-all"
                        title="Next action"
                      >
                        <FontAwesomeIcon icon={faStepForward} />
                      </motion.button>

                      <motion.button
                        whileHover={{ scale: 1.1 }}
                        whileTap={{ scale: 0.95 }}
                        onClick={seekToEnd}
                        className="px-3 py-2 rounded-lg bg-slate-800/50 hover:bg-slate-700/50 border border-slate-700/50 text-slate-400 hover:text-white transition-all text-xs"
                        title="Go to end"
                      >
                        <FontAwesomeIcon icon={faHistory} />
                      </motion.button>
                    </div>
                  </div>
                </div>

                {/* Body - Side-by-side Layout */}
                <div className="flex-1 flex overflow-hidden">
                  {/* Sync Log Panel - Left (40%) */}
                  <SyncLogPanel
                    replay={replay}
                    actions={playback.actions}
                    currentIndex={currentAction?.index ?? -1}
                    onSeek={playback.seek}
                    isPlaying={isPlaying}
                  />

                  {/* Focus Stage - Right (60%) */}
                  <div className="flex-1 overflow-auto p-6">
                  {currentAction ? (
                    <div className="h-full flex flex-col gap-4">
                      {/* Action Header - Actor vs Target */}
                      <motion.div
                        initial={{ opacity: 0, y: -10 }}
                        animate={{ opacity: 1, y: 0 }}
                        className="relative overflow-hidden rounded-2xl border border-purple-500/30 bg-gradient-to-br from-purple-900/20 via-slate-900/50 to-blue-900/20 backdrop-blur-sm"
                      >
                        {/* Top border accent */}
                        <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-purple-500 via-purple-400 to-blue-500" />

                        <div className="p-6">
                          {/* Round & Turn Badge */}
                          <div className="absolute top-4 right-4 flex flex-col items-end">
                            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-lg bg-slate-900/60 border border-purple-500/30">
                              <FontAwesomeIcon icon={faClock} className="text-purple-400 text-xs" />
                              <span className="text-sm font-mono text-purple-300">
                                R{currentAction.roundNumber} • T{currentAction.turnIndex + 1}
                              </span>
                            </div>
                          </div>

                          {/* Actor vs Target Layout */}
                          <div className="relative flex items-stretch gap-6">
                            {/* Actor Panel */}
                            <motion.div
                              initial={{ opacity: 0, x: -20 }}
                              animate={{ opacity: 1, x: 0 }}
                              transition={{ delay: 0.1 }}
                              className="relative z-10 flex-1 bg-gradient-to-br from-purple-900/40 to-purple-800/20 border-2 border-purple-500/40 rounded-xl p-5"
                            >
                              <div className="flex items-center gap-3 mb-3">
                                <div className="w-12 h-12 rounded-lg bg-purple-600/30 border border-purple-500/50 flex items-center justify-center">
                                  <FontAwesomeIcon icon={faUser} className="text-purple-400 text-lg" />
                                </div>
                                <div>
                                  <p className="text-xs text-purple-400 font-mono uppercase tracking-wider">Actor</p>
                                  <p className="text-lg font-semibold text-white">{currentAction.unitId}</p>
                                </div>
                              </div>
                              <div className="flex items-center gap-2">
                                <FontAwesomeIcon icon={faGavel} className="text-purple-400/70 text-sm" />
                                <span className="text-sm text-slate-300">Action: </span>
                                <span className="text-sm font-mono text-purple-300 bg-purple-500/20 px-2 py-0.5 rounded">
                                  {currentAction.action.actionId}
                                </span>
                              </div>
                            </motion.div>

                            {/* Interaction Arrow */}
                            {(() => {
                              const targetId = extractTargetId(currentAction.action.subEvents)
                              const interactionType = getInteractionType(currentAction.action.subEvents)

                              if (!targetId || !interactionType) return null

                              const arrowColor = interactionType === 'attack' ? 'text-red-400' :
                                                  interactionType === 'heal' ? 'text-emerald-400' :
                                                  interactionType === 'buff' ? 'text-amber-400' : 'text-purple-400'

                              return (
                                <motion.div
                                  initial={{ opacity: 0, scale: 0.5 }}
                                  animate={{ opacity: 1, scale: 1 }}
                                  transition={{ delay: 0.2 }}
                                  className="relative z-20 flex-shrink-0 flex items-center justify-center self-center"
                                >
                                  <div className={`w-14 h-14 rounded-full bg-slate-900/80 border-2 ${arrowColor.replace('text', 'border')} flex items-center justify-center shadow-lg`}>
                                    <FontAwesomeIcon
                                      icon={interactionType === 'attack' ? faBolt :
                                             interactionType === 'heal' ? faHeart :
                                             interactionType === 'buff' ? faMagic : faExchangeAlt}
                                      className={`${arrowColor} text-lg`}
                                    />
                                  </div>
                                </motion.div>
                              )
                            })()}

                            {/* Target Panel */}
                            {(() => {
                              const targetId = extractTargetId(currentAction.action.subEvents)
                              const interactionType = getInteractionType(currentAction.action.subEvents)

                              if (!targetId || !interactionType) return null

                              const borderColor = interactionType === 'attack' ? 'border-red-500/40' :
                                                  interactionType === 'heal' ? 'border-emerald-500/40' :
                                                  interactionType === 'buff' ? 'border-amber-500/40' : 'border-purple-500/40'

                              const bgGradient = interactionType === 'attack' ? 'from-red-900/40 to-rose-800/20' :
                                                interactionType === 'heal' ? 'from-emerald-900/40 to-green-800/20' :
                                                interactionType === 'buff' ? 'from-amber-900/40 to-yellow-800/20' : 'from-purple-900/40 to-purple-800/20'

                              const iconColor = interactionType === 'attack' ? 'text-red-400' :
                                              interactionType === 'heal' ? 'text-emerald-400' :
                                              interactionType === 'buff' ? 'text-amber-400' : 'text-purple-400'

                              return (
                                <motion.div
                                  initial={{ opacity: 0, x: 20 }}
                                  animate={{ opacity: 1, x: 0 }}
                                  transition={{ delay: 0.15 }}
                                  className={`relative z-10 flex-1 bg-gradient-to-br ${bgGradient} border-2 ${borderColor} rounded-xl p-5`}
                                >
                                  <div className="flex items-center gap-3 mb-3">
                                    <div className={`w-12 h-12 rounded-lg bg-slate-900/40 border ${borderColor.replace('/40', '/50')} flex items-center justify-center`}>
                                      <FontAwesomeIcon icon={faUserFriends} className={`${iconColor} text-lg`} />
                                    </div>
                                    <div>
                                      <p className={`text-xs font-mono uppercase tracking-wider ${iconColor}`}>Target</p>
                                      <p className="text-lg font-semibold text-white">{targetId}</p>
                                    </div>
                                  </div>
                                  <div className="flex items-center gap-2">
                                    <FontAwesomeIcon icon={faCrosshairs} className={`${iconColor.replace('400', '400/70')} text-sm`} />
                                    <span className="text-sm text-slate-300">Interaction: </span>
                                    <span className={`text-sm font-medium capitalize ${iconColor}`}>
                                      {interactionType}
                                    </span>
                                  </div>
                                </motion.div>
                              )
                            })()}
                          </div>
                        </div>
                      </motion.div>

                      {/* Sub-Events Timeline */}
                      <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        transition={{ delay: 0.1 }}
                        className="flex-1 overflow-auto"
                      >
                        <div className="flex items-center gap-2 mb-3">
                          <FontAwesomeIcon icon={faMagic} className="text-purple-400 text-xs" />
                          <h3 className="text-sm font-medium text-slate-300">Sub-Events</h3>
                          <span className="text-xs text-slate-500">({currentAction.action.subEvents.length})</span>
                        </div>
                        <div className="space-y-2 pr-2">
                          {currentAction.action.subEvents.length > 0 ? (
                            currentAction.action.subEvents.map((event, index) => (
                              <SubEventCard key={index} event={event} index={index} />
                            ))
                          ) : (
                            <div className="flex flex-col items-center justify-center py-12 text-slate-500">
                              <FontAwesomeIcon icon={faClock} className="text-2xl mb-2" />
                              <p className="text-sm">No sub-events recorded</p>
                            </div>
                          )}
                        </div>
                      </motion.div>
                    </div>
                  ) : (
                    <div className="h-full min-h-[300px] rounded-2xl border-2 border-dashed border-slate-700/50 bg-slate-800/10 flex flex-col items-center justify-center">
                      <motion.div
                        initial={{ opacity: 0, y: 10 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ delay: 0.2 }}
                        className="text-center"
                      >
                        <div className="w-20 h-20 rounded-full bg-slate-800/30 border border-slate-700/50 flex items-center justify-center mb-4 mx-auto">
                          <FontAwesomeIcon icon={faClock} className="text-3xl text-slate-600" />
                        </div>
                        <h3 className="text-xl font-medium text-slate-300 mb-2">No Action Selected</h3>
                        <p className="text-sm text-slate-500 max-w-md mx-auto">
                          Use the playback controls to navigate through the combat timeline.
                        </p>
                      </motion.div>
                    </div>
                  )}
                </div>
                </div>

                {/* Footer - Round Navigator */}
                <div className="border-t border-slate-800/50 px-6 py-4 bg-slate-900/60 backdrop-blur-xl">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <FontAwesomeIcon icon={faHistory} className="text-slate-500 text-xs" />
                      <span className="text-xs text-slate-400 font-medium">Round Navigator</span>
                    </div>
                    <div className="flex gap-1.5 flex-wrap justify-end">
                      {Array.from({ length: Math.min(totalRounds, 12) }).map((_, i) => (
                        <motion.button
                          key={i}
                          whileHover={{ scale: 1.05 }}
                          whileTap={{ scale: 0.95 }}
                          onClick={() => seekToRound(i)}
                          className={`relative w-9 h-9 rounded-lg text-xs font-mono font-semibold transition-all border ${
                            currentRoundIndex === i
                              ? 'bg-gradient-to-br from-purple-600 to-purple-700 text-white border-purple-500 shadow-lg shadow-purple-500/20'
                              : 'bg-slate-800/50 text-slate-400 hover:bg-slate-700/50 hover:text-white border-slate-700/30'
                          }`}
                        >
                          {i + 1}
                          {currentRoundIndex === i && (
                            <motion.div
                              layoutId="activeRound"
                              className="absolute inset-0 rounded-lg bg-white/10"
                              transition={{ type: 'spring', stiffness: 300, damping: 30 }}
                            />
                          )}
                        </motion.button>
                      ))}
                      {totalRounds > 12 && (
                        <div className="flex items-center gap-1 px-2">
                          <span className="text-xs text-slate-500">+</span>
                          <span className="text-xs text-slate-400 font-mono">{totalRounds - 12}</span>
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              </motion.div>
            </Dialog.Content>
          </>
        )}
      </AnimatePresence>
    </Dialog.Root>
  )
}
