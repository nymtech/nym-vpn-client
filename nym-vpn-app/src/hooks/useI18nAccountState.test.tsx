import { describe, expect, it } from 'vitest';
import type { AccountState } from '../types';
import { renderHookWithProviders } from '../test/harness';
import useI18nAccountState from './useI18nAccountState';

const states: AccountState[] = [
  'bandwidth-exceeded',
  'status-not-active',
  'no-subscription',
  'max-device-reached',
  'pending-subscription',
];

describe('useI18nAccountState', () => {
  it('translates every mapped account state to a non-empty string', () => {
    const { result } = renderHookWithProviders(() => useI18nAccountState());
    for (const state of states) {
      const message = result.current.t(state);
      expect(typeof message).toBe('string');
      expect(message.length).toBeGreaterThan(0);
    }
  });

  it('falls back to the internal account error for an unmapped state', () => {
    const { result } = renderHookWithProviders(() => useI18nAccountState());
    // 'ready' is not handled explicitly and hits the default branch.
    expect(result.current.t('ready')).toBe('Internal account error');
  });

  it('maps distinct states to distinct translations', () => {
    const { result } = renderHookWithProviders(() => useI18nAccountState());
    expect(result.current.t('bandwidth-exceeded')).not.toBe(
      result.current.t('no-subscription'),
    );
  });
});
