import { FC, useState } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faFileImport } from "@fortawesome/free-solid-svg-icons";
import { Creature } from "@/model/model";
import ImportModal from "./ImportModal";
import styles from "./creatureForm.module.scss"; // Reuse some styles

type PropType = {
    onImport: (creature: Creature) => void;
    className?: string;
}

const ImportButton: FC<PropType> = ({ onImport, className }) => {
    const [showImport, setShowImport] = useState(false);

    function handleImport(creature: Creature) {
        onImport(creature);
        setShowImport(false);
    }

    return (
        <>
            <button
                className={className}
                onClick={() => setShowImport(true)}
                title="Import from 5e.tools"
            >
                <FontAwesomeIcon icon={faFileImport} />
            </button>

            {showImport && (
                <ImportModal 
                    onImport={handleImport} 
                    onCancel={() => setShowImport(false)} 
                />
            )}
        </>
    );
};

export default ImportButton;
