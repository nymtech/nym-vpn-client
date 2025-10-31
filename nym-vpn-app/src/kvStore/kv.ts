import { invoke } from '@tauri-apps/api/core';
import { DbKey } from '../types';

/**
 * Get a key
 *
 * @param k - Key
 * @returns The value for that key if any
 */
export async function kvGet<V>(k: DbKey): Promise<V | null> {
  try {
    return await invoke<V>('db_get', { key: k });
  } catch {
    return null;
  }
}

/**
 * Insert a key to a new value
 *
 * @param k - Key
 * @param v - Value
 * @returns The last value if it was set
 */
export async function kvSet<V>(k: DbKey, v: V): Promise<V | null> {
  try {
    return await invoke<V>('db_set', { key: k, value: v });
  } catch {
    return null;
  }
}

/**
 * Remove a key
 *
 * @param k - Key
 * @returns The previous value if any
 */
export async function kvDel<V>(k: DbKey): Promise<V | null> {
  try {
    return await invoke<V>('db_del', { key: k });
  } catch {
    return null;
  }
}

/**
 * Flushes all dirty IO buffers and calls fsync.
 * If this succeeds, it is guaranteed that all previous
 * writes will be recovered if the system crashes
 *
 * @returns The number of bytes flushed during this call
 */
export async function kvFlush(): Promise<number | null> {
  try {
    return await invoke<number>('db_flush');
  } catch {
    return null;
  }
}
