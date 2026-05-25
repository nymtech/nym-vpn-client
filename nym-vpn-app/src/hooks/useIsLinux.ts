import { type as osType } from '@tauri-apps/plugin-os';

// Cached: OS doesn't change during runtime.
const isLinuxCached = osType() === 'linux';

export function useIsLinux(): boolean {
  return isLinuxCached;
}

export default useIsLinux;
