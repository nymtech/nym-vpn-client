import { kvDel, kvFlush, kvGet, kvSet } from '../kvStore';
import { DbKey } from '../types';

export type CCached<T> = {
  value: T;
  // timestamp in ms
  expiry?: number;
};

export type CKey = Extract<
  DbKey,
  | 'cache-mx-entry-gateways'
  | 'cache-mx-exit-gateways'
  | 'cache-wg-gateways'
  | 'cache-account-id'
  | 'cache-device-id'
>;

/**
 * Cache on-db, with optional expiry.
 * Simple wrapper around the kvStore that attach
 * a 'time-to-live' to the stored values.
 */
export const CCache = {
  /**
   * Get a key
   *
   * @param key - Key
   * @param stale - Accept stale (expired) data
   * @returns The cached value if any
   */
  get: async <T>(key: CKey, stale = false): Promise<T | null> => {
    const cached = await kvGet<CCached<T>>(key);
    if (!cached) {
      console.log(`no cache data for [${key}]`);
      return null;
    }
    if (!cached.expiry) {
      console.log(`cache data available [${key}]`);
      return cached.value;
    }
    if (Date.now() < cached.expiry) {
      console.log(`cache data available [${key}]`);
      return cached.value;
    }
    console.log(`cache data is stale [${key}]`);
    if (stale) {
      console.log(`cache data available [${key}]`);
      await kvDel(key);
      return cached.value;
    }
    await kvDel(key);
    return null;
  },
  /**
   * Set a key
   *
   * @param key - Key
   * @param value - The date to cache
   * @param ttl - The time to live from now in seconds
   */
  set: async <T>(key: CKey, value: T, ttl?: number): Promise<void> => {
    if (!ttl) {
      console.log(`set cache [${key}]`);
      await kvSet(key, { value: value });
      return;
    }
    const expiry = Date.now() + ttl * 1000;
    console.log(`set cache [${key}] (${ttl}s)`);
    await kvSet(key, { value: value, expiry });
  },
  /**
   * Remove a key
   *
   * @param key - Key
   */
  del: async <T>(key: CKey): Promise<void> => {
    console.log(`delete cache [${key}]`);
    await kvDel<CCached<T>>(key);
  },
  /**
   * Clear all cache
   */
  clear: async (): Promise<void> => {
    console.log(`clear cache`);
    await kvDel('cache-mx-entry-gateways');
    await kvDel('cache-mx-exit-gateways');
    await kvDel('cache-wg-gateways');
    await kvDel('cache-account-id');
    await kvDel('cache-device-id');
    await kvFlush();
  },
} as const;
