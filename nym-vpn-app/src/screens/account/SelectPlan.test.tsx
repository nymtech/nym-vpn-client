import { beforeEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { DeeplinkTimeout } from '../../errors';
import { mockTauriCommands, renderWithProviders } from '../../test/harness';
import SelectPlan from './SelectPlan';

// `SelectPlan` pulls `Button`/`PageAnim`/`Spinner`/`MsIcon` from the `../../ui`
// barrel, which loads `DaemonDot` reading `window._APP.devMode` at module-load
// time; `vi.hoisted` runs before the static imports below so the global exists.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const navigate = vi.fn();
vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return { ...actual, useNavigate: () => navigate };
});

const autologin = vi.fn<() => Promise<void>>();
const closeDialog = vi.fn();
vi.mock('../../contexts/autologin/context', () => ({
  useAutologin: () => ({ autologin, closeDialog }),
}));

const startListening = vi.fn<(timeoutMs?: number) => Promise<string>>();
const add = vi.fn();
vi.mock('../../hooks', () => ({
  useDeepLink: (): { startListening: typeof startListening } => ({
    startListening,
  }),
  useToast: (): { add: typeof add } => ({ add }),
}));

describe('SelectPlan', () => {
  beforeEach(() => {
    navigate.mockReset();
    autologin.mockReset();
    closeDialog.mockReset();
    startListening.mockReset();
    add.mockReset();
    autologin.mockResolvedValue(undefined);
    mockTauriCommands(() => null);
  });

  it('renders the title, feature list and choose-plan button', () => {
    renderWithProviders(<SelectPlan />);

    expect(
      screen.getByRole('heading', { name: 'Choose your subscription plan' }),
    ).toBeInTheDocument();
    expect(screen.getByText('All features included')).toBeInTheDocument();
    expect(screen.getByText('No ads')).toBeInTheDocument();
    expect(screen.getByText('Cancel anytime')).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /Choose plan/ }),
    ).toBeInTheDocument();
  });

  it('runs autologin then closes the dialog and navigates to root', async () => {
    startListening.mockResolvedValue('nym://callback');

    renderWithProviders(<SelectPlan />);

    await userEvent.click(screen.getByRole('button', { name: /Choose plan/ }));

    await waitFor(() => expect(navigate).toHaveBeenCalledWith('/home'));
    expect(autologin).toHaveBeenCalledWith('autologinRenew');
    expect(closeDialog).toHaveBeenCalledOnce();
  });

  it('shows a timeout toast when the deep-link listener times out', async () => {
    startListening.mockRejectedValue(new DeeplinkTimeout());

    renderWithProviders(<SelectPlan />);

    await userEvent.click(screen.getByRole('button', { name: /Choose plan/ }));

    await waitFor(() =>
      expect(add).toHaveBeenCalledWith(
        expect.objectContaining({ type: 'error' }),
      ),
    );
    expect(navigate).not.toHaveBeenCalled();
  });
});
