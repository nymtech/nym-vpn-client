import { invoke } from '@tauri-apps/api';

type Level = 'Trace' | 'Debug' | 'Info' | 'Warn' | 'Error';

async function logJs(level: Level, message: string) {
  try {
    await invoke<void>('log_js', { level, message });
  } catch (e) {
    console.error(`invoke log_js failed: ${e}`);
  }
}

/**
 * Rust logger
 */
const logu = {
  /**
   * Log a `trace` message
   *
   * @param msg - The message to log
   */
  trace: (msg: string) => logJs('Trace', msg),
  /**
   * Log a `debug` message
   *
   * @param msg - The message to log
   */
  debug: (msg: string) => logJs('Debug', msg),
  /**
   * Log an `info` message
   *
   * @param msg - The message to log
   */
  info: (msg: string) => logJs('Info', msg),
  /**
   * Log a `warn` message
   *
   * @param msg - The message to log
   */
  warn: (msg: string) => logJs('Warn', msg),
  /**
   * Log an `error` message
   *
   * @param msg - The message to log
   */
  error: (msg: string) => logJs('Error', msg),
};

export default logu;
