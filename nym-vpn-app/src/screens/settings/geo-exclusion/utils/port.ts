// Valid geo-exclusion SOCKS5 listen port range (matches the design: 1024–65535).
export const GeoExclusionPortMin = 1024;
export const GeoExclusionPortMax = 65535;

export function isValidGeoExclusionPort(value: string): boolean {
  if (!/^\d+$/.test(value)) {
    return false;
  }
  const port = Number(value);
  return port >= GeoExclusionPortMin && port <= GeoExclusionPortMax;
}
