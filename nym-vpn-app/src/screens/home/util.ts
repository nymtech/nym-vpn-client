import { allCountries } from 'country-region-data';
import { TunnelData, isWireguardData } from '../../types';

// find country code (lowercase) for a given region name
export function regionToCountryCode(region: string): string | null {
  for (const data of allCountries) {
    const res = data[2].some(
      (r) => r[0].toLowerCase() === region.toLowerCase(),
    );
    if (res) {
      return data[1].toLowerCase();
    }
  }
  console.warn(`country not found for region [${region}]`);
  return null;
}

export function isBridgeMode(data?: TunnelData | null) {
  if (!data) {
    return false;
  }
  return isWireguardData(data) && data.entryBridgeAddr;
}
