import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useEffect, useState } from 'react';
import { UiTheme } from '../types';

const appWindow = getCurrentWebviewWindow();

export function useSystemTheme() {
  const [theme, setTheme] = useState<UiTheme>('Light');

  useEffect(() => {
    async function getTheme() {
      const winTheme = await appWindow.theme();
      setTheme(winTheme === 'dark' ? 'Dark' : 'Light');
    }
    getTheme().catch((e: unknown) =>
      console.warn('Failed to get system theme', e),
    );
  }, []);

  return { theme };
}
