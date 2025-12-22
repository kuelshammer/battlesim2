import { FC, useState } from "react";
import Modal from "../utils/modal";
import styles from "./importModal.module.scss";
import { Monster5eSchema } from "@/model/import/5etools-schema";
import { mapMonster5eToCreature } from "@/model/import/5etools-mapper";
import { Creature } from "@/model/model";
import { cleanJsonInput } from "@/model/import/utils";

type PropType = {
    onImport: (creature: Creature) => void;
    onCancel: () => void;
}

const ImportModal: FC<PropType> = ({ onImport, onCancel }) => {
    const [jsonText, setJsonText] = useState("");
    const [error, setError] = useState<string | null>(null);

    const handleImport = () => {
        try {
            const cleaned = cleanJsonInput(jsonText);
            if (!cleaned) {
                setError("No valid JSON found in input. Please ensure you pasted the monster's JSON.");
                return;
            }

            const rawData = JSON.parse(cleaned);
            const validation = Monster5eSchema.safeParse(rawData);
            
            if (!validation.success) {
                setError("Invalid 5e.tools monster format. Please check your JSON.");
                console.error(validation.error);
                return;
            }

            const creature = mapMonster5eToCreature(validation.data);
            onImport(creature);
        } catch (e) {
            setError("Failed to parse JSON. Please ensure you pasted valid JSON text.");
        }
    };

    return (
        <Modal onCancel={onCancel} className={styles.importModal}>
            <h2>Import from 5e.tools</h2>
            <p>Paste the monster's "View JSON" content from 5e.tools below:</p>
            
            <textarea 
                className={styles.jsonInput}
                value={jsonText}
                onChange={(e) => {
                    setJsonText(e.target.value);
                    setError(null);
                }}
                placeholder='{ "name": "Beholder", ... }'
            />

            {error && <div className={styles.error}>{error}</div>}

            <div className={styles.actions}>
                <button className={styles.cancelBtn} onClick={onCancel}>Cancel</button>
                <button 
                    className={styles.importBtn} 
                    onClick={handleImport}
                    disabled={!jsonText.trim()}
                >
                    Import Creature
                </button>
            </div>
        </Modal>
    );
};

export default ImportModal;
