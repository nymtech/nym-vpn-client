import type { ReactElement } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Toast } from '@base-ui/react';
import { CardAnimationProvider } from '../../contexts/CardAnimationContext';
import { initialState } from '../../store/slices/createMainSlice';
import {
  mockTauriCommands,
  renderWithProviders,
  seedStore,
} from '../../test/harness';
import { NewBottomComponent } from './NewBottomComponent';

// `../../ui` barrel loads `DaemonDot` (`window._APP.devMode`) at module load.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

// `useConnect` (invoked from the connect button) reads the gateway-independence
// warning context that the shared harness does not provide.
const requestConfirmation = vi.fn<() => Promise<boolean>>();
vi.mock('../../contexts/gatewayIndependence', () => ({
  useGwIndependenceWarning: () => ({
    isOpen: false,
    requestConfirmation,
    accept: () => undefined,
    cancel: () => undefined,
  }),
}));

// `useToast` + `InteractiveCard`/`useAnimatedNavigate` require their providers.
function renderBottom(ui: ReactElement) {
  return renderWithProviders(
    <Toast.Provider>
      <CardAnimationProvider>{ui}</CardAnimationProvider>
    </Toast.Provider>,
  );
}

beforeEach(() => {
  mockTauriCommands(() => null);
});

afterEach(() => {
  seedStore({ ...initialState });
  requestConfirmation.mockReset();
});

describe('NewBottomComponent', () => {
  it('prompts to get started when there is no account', () => {
    seedStore({ account: false, state: 'disconnected', daemonStatus: 'ok' });

    renderBottom(<NewBottomComponent />);

    expect(
      screen.getByRole('button', { name: 'Get started' }),
    ).toBeInTheDocument();
  });

  it('shows the authenticate label when the daemon denied auth', () => {
    seedStore({ account: true, daemonStatus: 'auth-denied' });

    renderBottom(<NewBottomComponent />);

    expect(
      screen.getByRole('button', { name: 'Authenticate' }),
    ).toBeInTheDocument();
  });

  it('shows the connect label when disconnected with an account', () => {
    seedStore({ account: true, state: 'disconnected', daemonStatus: 'ok' });

    renderBottom(<NewBottomComponent />);

    expect(
      screen.getByRole('button', { name: 'Tap to connect' }),
    ).toBeInTheDocument();
  });

  it('disconnects when the connected button is pressed', async () => {
    seedStore({ account: true, state: 'connected', daemonStatus: 'ok' });
    const calls: string[] = [];
    mockTauriCommands((cmd) => {
      calls.push(cmd);
      return null;
    });
    const user = userEvent.setup();
    renderBottom(<NewBottomComponent />);

    await user.click(screen.getByRole('button', { name: 'Tap to disconnect' }));

    await waitFor(() => expect(calls).toContain('disconnect'));
  });

  it('disables the connect button while the daemon is down', () => {
    seedStore({ account: true, state: 'disconnected', daemonStatus: 'down' });

    renderBottom(<NewBottomComponent />);

    expect(
      screen.getByRole('button', { name: 'Tap to connect' }),
    ).toBeDisabled();
  });
});
