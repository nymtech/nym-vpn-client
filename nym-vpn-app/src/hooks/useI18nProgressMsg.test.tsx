import { describe, expect, it } from 'vitest';
import type { ConnectingProgress, ProgressMsg } from '../types';
import { renderHookWithProviders } from '../test/harness';
import useI18nProgressMsg from './useI18nProgressMsg';

const messages: (ConnectingProgress | ProgressMsg)[] = [
  'canceling',
  'resolving-api-addresses',
  'awaiting-account-readiness',
  'refreshing-gateways',
  'selecting-gateways',
  'registering-with-gateways',
  'connecting-tunnel',
];

describe('useI18nProgressMsg', () => {
  it('translates every progress message to a non-empty string', () => {
    const { result } = renderHookWithProviders(() => useI18nProgressMsg());
    for (const message of messages) {
      const label = result.current.t(message);
      expect(typeof label).toBe('string');
      expect(label?.length).toBeGreaterThan(0);
    }
  });

  it('maps distinct progress states to distinct translations', () => {
    const { result } = renderHookWithProviders(() => useI18nProgressMsg());
    expect(result.current.t('canceling')).not.toBe(
      result.current.t('connecting-tunnel'),
    );
  });
});
