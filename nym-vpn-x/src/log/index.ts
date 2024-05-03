import { invoke } from '@tauri-apps/api';

export type Level = 'Trace' | 'Debug' | 'Info' | 'Warn' | 'Error';

/**
 * Rust logger
 */
export interface Logu {
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
}

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

export default logu;
