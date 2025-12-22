import { z } from 'zod';

export const Monster5eSchema = z.object({
    name: z.string(),
    src: z.string().optional(),
    source: z.string().optional(),
    hp: z.object({
        average: z.number().optional(),
        formula: z.string().optional(),
    }).passthrough(),
    ac: z.array(z.union([
        z.number(), 
        z.object({ ac: z.number() }).passthrough()
    ])),
    str: z.number().optional(),
    dex: z.number().optional(),
    con: z.number().optional(),
    int: z.number().optional(),
    wis: z.number().optional(),
    cha: z.number().optional(),
    type: z.union([
        z.string(),
        z.object({ type: z.string() }).passthrough()
    ]).optional(),
    save: z.record(z.string()).optional(),
    action: z.array(z.object({
        name: z.string(),
        entries: z.array(z.any()).optional(),
    }).passthrough()).optional(),
}).passthrough();

export const Monster5eImportSchema = Monster5eSchema.deepPartial();
export type Monster5eImport = z.infer<typeof Monster5eImportSchema>;

export type Monster5e = z.infer<typeof Monster5eSchema>;
