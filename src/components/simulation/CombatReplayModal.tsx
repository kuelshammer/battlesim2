import { FC, useState } from 'react'
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
  faArrowRight,
  faMagic,
  faBolt,
  faUser,
  faCrosshairs,
  faGavel,
  faExchangeAlt,
  faUserFriends
} from '@fortawesome/free-solid-svg-icons'
import { useCombatPlayback } from '@/hooks/useCombatPlayback'
import type { Replay } from '@/model/replayTypes'
import type { Event } from '@/model/model'

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
    play,
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

                {/* Body - Focus Stage */}
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
