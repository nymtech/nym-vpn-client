// Binary basename for problematic apps
export const PROBLEMATIC_APPS = {
  DISABLED: new Set(['gnome-terminal']),
  WITH_WARNING: new Set([
    'brave-browser-stable',
    'chromium-browser',
    'firefox',
    'firefox-esr',
    'google-chrome-stable',
    'mate-terminal',
    'opera',
    'xfce4-terminal',
  ]),
};
