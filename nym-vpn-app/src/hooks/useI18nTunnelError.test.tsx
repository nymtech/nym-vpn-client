import { describe, expect, it } from 'vitest';
import type { TunnelError } from '../types';
import { renderHookWithProviders } from '../test/harness';
import useI18nTunnelError from './useI18nTunnelError';

const stringErrors: Exclude<TunnelError, { internal: string }>[] = [
  'tun-device',
  'tunnel-provider',
  'inactive-account',
  'device-logged-out',
  'set-firewall-policy',
  'set-routing',
  'set-dns',
  'same-entry-and-exit-gw',
  'invalid-entry-gw-country',
  'invalid-exit-gw-country',
  'invalid-entry-gw-id',
  'invalid-exit-gw-id',
  'max-devices-reached',
  'bandwidth-exceeded',
  'inactive-subscription',
  'device-time-out-of-sync',
  'ipv6-unavailable',
  'credential-wasted-on-entry-gateway',
  'credential-wasted-on-exit-gateway',
  'performant-entry-gw-unavailable',
  'performant-exit-gw-unavailable',
  'needs-relaxed-independence-criteria',
];

describe('useI18nTunnelError', () => {
  it('translates every mapped tunnel error to a non-empty string', () => {
    const { result } = renderHookWithProviders(() => useI18nTunnelError());
    for (const error of stringErrors) {
      const message = result.current.tTE(error);
      expect(typeof message).toBe('string');
      expect(message.length).toBeGreaterThan(0);
    }
  });

  it('appends the internal detail for a structured internal error', () => {
    const { result } = renderHookWithProviders(() => useI18nTunnelError());
    const message = result.current.tTE({ internal: 'boom' });
    // Internal errors are formatted as "<translation> - <detail>".
    expect(message).toMatch(/ - boom$/);
    expect(message.startsWith('Internal error')).toBe(true);
  });

  it('falls back to the "unknown" translation for an unhandled variant', () => {
    const { result } = renderHookWithProviders(() => useI18nTunnelError());
    // 'split-tunnel' exists in the TunnelError union but has no switch arm.
    expect(result.current.tTE('split-tunnel')).toBe('Unknown error');
  });
});
