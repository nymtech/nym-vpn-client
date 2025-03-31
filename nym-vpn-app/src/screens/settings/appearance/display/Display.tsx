import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useMainDispatch, useMainState } from '../../../../contexts';
import { kvSet } from '../../../../kvStore';
import { useSystemTheme } from '../../../../state';
import { StateDispatch, ThemeMode, UiTheme } from '../../../../types';
import { PageAnim, RadioGroup, RadioGroupOption } from '../../../../ui';
import { ColorMainBgDark, ColorMainBgLight } from '../../../../constants';
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
        theme: mode === 'system' ? systemTheme : mode,
      });
      dispatch({
        type: 'set-theme-mode',
        mode,
      });
      kvSet('ui-theme', mode);
      try {
        let theme: UiTheme;
        if (mode === 'system') {
          const window = getCurrentWindow();
          const systemTheme = await window.theme();
          theme = systemTheme === 'dark' ? 'dark' : 'light';
        } else {
          theme = mode;
        }
        await invoke('set_background_color', {
          hexColor: theme === 'dark' ? ColorMainBgDark : ColorMainBgLight,
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
        key: 'system',
        label: t('options.system'),
        desc: t('system-desc'),
      },
      {
        key: 'light',
        label: t('options.light'),
        className: 'min-h-11',
      },
      {
        key: 'dark',
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
      <div className="mt-3 text-base font-medium cursor-default">
        {t('zoom-section-title')}
      </div>
      <UiScaler />
    </PageAnim>
  );
}

export default Display;
