import { appWindow } from '@tauri-apps/api/window';
import { useEffect, useState } from 'react';
import { UiTheme } from '../types';

export function useSystemTheme() {
  const [theme, setTheme] = useState<UiTheme>('Light');

  useEffect(() => {
    async function getTheme() {
      const winTheme = await appWindow.theme();
      setTheme(winTheme === 'dark' ? 'Dark' : 'Light');
    }
    getTheme().catch((e) => console.warn('Failed to get system theme', e));
  }, []);

  return { theme };
}
