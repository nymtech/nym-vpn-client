import i18n from 'i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import { initReactI18next } from 'react-i18next';

import { en } from './resources';
import { LngTag, Locale, LocaleResource, Namespaces } from './types';

export const defaultNS = 'common';
export const ns = [
  'account',
  'add-credential',
  'common',
  'home',
  'settings',
  'node-location',
  'backend-messages',
  'display',
  'licenses',
  'errors',
  'welcome',
  'glossary',
  'notifications',
  'login',
] as const;

export const languages = [
  { code: 'ar', name: 'العربية' }, // rtl
  { code: 'fa', name: 'فارسی' }, // rtl
  { code: 'bn', name: 'বাংলা' },
  { code: 'de', name: 'Deutsch' },
  { code: 'en', name: 'English' },
  { code: 'es', name: 'Español' },
  { code: 'fr', name: 'Français' },
  { code: 'hi', name: 'हिन्दी' },
  { code: 'pt', name: 'Português Brasileiro' },
  { code: 'ru', name: 'Русский язык' },
  { code: 'tr', name: 'Türkçe' },
  { code: 'uk', name: 'Українська' },
  { code: 'vi', name: 'Tiếng Việt' },
  { code: 'zh', name: '中文' },

  // { code: 'cs', name: 'Čeština (Czech)' },
  // { code: 'hu', name: 'Magyar (Hungarian)' },
  // { code: 'el', name: 'ελληνικά' },
  // { code: 'it', name: 'Italiano' },
  // { code: 'ja', name: '日本語' },
] as const;

export const supportedLngs = languages.map((lang) => lang.code);

const loadLocaleNs = async (
  locale: LngTag,
  ns: Namespaces,
): Promise<Locale> => {
  const module = (await import(`./${locale}/${ns}.json`)) as {
    default: Locale;
  };
  return module.default;
};

export const loadLocale = async (locale: LngTag) => {
  return ns.reduce<Promise<LocaleResource>>(
    async (accPromise, namespace) => {
      if (i18n.hasResourceBundle(locale, namespace)) {
        return accPromise;
      }
      const acc = await accPromise;
      try {
        acc[namespace] = await loadLocaleNs(locale, namespace);
      } catch (error) {
        console.error(
          `Failed to load namespace ${namespace} for locale ${locale}:`,
          error,
        );
      }
      return acc;
    },
    Promise.resolve({} as LocaleResource),
  );
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    supportedLngs,
    fallbackLng: ['en'],
    debug: import.meta.env.DEV,
    defaultNS,
    resources: {
      en: en,
    },
    ns,
    interpolation: {
      escapeValue: false, // not needed for react as it escapes by default
    },
  });

export default i18n;
