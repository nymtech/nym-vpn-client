import { defaultNS, resources } from '../i18n/config.ts';

// based on https://www.i18next.com/overview/typescript
declare module 'i18next' {
  // eslint-disable-next-line @typescript-eslint/consistent-type-definitions
  interface CustomTypeOptions {
    defaultNS: typeof defaultNS;
    resources: (typeof resources)['en'];
  }
}
