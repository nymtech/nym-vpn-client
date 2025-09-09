type StartupError = {
  key: 'internal' | 'db-open' | 'db-locked';
  detail: string | null;
};

// binding to WindowInitEnv Rust struct
type JsEnv = {
  devMode: boolean;
  updaterEnabled: boolean;
  noSplash: boolean;
  defaultVpnMode: 'wg' | 'mixnet';
  defaultSentryEnabled: boolean;
  defaultNetstatsEnabled: boolean;
  startupError?: StartupError | null;
};

// eslint-disable-next-line @typescript-eslint/consistent-type-definitions
interface Window {
  _APP: JsEnv;
}
