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

type PropType = {
    currentPlayers: Creature[],
    currentEncounters: Encounter[],
    onCancel: () => void,
    onApplyChanges: (newPlayers: Creature[], newEncounters: Encounter[]) => void, // Callback to update parent state
    onEditingChange?: (isEditing: boolean) => void,
}

function carefulSave(key: string, value: string) {
    if (!localStorage.getItem('useLocalStorage')) return
    localStorage.setItem(key, value)
}

const SaveFileSchema = z.object({
    updated: z.number(),
    name: z.string(),
    players: z.array(CreatureSchema),
    encounters: z.array(EncounterSchema),
})
type SaveFile = z.infer<typeof SaveFileSchema>

const SaveCollectionSchema = z.array(SaveFileSchema)
type SaveCollection = z.infer<typeof SaveCollectionSchema>

const ExampleAdventuringDay: SaveFile = {
    updated: Date.now(),
    name: 'Example',
    players: [
        PlayerTemplates.barbarian(3, { gwm: false, weaponBonus: 0 }),
        PlayerTemplates.cleric(3, {}),
        PlayerTemplates.rogue(3, { ss: false, weaponBonus: 0 }),
        PlayerTemplates.wizard(3, {}),
    ],
    encounters: [
        {
            monsters: [
                getMonster('Bandit Captain')!,
                { ...getMonster('Bandit')!, count: 5 },
            ],
            playersSurprised: false,
            monstersSurprised: false,
            shortRest: false,
            playersPrecast: false,
            monstersPrecast: false,
        },
    ]
}

function loadSaves(): SaveCollection {
    if (typeof localStorage === undefined) return []

    const json = localStorage.getItem('saveFiles')
    if (!json) return [ExampleAdventuringDay]

    const obj = JSON.parse(json)
    const parsed = SaveCollectionSchema.safeParse(obj)

    if (parsed.success) {
        return parsed.data
    }
    return []
}

function currentSaveName(): string {
    if (typeof localStorage === undefined) return ''

    return localStorage.getItem('saveName') || ''
}

const AdventuringDayForm: FC<PropType> = ({ currentPlayers, currentEncounters, onCancel, onApplyChanges, onEditingChange }) => {
    const [editedPlayers, setEditedPlayers] = useState<Creature[]>(currentPlayers);
    const [editedEncounters, setEditedEncounters] = useState<Encounter[]>(currentEncounters);
    const [editingPlayer, setEditingPlayer] = useState<Creature | null>(null);
    const [editingMonster, setEditingMonster] = useState<Creature | null>(null);
    const [editingMonsterEncounterIndex, setEditingMonsterEncounterIndex] = useState<number | null>(null);

    useEffect(() => {
        onEditingChange?.(editingPlayer !== null || editingMonster !== null);
    }, [editingPlayer, editingMonster, onEditingChange]);

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
        const newMonster = getMonster('Bandit')!; // Default Monster
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

    // Adapt existing save/load logic to work with edited state
    const useSharedContext = sharedStateGenerator('adventuringDayForm')
    const [saveName, setSaveName] = useSharedContext(currentSaveName())
    const [deleted, setDeleted] = useState(0)
    const [error, setError] = useState<string | null>(null)

    const isValidSaveName = useCalculatedState(() => !!saveName, [saveName])
    const searchResults = useCalculatedState(loadSaves, [saveName, deleted])


    function saveEditedDay() {
        if (!isValidSaveName) return;

        const newSaveFile: SaveFile = {
            updated: Date.now(),
            name: saveName,
            players: editedPlayers, // Use edited state
            encounters: editedEncounters, // Use edited state
        }

        const saveFiles = loadSaves()

        const existingIndex = saveFiles.findIndex(save => (save.name === newSaveFile.name))
        if (existingIndex !== -1) saveFiles[existingIndex] = newSaveFile
        else saveFiles.push(newSaveFile)

        carefulSave('saveFiles', JSON.stringify(saveFiles))
        carefulSave('saveName', saveName)
        onApplyChanges(editedPlayers, editedEncounters); // Pass current edited state back to parent
    }

    function loadSavedDay(nameToLoad: string) {
        const saveFile = loadSaves().find(save => (save.name === nameToLoad))

        if (!saveFile) return

        onApplyChanges(saveFile.players, saveFile.encounters) // Pass loaded state to parent
        setEditedPlayers(saveFile.players); // Update local state
        setEditedEncounters(saveFile.encounters); // Update local state
        setSaveName(nameToLoad);
        carefulSave('saveName', nameToLoad);
    }

    function deleteSave(nameToDelete: string) {
        setDeleted(deleted + 1)
        const saveFiles = loadSaves()
        const index = saveFiles.findIndex(save => (save.name === nameToDelete))

        if (index === -1) return

        saveFiles.splice(index, 1)
        carefulSave('saveFiles', JSON.stringify(saveFiles))

        if (saveName === nameToDelete) {
            setSaveName('');
            localStorage.removeItem('saveName');
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

        const saveFiles = loadSaves()

        const existingIndex = saveFiles.findIndex(save => (save.name === newSave.name))
        if (existingIndex !== -1) saveFiles[existingIndex] = newSave
        else saveFiles.push(newSave)

        carefulSave('saveFiles', JSON.stringify(saveFiles))
        carefulSave('saveName', newSave.name)
        onApplyChanges(newSave.players, newSave.encounters) // Update parent
        setEditedPlayers(newSave.players); // Update local state
        setEditedEncounters(newSave.encounters); // Update local state
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
                {searchResults.map(save => (
                    <div key={save.name} className={styles.saveItem}>
                        <span>{save.name} ({new Date(save.updated).toLocaleDateString()})</span>
                        <button onClick={() => loadSavedDay(save.name)}>Load</button>
                        <button onClick={() => deleteSave(save.name)}>Delete</button>
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