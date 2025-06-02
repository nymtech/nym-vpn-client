if (import.meta.env.MODE === 'dev-browser') {
  window._APP = {
    devMode: true,
    updaterEnabled: true,
  };
}
