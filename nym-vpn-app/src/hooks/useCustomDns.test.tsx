import { afterEach, describe, expect, it, vi } from 'vitest';
import { act } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { useAppStore } from '../store';
import { renderHookWithProviders, seedStore } from '../test/harness';
import useCustomDns from './useCustomDns';

const add = vi.fn();

// `useCustomDns` pulls `useToast` from the hooks barrel; stub it so no toast
// provider is needed and error toasts can be asserted.
vi.mock('./index', () => ({
  useToast: () => ({ add, close: vi.fn() }),
}));

afterEach(() => {
  add.mockReset();
  seedStore({ customDnsEnabled: false, customDns: [], defaultDns: [] });
});

describe('useCustomDns', () => {
  it('reflects the store values it reads', () => {
    seedStore({
      customDnsEnabled: true,
      customDns: ['1.1.1.1'],
      defaultDns: ['8.8.8.8'],
    });
    const { result } = renderHookWithProviders(() => useCustomDns());

    expect(result.current.enabled).toBe(true);
    expect(result.current.customDns).toEqual(['1.1.1.1']);
    expect(result.current.defaultDns).toEqual(['8.8.8.8']);
  });

  it('toggle invokes set_custom_dns_enabled and updates the store', async () => {
    const calls: { cmd: string; payload?: unknown }[] = [];
    mockIPC((cmd, payload) => {
      calls.push({ cmd, payload });
      return undefined;
    });
    const { result } = renderHookWithProviders(() => useCustomDns());

    await act(async () => {
      await result.current.toggle(true);
    });

    expect(calls).toContainEqual({
      cmd: 'set_custom_dns_enabled',
      payload: { enabled: true },
    });
    expect(useAppStore.getState().customDnsEnabled).toBe(true);
    expect(add).not.toHaveBeenCalled();
  });

  it('toggle shows an error toast and skips the store update on failure', async () => {
    mockIPC((cmd) => {
      if (cmd === 'set_custom_dns_enabled') throw new Error('nope');
      return undefined;
    });
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(vi.fn());
    seedStore({ customDnsEnabled: false });
    const { result } = renderHookWithProviders(() => useCustomDns());

    await act(async () => {
      await result.current.toggle(true);
    });

    expect(useAppStore.getState().customDnsEnabled).toBe(false);
    expect(add).toHaveBeenCalledExactlyOnceWith({
      title: 'Failed to apply DNS changes',
      type: 'error',
    });
    errorSpy.mockRestore();
  });

  it('setCustomDns invokes set_custom_dns and updates the store', async () => {
    const calls: { cmd: string; payload?: unknown }[] = [];
    mockIPC((cmd, payload) => {
      calls.push({ cmd, payload });
      return undefined;
    });
    const { result } = renderHookWithProviders(() => useCustomDns());

    await act(async () => {
      await result.current.setCustomDns(['9.9.9.9']);
    });

    expect(calls).toContainEqual({
      cmd: 'set_custom_dns',
      payload: { dns: ['9.9.9.9'] },
    });
    expect(useAppStore.getState().customDns).toEqual(['9.9.9.9']);
    expect(add).not.toHaveBeenCalled();
  });

  it('setCustomDns shows an error toast and skips the store update on failure', async () => {
    mockIPC((cmd) => {
      if (cmd === 'set_custom_dns') throw new Error('nope');
      return undefined;
    });
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(vi.fn());
    seedStore({ customDns: [] });
    const { result } = renderHookWithProviders(() => useCustomDns());

    await act(async () => {
      await result.current.setCustomDns(['9.9.9.9']);
    });

    expect(useAppStore.getState().customDns).toEqual([]);
    expect(add).toHaveBeenCalledExactlyOnceWith({
      title: 'Failed to apply DNS changes',
      type: 'error',
    });
    errorSpy.mockRestore();
  });
});
