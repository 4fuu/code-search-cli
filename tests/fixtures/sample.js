export class EventEmitter {
  on(event, callback) {
    return callback(event);
  }
}

export function createLogger() {
  return new EventEmitter();
}

const internalHelper = (value) => value.trim();
export const VERSION = "1.0";
let currentLevel = VERSION;

new EventEmitter().on("ready", internalHelper);
