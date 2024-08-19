import common from '../hi/common.json';
import home from '../hi/home.json';
import settings from '../hi/settings.json';
import nodeLocation from '../hi/node-location.json';
import backendMessages from '../hi/backend-messages.json';
import display from '../hi/display.json';
import addCredential from '../hi/add-credential.json';
import licenses from '../hi/licenses.json';
import errors from '../hi/errors.json';
import welcome from '../hi/welcome.json';
import glossary from '../hi/glossary.json';
import notifications from '../hi/notifications.json';

export const hi = {
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
