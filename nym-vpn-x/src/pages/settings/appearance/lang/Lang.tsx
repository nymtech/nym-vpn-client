import { Button } from '@headlessui/react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { kvSet } from '../../../../kvStore';
import { PageAnim } from '../../../../ui';

type Lang = { code: string; name: string };

const languages: Lang[] = [
  { code: 'en', name: 'English' },
  { code: 'es', name: 'Español' },
  { code: 'fr', name: 'Français' },
  { code: 'it', name: 'Italiano' },
  { code: 'pt-BR', name: 'Português brasileiro' },
];

function Lang() {
  const { t, i18n } = useTranslation();

  const onSelect = (lang: Lang) => {
    if (i18n.language === lang.code) {
      return;
    }
    i18n.changeLanguage(lang.code);
    kvSet('UiLanguage', lang.code);
  };

  return (
    <PageAnim className="h-full flex flex-col py-6 gap-6">
      <ul className="flex flex-col w-full items-stretch gap-1">
        {languages.map((lang) => (
          <li key={lang.code} className="list-none w-full">
            <Button
              role="presentation"
              className={clsx([
                'flex flex-row justify-between items-center w-full',
                'hover:bg-gun-powder hover:bg-opacity-10',
                'dark:hover:bg-laughing-jack dark:hover:bg-opacity-10',
                'rounded-lg px-3 py-1 transition duration-75 cursor-default',
              ])}
              onClick={() => onSelect(lang)}
            >
              <div className="flex flex-row items-center m-1 gap-3 p-1 overflow-hidden">
                {lang.name}
              </div>
              <div
                className={clsx([
                  'pr-4 ml-2 flex items-center font-medium text-xs',
                  'text-cement-feet dark:text-mercury-mist',
                ])}
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
