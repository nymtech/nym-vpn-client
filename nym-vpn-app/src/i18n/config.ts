import i18n from 'i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import { initReactI18next } from 'react-i18next';

import {
  bn,
  de,
  el,
  en,
  es,
  fr,
  hi,
  it,
  ja,
  ptBR,
  ptPT,
  ru,
  tr,
  uk,
  vi,
  zhHans,
} from './resources';
import { Lang } from './types';

export const defaultNS = 'common';
export const resources = {
  bn,
  de,
  el,
  en,
  es,
  fr,
  hi,
  it,
  ja,
  ['pt-BR']: ptBR,
  ['pt-PT']: ptPT,
  ru,
  tr,
  uk,
  vi,
  ['zh-Hans']: zhHans,
} as const;

export const languages: Lang[] = [
  { code: 'bn', name: 'বাংলা' },
  { code: 'de', name: 'Deutsch' },
  { code: 'el', name: 'ελληνικά' },
  { code: 'en', name: 'English' },
  { code: 'es', name: 'Español' },
  { code: 'fr', name: 'Français' },
  { code: 'hi', name: 'हिन्दी' },
  { code: 'it', name: 'Italiano' },
  { code: 'ja', name: '日本語' },
  { code: 'pt-BR', name: 'Português brasileiro' },
  { code: 'pt-PT', name: 'Português de Portugal' },
  { code: 'ru', name: 'Русский язык' },
  { code: 'tr', name: 'Türkçe' },
  { code: 'uk', name: 'Українська' },
  { code: 'vi', name: 'Tiếng Việt' },
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
      'account',
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
      'login',
    ],

    interpolation: {
      escapeValue: false, // not needed for react as it escapes by default
    },
  });

export default i18n;
