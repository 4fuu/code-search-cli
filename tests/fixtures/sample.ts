export const VERSION = "1.0.0";

export type Result<T> = { ok: true; value: T } | { ok: false; error: Error };

export interface Logger {
    log(message: string): void;
    error(message: string): void;
    level: string;
}

export enum LogLevel {
    Debug = "debug",
    Info = "info",
    Warn = "warn",
    Error = "error",
}

export class EventEmitter {
    private listeners: Map<string, Function[]> = new Map();

    on(event: string, callback: Function): void {
        const list = this.listeners.get(event) || [];
        list.push(callback);
        this.listeners.set(event, list);
    }

    emit(event: string, ...args: any[]): void {
        const list = this.listeners.get(event) || [];
        for (const cb of list) {
            cb(...args);
        }
    }
}

export function createLogger(name: string): Logger {
    return {
        log: (msg: string) => console.log(`[${name}] ${msg}`),
        error: (msg: string) => console.error(`[${name}] ${msg}`),
        level: "info",
    };
}

function internalHelper(): void {
    // not exported
}

export class HttpClient extends EventEmitter {
    private baseUrl: string;

    constructor(baseUrl: string) {
        super();
        this.baseUrl = baseUrl;
    }

    async get(path: string): Promise<Response> {
        return fetch(`${this.baseUrl}${path}`);
    }

    async post(path: string, body: any): Promise<Response> {
        return fetch(`${this.baseUrl}${path}`, {
            method: "POST",
            body: JSON.stringify(body),
        });
    }
}
