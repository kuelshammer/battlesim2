import React, { FC, useState, Suspense, memo } from "react"
import { clone } from "@/model/utils"
import styles from './simulation.module.scss'
import OnboardingTour from "./OnboardingTour"
import PerformanceDashboard from "../debug/PerformanceDashboard"
import { semiPersistentContext } from "@/model/semiPersistentContext"
import AdventuringDayForm from "./adventuringDayForm"
import { UIToggleProvider } from "@/model/uiToggleState"
import { CrosshairProvider } from "./CrosshairContext"
import { CrosshairTooltip } from "./CrosshairLine"

// New hooks
import { useSimulationSession } from "./hooks/useSimulationSession"
import { useAutoSimulation } from "./hooks/useAutoSimulation"

// New components
import { SimulationHeader } from "./components/SimulationHeader"
import { BackendStatusPanel } from "./components/BackendStatusPanel"
import { PlayerFormSection } from "./components/PlayerFormSection"
import { TimelineItem } from "./components/TimelineItem"
import { AddTimelineButtons } from "./components/AddTimelineButtons"
import { OverallSummary } from "./components/OverallSummary"
import { SimulationModals } from "./components/SimulationModals"

type PropType = object

const Simulation: FC<PropType> = memo(() => {
    // UI state
    const [state, setState] = useState(new Map<string, unknown>())
    const [saving, setSaving] = useState(false)
    const [loading, setLoading] = useState(false)
    const [showLogModal, setShowLogModal] = useState(false)
    const [selectedEncounterIndex, setSelectedEncounterIndex] = useState<number | null>(null)
    const [selectedDecileIndex, setSelectedDecileIndex] = useState<number>(5) // Default to 50% Median
    const [runTour, setRunTour] = useState(false)
    const [showPerformanceDashboard, setShowPerformanceDashboard] = useState(false)

    // Use the new hooks
    const session = useSimulationSession()
    const simulation = useAutoSimulation(
        session.players,
        session.timeline,
        session.isPlayersLoaded,
        session.isTimelineLoaded
    )

    function applyOptimizedResult() {
        if (selectedEncounterIndex === null || !simulation.worker.optimizedResult) return;

        const timelineClone = clone(session.timeline);
        const item = timelineClone[selectedEncounterIndex];

        if (item.type === 'combat') {
            item.monsters = simulation.worker.optimizedResult.monsters;
            session.setTimeline(timelineClone);
        }

        simulation.worker.clearOptimizedResult();
        setSelectedEncounterIndex(null);
    }

    return (
        <UIToggleProvider>
            <div className={styles.simulation}>
                <semiPersistentContext.Provider value={{ state, setState }}>
                    <Suspense fallback={<div>Loading...</div>}>
                        <SimulationHeader
                            runTour={runTour}
                            setRunTour={setRunTour}
                            showPerformanceDashboard={showPerformanceDashboard}
                            setShowPerformanceDashboard={setShowPerformanceDashboard}
                        />

                        <BackendStatusPanel
                            worker={simulation.worker}
                            highPrecision={simulation.highPrecision}
                            setHighPrecision={simulation.setHighPrecision}
                            isEditing={simulation.isEditing}
                            simulationEvents={simulation.simulationEvents}
                            players={session.players}
                            timeline={session.timeline}
                        />

                        <PlayerFormSection
                            players={session.players}
                            setPlayers={session.setPlayers}
                            isEmptyResult={session.isEmptyResult}
                            canSave={simulation.canSave}
                            setSaving={simulation.setSaving}
                            setLoading={simulation.setLoading}
                            setIsEditing={simulation.setIsEditing}
                        />

                        <CrosshairProvider>
                            {session.timeline.map((item, index) => (
                                <TimelineItem
                                    key={index}
                                    item={item}
                                    index={index}
                                    timeline={session.timeline}
                                    players={session.players}
                                    combatantNames={session.combatantNames}
                                    isStale={simulation.isStale}
                                    encounterWeights={session.encounterWeights}
                                    worker={simulation.worker}
                                    simulationResults={simulation.simulationResults}
                                    setIsEditing={simulation.setIsEditing}
                                    updateTimelineItem={session.updateTimelineItem}
                                    deleteTimelineItem={session.deleteTimelineItem}
                                    swapTimelineItems={session.swapTimelineItems}
                                    setSelectedEncounterIndex={setSelectedEncounterIndex}
                                    setSelectedDecileIndex={setSelectedDecileIndex}
                                    setShowLogModal={setShowLogModal}
                                />
                            ))}

                            <AddTimelineButtons
                                createCombat={session.createCombat}
                                createShortRest={session.createShortRest}
                            />

                            <OverallSummary
                                worker={simulation.worker}
                                timeline={session.timeline}
                                encounterWeights={session.encounterWeights}
                                combatantNames={session.combatantNames}
                            />

                            <CrosshairTooltip />
                        </CrosshairProvider>

                        <SimulationModals
                            showLogModal={showLogModal}
                            selectedEncounterIndex={selectedEncounterIndex}
                            selectedDecileIndex={selectedDecileIndex}
                            setSelectedDecileIndex={setSelectedDecileIndex}
                            setShowLogModal={setShowLogModal}
                            setSelectedEncounterIndex={setSelectedEncounterIndex}
                            worker={simulation.worker}
                            combatantNames={session.combatantNames}
                            actionNames={session.actionNames}
                            timeline={session.timeline}
                            applyOptimizedResult={applyOptimizedResult}
                        />

                        {/* Adventuring Day Editor (Save/Load) */}
                        {(saving || loading) && (
                            <AdventuringDayForm
                                currentPlayers={session.players}
                                currentTimeline={session.timeline}
                                onCancel={() => { simulation.setSaving(false); simulation.setLoading(false); }}
                                onApplyChanges={(newPlayers, newTimeline) => {
                                    session.setPlayers(newPlayers);
                                    session.setTimeline(newTimeline);
                                    simulation.setSaving(false);
                                    simulation.setLoading(false);
                                }}
                                onEditingChange={simulation.setIsEditing}
                            />
                        )}
                    </Suspense>
                </semiPersistentContext.Provider>

                {/* Onboarding Tour */}
                <OnboardingTour
                    forceRun={runTour}
                    onTourEnd={() => setRunTour(false)}
                />

                {/* Performance Dashboard */}
                <PerformanceDashboard
                    isVisible={showPerformanceDashboard}
                    onClose={() => setShowPerformanceDashboard(false)}
                />
            </div>
        </UIToggleProvider>
    )
})

export default Simulation