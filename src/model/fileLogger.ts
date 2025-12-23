import { Event } from "./model";
import { LogFormatter } from "./logFormatter";

export interface FileLoggerOptions {
    includeRaw?: boolean;
}

export class FileLogger {
    constructor(
        private names: Record<string, string>,
        private options: FileLoggerOptions = {}
    ) {}

    formatLog(events: Event[]): string {
        return events.map(event => {
            const summary = LogFormatter.toSummary(event, this.names);
            const details = LogFormatter.toDetails(event, this.names);
            
            let entry = `${summary}\n`;
            if (details && details !== summary && details !== JSON.stringify(event, null, 2)) {
                entry += `${details}\n`;
            }
            
            if (this.options.includeRaw) {
                entry += `Raw: ${JSON.stringify(event)}\n`;
            }
            
            return entry;
        }).join('\n');
    }
}
