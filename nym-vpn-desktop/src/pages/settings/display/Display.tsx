import { useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api';
import { useTranslation } from 'react-i18next';
import { appWindow } from '@tauri-apps/api/window';
import { useMainDispatch, useMainState } from '../../../contexts';
import { StateDispatch, UiTheme } from '../../../types';
import { RadioGroup, RadioGroupOption } from '../../../ui';
import UiScaler from './UiScaler';

type ThemeModes = UiTheme | 'System';

function Display() {
  const state = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('display');

  const handleThemeChange = async (mode: ThemeModes) => {
    let newMode: UiTheme = 'Light';
    if (mode === 'System') {
      const systemTheme = await appWindow.theme();
      systemTheme === 'dark' ? (newMode = 'Dark') : (newMode = 'Light');
    } else if (mode === 'Dark') {
      newMode = 'Dark';
    } else if (mode === 'Light') {
      newMode = 'Light';
    }
    if (newMode !== state.uiTheme) {
      dispatch({ type: 'set-ui-theme', theme: newMode });
      invoke<void>('set_ui_theme', {
        theme: newMode,
      }).catch((e) => {
        console.log(e);
      });
    }
  };

  const options = useMemo<RadioGroupOption<ThemeModes>[]>(() => {
    return [
      {
        key: 'System',
        label: t('options.system'),
        desc: t('system-desc'),
        cursor: 'pointer',
      },
      {
        key: 'Light',
        label: t('options.light'),
        cursor: 'pointer',
        style: 'min-h-11',
      },
      {
        key: 'Dark',
        label: t('options.dark'),
        cursor: 'pointer',
        style: 'min-h-11',
      },
    ];
  }, [t]);

  return (
    <div className="h-full flex flex-col py-6 gap-6">
      <RadioGroup
        defaultValue={state.uiTheme}
        options={options}
        onChange={handleThemeChange}
        rootLabel={t('theme-section-title')}
      />
      <div className="mt-3 text-base font-semibold cursor-default">
        {t('zoom-section-title')}
      </div>
      <UiScaler />
    </div>
  );
}

export default Display;
