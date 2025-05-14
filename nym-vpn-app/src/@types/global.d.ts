type JsEnv = {
  devMode: boolean;
};

// eslint-disable-next-line @typescript-eslint/consistent-type-definitions
interface Window {
  _APP: JsEnv;
}
