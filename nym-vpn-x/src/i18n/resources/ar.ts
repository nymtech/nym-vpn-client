import common from '../ar/common.json';
import home from '../ar/home.json';
import settings from '../ar/settings.json';
import nodeLocation from '../ar/node-location.json';
import backendMessages from '../ar/backend-messages.json';
import display from '../ar/display.json';
import addCredential from '../ar/add-credential.json';
import licenses from '../ar/licenses.json';
import errors from '../ar/errors.json';
import welcome from '../ar/welcome.json';
import glossary from '../ar/glossary.json';
import notifications from '../ar/notifications.json';

export const ar = {
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
} as const;
