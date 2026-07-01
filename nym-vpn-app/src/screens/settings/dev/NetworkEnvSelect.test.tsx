import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { renderWithProviders } from '../../../test/harness';
import NetworkEnvSelect from './NetworkEnvSelect';

// `NetworkEnvSelect` pulls `MsIcon` from the `../../../ui` barrel, which loads
// `DaemonDot` reading `window._APP.devMode` and calls the Tauri OS plugin's
// `type()` at module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

describe('NetworkEnvSelect', () => {
  it('renders every network option with the current value selected', () => {
    renderWithProviders(<NetworkEnvSelect current="mainnet" />);

    const select = screen.getByTestId('network-env-select');
    expect(select).toHaveValue('mainnet');
    expect(
      screen.getByTestId('network-env-option-mainnet'),
    ).toBeInTheDocument();
    expect(screen.getByTestId('network-env-option-canary')).toBeInTheDocument();
    expect(
      screen.getByTestId('network-env-option-sandbox'),
    ).toBeInTheDocument();
  });

  it('invokes set_network with the chosen environment on change', async () => {
    const user = userEvent.setup();
    const calls: { cmd: string; payload?: unknown }[] = [];
    mockIPC((cmd, payload) => {
      calls.push({ cmd, payload });
      return undefined;
    });
    renderWithProviders(<NetworkEnvSelect current="mainnet" />);

    await user.selectOptions(
      screen.getByTestId('network-env-select'),
      'sandbox',
    );

    expect(calls).toContainEqual({
      cmd: 'set_network',
      payload: { network: 'sandbox' },
    });
  });

  it('surfaces an error message when set_network fails', async () => {
    const user = userEvent.setup();
    mockIPC((cmd) => {
      if (cmd === 'set_network') {
        // The command surfaces a `BackendError` ({ key, message }); model it as
        // an Error carrying a `key` so the throw is a real Error object.
        throw Object.assign(new Error('nope'), { key: 'boom' });
      }
      return undefined;
    });
    renderWithProviders(<NetworkEnvSelect current="mainnet" />);

    await user.selectOptions(
      screen.getByTestId('network-env-select'),
      'canary',
    );

    expect(await screen.findByTestId('network-env-error')).toHaveTextContent(
      'Failed to set network: boom - nope',
    );
  });
});
