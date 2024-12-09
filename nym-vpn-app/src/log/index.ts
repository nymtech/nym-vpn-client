import { invoke } from '@tauri-apps/api/core';

// Log level used by Rust
type Level = 'trace' | 'debug' | 'info' | 'warn' | 'error';

// Js console API
type ConsoleFn = 'log' | 'debug' | 'info' | 'warn' | 'error';

// Map from Js console API to Rust log level
const ConsoleMap: Record<ConsoleFn, Level> = {
  log: 'trace',
  debug: 'debug',
  info: 'info',
  warn: 'warn',
  error: 'error',
};

async function logToRust(level: Level, message: string) {
  try {
    await invoke<void>('log_js', { level, message });
  } catch (e) {
    console.error('invoke log_js failed:', e);
  }
}

function forwardConsole(fnName: ConsoleFn, level: Level) {
  const original = console[fnName];
  console[fnName] = (message?: unknown, ...rest: unknown[]) => {
    original(message, ...rest);
    const messageStr =
      typeof message === 'string' ? message : JSON.stringify(message);
    const restStr = rest
      .map((r) => (typeof r === 'string' ? r : JSON.stringify(r)))
      .join(' ');
    if (restStr.length > 0) {
      logToRust(level, `${messageStr} ${restStr}`);
    } else {
      logToRust(level, messageStr);
    }
  };
}

export function init() {
  Object.keys(ConsoleMap).forEach((fnName) => {
    forwardConsole(fnName as ConsoleFn, ConsoleMap[fnName as ConsoleFn]);
  });
}
