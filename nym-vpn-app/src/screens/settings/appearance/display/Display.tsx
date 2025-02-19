import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useMainDispatch, useMainState } from '../../../../contexts';
import { kvSet } from '../../../../kvStore';
import { useSystemTheme } from '../../../../state';
import { StateDispatch, ThemeMode } from '../../../../types';
import { PageAnim, RadioGroup, RadioGroupOption } from '../../../../ui';
import UiScaler from './UiScaler';

function Display() {
  const state = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('display');

  const { theme: systemTheme } = useSystemTheme();

  const handleThemeChange = async (mode: ThemeMode) => {
    if (mode !== state.themeMode) {
      dispatch({
        type: 'set-ui-theme',
        theme: mode === 'System' ? systemTheme : mode,
      });
      dispatch({
        type: 'set-theme-mode',
        mode,
      });
      kvSet('ui-theme', mode);
      try {
        let theme: 'Dark' | 'Light';
        if (mode === 'System') {
          const window = getCurrentWindow();
          const systemTheme = await window.theme();
          theme = systemTheme === 'dark' ? 'Dark' : 'Light';
        } else {
          theme = mode;
        }
        await invoke('set_background_color', {
          hexColor: theme === 'Dark' ? '#1C1B1F' : '#F2F4F6',
        });
        console.log('updated webview window background color');
      } catch (e) {
        console.error('failed to set the webview window background color', e);
      }
    }
  };

  const options = useMemo<RadioGroupOption<ThemeMode>[]>(() => {
    return [
      {
        key: 'System',
        label: t('options.system'),
        desc: t('system-desc'),
      },
      {
        key: 'Light',
        label: t('options.light'),
        className: 'min-h-11',
      },
      {
        key: 'Dark',
        label: t('options.dark'),
        className: 'min-h-11',
      },
    ];
  }, [t]);

  return (
    <PageAnim className="h-full flex flex-col py-6 gap-6">
      <RadioGroup
        defaultValue={state.themeMode}
        options={options}
        onChange={handleThemeChange}
        rootLabel={t('theme-section-title')}
      />
      <div className="mt-3 text-base font-semibold cursor-default">
        {t('zoom-section-title')}
      </div>
      <UiScaler />
    </PageAnim>
  );
}

export default Display;
