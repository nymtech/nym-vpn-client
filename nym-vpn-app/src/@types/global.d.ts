type StartupError = {
  key: 'internal' | 'db-open' | 'db-locked';
  detail: string | null;
};

// binding to WindowInitEnv Rust struct
type JsEnv = {
  devMode: boolean;
  updaterEnabled: boolean;
  noSplash: boolean;
  defaultVpnMode: 'mixnet' | 'wg';
  defaultSentry: boolean;
  defaultNetstats: boolean;
  defaultQuic: boolean;
  defaultDomainFronting: boolean;
  startupError: StartupError | null;
};

// eslint-disable-next-line @typescript-eslint/consistent-type-definitions
interface Window {
  _APP: JsEnv;
}
