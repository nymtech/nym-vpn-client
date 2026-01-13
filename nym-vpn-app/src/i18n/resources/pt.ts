import account from '../pt/account.json';
import common from '../pt/common.json';
import home from '../pt/home.json';
import settings from '../pt/settings.json';
import nodeLocation from '../pt/node-location.json';
import backendMessages from '../pt/backend-messages.json';
import display from '../pt/display.json';
import addCredential from '../pt/add-credential.json';
import licenses from '../pt/licenses.json';
import errors from '../pt/errors.json';
import welcome from '../pt/welcome.json';
import glossary from '../pt/glossary.json';
import notifications from '../pt/notifications.json';

export const pt = {
  account,
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
