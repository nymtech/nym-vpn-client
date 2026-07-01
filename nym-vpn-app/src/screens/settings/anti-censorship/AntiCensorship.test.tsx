import { afterEach, describe, expect, it, vi } from 'vitest';
import { act, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { initialState } from '../../../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../../../test/harness';
import AntiCensorship from './AntiCensorship';

// The `../../../ui` barrel loads `DaemonDot`, which reads `window._APP.devMode`
// at module-load time and calls the Tauri OS plugin; stub both before import.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const add = vi.fn();

// `AntiCensorship` pulls `useToast` from the hooks barrel; stub it so no toast
// provider is required and error/info toasts can be asserted.
vi.mock('../../../hooks', () => ({
  useToast: () => ({ add, close: vi.fn() }),
}));

afterEach(() => {
  add.mockReset();
  seedStore({
    ...initialState,
    backendFlags: { quic: true, domainFronting: true, zknymCredential: false },
    frontingMode: 'onRetry',
    quic: false,
  });
});

describe('AntiCensorship', () => {
  it('shows the unavailable message when neither feature flag is set', () => {
    seedStore({
      backendFlags: {
        quic: false,
        domainFronting: false,
        zknymCredential: false,
      },
    });
    renderWithProviders(<AntiCensorship />);

    expect(
      screen.getByText('This feature is not available'),
    ).toBeInTheDocument();
    expect(screen.queryByRole('switch')).not.toBeInTheDocument();
  });

  it('renders the QUIC card only when the quic flag is on', () => {
    seedStore({
      backendFlags: {
        quic: true,
        domainFronting: false,
        zknymCredential: false,
      },
    });
    renderWithProviders(<AntiCensorship />);

    expect(screen.getByText('Enhanced connection (QUIC)')).toBeInTheDocument();
  });

  it('hides the QUIC card when only the domainFronting flag is on', () => {
    seedStore({
      backendFlags: {
        quic: false,
        domainFronting: true,
        zknymCredential: false,
      },
    });
    renderWithProviders(<AntiCensorship />);

    expect(
      screen.queryByText('Enhanced connection (QUIC)'),
    ).not.toBeInTheDocument();
    expect(screen.getByText('Stealth API connect')).toBeInTheDocument();
  });

  it('reflects the stored fronting mode on the stealth switch', () => {
    seedStore({
      backendFlags: {
        quic: false,
        domainFronting: true,
        zknymCredential: false,
      },
      frontingMode: 'always',
    });
    renderWithProviders(<AntiCensorship />);

    const stealthCard = screen.getByRole('button', {
      name: /Stealth API connect/,
    });
    expect(within(stealthCard).getByRole('switch')).toBeChecked();
  });

  it('invokes set_fronting_mode with "always" when toggling the stealth switch on', async () => {
    const calls: { cmd: string; payload?: unknown }[] = [];
    mockIPC((cmd, payload) => {
      calls.push({ cmd, payload });
      return undefined;
    });
    seedStore({
      backendFlags: {
        quic: false,
        domainFronting: true,
        zknymCredential: false,
      },
      frontingMode: 'onRetry',
    });
    renderWithProviders(<AntiCensorship />);

    const stealthSwitch = within(
      screen.getByRole('button', { name: /Stealth API connect/ }),
    ).getByRole('switch');
    await act(async () => {
      await userEvent.click(stealthSwitch);
    });

    expect(calls).toContainEqual({
      cmd: 'set_fronting_mode',
      payload: { mode: 'always' },
    });
    expect(add).not.toHaveBeenCalled();
  });

  it('shows an error toast when set_fronting_mode fails', async () => {
    mockIPC((cmd) => {
      if (cmd === 'set_fronting_mode') throw new Error('nope');
      return undefined;
    });
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(vi.fn());
    seedStore({
      backendFlags: {
        quic: false,
        domainFronting: true,
        zknymCredential: false,
      },
      frontingMode: 'onRetry',
    });
    renderWithProviders(<AntiCensorship />);

    const stealthSwitch = within(
      screen.getByRole('button', { name: /Stealth API connect/ }),
    ).getByRole('switch');
    await act(async () => {
      await userEvent.click(stealthSwitch);
    });

    // The switch's onChange and the wrapping card's bubbled onClick both fire
    // the handler, so assert the payload rather than an exact call count.
    expect(add).toHaveBeenCalledWith({
      id: 'fronting-mode-switch-always',
      title: 'Failed to enable Stealth API connect',
      type: 'error',
    });
    errorSpy.mockRestore();
  });

  it('invokes set_quic when toggling the QUIC switch', async () => {
    const calls: { cmd: string; payload?: unknown }[] = [];
    mockIPC((cmd, payload) => {
      calls.push({ cmd, payload });
      return undefined;
    });
    seedStore({
      backendFlags: {
        quic: true,
        domainFronting: false,
        zknymCredential: false,
      },
      quic: false,
    });
    renderWithProviders(<AntiCensorship />);

    const quicSwitch = within(
      screen.getByRole('button', { name: /Enhanced connection/ }),
    ).getByRole('switch');
    await act(async () => {
      await userEvent.click(quicSwitch);
    });

    expect(calls).toContainEqual({
      cmd: 'set_quic',
      payload: { enabled: true },
    });
  });
});
