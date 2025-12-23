import { FC, useState, useEffect, useCallback } from "react"
import { Creature, CreatureSchema, TimelineEvent, TimelineEventSchema } from "@/model/model"
import styles from './adventuringDayForm.module.scss'
import { z } from 'zod'
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faDownload, faSave, faTrash, faUpload, faPlus, faPen, faTimes, faBed } from "@fortawesome/free-solid-svg-icons"
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
    currentTimeline: TimelineEvent[],
    onCancel: () => void,
    onApplyChanges: (newPlayers: Creature[], newTimeline: TimelineEvent[]) => void,
    onEditingChange?: (isEditing: boolean) => void,
}

const SaveFileSchema = z.object({
    updated: z.number(),
    name: z.string(),
    filename: z.string().optional(),
    players: z.array(CreatureSchema),
    timeline: z.array(TimelineEventSchema),
})
type SaveFile = z.infer<typeof SaveFileSchema>

const AdventuringDayForm: FC<PropType> = ({ currentPlayers, currentTimeline, onCancel, onApplyChanges, onEditingChange }) => {
    const [editedPlayers, setEditedPlayers] = useState<Creature[]>(currentPlayers);
    const [editedTimeline, setEditedTimeline] = useState<TimelineEvent[]>(currentTimeline);
    const [saveName, setSaveName] = useState("");
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const [savedDays, setSavedDays] = useState<SaveFile[]>([]);
    
    const [editingPlayer, setEditingPlayer] = useState<Creature | null>(null);
    const [editingMonster, setEditingMonster] = useState<Creature | null>(null);
    const [editingMonsterEncounterIndex, setEditingMonsterEncounterIndex] = useState<number | null>(null);

    const fetchSaves = useCallback(async () => {
        setLoading(true);
        try {
            const response = await fetch('/api/adventuring-days');
            if (response.ok) {
                const data = await response.json();
                setSavedDays(data);
            }
        } catch (e) {
            console.error("Failed to fetch saves:", e);
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        fetchSaves();
    }, [fetchSaves]);

    // Sync external changes
    useEffect(() => {
        setEditedPlayers(currentPlayers);
    }, [currentPlayers]);

    useEffect(() => {
        setEditedTimeline(currentTimeline);
    }, [currentTimeline]);

    function addPlayer() {
        const newPlayer = PlayerTemplates.barbarian(1, { gwm: false, weaponBonus: 0 }); 
        newPlayer.id = uuidv4();
        setEditedPlayers([...editedPlayers, newPlayer]);
    }

    function updatePlayer(updatedPlayer: Creature) {
        setEditedPlayers(editedPlayers.map(p => p.id === updatedPlayer.id ? updatedPlayer : p));
        setEditingPlayer(null); 
    }

    function removePlayer(id: string) {
        setEditedPlayers(editedPlayers.filter(p => p.id !== id));
    }

    function addEncounter() {
        const newEncounter: TimelineEvent = {
            type: 'combat',
            id: uuidv4(),
            monsters: [],
            playersSurprised: false,
            monstersSurprised: false,
            playersPrecast: false,
            monstersPrecast: false,
            targetRole: 'Standard',
        };
        setEditedTimeline([...editedTimeline, newEncounter]);
    }

    function addShortRest() {
        const newRest: TimelineEvent = {
            type: 'shortRest',
            id: uuidv4(),
        };
        setEditedTimeline([...editedTimeline, newRest]);
    }

    function updateTimelineItem(index: number, updatedEvent: TimelineEvent) {
        setEditedTimeline(editedTimeline.map((e, i) => i === index ? updatedEvent : e));
    }

    function removeTimelineItem(index: number) {
        setEditedTimeline(editedTimeline.filter((_, i) => i !== index));
    }

    function swapTimelineItems(idx1: number, idx2: number) {
        if (idx1 < 0 || idx1 >= editedTimeline.length || idx2 < 0 || idx2 >= editedTimeline.length) return;
        const newTimeline = [...editedTimeline];
        [newTimeline[idx1], newTimeline[idx2]] = [newTimeline[idx2], newTimeline[idx1]];
        setEditedTimeline(newTimeline);
    }

    function addMonsterToEncounter(timelineIndex: number) {
        const item = editedTimeline[timelineIndex];
        if (item?.type !== 'combat') return;

        const bandit = getMonster('Bandit');
        if (!bandit) return;
        const newMonster = clone(bandit);
        newMonster.id = uuidv4();
        
        const updatedTimeline = editedTimeline.map((event, i) => {
            if (i === timelineIndex && event.type === 'combat') {
                return { ...event, monsters: [...event.monsters, newMonster] };
            }
            return event;
        });
        setEditedTimeline(updatedTimeline);
    }

    function updateMonsterInEncounter(timelineIndex: number, updatedMonster: Creature) {
        const updatedTimeline = editedTimeline.map((event, i) => {
            if (i === timelineIndex && event.type === 'combat') {
                return {
                    ...event,
                    monsters: event.monsters.map(m => m.id === updatedMonster.id ? updatedMonster : m)
                };
            }
            return event;
        });
        setEditedTimeline(updatedTimeline);
        setEditingMonster(null); 
        setEditingMonsterEncounterIndex(null);
    }

    function removeMonsterFromEncounter(timelineIndex: number, monsterId: string) {
        const updatedTimeline = editedTimeline.map((event, i) => {
            if (i === timelineIndex && event.type === 'combat') {
                return {
                    ...event,
                    monsters: event.monsters.filter(m => m.id !== monsterId)
                };
            }
            return event;
        });
        setEditedTimeline(updatedTimeline);
    }

    const isValidSaveName = !!saveName

    async function applyAndSave() {
        onApplyChanges(editedPlayers, editedTimeline);
        
        if (isValidSaveName) {
            try {
                await fetch('/api/adventuring-days', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        updated: Date.now(),
                        name: saveName,
                        players: editedPlayers,
                        timeline: editedTimeline,
                    })
                })
                fetchSaves()
            } catch (e) {
                setError('Failed to save')
            }
        }
    }

    function loadSavedDay(save: SaveFile) {
        setEditedPlayers(save.players);
        setEditedTimeline(save.timeline);
        setSaveName(save.name);
        onApplyChanges(save.players, save.timeline);
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
            timeline: editedTimeline,
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
                    timeline: newSave.timeline,
                })
            })
            fetchSaves()
            onApplyChanges(newSave.players, newSave.timeline)
            setEditedPlayers(newSave.players);
            setEditedTimeline(newSave.timeline);
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
                <button disabled={!isValidSaveName} onClick={applyAndSave}>
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

            {/* Existing Save Files List */}
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

            {/* Timeline Section */}
            <section className={styles.encountersSection}>
                <h2>Timeline</h2>
                <div className={styles.timelineControls}>
                    <button onClick={addEncounter}><FontAwesomeIcon icon={faPlus} /> Add Combat</button>
                    <button onClick={addShortRest} className={styles.restBtn}><FontAwesomeIcon icon={faBed} /> Add Short Rest</button>
                </div>
                
                <div className={styles.encounterList}>
                    {editedTimeline.map((item, index) => (
                        <div key={item.id} className={item.type === 'combat' ? styles.encounterItem : styles.restItem}>
                            <div className={styles.itemHeader}>
                                <h3>{item.type === 'combat' ? `Encounter ${index + 1}` : 'Short Rest'}</h3>
                                <div className={styles.itemControls}>
                                    <button onClick={() => swapTimelineItems(index, index - 1)} disabled={index === 0}>↑</button>
                                    <button onClick={() => swapTimelineItems(index, index + 1)} disabled={index === editedTimeline.length - 1}>↓</button>
                                    <button onClick={() => removeTimelineItem(index)} className={styles.deleteBtn}><FontAwesomeIcon icon={faTrash} /></button>
                                </div>
                            </div>

                            {item.type === 'combat' ? (
                                <>
                                    <div className={styles.encounterSettings}>
                                        <Checkbox value={item.playersSurprised || false} onToggle={() => updateTimelineItem(index, { ...item, playersSurprised: !item.playersSurprised })}>Players Surprised</Checkbox>
                                        <Checkbox value={item.monstersSurprised || false} onToggle={() => updateTimelineItem(index, { ...item, monstersSurprised: !item.monstersSurprised })}>Monsters Surprised</Checkbox>
                                        <Checkbox value={item.playersPrecast || false} onToggle={() => updateTimelineItem(index, { ...item, playersPrecast: !item.playersPrecast })}>Players Precast</Checkbox>
                                        <Checkbox value={item.monstersPrecast || false} onToggle={() => updateTimelineItem(index, { ...item, monstersPrecast: !item.monstersPrecast })}>Monsters Precast</Checkbox>
                                    </div>
                                    <h4>Monsters</h4>
                                    <button onClick={() => addMonsterToEncounter(index)}><FontAwesomeIcon icon={faPlus} /> Add Monster</button>
                                    <div className={styles.monsterList}>
                                        {item.monsters.map(monster => (
                                            <div key={monster.id} className={styles.monsterItem}>
                                                <span>{monster.name} (x{monster.count})</span>
                                                <button onClick={() => { setEditingMonster(monster); setEditingMonsterEncounterIndex(index); }}><FontAwesomeIcon icon={faPen} /> Edit</button>
                                                <button onClick={() => removeMonsterFromEncounter(index, monster.id)}><FontAwesomeIcon icon={faTimes} /> Remove</button>
                                            </div>
                                        ))}
                                    </div>
                                </>
                            ) : (
                                <div className={styles.restBody}>
                                    <p><FontAwesomeIcon icon={faBed} size="2x" /></p>
                                    <p>Standard 1-hour rest. Characters spend Hit Dice to recover HP and reset "Short Rest" resources.</p>
                                </div>
                            )}
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