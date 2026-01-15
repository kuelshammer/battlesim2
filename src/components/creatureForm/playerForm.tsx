import { FC, useEffect } from "react"
import { Creature } from "@/model/model"
import styles from './playerForm.module.scss'
import { Class, ClassesList } from "@/model/enums"
import { capitalize, clone, range } from "@/model/utils"
import { PlayerTemplates } from "@/data/data"
import ClassOptions from "@/model/classOptions"
import { z } from "zod"
import Checkbox from "@/utils/checkbox"
import Range from "@/utils/range"

type PropType = {
    value?: Creature,
    onChange: (newvalue: Creature) => void,
}

type ClassForm = { type: 'artificer', options: z.infer<typeof ClassOptions.artificer> }
    | { type: 'barbarian', options: z.infer<typeof ClassOptions.barbarian> }
    | { type: 'bard', options: z.infer<typeof ClassOptions.bard> }
    | { type: 'cleric', options: z.infer<typeof ClassOptions.cleric> }
    | { type: 'druid', options: z.infer<typeof ClassOptions.druid> }
    | { type: 'fighter', options: z.infer<typeof ClassOptions.fighter> }
    | { type: 'monk', options: z.infer<typeof ClassOptions.monk> }
    | { type: 'paladin', options: z.infer<typeof ClassOptions.paladin> }
    | { type: 'ranger', options: z.infer<typeof ClassOptions.ranger> }
    | { type: 'rogue', options: z.infer<typeof ClassOptions.rogue> }
    | { type: 'sorcerer', options: z.infer<typeof ClassOptions.sorcerer> }
    | { type: 'warlock', options: z.infer<typeof ClassOptions.warlock> }
    | { type: 'wizard', options: z.infer<typeof ClassOptions.wizard> }

type ClassOptionsMap = {
    [K in Class]: z.infer<typeof ClassOptions[K]>
}

const DefaultOptions: ClassOptionsMap = {
    artificer: {},
    barbarian: { gwm: false, weaponBonus: 0 },
    bard: {},
    cleric: {},
    druid: {},
    fighter: { gwm: false, weaponBonus: 0 },
    monk: {},
    paladin: { gwm: false, weaponBonus: 0 },
    ranger: { ss: false, weaponBonus: 0 },
    rogue: { ss: false, weaponBonus: 0 },
    sorcerer: {},
    warlock: {},
    wizard: {},
}

const DefaultClass: ClassForm = { type: 'barbarian', options: DefaultOptions.barbarian }
const DefaultLevel = 1

