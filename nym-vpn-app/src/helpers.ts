import { NetworkEnv } from './types';
import {
  AccountCreateUrlPath,
  AccountLoginUrlPath,
  NymDotComCanaryUrl,
  NymDotComQAUrl,
  NymDotComUrl,
} from './constants';

export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Capitalize the first letter of a string
export function capFirst(string: string) {
  return string.charAt(0).toUpperCase() + string.slice(1);
}

// Given a set of strings, return the strings concatenated by a white space
export function setToString(obj: Record<string, string>): string {
  return Object.values(obj).reduce((prev, s) => `${prev} ${s}`, '');
}

export function getCreateAccountUrl(env: NetworkEnv) {
  switch (env) {
    case 'mainnet':
      return `${NymDotComUrl}${AccountCreateUrlPath}`;
    case 'canary':
      return `${NymDotComCanaryUrl}${AccountCreateUrlPath}`;
    case 'qa':
      return `${NymDotComQAUrl}${AccountCreateUrlPath}`;
    default:
      return null;
  }
}

export function getLoginAccountUrl(env: NetworkEnv) {
  switch (env) {
    case 'mainnet':
      return `${NymDotComUrl}${AccountLoginUrlPath}`;
    case 'canary':
      return `${NymDotComCanaryUrl}${AccountLoginUrlPath}`;
    case 'qa':
      return `${NymDotComQAUrl}${AccountLoginUrlPath}`;
    default:
      return null;
  }
}
