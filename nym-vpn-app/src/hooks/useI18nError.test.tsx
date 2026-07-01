import { describe, expect, it } from 'vitest';
import type { ErrorKey } from '../types';
import { renderHookWithProviders } from '../test/harness';
import useI18nError from './useI18nError';

const errorKeys: ErrorKey[] = [
  'entry-gw-down',
  'exit-gw-down-ipv4',
  'exit-gw-down-ipv6',
  'exit-gw-routing-error-ipv4',
  'exit-gw-routing-error-ipv6',
  'mixnet-no-bandwidth',
  'internal',
  'vpnd-client',
  'not-connected-to-daemon',
  'auth-denied',
  'account-invalid-mnemonic',
  'account-invalid-secret',
  'get-mixnet-entry-countries-query',
  'get-mixnet-exit-countries-query',
  'get-wg-countries-query',
  'no-account-stored',
  'no-device-stored',
  'existing-account',
  'account-status-not-active',
  'no-subscription',
  'max-device-reached',
  'device-time-desync',
  'bandwidth-exceeded',
  'split-tunnel-app-invalid',
  'split-tunnel-app-duplicate',
];

describe('useI18nError', () => {
  it('translates every known error key to a non-empty string', () => {
    const { result } = renderHookWithProviders(() => useI18nError());
    for (const key of errorKeys) {
      const message = result.current.tE(key);
      expect(typeof message).toBe('string');
      expect(message.length).toBeGreaterThan(0);
    }
  });

  it('maps distinct keys to their specific translations', () => {
    const { result } = renderHookWithProviders(() => useI18nError());
    // Both invalid-phrase keys collapse onto the same translation.
    expect(result.current.tE('account-invalid-mnemonic')).toBe(
      result.current.tE('account-invalid-secret'),
    );
    // A general key resolves to something different from an account key.
    expect(result.current.tE('internal')).not.toBe(
      result.current.tE('no-subscription'),
    );
  });

  it('falls back to the "unknown" translation for an unhandled key', () => {
    const { result } = renderHookWithProviders(() => useI18nError());
    const unknown = result.current.tE('unknown' as unknown as ErrorKey);
    expect(unknown).toBe('Unknown error');
  });
});
