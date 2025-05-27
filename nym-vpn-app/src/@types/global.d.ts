type StartupError = {
  key: 'startup-db-open' | 'startup-db-locked';
  detail: string | null;
};

type JsEnv = {
  devMode: boolean;
  startupError?: StartupError | null;
  updaterEnabled?: boolean;
};

// eslint-disable-next-line @typescript-eslint/consistent-type-definitions
interface Window {
  _APP: JsEnv;
}
