import { beforeEach, describe, expect, it, vi } from 'vitest';
import { act, waitFor } from '@testing-library/react';
import {
  mockTauriCommands,
  renderHookWithProviders,
  seedStore,
} from '../test/harness';
import useGatewayIndependenceWatcher from './useGatewayIndependenceWatcher';

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

function install() {
  const calls: string[] = [];
  mockTauriCommands((cmd) => {
    calls.push(cmd);
    return undefined;
  });
  return calls;
}

// Flip the store into the relaxed-independence error state and let the watcher's
// async handler start before assertions run.
async function raiseError() {
  await act(async () => {
    seedStore({ tunnelError: 'needs-relaxed-independence-criteria' });
    await Promise.resolve();
  });
}

beforeEach(() => {
  requestConfirmation.mockReset();
  seedStore({ tunnelError: null, gatewayIndependenceNotifications: true });
});

describe('useGatewayIndependenceWatcher', () => {
  it('does nothing while there is no relaxed-independence error', async () => {
    const calls = install();
    renderHookWithProviders(() => useGatewayIndependenceWatcher());

    // give any (unexpected) async handler a chance to run
    await act(async () => {
      await Promise.resolve();
    });
    expect(calls).toHaveLength(0);
    expect(requestConfirmation).not.toHaveBeenCalled();
  });

  it('prompts, relaxes and reconnects when the user accepts (notifications ON)', async () => {
    requestConfirmation.mockResolvedValue(true);
    const calls = install();
    renderHookWithProviders(() => useGatewayIndependenceWatcher());

    await raiseError();

    await waitFor(() => expect(calls).toContain('reconnect'));
    expect(requestConfirmation).toHaveBeenCalledOnce();
    expect(calls).toContain('set_gateway_independence');
  });

  it('stays in error without reconnecting when the user declines', async () => {
    requestConfirmation.mockResolvedValue(false);
    const calls = install();
    renderHookWithProviders(() => useGatewayIndependenceWatcher());

    await raiseError();

    await waitFor(() => expect(requestConfirmation).toHaveBeenCalledOnce());
    // declined: neither relax nor reconnect
    expect(calls).not.toContain('set_gateway_independence');
    expect(calls).not.toContain('reconnect');
  });

  it('relaxes silently and reconnects when notifications are OFF', async () => {
    seedStore({ gatewayIndependenceNotifications: false });
    const calls = install();
    renderHookWithProviders(() => useGatewayIndependenceWatcher());

    await raiseError();

    await waitFor(() => expect(calls).toContain('reconnect'));
    expect(requestConfirmation).not.toHaveBeenCalled();
    expect(calls).toContain('set_gateway_independence');
  });

  it('handles a failing daemon call without crashing', async () => {
    seedStore({ gatewayIndependenceNotifications: false });
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(noop);
    mockTauriCommands(() => {
      throw new Error('daemon down');
    });
    renderHookWithProviders(() => useGatewayIndependenceWatcher());

    await raiseError();

    await waitFor(() =>
      expect(errorSpy).toHaveBeenCalledWith(
        'gateway independence watcher failed',
        expect.anything(),
      ),
    );
    errorSpy.mockRestore();
  });
});
