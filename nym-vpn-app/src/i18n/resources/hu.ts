import account from '../hu/account.json';
import common from '../hu/common.json';
import home from '../hu/home.json';
import settings from '../hu/settings.json';
import nodeLocation from '../hu/node-location.json';
import backendMessages from '../hu/backend-messages.json';
import display from '../hu/display.json';
import addCredential from '../hu/add-credential.json';
import licenses from '../hu/licenses.json';
import errors from '../hu/errors.json';
import welcome from '../hu/welcome.json';
import glossary from '../hu/glossary.json';
import notifications from '../hu/notifications.json';

export const hu = {
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
