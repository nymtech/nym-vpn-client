import { ns, supportedLngs } from './config';

export type LngTag = (typeof supportedLngs)[number];
export type Locale = Record<string, unknown>;
export type Namespaces = (typeof ns)[number];
export type LocaleResource = Partial<Record<Namespaces, Locale>>;