const PlayerForm:FC<PropType> = ({ value, onChange }) => {
    const chosenClass: ClassForm = (value && value.class) ? { type: value.class.type, options: value.class.options } as ClassForm : DefaultClass;
    const level = (value && value.class) ? value.class.level : DefaultLevel;

    // Apply template on first mount if no class exists
    useEffect(() => {
        if (value && !value.class) {
            applyTemplate(DefaultClass.type, DefaultLevel, DefaultOptions[DefaultClass.type]);
        }
    }, []);

    function applyTemplate(type: Class, lvl: number, options: ClassOptionsMap[Class]) {
        const template = PlayerTemplates[type];
        const creature = (template as (level: number, options: ClassOptionsMap[Class]) => Creature)(lvl, options);
        creature.id = value?.id || creature.id;
        creature.class = {
            type: type,
            level: lvl,
            options: options,
        };
        creature.count = value?.count || 1;
        creature.name = value?.name || creature.name;
        
        // If HP/AC were default 10, let template decide. Otherwise keep parent's.
        if (value && value.hp !== 10) creature.hp = value.hp;
        if (value && value.ac !== 10) creature.ac = value.ac;

        onChange(creature);
    }

    function setClass(type: Class) {
        applyTemplate(type, level, DefaultOptions[type]);
    }

    function setLevel(lvl: number) {
        applyTemplate(chosenClass.type, lvl, chosenClass.options);
    }

    function setClassOptions(callback: (classOptions: ClassForm['options']) => void) {
        const chosenClassClone = clone(chosenClass)
        callback(chosenClassClone.options)
        applyTemplate(chosenClassClone.type, level, chosenClassClone.options);
    }

    return (
        <div className={styles.playerForm} data-testid="player-form">
            <h3>Class</h3>
            <section className={styles.classes} data-testid="class-selector">
                { ClassesList.map(className => (
                    <button
                        key={className}
                        className={`${styles.class} ${(chosenClass?.type === className) ? styles.active : ''}`}
                        onClick={() => { setClass(className) }}
                        data-testid={`class-${className}`}
                    >
                        <img src={`./classes/${className}.jpeg`} />
                        {capitalize(className)}
                    </button>
                )) }
            </section>
            
            <h3>Level</h3>
            <section className={styles.levels} data-testid="level-selector">
                { range(20).map(i => i+1).map(lvl => (
                    <button
                        key={lvl}
                        className={`${styles.level} ${(level === lvl) ? styles.active : ''}`}
                        onClick={() => setLevel(lvl)}
                        data-testid={`level-${lvl}`}
                    >
                        {lvl}
                    </button>
                )) }
            </section>

            <h3>Hit Dice (e.g., "3d8+5d10")</h3>
            <section>
                <input
                    type="text"
                    value={value?.hitDice || ''}
                    onChange={(e) => {
                        onChange({ ...value!, hitDice: e.target.value || undefined });
                    }}
                    data-testid="hit-dice-input"
                />
            </section>

            <h3>Constitution Modifier</h3>
            <section>
                <input
                    type="number"
                    value={value?.conModifier || 0}
                    onChange={(e) => {
                        onChange({ ...value!, conModifier: parseFloat(e.target.value) || 0 });
                    }}
                    data-testid="con-modifier-input"
                />
            </section>

            { !chosenClass ? null : (
                (chosenClass.type === 'barbarian') ? (
                    <>
                        <h3>Barbarian-specific Options</h3>
                        <section className={styles.classOptions}>
                            <Checkbox 
                                value={chosenClass.options.gwm} 
                                onToggle={() => setClassOptions(options => { (options as z.infer<typeof ClassOptions.barbarian>).gwm = !(options as z.infer<typeof ClassOptions.barbarian>).gwm })}>
                                Use Great Weapon Master
                            </Checkbox>
                            <div className={styles.option}>
                                Weapon:
                                <Range
                                    min={0}
                                    max={3}
                                    value={chosenClass.options.weaponBonus}
                                    onChange={(newValue) => setClassOptions(options => { (options as z.infer<typeof ClassOptions.barbarian>).weaponBonus = newValue })}
                                    label={`+${chosenClass.options.weaponBonus}`}
                                />
                            </div>
                        </section>
                    </>
                ) : (chosenClass.type === 'bard') ? (
                    <></>
                ) : (chosenClass.type === 'cleric') ? (
                    <></>
                ) : (chosenClass.type === 'druid') ? (
                    <></>
                ) : (chosenClass.type === 'fighter') ? (
                    <>
                        <h3>Fighter-specific Options</h3>
                        <section className={styles.classOptions}>
                            <Checkbox 
                                value={chosenClass.options.gwm} 
                                onToggle={() => setClassOptions(options => { (options as z.infer<typeof ClassOptions.barbarian>).gwm = !(options as z.infer<typeof ClassOptions.barbarian>).gwm })}>
                                Use Great Weapon Master
                            </Checkbox>
                            <div className={styles.option}>
                                Weapon:
                                <Range
                                    min={0}
                                    max={3}
                                    value={chosenClass.options.weaponBonus}
                                    onChange={(newValue) => setClassOptions(options => { (options as z.infer<typeof ClassOptions.barbarian>).weaponBonus = newValue })}
                                    label={`+${chosenClass.options.weaponBonus}`}
                                />
                            </div>
                        </section>
                    </>
                ) : (chosenClass.type === 'monk') ? (
                    <></>
                ) : (chosenClass.type === 'paladin') ? (
                    <>
                        <h3>Paladin-specific Options</h3>
                        <section className={styles.classOptions}>
                            <Checkbox 
                                value={chosenClass.options.gwm} 
                                onToggle={() => setClassOptions(options => { (options as z.infer<typeof ClassOptions.barbarian>).gwm = !(options as z.infer<typeof ClassOptions.barbarian>).gwm })}>
                                Use Great Weapon Master
                            </Checkbox>
                            <div className={styles.option}>
                                Weapon:
                                <Range
                                    min={0}
                                    max={3}
                                    value={chosenClass.options.weaponBonus}
                                    onChange={(newValue) => setClassOptions(options => { (options as z.infer<typeof ClassOptions.barbarian>).weaponBonus = newValue })}
                                    label={`+${chosenClass.options.weaponBonus}`}
                                />
                            </div>
                        </section>
                    </>
                 ) : (chosenClass.type === 'ranger') ? (
                     <>
                         <h3>Ranger-specific Options</h3>
                         <section className={styles.classOptions}>
                             <Checkbox 
                                 value={chosenClass.options.ss} 
                                 onToggle={() => setClassOptions(options => { (options as z.infer<typeof ClassOptions.ranger>).ss = !(options as z.infer<typeof ClassOptions.ranger>).ss })}>
                                 Use Sharpshooter
                             </Checkbox>
                             <div className={styles.option}>
                                 Weapon:
                                 <Range
                                     min={0}
                                     max={3}
                                     value={chosenClass.options.weaponBonus}
                                     onChange={(newValue) => setClassOptions(options => { (options as z.infer<typeof ClassOptions.barbarian>).weaponBonus = newValue })}
                                     label={`+${chosenClass.options.weaponBonus}`}
                                 />
                             </div>
                         </section>
                     </>
                 ) : (chosenClass.type === 'rogue') ? (
                     <>
                         <h3>Rogue-specific Options</h3>
                         <section className={styles.classOptions}>
                             <Checkbox 
                                 value={chosenClass.options.ss} 
                                 onToggle={() => setClassOptions(options => { (options as z.infer<typeof ClassOptions.ranger>).ss = !(options as z.infer<typeof ClassOptions.ranger>).ss })}>
                                 Use Sharpshooter
                             </Checkbox>
                             <div className={styles.option}>
                                 Weapon:
                                 <Range
                                     min={0}
                                     max={3}
                                     value={chosenClass.options.weaponBonus}
                                     onChange={(newValue) => setClassOptions(options => { (options as z.infer<typeof ClassOptions.barbarian>).weaponBonus = newValue })}
                                     label={`+${chosenClass.options.weaponBonus}`}
                                 />
                             </div>
                         </section>
                     </>
                 ) : (chosenClass.type === 'sorcerer') ? (
                    <></>
                ) : (chosenClass.type === 'warlock') ? (
                    <></>
                ) : (chosenClass.type === 'wizard') ? (
                    <></>
                ) : null
            ) }
        </div>
    )
}

export default PlayerForm