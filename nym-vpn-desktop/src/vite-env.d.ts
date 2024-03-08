/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly APP_NOSPLASH: string | undefined;
  readonly APP_CREDENTIAL: string | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
