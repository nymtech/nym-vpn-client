import { describe, expect, it, vi } from 'vitest';
import { act, waitFor } from '@testing-library/react';
import {
  i18n,
  mockTauriCommands,
  renderHookWithProviders,
} from '../test/harness';
import useLang from './useLang';

describe('useLang', () => {
  it('resolves localized country names for valid codes', () => {
    const { result } = renderHookWithProviders(() => useLang());
    // English display names are deterministic in jsdom's Intl.
    expect(result.current.getCountryName('DE')).toBe('Germany');
    expect(result.current.getCountryName('FR')).toBe('France');
  });

  it('returns null for a malformed country code without throwing', () => {
    const { result } = renderHookWithProviders(() => useLang());
    // A structurally invalid region code makes Intl throw; the hook swallows
    // it and returns null rather than propagating.
    expect(result.current.getCountryName('X')).toBeNull();
  });

  it('orders strings using the collator for the active language', () => {
    const { result } = renderHookWithProviders(() => useLang());
    expect(result.current.compare('apple', 'banana')).toBeLessThan(0);
    expect(result.current.compare('banana', 'apple')).toBeGreaterThan(0);
    expect(result.current.compare('apple', 'apple')).toBe(0);
  });

  it('set() persists the language to the KV store and applies it to i18n', async () => {
    const setCalls: Record<string, unknown>[] = [];
    mockTauriCommands((cmd, payload) => {
      if (cmd === 'db_set' && payload) {
        setCalls.push(payload);
      }
      return null;
    });

    const { result } = renderHookWithProviders(() => useLang());
    await act(async () => {
      await result.current.set('en');
    });

    await waitFor(() => {
      expect(i18n.language).toBe('en');
    });
    expect(setCalls).toContainEqual({ key: 'ui-language', value: 'en' });
    expect(document.documentElement.getAttribute('lang')).toBe('en');
  });

  it('set() skips the KV store write when updateDb is false', async () => {
    const setCommand = vi.fn();
    mockTauriCommands((cmd) => {
      if (cmd === 'db_set') {
        setCommand();
      }
      return null;
    });

    const { result } = renderHookWithProviders(() => useLang());
    await act(async () => {
      await result.current.set('en', false);
    });

    expect(setCommand).not.toHaveBeenCalled();
  });

  it('setSystem() clears the stored preference and applies a system locale', async () => {
    const delCalls: Record<string, unknown>[] = [];
    mockTauriCommands((cmd, payload) => {
      if (cmd === 'db_del' && payload) {
        delCalls.push(payload);
      }
      return null;
    });

    const { result } = renderHookWithProviders(() => useLang());
    await act(async () => {
      await result.current.setSystem();
    });

    expect(delCalls).toContainEqual({ key: 'ui-language' });
    // jsdom's navigator.language is English-based, so it resolves to 'en'.
    await waitFor(() => {
      expect(i18n.language).toBe('en');
    });
  });
});
