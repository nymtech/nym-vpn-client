import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type {
  MixnetTrafficConfig,
  MixnetTrafficDefaults,
} from '../../../types';
import { renderWithProviders, seedStore } from '../../../test/harness';
import { MixingDelayCard } from './MixingDelayCard';
import { MixnetTrafficConfigProvider } from './context';

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
  allContinuousTraffic: [{ value: 20, throughput: '2 Mbps' }],
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
      <MixingDelayCard />
    </MixnetTrafficConfigProvider>,
  );
}

describe('MixingDelayCard', () => {
  it('renders the mixing-delay slider bounded by the store defaults', () => {
    seed({ averagePacketDelay: 50 });
    renderCard();

    const slider = screen.getByRole('slider', { name: 'Mixing delays' });
    expect(slider).toHaveAttribute('aria-valuenow', '50');
    expect(slider).toHaveAttribute('min', '0');
    expect(slider).toHaveAttribute('max', '200');
  });

  it('shows the standard description when the delay is non-zero', () => {
    seed({ averagePacketDelay: 50 });
    renderCard();

    expect(
      screen.getByText(
        'Adjust timing delays to change how your packets are mixed.',
      ),
    ).toBeInTheDocument();
  });

  it('shows the privacy warning when the delay is zero', () => {
    seed({ averagePacketDelay: 0 });
    renderCard();

    expect(
      screen.getByText(/Timing protection is currently turned off/),
    ).toBeInTheDocument();
  });

  it('updates the delay when the slider is moved with the keyboard', async () => {
    seed({ averagePacketDelay: 50 });
    renderCard();

    const slider = screen.getByRole('slider', { name: 'Mixing delays' });
    slider.focus();
    await userEvent.keyboard('{ArrowRight}');

    expect(slider).toHaveAttribute('aria-valuenow', '51');
  });
});
