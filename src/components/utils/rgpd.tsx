import { FC, useEffect } from "react";

type PropType = object

const RGPD: FC<PropType> = () => {
    useEffect(() => {
        if (typeof window !== 'undefined' && localStorage) {
            localStorage.setItem('useLocalStorage', 'true');
        }
    }, []);

    return null;
}

export default RGPD