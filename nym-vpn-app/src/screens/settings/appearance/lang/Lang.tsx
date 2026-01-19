import { Button } from '@headlessui/react';
import clsx from 'clsx';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useTranslation } from 'react-i18next';
import { useLang } from '../../../../hooks';
import { languages } from '../../../../i18n';
import { PageAnim, SettingsMenuCard } from '../../../../ui';
import { TranslationHelpUrl } from '../../../../constants';

function Lang() {
  const { t, i18n } = useTranslation();
  const { set } = useLang();

  return (
    <PageAnim
      className="relative h-full flex flex-col"
      data-testid="language-page"
    >
      <div className="sticky -top-4 left-0 right-0 w-full pt-4 mb-4 bg-faded-lavender dark:bg-ash">
        <SettingsMenuCard
          title={t('support.help.title', { ns: 'settings' })}
          onClick={() => openUrl(TranslationHelpUrl)}
          description={t('support.help.description', { ns: 'settings' })}
          leadingIcon="language"
          trailingIcon="open_in_new"
          className="mx-3"
        />
      </div>

      <ul
        className="flex flex-col w-full items-stretch gap-1"
        data-testid="language-list"
      >
        {languages.map((lang) => (
          <li
            key={lang.code}
            className="list-none w-full"
            data-testid={`language-item-${lang.code}`}
          >
            <Button
              role="presentation"
              className={clsx([
                'flex flex-row justify-between items-center w-full',
                'hover:bg-iron/10 dark:hover:bg-bombay/10',
                'rounded-lg px-3 py-1 transition duration-75 cursor-default',
              ])}
              onClick={() => set(lang.code)}
              data-testid={`language-button-${lang.code}`}
              data-selected={i18n.language === lang.code}
            >
              <div
                className="flex flex-row items-center m-1 gap-3 p-1 overflow-hidden"
                data-testid={`language-name-${lang.code}`}
              >
                {lang.name}
              </div>
              <div
                className={clsx([
                  'pr-4 ml-2 flex items-center font-medium text-xs',
                  'text-iron dark:text-bombay',
                ])}
                data-testid={`language-selected-indicator-${lang.code}`}
              >
                {i18n.language === lang.code &&
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
