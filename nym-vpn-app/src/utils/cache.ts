import { invoke } from '@tauri-apps/api/core';
import { CCache } from '../cache';

const IdsTimeToLive = 120; // sec

export const getAccountId = async () => {
  const accountId = await CCache.get<string>('cache-account-id');
  if (accountId) {
    return accountId;
  }
  try {
    const accountId = await invoke<string>('get_account_id');
    CCache.set('cache-account-id', accountId, IdsTimeToLive);
    return accountId;
  } catch {
    return null;
  }
};

export const getDeviceId = async () => {
  const deviceId = await CCache.get<string>('cache-device-id');
  if (deviceId) {
    return deviceId;
  }
  try {
    const deviceId = await invoke<string>('get_device_id');
    CCache.set('cache-device-id', deviceId, IdsTimeToLive);
    return deviceId;
  } catch {
    return null;
  }
};
