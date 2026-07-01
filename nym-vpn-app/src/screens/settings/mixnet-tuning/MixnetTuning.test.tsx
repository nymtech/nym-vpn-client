import { describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type {
  MixnetTrafficConfig,
  MixnetTrafficDefaults,
} from '../../../types';
import {
  mockTauriCommands,
  renderWithProviders,
  seedStore,
} from '../../../test/harness';
import MixnetTuningWrapper from './MixnetTuning';

// The `../../../ui` barrel loads `DaemonDot`, which reads `window._APP.devMode`
// at module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

// framer-motion's `useInView` (via `PageAnim`) relies on IntersectionObserver.
class MockIntersectionObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
  takeRecords = vi.fn(() => []);
}

const defaults: MixnetTrafficDefaults = {
  mixingDelay: { minValue: 0, maxValue: 200, defaultValue: 50 },
  disablePoissonRate: false,
  defaultBackgroundTraffic: { value: 1, multiplier: '1x' },
  defaultContinuousTraffic: { value: 20, throughput: '2 Mbps' },
  allBackgroundTraffic: [
    { value: 1, multiplier: '1x' },
    { value: 2, multiplier: '2x' },
  ],
  allContinuousTraffic: [
    { value: 10, throughput: '1 Mbps' },
    { value: 20, throughput: '2 Mbps' },
  ],
};

const savedConfig: MixnetTrafficConfig = {
  poissonParameterForLoopCoverStream: 1,
  averagePacketDelay: 50,
  messageSendingAverageDelay: 20,
  disablePoissonRate: false,
  disableBackgroundCoverTraffic: false,
  minMixnodePerformance: null,
  minGatewayMixnetPerformance: null,
};

function seed(config: MixnetTrafficConfig) {
  seedStore({ mixnetTrafficConfig: config, mixnetTrafficDefaults: defaults });
}

describe('MixnetTuning', () => {
  it('renders the tuning cards and action buttons', () => {
    vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);
    seed(savedConfig);
    mockTauriCommands(() => 0);
    renderWithProviders(<MixnetTuningWrapper />);

    expect(
      screen.getByRole('button', { name: 'Save custom settings' }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Reset to default settings' }),
    ).toBeInTheDocument();
  });

  it('disables both actions when the config already matches the saved defaults', () => {
    vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);
    seed(savedConfig);
    mockTauriCommands(() => 0);
    renderWithProviders(<MixnetTuningWrapper />);

    expect(
      screen.getByRole('button', { name: 'Save custom settings' }),
    ).toBeDisabled();
    expect(
      screen.getByRole('button', { name: 'Reset to default settings' }),
    ).toBeDisabled();
  });

  it('enables the reset button when the config differs from the defaults', () => {
    vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);
    seed({ ...savedConfig, averagePacketDelay: 120 });
    mockTauriCommands(() => 0);
    renderWithProviders(<MixnetTuningWrapper />);

    expect(
      screen.getByRole('button', { name: 'Reset to default settings' }),
    ).toBeEnabled();
  });

  it('resets to defaults and then persists them via the backend', async () => {
    vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);
    seed({ ...savedConfig, averagePacketDelay: 120 });
    const commands: string[] = [];
    mockTauriCommands((cmd) => {
      commands.push(cmd);
      return 0;
    });
    renderWithProviders(<MixnetTuningWrapper />);

    // reset moves state back to defaults, which now differs from the saved
    // config -> save becomes enabled
    await userEvent.click(
      screen.getByRole('button', { name: 'Reset to default settings' }),
    );
    const save = screen.getByRole('button', { name: 'Save custom settings' });
    await waitFor(() => expect(save).toBeEnabled());

    await userEvent.click(save);

    await waitFor(() =>
      expect(commands).toContain('set_mixnet_traffic_config'),
    );
  });
});
