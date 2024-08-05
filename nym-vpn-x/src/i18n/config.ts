import i18n from 'i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import { initReactI18next } from 'react-i18next';

import { en, es, fr, it, ptBR, ru, tr, uk, zhHans } from './resources';
import { Lang } from './types';

export const defaultNS = 'common';
export const resources = {
  en,
  es,
  fr,
  it,
  ['pt-BR']: ptBR,
  ru,
  tr,
  uk,
  ['zh-Hans']: zhHans,
} as const;

export const languages: Lang[] = [
  { code: 'en', name: 'English' },
  { code: 'es', name: 'Español' },
  { code: 'fr', name: 'Français' },
  { code: 'it', name: 'Italiano' },
  { code: 'pt-BR', name: 'Português brasileiro' },
  { code: 'ru', name: 'Русский язык' },
  { code: 'uk', name: 'Українська' },
  { code: 'tr', name: 'Türkçe' },
  { code: 'zh-Hans', name: '中文' },
];

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    // add 'pt' and 'zh' to supportedLngs to avoid i18next warnings
    supportedLngs: ['pt', 'zh', ...Object.keys(resources)],
    fallbackLng: ['en'],
    debug: import.meta.env.DEV,
    defaultNS,
    resources,
    ns: [
      'addCredential',
      'common',
      'home',
      'settings',
      'nodeLocation',
      'backendMessages',
      'display',
      'licenses',
      'errors',
      'welcome',
      'glossary',
      'notifications',
    ],

    interpolation: {
      escapeValue: false, // not needed for react as it escapes by default
    },
  });

export default i18n;
