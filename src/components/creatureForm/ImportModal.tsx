import { FC, useState } from "react";
import Modal from "../utils/modal";
import styles from "./importModal.module.scss";
import { Monster5eImportSchema } from "@/model/import/5etools-schema";
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
        setError(null);
        try {
            const cleaned = cleanJsonInput(jsonText);
            if (!cleaned) {
                setError("No valid JSON found in input. Please ensure you pasted the monster's JSON.");
                return;
            }

            let rawData;
            try {
                rawData = JSON.parse(cleaned);
            } catch (e) {
                setError("Failed to parse JSON syntax. Please check for missing braces or extra characters.");
                return;
            }

            const validation = Monster5eImportSchema.safeParse(rawData);
            if (!validation.success) {
                const issues = validation.error.issues.map(i => `${i.path.join('.')} (${i.message})`).join(', ');
                setError(`Invalid format: ${issues}`);
                return;
            }

            const creature = mapMonster5eToCreature(validation.data);
            onImport(creature);
        } catch (e) {
            setError("An unexpected error occurred during import.");
            console.error("Import error:", e);
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
