import account from '../cs/account.json';
import common from '../cs/common.json';
import home from '../cs/home.json';
import settings from '../cs/settings.json';
import nodeLocation from '../cs/node-location.json';
import backendMessages from '../cs/backend-messages.json';
import display from '../cs/display.json';
import addCredential from '../cs/add-credential.json';
import licenses from '../cs/licenses.json';
import errors from '../cs/errors.json';
import welcome from '../cs/welcome.json';
import glossary from '../cs/glossary.json';
import notifications from '../cs/notifications.json';

export const cs = {
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
