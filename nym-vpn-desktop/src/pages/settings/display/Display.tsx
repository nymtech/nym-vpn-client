import { useEffect, useState } from 'react';
import clsx from 'clsx';
import { invoke } from '@tauri-apps/api';
import { useTranslation } from 'react-i18next';
import { useMainDispatch, useMainState } from '../../../contexts';
import { StateDispatch } from '../../../types';
import { Switch } from '../../../ui';
import UiScaler from './UiScaler';

function Display() {
  const state = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation();

  const [darkModeEnabled, setDarkModeEnabled] = useState(
    state.uiTheme === 'Dark',
  );

  useEffect(() => {
    setDarkModeEnabled(state.uiTheme === 'Dark');
  }, [state]);

  const handleThemeChange = async (darkMode: boolean) => {
    if (darkMode && state.uiTheme === 'Light') {
      dispatch({ type: 'set-ui-theme', theme: 'Dark' });
    } else if (!darkMode && state.uiTheme === 'Dark') {
      dispatch({ type: 'set-ui-theme', theme: 'Light' });
    }
    invoke<void>('set_ui_theme', { theme: darkMode ? 'Dark' : 'Light' }).catch(
      (e) => {
        console.log(e);
      },
    );
  };

  return (
    <div className="h-full flex flex-col py-6 gap-6">
      <div
        className={clsx([
          'flex flex-row justify-between items-center',
          'bg-white dark:bg-baltic-sea-jaguar',
          'px-6 py-4 rounded-lg',
        ])}
      >
        <p className="text-base text-baltic-sea dark:text-mercury-pinkish select-none">
          {t('ui-mode.dark')}
        </p>
        <Switch checked={darkModeEnabled} onChange={handleThemeChange} />
      </div>
      <UiScaler />
    </div>
  );
}

export default Display;
