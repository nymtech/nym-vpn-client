import { resources } from './config';

export type LngTag = keyof typeof resources;
export type Lang = { code: LngTag; name: string };
