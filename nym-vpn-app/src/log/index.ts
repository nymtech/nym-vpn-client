import { invoke } from '@tauri-apps/api/core';

export type Level = 'Trace' | 'Debug' | 'Info' | 'Warn' | 'Error';

/**
 * Rust logger
 */
export type Logu = {
  /**
   * Log a `trace` message
   *
   * @param msg - The message to log
   */
  readonly trace: (msg: string) => void;
  /**
   * Log a `debug` message
   *
   * @param msg - The message to log
   */
  readonly debug: (msg: string) => void;
  /**
   * Log an `info` message
   *
   * @param msg - The message to log
   */
  readonly info: (msg: string) => void;
  /**
   * Log a `warn` message
   *
   * @param msg - The message to log
   */
  readonly warn: (msg: string) => void;
  /**
   * Log an `error` message
   *
   * @param msg - The message to log
   */
  readonly error: (msg: string) => void;
};

async function logJs(level: Level, message: string) {
  try {
    await invoke<void>('log_js', { level, message });
  } catch (e) {
    console.error('invoke log_js failed:', e);
  }
}

/**
 * Rust logger
 */
const logu: Logu = Object.freeze({
  trace: (msg: string) => logJs('Trace', msg),
  debug: (msg: string) => logJs('Debug', msg),
  info: (msg: string) => logJs('Info', msg),
  warn: (msg: string) => logJs('Warn', msg),
  error: (msg: string) => logJs('Error', msg),
});

type ConsoleFn = 'log' | 'debug' | 'info' | 'warn' | 'error';

const ConsoleLoggerMap: Record<ConsoleFn, keyof Logu> = {
  log: 'trace',
  debug: 'debug',
  info: 'info',
  warn: 'warn',
  error: 'error',
};

function forwardConsole(fnName: ConsoleFn, loguFn: keyof Logu) {
  const original = console[fnName];
  console[fnName] = (message?: unknown, ...rest: unknown[]) => {
    original(message, ...rest);
    const messageStr =
      typeof message === 'string' ? message : JSON.stringify(message);
    const restStr = rest
      .map((r) => (typeof r === 'string' ? r : JSON.stringify(r)))
      .join(' ');
    if (restStr.length > 0) {
      logu[loguFn](`${messageStr} ${restStr}`);
    } else {
      logu[loguFn](messageStr);
    }
  };
}

export function init() {
  Object.keys(ConsoleLoggerMap).forEach((fnName) => {
    forwardConsole(fnName as ConsoleFn, ConsoleLoggerMap[fnName as ConsoleFn]);
  });
}

export default logu;
