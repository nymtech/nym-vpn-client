// simple in-memory cache

export type MCached<T> = {
  value: T;
  // timestamp in ms
  expiry?: number;
};

export type CKey =
  | 'mn-entry-countries'
  | 'mn-exit-countries'
  | 'wg-countries'
  | 'account-id'
  | 'device-id';

const cache = new Map<CKey, MCached<never>>();

/**
 * In-memory cache, with optional expiry
 */
export const MCache = {
  /**
   * Get a key
   *
   * @param key - Key
   * @param stale - Accept stale (expired) data
   * @returns The cached value if any
   */
  get: <T>(key: CKey, stale = false): T | null => {
    const cached = cache.get(key);
    if (!cached) {
      console.log(`no cache data for [${key}]`);
      return null;
    }
    if (!cached.expiry) {
      console.log(`cache data [${key}]`, cached.value);
      return cached.value as T;
    }
    if (Date.now() < cached.expiry) {
      console.log(`cache data [${key}]`, cached.value);
      return cached.value as T;
    }
    console.log(`cache data is stale [${key}]`);
    if (stale) {
      console.log(`cache data [${key}]`, cached.value);
      cache.delete(key);
      return cached.value as T;
    }
    cache.delete(key);
    return null;
  },
  /**
   * Set a key
   *
   * @param key - Key
   * @param value - The date to cache
   * @param ttl - The time to live from now in seconds
   */
  set: <T>(key: CKey, value: T, ttl?: number): void => {
    if (!ttl) {
      console.log(`set cache [${key}]`, value);
      cache.set(key, { value: value as never });
      return;
    }
    const expiry = Date.now() + ttl * 1000;
    console.log(
      `set cache [${key}], expiry ${new Date(expiry).toString()}`,
      value,
    );
    cache.set(key, { value: value as never, expiry });
  },
  /**
   * Remove a key
   *
   * @param key - Key
   */
  del: (key: CKey): void => {
    console.log(`delete cache [${key}]`);
    cache.delete(key);
  },
  /**
   * Clear all cache
   */
  clear: (): void => {
    console.log(`clear cache`);
    cache.clear();
  },
} as const;
