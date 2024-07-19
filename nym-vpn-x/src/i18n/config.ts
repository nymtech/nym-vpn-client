import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import common from './en/common.json';
import home from './en/home.json';
import settings from './en/settings.json';
import nodeLocation from './en/node-location.json';
import backendMessages from './en/backend-messages.json';
import display from './en/display.json';
import addCredential from './en/add-credential.json';
import licenses from './en/licenses.json';
import errors from './en/errors.json';
import welcome from './en/welcome.json';
import glossary from './en/glossary.json';
import notifications from './en/notifications.json';

export const defaultNS = 'common';
export const resources = {
  en: {
    common,
    home,
    settings,
    nodeLocation,
    backendMessages,
    display,
    addCredential,
    licenses,
    errors,
    welcome,
    glossary,
    notifications,
  },
} as const;

i18n.use(initReactI18next).init({
  lng: 'en',
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
