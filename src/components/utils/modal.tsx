import { FC, ReactNode } from "react"
import * as Dialog from '@radix-ui/react-dialog'
import styles from './modal.module.scss'
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faClose } from "@fortawesome/free-solid-svg-icons"

type PropType = {
    className?: string,
    onCancel: () => void,
    children: ReactNode,
    title?: string,
}

/**
 * Modal component using Radix Dialog primitives (Library Discipline compliance)
 * Provides accessible, unstyled dialog primitives that we style with Grimoire aesthetic
 */
const Modal:FC<PropType> = ({ onCancel, children, className, title }) => {
    return (
        <Dialog.Root open onOpenChange={(open) => !open && onCancel()}>
            <Dialog.Portal>
                <Dialog.Overlay asChild>
                    <div className={styles.overlay} />
                </Dialog.Overlay>
                <Dialog.Content asChild>
                    <div
                        className={`${styles.modal} ${className}`}
                        role="dialog"
                        aria-modal="true"
                        aria-label={title || "Modal Dialog"}
                    >
                        {/* Close button using Radix's close functionality */}
                        <Dialog.Close asChild>
                            <button
                                className={styles.closeBtn}
                                aria-label="Close modal"
                            >
                                <FontAwesomeIcon icon={faClose} />
                            </button>
                        </Dialog.Close>

                        {children}
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    )
}

export default Modal
