import { FC, useState, useEffect } from "react"
import { Creature, CreatureSchema, Encounter, EncounterSchema } from "@/model/model"
import styles from './adventuringDayForm.module.scss'
import { sharedStateGenerator, useCalculatedState } from "@/model/utils"
import { z } from 'zod'
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faDownload, faFolder, faSave, faTrash, faUpload, faPlus, faPen, faTimes, faEye } from "@fortawesome/free-solid-svg-icons"
import { PlayerTemplates } from "@/data/data"
import { getMonster } from "@/data/monsters"
import Modal from "@/utils/modal"
import Checkbox from "@/utils/checkbox"
import PlayerForm from "../creatureForm/playerForm"
import { v4 as uuidv4 } from 'uuid';
import MonsterForm from "../creatureForm/monsterForm"
import { clone } from "@/model/utils"

type PropType = {
    currentPlayers: Creature[],
    currentEncounters: Encounter[],
    onCancel: () => void,
    onApplyChanges: (newPlayers: Creature[], newEncounters: Encounter[]) => void, // Callback to update parent state
    onEditingChange?: (isEditing: boolean) => void,
}

const SaveFileSchema = z.object({
    updated: z.number(),
    name: z.string(),
    filename: z.string().optional(),
    players: z.array(CreatureSchema),
    encounters: z.array(EncounterSchema),
})
type SaveFile = z.infer<typeof SaveFileSchema>

const SaveCollectionSchema = z.array(SaveFileSchema)
type SaveCollection = z.infer<typeof SaveCollectionSchema>

