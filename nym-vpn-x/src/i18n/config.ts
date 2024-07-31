import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import { en, es, fr, it, ptBR, ru, tr, uk, zhHans } from './resources';

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

i18n.use(initReactI18next).init({
  lng: 'en',
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
