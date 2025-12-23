import { Event } from "./model";

export interface FileLoggerOptions {
    includeRaw?: boolean;
}

export class FileLogger {
    constructor(
        private names: Record<string, string>,
        private options: FileLoggerOptions = {}
    ) {}

    formatLog(events: Event[]): string {
        return "";
    }
}
