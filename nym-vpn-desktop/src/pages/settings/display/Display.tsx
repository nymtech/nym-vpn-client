import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { useMainDispatch, useMainState } from '../../../contexts';
import { kvSet } from '../../../kvStore';
import { useSystemTheme } from '../../../state';
import { StateDispatch, ThemeMode } from '../../../types';
import { RadioGroup, RadioGroupOption } from '../../../ui';
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
      kvSet('UiTheme', mode);
    }
  };

  const options = useMemo<RadioGroupOption<ThemeMode>[]>(() => {
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
        className: 'min-h-11',
      },
      {
        key: 'Dark',
        label: t('options.dark'),
        cursor: 'pointer',
        className: 'min-h-11',
      },
    ];
  }, [t]);

  return (
    <div className="h-full flex flex-col py-6 gap-6">
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
    </div>
  );
}

export default Display;
