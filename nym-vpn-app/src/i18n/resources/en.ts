import account from '../en/account.json';
import common from '../en/common.json';
import home from '../en/home.json';
import settings from '../en/settings.json';
import nodeLocation from '../en/node-location.json';
import backendMessages from '../en/backend-messages.json';
import display from '../en/display.json';
import addCredential from '../en/add-credential.json';
import licenses from '../en/licenses.json';
import errors from '../en/errors.json';
import welcome from '../en/welcome.json';
import onboarding from '../en/onboarding.json';
import glossary from '../en/glossary.json';
import notifications from '../en/notifications.json';
import login from '../en/login.json';
import tray from '../en/tray.json';
import systemAuthentication from '../en/system-authentication.json';
import recoveryPhrase from '../en/recovery-phrase.json';

export const en = {
  account,
  common,
  home,
  settings,
  'node-location': nodeLocation,
  'backend-messages': backendMessages,
  display,
  'add-credential': addCredential,
  licenses,
  errors,
  welcome,
  onboarding,
  glossary,
  notifications,
  login,
  tray,
  'system-authentication': systemAuthentication,
  'recovery-phrase': recoveryPhrase,
} as const;
