import type { NetworkCompat } from '../../types';

export function showsNetworkUpdateDialog(
  compat: NetworkCompat | null | undefined,
  devMode: boolean,
): boolean {
  if (devMode || !compat) {
    return false;
  }
  return compat.core === false || compat.tauri === false;
}

export function blocksNewConnect(
  compat: NetworkCompat | null | undefined,
  devMode: boolean,
): boolean {
  return (
    showsNetworkUpdateDialog(compat, devMode) &&
    compat?.appUpdatePolicy === 'required'
  );
}
