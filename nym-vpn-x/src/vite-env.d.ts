/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly APP_NOSPLASH: string | undefined;
  readonly APP_SENTRY_DSN: string | undefined;
  readonly APP_DISABLE_DATA_STORAGE: string | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
