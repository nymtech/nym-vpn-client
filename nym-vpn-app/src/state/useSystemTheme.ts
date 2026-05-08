import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ThemeMode, UiTheme } from '../types';
import { dispatch } from '../store';
import { ColorMainBgDark, ColorMainBgLight } from '../constants';
import { kvSet } from '../kvStore/index';

export function useSystemTheme() {
  const [theme, setTheme] = useState<UiTheme>('light');

  useEffect(() => {
    async function getTheme() {
      const window = getCurrentWebviewWindow();
      const winTheme = await window.theme();
      setTheme(winTheme === 'dark' ? 'dark' : 'light');
    }
    getTheme().catch((e: unknown) =>
      console.warn('Failed to get system theme', e),
    );
  }, []);

  const handleThemeChange = useCallback(
    async (mode: ThemeMode) => {
      dispatch({
        type: 'set-ui-theme',
        theme: mode === 'system' ? theme : mode,
      });
      dispatch({
        type: 'set-theme-mode',
        mode,
      });
      kvSet('ui-theme', mode);
      try {
        let theme: UiTheme;
        if (mode === 'system') {
          const window = getCurrentWebviewWindow();
          const systemTheme = await window.theme();
          theme = systemTheme === 'dark' ? 'dark' : 'light';
        } else {
          theme = mode;
        }
        await invoke('set_background_color', {
          hexColor: theme === 'dark' ? ColorMainBgDark : ColorMainBgLight,
        });
        console.info('updated webview window background color');
      } catch {
        console.error('Failed to update UI theme');
      }
    },
    [theme],
  );

  return { theme, handleThemeChange };
}
