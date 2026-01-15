import { FC, useEffect, useState } from "react"
import { Creature, CreatureSchema } from "@/model/model"
import styles from './monsterForm.module.scss'
import { ChallengeRating, ChallengeRatingList, CreatureType, CreatureTypeList, numericCR } from "@/model/enums"
import { capitalize, clone, sharedStateGenerator, useCalculatedState } from "@/model/utils"
import Range from "@/utils/range"
import SortTable from "@/utils/sortTable"

type PropType = {
    value: Creature,
    onChange: (newvalue: Creature) => void,
}

const defaultTypeFilter: {[type in CreatureType]: boolean} = Object.fromEntries(CreatureTypeList.map(t => [t, true])) as {[type in CreatureType]: boolean}

const MonsterForm:FC<PropType> = ({ onChange, value }) => {
    const useSharedState = sharedStateGenerator('monsterForm')
    const [creatureType, setCreatureType] = useSharedState(defaultTypeFilter)
    const [minCR, setMinCR] = useSharedState<ChallengeRating>(ChallengeRatingList[0])
    const [maxCR, setMaxCR] = useSharedState<ChallengeRating>(ChallengeRatingList[ChallengeRatingList.length - 2])
    const [search, setSearch] = useSharedState<string>('')
    const [monsters, setMonsters] = useState<Creature[]>([])

    useEffect(() => {
        const loadMonsters = async () => {
            try {
                const response = await fetch('./data/monsters.json');
                const data = await response.json();
                setMonsters(data);
            } catch (error) {
                console.error("Failed to load monsters:", error);
            }
        };
        loadMonsters();
    }, []);

    const searchResults = useCalculatedState(() => monsters.filter(monster => {
        if (!monster.name.toLocaleLowerCase().includes(search.toLocaleLowerCase())) return false
        if (!monster.cr) return false
        if (numericCR(monster.cr) > numericCR(maxCR)) return false
        if (numericCR(monster.cr) < numericCR(minCR)) return false
        if (!monster.type) return false
        if (!creatureType[monster.type]) return false

        return true
    }), [creatureType, minCR, maxCR, search, monsters])
    
    function toggleCreatureType(type: CreatureType) {
        const newValue = clone(creatureType)
        newValue[type] = !newValue[type]
        setCreatureType(newValue)
    }

    function selectMonster(monster: Creature) {
        const templates = JSON.parse(localStorage.getItem('monsterTemplates') || "{}")
        const monsterTemplate = CreatureSchema.safeParse(templates[monster.id])

        const creature = monsterTemplate.success ? monsterTemplate.data : clone(monster)
        
        // Preserve parent's ID
        creature.id = value?.id || creature.id;
        
        // Keep parent's count if already set
        creature.count = value?.count || 1;
        
        // IMPORTANT: In monster search, selecting a monster SHOULD overwrite the Name/HP/AC
        // unless they were dirty? No, usually you want the monster stats.
        // We'll let onChange propagate the new monster data.
        
        onChange(creature)
    }

    return (
        <div className={styles.monsterForm}>
            <section>
                <h3>Search Monster</h3>
                <input 
                    type='text' 
                    value={search} 
                    onChange={e => setSearch(e.target.value)} 
                    placeholder='Bandit...'  
                    autoFocus={true} 
                    data-testid="monster-search" 
                />
            </section>

            <section>
                <h3>Creature Type</h3>
                <div className={styles.creatureTypes} data-testid="creature-type-selector">
                    <button 
                        onClick={() => setCreatureType(defaultTypeFilter)} 
                        className={!Object.entries(creatureType).find(([, b]) => !b) ? styles.active : undefined}
                        data-testid="type-all"
                    >
                            All
                    </button>
                    <button
                        onClick={() => setCreatureType(Object.fromEntries(Object.keys(creatureType).map(t => [t, false])) as {[type in CreatureType]: boolean})}
                        className={!Object.entries(creatureType).find(([, b]) => b) ? styles.active : undefined}
                        data-testid="type-none"
                    >
                            None
                    </button>
                    { CreatureTypeList.map(type => (
                        <button
                            key={type}
                            onClick={() => toggleCreatureType(type)}
                            className={creatureType[type] ? styles.active : undefined}
                            data-testid={`type-${type}`}
                        >
                                {capitalize(type)}
                        </button>
                    )) }
                </div>
            </section>
            
            <section className={styles.challengeRating} data-testid="cr-selector">
                <h3>Challenge Rating</h3>
                <Range
                    values={[ChallengeRatingList.indexOf(minCR), ChallengeRatingList.indexOf(maxCR)]}
                    min={0}
                    max={ChallengeRatingList.length - 2}
                    onChange={async (values: number[]) => { await setMaxCR(ChallengeRatingList[values[1]]); await setMinCR(ChallengeRatingList[values[0]]) }}
                    label={minCR}
                    upperLabel={maxCR}
                />
            </section>

            <SortTable
                value={value}
                list={searchResults}
                comparators={{
                    Name: (a: Creature, b: Creature) => a.name.localeCompare(b.name),
                    CR: (a: Creature, b: Creature) => (numericCR(a.cr!) - numericCR(b.cr!)),
                }}
                onChange={selectMonster}
                data-testid="monster-select">
                    { monster => (
                        <div className={styles.monster}>
                            <span className={styles.name}>{monster.name}</span>
                            <span className={styles.stats}>{monster.type}, {monster.src}</span>
                            <span className={styles.stats}>CR {monster.cr}</span>
                            {monster.speed_fly && <span className={styles.stats}>Speed Fly {monster.speed_fly}</span>}
                        </div>
                    )}
            </SortTable>
        </div>
    )
}

export default MonsterForm