const AdventuringDayForm: FC<PropType> = ({ currentPlayers, currentEncounters, onCancel, onApplyChanges, onEditingChange }) => {
    const [editedPlayers, setEditedPlayers] = useState<Creature[]>(currentPlayers);
    const [editedEncounters, setEditedEncounters] = useState<Encounter[]>(currentEncounters);
    const [editingPlayer, setEditingPlayer] = useState<Creature | null>(null);
    const [editingMonster, setEditingMonster] = useState<Creature | null>(null);
    const [editingMonsterEncounterIndex, setEditingMonsterEncounterIndex] = useState<number | null>(null);

    const [saveName, setSaveName] = useState('')
    const [savedDays, setSavedDays] = useState<SaveFile[]>([])
    const [error, setError] = useState<string | null>(null)
    const [loading, setLoading] = useState(false)

    useEffect(() => {
        onEditingChange?.(editingPlayer !== null || editingMonster !== null);
    }, [editingPlayer, editingMonster, onEditingChange]);

    useEffect(() => {
        fetchSaves()
    }, [])

    async function fetchSaves() {
        setLoading(true)
        try {
            const response = await fetch('/api/adventuring-days')
            const data = await response.json()
            setSavedDays(data)
        } catch (e) {
            setError('Failed to fetch saves')
        } finally {
            setLoading(false)
        }
    }

    // Sync external changes (if parent re-renders with new props)
    useEffect(() => {
        setEditedPlayers(currentPlayers);
    }, [currentPlayers]);

    useEffect(() => {
        setEditedEncounters(currentEncounters);
    }, [currentEncounters]);

    function addPlayer() {
        const newPlayer = PlayerTemplates.barbarian(1, { gwm: false, weaponBonus: 0 }); // Default Barbarian with correct options
        newPlayer.id = uuidv4();
        setEditedPlayers([...editedPlayers, newPlayer]);
    }

    function updatePlayer(updatedPlayer: Creature) {
        setEditedPlayers(editedPlayers.map(p => p.id === updatedPlayer.id ? updatedPlayer : p));
        setEditingPlayer(null); // Close modal
    }

    function removePlayer(id: string) {
        setEditedPlayers(editedPlayers.filter(p => p.id !== id));
    }

    function addEncounter() {
        const newEncounter: Encounter = {
            id: uuidv4(),
            monsters: [],
            playersSurprised: false,
            monstersSurprised: false,
            shortRest: false,
            playersPrecast: false,
            monstersPrecast: false,
        };
        setEditedEncounters([...editedEncounters, newEncounter]);
    }

    function updateEncounter(index: number, updatedEncounter: Encounter) {
        setEditedEncounters(editedEncounters.map((e, i) => i === index ? updatedEncounter : e));
    }

    function removeEncounter(index: number) {
        setEditedEncounters(editedEncounters.filter((_, i) => i !== index));
    }

    function addMonsterToEncounter(encounterIndex: number) {
        const bandit = getMonster('Bandit');
        if (!bandit) return;
        const newMonster = clone(bandit);
        newMonster.id = uuidv4();
        const updatedEncounters = editedEncounters.map((enc, i) => {
            if (i === encounterIndex) {
                return { ...enc, monsters: [...enc.monsters, newMonster] };
            }
            return enc;
        });
        setEditedEncounters(updatedEncounters);
    }

    function updateMonsterInEncounter(encounterIndex: number, updatedMonster: Creature) {
        const updatedEncounters = editedEncounters.map((enc, i) => {
            if (i === encounterIndex) {
                return {
                    ...enc,
                    monsters: enc.monsters.map(m => m.id === updatedMonster.id ? updatedMonster : m)
                };
            }
            return enc;
        });
        setEditedEncounters(updatedEncounters);
        setEditingMonster(null); // Close modal
        setEditingMonsterEncounterIndex(null);
    }

    function removeMonsterFromEncounter(encounterIndex: number, monsterId: string) {
        const updatedEncounters = editedEncounters.map((enc, i) => {
            if (i === encounterIndex) {
                return {
                    ...enc,
                    monsters: enc.monsters.filter(m => m.id !== monsterId)
                };
            }
            return enc;
        });
        setEditedEncounters(updatedEncounters);
    }

    const isValidSaveName = !!saveName

    async function saveEditedDay() {
        if (!isValidSaveName) return;

        try {
            const response = await fetch('/api/adventuring-days', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name: saveName,
                    players: editedPlayers,
                    encounters: editedEncounters,
                })
            })
            if (response.ok) {
                fetchSaves()
                onApplyChanges(editedPlayers, editedEncounters);
            }
        } catch (e) {
            setError('Failed to save')
        }
    }

    function loadSavedDay(save: SaveFile) {
        onApplyChanges(save.players, save.encounters)
        setEditedPlayers(save.players);
        setEditedEncounters(save.encounters);
        setSaveName(save.name);
    }

    async function deleteSave(filename: string) {
        try {
            const response = await fetch(`/api/adventuring-days/${filename}`, {
                method: 'DELETE'
            })
            if (response.ok) {
                fetchSaves()
            }
        } catch (e) {
            setError('Failed to delete')
        }
    }

    async function onDownload() {
        const newSaveFile: SaveFile = {
            updated: Date.now(),
            name: saveName,
            players: editedPlayers,
            encounters: editedEncounters,
        }

        const file = new Blob([JSON.stringify(newSaveFile)], { type: 'json' })
        const a = document.createElement("a")
        const url = URL.createObjectURL(file)
        a.href = url
        a.download = `${newSaveFile.name}.json`
        document.body.appendChild(a)
        a.click()
        setTimeout(() => {
            document.body.removeChild(a)
            window.URL.revokeObjectURL(url)
        }, 0)
    }

    async function onUpload(files: FileList | null) {
        if (!files || !files.length) { setError('No files uploaded'); return }

        const file = files[0]
        if (!file) { setError('No file uploaded'); return }

        const json = await file.text()
        if (!json) return

        let obj
        try { obj = JSON.parse(json) }
        catch (e) { setError('File is not valid JSON'); return }

        const parsed = SaveFileSchema.safeParse(obj)
        if (!parsed.success) { setError('Invalid schema'); return }

        const newSave: SaveFile = parsed.data

        try {
            await fetch('/api/adventuring-days', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name: newSave.name,
                    players: newSave.players,
                    encounters: newSave.encounters,
                })
            })
            fetchSaves()
            onApplyChanges(newSave.players, newSave.encounters)
            setEditedPlayers(newSave.players);
            setEditedEncounters(newSave.encounters);
            setSaveName(newSave.name);
        } catch (e) {
            setError('Failed to upload')
        }
    }


    return (
        <Modal onCancel={onCancel} className={styles.adventuringDayEditor}>
            <h1>Adventuring Day Editor</h1>

            {/* Save/Load Toolbar */}
            <section className={styles.toolbar}>
                <h3>Save Name:</h3>
                <input type="text" value={saveName} onChange={e => setSaveName(e.target.value)} />
                <button disabled={!isValidSaveName} onClick={saveEditedDay}>
                    <FontAwesomeIcon icon={faSave} /> Save
                </button>
                <button className="tooltipContainer" onClick={onDownload}>
                    <FontAwesomeIcon icon={faDownload} /> Download
                </button>
                <label htmlFor="file" className="tooltipContainer">
                    <FontAwesomeIcon icon={faUpload} /> Upload
                </label>
                <input
                    type="file"
                    id="file"
                    accept="application/json"
                    style={{ display: "none" }}
                    onChange={(e) => onUpload(e.target.files)} />
            </section>

            {/* Existing Save Files List (still useful for loading others) */}
            <section className={styles.saveFilesList}>
                <h3>Saved Days:</h3>
                {loading ? <p>Loading saves...</p> : 
                 savedDays.length === 0 ? <p>No saved adventuring days found.</p> :
                 savedDays.map(save => (
                    <div key={save.name} className={styles.saveItem}>
                        <span>{save.name} ({new Date(save.updated).toLocaleDateString()})</span>
                        <button onClick={() => loadSavedDay(save)}>Load</button>
                        <button onClick={() => deleteSave(save.filename || save.name)}>Delete</button>
                    </div>
                ))}
            </section>

            {/* Players Section */}
            <section className={styles.playersSection}>
                <h2>Players</h2>
                <button onClick={addPlayer}><FontAwesomeIcon icon={faPlus} /> Add Player</button>
                <div className={styles.playerList}>
                    {editedPlayers.map(player => (
                        <div key={player.id} className={styles.playerItem}>
                            <span>{player.name} (Lvl {player.class?.level} {player.class?.type})</span>
                            <button onClick={() => setEditingPlayer(player)}><FontAwesomeIcon icon={faPen} /> Edit</button>
                            <button onClick={() => removePlayer(player.id)}><FontAwesomeIcon icon={faTimes} /> Remove</button>
                        </div>
                    ))}
                </div>
            </section>

            {/* Encounters Section */}
            <section className={styles.encountersSection}>
                <h2>Encounters</h2>
                <button onClick={addEncounter}><FontAwesomeIcon icon={faPlus} /> Add Encounter</button>
                <div className={styles.encounterList}>
                    {editedEncounters.map((encounter, encIndex) => (
                        <div key={encIndex} className={styles.encounterItem}>
                            <h3>Encounter {encIndex + 1}</h3>
                            <div className={styles.encounterSettings}>
                                <Checkbox value={encounter.playersSurprised || false} onToggle={() => updateEncounter(encIndex, { ...encounter, playersSurprised: !encounter.playersSurprised })}>Players Surprised</Checkbox>
                                <Checkbox value={encounter.monstersSurprised || false} onToggle={() => updateEncounter(encIndex, { ...encounter, monstersSurprised: !encounter.monstersSurprised })}>Monsters Surprised</Checkbox>
                                <Checkbox value={encounter.playersPrecast || false} onToggle={() => updateEncounter(encIndex, { ...encounter, playersPrecast: !encounter.playersPrecast })}>Players Precast</Checkbox>
                                <Checkbox value={encounter.monstersPrecast || false} onToggle={() => updateEncounter(encIndex, { ...encounter, monstersPrecast: !encounter.monstersPrecast })}>Monsters Precast</Checkbox>
                                <Checkbox value={encounter.shortRest || false} onToggle={() => updateEncounter(encIndex, { ...encounter, shortRest: !encounter.shortRest })}>Short Rest After</Checkbox>
                            </div>
                            <h4>Monsters</h4>
                            <button onClick={() => addMonsterToEncounter(encIndex)}><FontAwesomeIcon icon={faPlus} /> Add Monster</button>
                            <div className={styles.monsterList}>
                                {encounter.monsters.map(monster => (
                                    <div key={monster.id} className={styles.monsterItem}>
                                        <span>{monster.name} (x{monster.count})</span>
                                        <button onClick={() => { setEditingMonster(monster); setEditingMonsterEncounterIndex(encIndex); }}><FontAwesomeIcon icon={faPen} /> Edit</button>
                                        <button onClick={() => removeMonsterFromEncounter(encIndex, monster.id)}><FontAwesomeIcon icon={faTimes} /> Remove</button>
                                    </div>
                                ))}
                            </div>
                            <button onClick={() => removeEncounter(encIndex)}><FontAwesomeIcon icon={faTrash} /> Remove Encounter</button>
                        </div>
                    ))}
                </div>
            </section>

            {/* Player Edit Modal */}
            {editingPlayer && (
                <Modal onCancel={() => setEditingPlayer(null)}>
                    <PlayerForm value={editingPlayer} onChange={updatePlayer} />
                </Modal>
            )}

            {/* Monster Edit Modal */}
            {editingMonster && editingMonsterEncounterIndex !== null && (
                <Modal onCancel={() => { setEditingMonster(null); setEditingMonsterEncounterIndex(null); }}>
                    <MonsterForm value={editingMonster} onChange={(updatedMonster) => updateMonsterInEncounter(editingMonsterEncounterIndex, updatedMonster)} />
                </Modal>
            )}

            {(error !== null) ? (
                <div className={styles.error}>
                    {error}
                </div>
            ) : null}
        </Modal>
    )
}

export default AdventuringDayForm