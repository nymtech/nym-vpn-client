import { invoke } from '@tauri-apps/api';

type Level = 'Trace' | 'Debug' | 'Info' | 'Warn' | 'Error';

async function log_js(level: Level, message: string) {
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
  trace: (msg: string) => log_js('Trace', msg),
  /**
   * Log a `debug` message
   *
   * @param msg - The message to log
   */
  debug: (msg: string) => log_js('Debug', msg),
  /**
   * Log an `info` message
   *
   * @param msg - The message to log
   */
  info: (msg: string) => log_js('Info', msg),
  /**
   * Log a `warn` message
   *
   * @param msg - The message to log
   */
  warn: (msg: string) => log_js('Warn', msg),
  /**
   * Log an `error` message
   *
   * @param msg - The message to log
   */
  error: (msg: string) => log_js('Error', msg),
};

export default logu;
