import { z } from 'zod';

export const Monster5eSchema = z.object({
    name: z.string(),
    src: z.string().optional(),
    hp: z.object({
        average: z.number().optional(),
        formula: z.string().optional(),
    }).passthrough(),
    ac: z.array(z.union([z.number(), z.object({ ac: z.number() }).passthrough()])),
    str: z.number().optional(),
    dex: z.number().optional(),
    con: z.number().optional(),
    int: z.number().optional(),
    wis: z.number().optional(),
    cha: z.number().optional(),
    save: z.record(z.string()).optional(),
    action: z.array(z.object({
        name: z.string(),
        entries: z.array(z.any()).optional(),
    }).passthrough()).optional(),
}).passthrough();

export type Monster5e = z.infer<typeof Monster5eSchema>;