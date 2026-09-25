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

export function networkUpdateTitleKey(required: boolean): string {
  return required ? 'update-dialog.title' : 'update-dialog.title-available';
}

export function networkUpdateBodyKey(required: boolean): string {
  return required
    ? 'update-dialog.description-2'
    : 'update-dialog.description-available';
}

export function networkUpdateDownloadUrl(os: string, base: string): string | null {
  if (os === 'linux' || os === 'windows' || os === 'macos') {
    return `${base}/${os}`;
  }
  return null;
}
