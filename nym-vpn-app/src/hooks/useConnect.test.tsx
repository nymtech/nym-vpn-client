import { beforeEach, describe, expect, it, vi } from 'vitest';
import { act } from '@testing-library/react';
import type { TentativeGateways } from '../types/tauri';
import {
  mockTauriCommands,
  renderHookWithProviders,
  seedStore,
} from '../test/harness';
import useConnect from './useConnect';

// The gateway-independence warning modal lives behind a React context that the
// shared harness does not provide. Mock it so we can drive `requestConfirmation`.
const requestConfirmation = vi.fn<() => Promise<boolean>>();
const noop = () => undefined;
vi.mock('../contexts/gatewayIndependence', () => ({
  useGwIndependenceWarning: () => ({
    isOpen: false,
    requestConfirmation,
    accept: noop,
    cancel: noop,
  }),
}));

type Recorded = { cmd: string; payload?: Record<string, unknown> };

function install(tentative: TentativeGateways) {
  const calls: Recorded[] = [];
  mockTauriCommands((cmd, payload) => {
    calls.push({ cmd, payload });
    if (cmd === 'get_tentative_gateways') return tentative;
    return undefined;
  });
  return calls;
}

beforeEach(() => {
  requestConfirmation.mockReset();
});

describe('useConnect', () => {
  it('resets independence to ON, then connects when a pair is selected', async () => {
    seedStore({ gatewayIndependenceNotifications: true });
    const calls = install('selected');
    const { result } = renderHookWithProviders(() => useConnect());

    await act(async () => {
      await result.current();
    });

    const names = calls.map((c) => c.cmd);
    expect(names).toEqual([
      'set_gateway_independence',
      'get_tentative_gateways',
      'connect',
    ]);
    expect(calls[0].payload).toEqual({ enabled: true });
    // No relaxation and no confirmation prompt on the happy path.
    expect(requestConfirmation).not.toHaveBeenCalled();
  });

  it('prompts and relaxes independence when the user confirms', async () => {
    seedStore({ gatewayIndependenceNotifications: true });
    requestConfirmation.mockResolvedValue(true);
    const calls = install('needs-relaxed-independence-criteria');
    const { result } = renderHookWithProviders(() => useConnect());

    await act(async () => {
      await result.current();
    });

    expect(requestConfirmation).toHaveBeenCalledOnce();
    const relaxCall = calls.find(
      (c) =>
        c.cmd === 'set_gateway_independence' &&
        (c.payload as { enabled?: boolean }).enabled === false,
    );
    expect(relaxCall).toBeDefined();
    expect(calls.map((c) => c.cmd)).toContain('connect');
  });

  it('aborts without connecting when the user declines the warning', async () => {
    seedStore({ gatewayIndependenceNotifications: true });
    requestConfirmation.mockResolvedValue(false);
    const calls = install('needs-relaxed-independence-criteria');
    const { result } = renderHookWithProviders(() => useConnect());

    await act(async () => {
      await result.current();
    });

    expect(requestConfirmation).toHaveBeenCalledOnce();
    expect(calls.map((c) => c.cmd)).not.toContain('connect');
    // never relaxed independence after a decline
    expect(
      calls.some(
        (c) =>
          c.cmd === 'set_gateway_independence' &&
          (c.payload as { enabled?: boolean }).enabled === false,
      ),
    ).toBe(false);
  });

  it('relaxes silently when notifications are disabled', async () => {
    seedStore({ gatewayIndependenceNotifications: false });
    const calls = install('needs-relaxed-independence-criteria');
    const { result } = renderHookWithProviders(() => useConnect());

    await act(async () => {
      await result.current();
    });

    // no prompt, but independence is relaxed and connect still runs
    expect(requestConfirmation).not.toHaveBeenCalled();
    expect(
      calls.some(
        (c) =>
          c.cmd === 'set_gateway_independence' &&
          (c.payload as { enabled?: boolean }).enabled === false,
      ),
    ).toBe(true);
    expect(calls.map((c) => c.cmd)).toContain('connect');
  });
});
