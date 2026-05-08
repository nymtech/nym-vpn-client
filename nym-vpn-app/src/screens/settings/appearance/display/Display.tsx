import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { useSystemTheme } from '../../../../state';
import { ThemeMode } from '../../../../types';
import { PageAnim, RadioGroup, RadioGroupOption } from '../../../../ui';
import { useMainState } from '../../../../store';
import UiScaler from './UiScaler';

function Display() {
  const state = useMainState();
  const { t } = useTranslation('display');

  const { handleThemeChange } = useSystemTheme();

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
    <PageAnim
      className="flex h-full flex-col gap-6 py-6"
      data-testid="display-page"
    >
      <RadioGroup
        defaultValue={state.themeMode}
        options={options}
        onChange={(mode) => handleThemeChange(mode)}
        rootLabel={t('theme-section-title')}
        data-testid="theme-radio-group"
      />
      <div
        className="mt-3 cursor-default text-base font-medium"
        data-testid="zoom-section-title"
      >
        {t('zoom-section-title')}
      </div>
      <UiScaler />
    </PageAnim>
  );
}

export default Display;
