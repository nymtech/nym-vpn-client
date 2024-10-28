import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useEffect, useState } from 'react';
import { UiTheme } from '../types';

export function useSystemTheme() {
  const [theme, setTheme] = useState<UiTheme>('Light');

  useEffect(() => {
    async function getTheme() {
      const window = getCurrentWebviewWindow();
      const winTheme = await window.theme();
      setTheme(winTheme === 'dark' ? 'Dark' : 'Light');
    }
    getTheme().catch((e: unknown) =>
      console.warn('Failed to get system theme', e),
    );
  }, []);

  return { theme };
}
