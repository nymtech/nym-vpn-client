import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { kvGet } from './kvStore';
import { ThemeMode } from './types';

export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Capitalize the first letter of a string
export function capFirst(string: string) {
  return string.charAt(0).toUpperCase() + string.slice(1);
}

export function formatGb(gb: number | bigint) {
  return `${Number(gb).toLocaleString()} GB`;
}

// Given a set of strings, return the strings concatenated by a white space
export function setToString(obj: Record<string, string>): string {
  return Object.values(obj).reduce((prev, s) => `${prev} ${s}`, '');
}

export async function getTheme(): Promise<'light' | 'dark'> {
  const mode = await kvGet<ThemeMode>('ui-theme');
  if (mode === 'light') {
    return 'light';
  }
  if (mode === 'dark') {
    return 'dark';
  }
  const window = getCurrentWebviewWindow();
  const theme = await window.theme();
  if (theme === 'dark') {
    return 'dark';
  }
  return 'light';
}
