import { FC, useState, useRef, useEffect } from 'react'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { faList } from '@fortawesome/free-solid-svg-icons'
import type { SyncLogPanelProps } from './combatReplayTypes'
import { TurnCard } from './TurnCard'

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
export const SyncLogPanel: FC<SyncLogPanelProps> = ({
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
