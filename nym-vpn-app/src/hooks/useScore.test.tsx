import { describe, expect, it } from 'vitest';
import type { Score } from '../types';
import { renderHookWithProviders } from '../test/harness';
import useScore from './useScore';

const scores: Score[] = ['offline', 'low', 'medium', 'high'];

describe('useScore', () => {
  it('maps every performance score to a colour and a non-empty label', () => {
    const { result } = renderHookWithProviders(() => useScore());
    for (const score of scores) {
      const perf = result.current.performance(score);
      expect(perf?.color).toMatch(/^text-/);
      expect(perf?.label).toBeTruthy();
    }
  });

  it('maps every server-load score to a colour and a non-empty label', () => {
    const { result } = renderHookWithProviders(() => useScore());
    for (const score of scores) {
      const load = result.current.serverLoad(score);
      expect(load?.color).toMatch(/^text-/);
      expect(load?.label).toBeTruthy();
    }
  });

  it('uses distinct colours for performance vs server load on the same score', () => {
    const { result } = renderHookWithProviders(() => useScore());
    // "low" is good for load (brand) but bad for performance (error).
    expect(result.current.performance('low')?.color).toBe('text-status-error');
    expect(result.current.serverLoad('low')?.color).toBe('text-brand-primary');
  });
});
