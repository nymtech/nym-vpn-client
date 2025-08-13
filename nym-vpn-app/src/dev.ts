if (import.meta.env.MODE === 'dev-browser') {
  window._APP = {
    defaultNetstatsEnabled: true,
    defaultSentryEnabled: false,
    defaultVpnMode: 'wg',
    devMode: true,
    updaterEnabled: true,
    noSplash: true,
  };
  // @ts-expect-error mocking os plugin
  window.__TAURI_OS_PLUGIN_INTERNALS__ = {
    eol: '\n',
    os_type: 'linux',
    platform: 'linux',
    family: 'unix',
    arch: 'x86_64',
    version: '2001.0.1-arch1-1',
  };
}
