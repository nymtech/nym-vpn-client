import account from '../bn/account.json';
import common from '../bn/common.json';
import home from '../bn/home.json';
import settings from '../bn/settings.json';
import nodeLocation from '../bn/node-location.json';
import backendMessages from '../bn/backend-messages.json';
import display from '../bn/display.json';
import addCredential from '../bn/add-credential.json';
import licenses from '../bn/licenses.json';
import errors from '../bn/errors.json';
import welcome from '../bn/welcome.json';
import glossary from '../bn/glossary.json';
import notifications from '../bn/notifications.json';

export const bn = {
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
