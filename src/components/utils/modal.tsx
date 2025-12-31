import { FC, ReactNode, useEffect } from "react"
import styles from './modal.module.scss'
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faClose } from "@fortawesome/free-solid-svg-icons"

type PropType = {
    className?: string,
    onCancel: () => void,
    children: ReactNode,
    title?: string,
}

const Modal:FC<PropType> = ({ onCancel, children, className, title }) => {
    useEffect(() => {
        const handleEsc = (event: KeyboardEvent) => {
            if (event.key === 'Escape') {
                onCancel();
            }
        };
        window.addEventListener('keydown', handleEsc);
        return () => {
            window.removeEventListener('keydown', handleEsc);
        };
    }, [onCancel]);

    return (
        <div className={styles.overlay} role="presentation">
            <button 
                className={styles.closeBtn} 
                onClick={onCancel}
                aria-label="Close modal"
            >
                <FontAwesomeIcon icon={faClose} />
            </button>
            
            <div 
                className={`${styles.modal} ${className}`}
                role="dialog"
                aria-modal="true"
                aria-label={title || "Modal Dialog"}
            >
                {children}
            </div>
        </div>
    )
}

export default Modal