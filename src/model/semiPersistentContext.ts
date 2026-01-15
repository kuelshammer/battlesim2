import { createContext } from "react";

export const semiPersistentContext = createContext({
    state: new Map<string, unknown>(),
    setState: (newValue: Map<string, unknown>) => {}, // eslint-disable-line @typescript-eslint/no-unused-vars
})