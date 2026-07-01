import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { GatewaySelectionAlgorithm } from '../../types';
import { initialState } from '../../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../../test/harness';
import { NodeRow } from './NodeRow';

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

const navigate = vi.fn();
vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return { ...actual, useNavigate: () => navigate };
});

function seedExplicit(algorithm: GatewaySelectionAlgorithm = 'explicit') {
  seedStore({
    entryNode: 'random',
    exitNode: 'random',
    state: 'disconnected',
    vpnMode: 'wg',
    gatewaySelectionAlgorithmConfig: {
      enableGeoLocation: true,
      gatewaySelectionAlgorithm: algorithm,
    },
  });
}

afterEach(() => {
  seedStore({ ...initialState });
  navigate.mockReset();
});

describe('NodeRow', () => {
  it('renders the entry-server label', () => {
    seedExplicit();
    renderWithProviders(<NodeRow type="entry" />);

    expect(screen.getByText('Entry server')).toBeInTheDocument();
  });

  it('renders the exit-server label', () => {
    seedExplicit();
    renderWithProviders(<NodeRow type="exit" />);

    expect(screen.getByText('Exit server')).toBeInTheDocument();
  });

  it('shows the Random placeholder for an unpicked node in explicit mode', () => {
    seedExplicit();
    renderWithProviders(<NodeRow type="exit" />);

    expect(screen.getByText('Random')).toBeInTheDocument();
  });

  it('is interactive and navigates on click in explicit mode', async () => {
    seedExplicit();
    const user = userEvent.setup();
    renderWithProviders(<NodeRow type="exit" />);

    await user.click(screen.getByRole('button'));

    expect(navigate).toHaveBeenCalledOnce();
  });

  it('locks the entry row (non-interactive) when the daemon owns the pick', () => {
    seedExplicit('auto');
    renderWithProviders(<NodeRow type="entry" />);

    // In auto mode the daemon owns the entry hop, so the row is not a button.
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});
