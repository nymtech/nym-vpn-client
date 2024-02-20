/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly APP_NOSPLASH: string | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
