/// <reference types="vite/client" />

// eslint-disable-next-line @typescript-eslint/consistent-type-definitions
interface ImportMetaEnv {
  readonly APP_NOSPLASH: string | undefined;
  readonly APP_SENTRY_DSN: string | undefined;
  readonly APP_DISABLE_DATA_STORAGE: string | undefined;
}

// eslint-disable-next-line @typescript-eslint/consistent-type-definitions
interface ImportMeta {
  readonly env: ImportMetaEnv;
}
