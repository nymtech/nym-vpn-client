import { kvDel, kvFlush, kvGet, kvSet } from '../kvStore';
import { DbKey } from '../types';

export type CCached<T> = {
  value: T;
  // timestamp in ms
  expiry?: number;
};

export type CKey = Extract<
  DbKey,
  | 'mx-entry-gateways'
  | 'mx-exit-gateways'
  | 'wg-gateways'
  | 'account-id'
  | 'device-id'
>;

/**
 * Cache on-db, with optional expiry
 * Just a simple wrapper around the kvStore that adds time-to-live
 * to the values
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
      console.log(`cache data [${key}]`, cached.value);
      return cached.value;
    }
    if (Date.now() < cached.expiry) {
      console.log(`cache data [${key}]`, cached.value);
      return cached.value;
    }
    console.log(`cache data is stale [${key}]`);
    if (stale) {
      console.log(`cache data [${key}]`, cached.value);
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
      console.log(`set cache [${key}]`, value);
      await kvSet(key, { value: value });
      return;
    }
    const expiry = Date.now() + ttl * 1000;
    console.log(
      `set cache [${key}], expiry ${new Date(expiry).toString()}`,
      value,
    );
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
    await kvDel('mx-entry-gateways');
    await kvDel('mx-exit-gateways');
    await kvDel('wg-gateways');
    await kvDel('account-id');
    await kvDel('device-id');
    await kvFlush();
  },
} as const;
