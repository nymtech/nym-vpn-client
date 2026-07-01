import { beforeEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { mockTauriCommands, renderWithProviders } from '../../test/harness';
import TechnicalOptin from './TechnicalOptin';

// `TechnicalOptin` reads `window._APP.defaultSentry`/`defaultNetstats` at
// module-load time (and the `../../ui` barrel loads `DaemonDot`, which reads
// `devMode`); `vi.hoisted` runs before the static imports below so the global
// exists in time.
vi.hoisted(() => {
  (
    globalThis as unknown as {
      _APP: {
        devMode: boolean;
        defaultSentry: boolean;
        defaultNetstats: boolean;
      };
    }
  )._APP = {
    devMode: true,
    defaultSentry: false,
    defaultNetstats: false,
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

describe('TechnicalOptin', () => {
  beforeEach(() => {
    navigate.mockReset();
    mockTauriCommands(() => null);
  });

  it('renders the heading, both toggles and the continue button', () => {
    renderWithProviders(<TechnicalOptin />);

    expect(
      screen.getByRole('heading', { name: 'Help us improve NymVPN' }),
    ).toBeInTheDocument();
    expect(screen.getByTestId('welcome-monitoring-switch')).toBeInTheDocument();
    expect(screen.getByTestId('welcome-netstats-switch')).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Continue' }),
    ).toBeInTheDocument();
  });

  it('toggles the monitoring switch on click', async () => {
    renderWithProviders(<TechnicalOptin />);

    const monitoring = screen.getByTestId('welcome-monitoring-switch');
    // Defaults follow `_APP.defaultSentry` (false).
    expect(monitoring).toHaveAttribute('aria-checked', 'false');

    await userEvent.click(monitoring);

    expect(monitoring).toHaveAttribute('aria-checked', 'true');
  });

  it('navigates to root when continue is clicked', async () => {
    renderWithProviders(<TechnicalOptin />);

    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));

    await waitFor(() => expect(navigate).toHaveBeenCalledWith('/home'));
  });
});
