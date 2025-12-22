// Removed Zod due to persistent crashes with version 4.1.13 in this environment
export const Monster5eSchema = {
    safeParse: (data: any) => {
        // Simple manual validation as a temporary measure
        if (data && typeof data === 'object' && data.name) {
            return { success: true, data };
        }
        return { success: false, error: new Error("Invalid monster data") };
    },
    parse: (data: any) => {
        if (data && typeof data === 'object' && data.name) {
            return data;
        }
        throw new Error("Invalid monster data");
    }
};

export type Monster5e = any;