import { FC, useMemo } from "react"
import { Combattant, Round } from "@/model/model"
import styles from './ActionEconomyDisplay.module.scss'

// Action economy state enum
type ActionEconomyState = 'EnemyAdvantage' | 'Even' | 'PlayerAdvantage'

interface ActionEconomyStatus {
  playerCount: number
  monsterCount: number
  roundsToKillMonsters: number
  roundsToKillPlayers: number
  combinedRatio: number
  actionRatio: number
  timeRatio: number
  state: ActionEconomyState
}

// Calculate action economy status from current combatants
function calculateActionEconomy(round: Round): ActionEconomyStatus {
  const consciousPlayers = round.team1.filter(c => c.initialState.currentHp > 0)
  const consciousMonsters = round.team2.filter(c => c.initialState.currentHp > 0)

  const playerCount = consciousPlayers.length
  const monsterCount = consciousMonsters.length

  // Handle edge cases
  if (monsterCount === 0) {
    return {
      playerCount,
      monsterCount: 0,
      roundsToKillMonsters: 0,
      roundsToKillPlayers: Infinity,
      combinedRatio: Infinity,
      actionRatio: Infinity,
      timeRatio: 0,
      state: 'PlayerAdvantage',
    }
  }

  if (playerCount === 0) {
    return {
      playerCount: 0,
      monsterCount,
      roundsToKillMonsters: Infinity,
      roundsToKillPlayers: 0,
      combinedRatio: 0,
      actionRatio: 0,
      timeRatio: Infinity,
      state: 'EnemyAdvantage',
    }
  }

  // Calculate total HP
  const totalPlayerHp = consciousPlayers.reduce((sum, c) => sum + c.initialState.currentHp, 0)
  const totalMonsterHp = consciousMonsters.reduce((sum, c) => sum + c.initialState.currentHp, 0)

  // Estimate DPR using simple heuristics (action count + average damage)
  // In a full implementation, this would use the CombatantStats from the backend
  const playerDpr = consciousPlayers.length * 15 // Rough estimate
  const monsterDpr = consciousMonsters.length * 10 // Rough estimate

  // Calculate rounds to exhaustion
  const roundsToKillMonsters = playerDpr > 0 ? totalMonsterHp / playerDpr : Infinity
  const roundsToKillPlayers = monsterDpr > 0 ? totalPlayerHp / monsterDpr : Infinity

  // Calculate ratios
  const actionRatio = playerCount / monsterCount
  const timeRatio = roundsToKillMonsters / roundsToKillPlayers
  const combinedRatio = Math.sqrt(actionRatio * timeRatio)

  // Determine state
  let state: ActionEconomyState
  if (combinedRatio < 0.6) {
    state = 'EnemyAdvantage'
  } else if (combinedRatio > 1.5) {
    state = 'PlayerAdvantage'
  } else {
    state = 'Even'
  }

  return {
    playerCount,
    monsterCount,
    roundsToKillMonsters,
    roundsToKillPlayers,
    combinedRatio,
    actionRatio,
    timeRatio,
    state,
  }
}

// Get display info for state
function getStateInfo(state: ActionEconomyState) {
  switch (state) {
    case 'EnemyAdvantage':
      return {
        icon: '🔴',
        label: 'Enemy Advantage',
        recommendation: 'Nova! Use AoE and burst damage to reduce their action count fast.',
      }
    case 'Even':
      return {
        icon: '🟡',
        label: 'Even',
        recommendation: 'Normal tactics. Balance damage and resource conservation.',
      }
    case 'PlayerAdvantage':
      return {
        icon: '🟢',
        recommendation: 'Conserve! Use cantrips and basic attacks. Save resources for harder fights.',
      }
  }
}

type PropType = {
  round: Round
}

const ActionEconomyDisplay: FC<PropType> = ({ round }) => {
  const status = useMemo(() => calculateActionEconomy(round), [round])
  const stateInfo = getStateInfo(status.state)

  // Get state class for styling
  const stateClass = `${styles.actionEconomy} ${styles[status.state.toLowerCase()]}`

  return (
    <div className={stateClass}>
      <div className={styles.header}>
        <span className={styles.icon}>{stateInfo.icon}</span>
        <span className={styles.label}>Action Economy: {stateInfo.label}</span>
      </div>

      <div className={styles.details}>
        <span className={styles.counts}>
          ({status.playerCount} vs {status.monsterCount})
        </span>
        <span className={styles.rounds}>
          You {status.roundsToKillMonsters.toFixed(1)} rounds | Them {status.roundsToKillPlayers.toFixed(1)} rounds
        </span>
      </div>

      <div className={styles.recommendation}>
        {stateInfo.recommendation}
      </div>
    </div>
  )
}

export default ActionEconomyDisplay
