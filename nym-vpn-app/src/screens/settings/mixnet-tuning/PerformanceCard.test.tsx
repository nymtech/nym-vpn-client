import { describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import type {
  MixnetTrafficConfig,
  MixnetTrafficDefaults,
} from '../../../types';
import {
  mockTauriCommands,
  renderWithProviders,
  seedStore,
} from '../../../test/harness';
import { MixnetTrafficConfigProvider } from './context';
import { PerformanceCard } from './PerformanceCard';

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

const defaults: MixnetTrafficDefaults = {
  mixingDelay: { minValue: 0, maxValue: 200, defaultValue: 50 },
  disablePoissonRate: false,
  defaultBackgroundTraffic: { value: 1, multiplier: '1x' },
  defaultContinuousTraffic: { value: 20, throughput: '2 Mbps' },
  allBackgroundTraffic: [{ value: 1, multiplier: '1x' }],
  allContinuousTraffic: [
    { value: 10, throughput: '1 Mbps' },
    { value: 20, throughput: '2 Mbps' },
  ],
};

function seed(config: Partial<MixnetTrafficConfig>) {
  seedStore({
    mixnetTrafficConfig: {
      poissonParameterForLoopCoverStream: 1,
      averagePacketDelay: 50,
      messageSendingAverageDelay: 20,
      disablePoissonRate: false,
      disableBackgroundCoverTraffic: false,
      minMixnodePerformance: null,
      minGatewayMixnetPerformance: null,
      ...config,
    },
    mixnetTrafficDefaults: defaults,
  });
}

function renderCard() {
  return renderWithProviders(
    <MixnetTrafficConfigProvider>
      <PerformanceCard />
    </MixnetTrafficConfigProvider>,
  );
}

describe('PerformanceCard', () => {
  it('renders the speed derived from the selected continuous-traffic rate', () => {
    seed({ messageSendingAverageDelay: 20 });
    mockTauriCommands(() => 0);
    renderCard();

    expect(screen.getByText('Speed')).toBeInTheDocument();
    // continuous rate 20 -> throughput label "2 Mbps"
    expect(screen.getByText('Up to 2 Mbps')).toBeInTheDocument();
  });

  it('renders the latency returned by the backend', async () => {
    seed({ messageSendingAverageDelay: 20 });
    mockTauriCommands((cmd) => (cmd === 'calculate_traffic_latency' ? 42 : 0));
    renderCard();

    await waitFor(() =>
      expect(screen.getByText('At least 42 ms')).toBeInTheDocument(),
    );
  });

  it('requests the latency using the current config', async () => {
    seed({ messageSendingAverageDelay: 10 });
    const commands: string[] = [];
    mockTauriCommands((cmd) => {
      commands.push(cmd);
      return 7;
    });
    renderCard();

    await waitFor(() =>
      expect(commands).toContain('calculate_traffic_latency'),
    );
    // continuous rate 10 -> throughput label "1 Mbps"
    expect(screen.getByText('Up to 1 Mbps')).toBeInTheDocument();
  });
});
