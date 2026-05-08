import { Button } from '@headlessui/react';
import clsx from 'clsx';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useLang } from '../../../../hooks';
import { LngTag, languages } from '../../../../i18n';
import { kvGet } from '../../../../kvStore';
import { PageAnim, SettingsMenuCard } from '../../../../ui';
import { TranslationHelpUrl } from '../../../../constants';

function Lang() {
  const { t, i18n } = useTranslation();
  const { set, setSystem } = useLang();
  const [isSystemLang, setIsSystemLang] = useState<boolean>(false);

  useEffect(() => {
    kvGet<string>('ui-language').then((stored) => {
      setIsSystemLang(!stored);
    });
  }, []);

  const handleSystemLang = async () => {
    setIsSystemLang(true);
    await setSystem();
  };

  const handleLangSelect = async (code: LngTag) => {
    setIsSystemLang(false);
    await set(code);
  };

  return (
    <PageAnim
      className="relative flex h-full flex-col"
      data-testid="language-page"
    >
      <div
        className={clsx(
          'sticky -top-4 right-0 left-0 mb-4 w-full pt-4',
          'from-faded-lavender via-faded-lavender/98 dark:from-ash dark:via-ash/98 bg-linear-to-b to-transparent dark:to-transparent',
        )}
      >
        <SettingsMenuCard
          title={t('support.help.title', { ns: 'settings' })}
          onClick={() => openUrl(TranslationHelpUrl)}
          description={t('support.help.description', { ns: 'settings' })}
          leadingIcon="language"
          trailingIcon="open_in_new"
        />
      </div>

      <ul
        className="flex w-full flex-col items-stretch gap-1"
        data-testid="language-list"
      >
        <li
          key="system"
          className="w-full list-none"
          data-testid="language-item-system"
        >
          <Button
            role="presentation"
            className={clsx([
              'flex w-full flex-row items-center justify-between',
              'hover:bg-iron/10 dark:hover:bg-bombay/10',
              'cursor-default rounded-lg px-3 py-1 transition duration-75',
            ])}
            onClick={handleSystemLang}
            data-testid="language-button-system"
            data-selected={isSystemLang}
          >
            <div
              className="m-1 flex flex-row items-center gap-3 overflow-hidden p-1"
              data-testid="language-name-system"
            >
              {t('language-system', { ns: 'settings' })}
            </div>
            <div
              className={clsx([
                'ml-2 flex items-center pr-4 text-xs font-medium',
                'text-text-secondary',
              ])}
              data-testid="language-selected-indicator-system"
            >
              {isSystemLang && t('selected', { ns: 'glossary' })}
            </div>
          </Button>
        </li>

        {languages.map((lang) => (
          <li
            key={lang.code}
            className="w-full list-none"
            data-testid={`language-item-${lang.code}`}
          >
            <Button
              role="presentation"
              className={clsx([
                'flex w-full flex-row items-center justify-between',
                'hover:bg-iron/10 dark:hover:bg-bombay/10',
                'cursor-default rounded-lg px-3 py-1 transition duration-75',
              ])}
              onClick={() => handleLangSelect(lang.code)}
              data-testid={`language-button-${lang.code}`}
              data-selected={!isSystemLang && i18n.language === lang.code}
            >
              <div
                className="m-1 flex flex-row items-center gap-3 overflow-hidden p-1"
                data-testid={`language-name-${lang.code}`}
              >
                {lang.name}
              </div>
              <div
                className={clsx([
                  'ml-2 flex items-center pr-4 text-xs font-medium',
                  'text-text-secondary',
                ])}
                data-testid={`language-selected-indicator-${lang.code}`}
              >
                {!isSystemLang &&
                  i18n.language === lang.code &&
                  t('selected', { ns: 'glossary' })}
              </div>
            </Button>
          </li>
        ))}
      </ul>
    </PageAnim>
  );
}

export default Lang;
