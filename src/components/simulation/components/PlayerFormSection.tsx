import React, { memo } from "react"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faTrash, faSave, faFolder } from "@fortawesome/free-solid-svg-icons"
import EncounterForm from "../encounterForm"
import { Creature } from "@/model/model"
import styles from '../simulation.module.scss'

interface PlayerFormSectionProps {
    players: Creature[]
    setPlayers: (players: Creature[]) => void
    isEmptyResult: boolean
    canSave: boolean
    setSaving: (saving: boolean) => void
    setLoading: (loading: boolean) => void
    setIsEditing: (isEditing: boolean) => void
}

export const PlayerFormSection = memo<PlayerFormSectionProps>(({
    players,
    setPlayers,
    isEmptyResult,
    canSave,
    setSaving,
    setLoading,
    setIsEditing
}) => {
    return (
        <div className="encounter-builder-section player-form-section" data-testid="player-section">
            <EncounterForm
                mode='player'
                encounter={{ id: 'players', monsters: players, type: 'combat', targetRole: 'Standard' }}
                onUpdate={(newValue) => setPlayers(newValue.monsters)}
                onEditingChange={setIsEditing}>
                <>
                    {!isEmptyResult ? (
                        <button onClick={() => { setPlayers([]) }} data-testid="clear-all-btn">
                            <FontAwesomeIcon icon={faTrash} />
                            Clear Adventuring Day
                        </button>
                    ) : null}
                    {canSave ? (
                        <button onClick={() => setSaving(true)} data-testid="save-day-btn">
                            <FontAwesomeIcon icon={faSave} />
                            Save Adventuring Day
                        </button>
                    ) : null}
                    <button onClick={() => setLoading(true)} data-testid="load-day-btn">
                        <FontAwesomeIcon icon={faFolder} />
                        Load Adventuring Day
                    </button>
                </>
            </EncounterForm>
        </div>
    )
})

PlayerFormSection.displayName = 'PlayerFormSection'