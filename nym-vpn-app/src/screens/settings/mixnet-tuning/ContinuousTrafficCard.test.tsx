import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type {
  MixnetTrafficConfig,
  MixnetTrafficDefaults,
} from '../../../types';
import { renderWithProviders, seedStore } from '../../../test/harness';
import { ContinuousTrafficCard } from './ContinuousTrafficCard';
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
  allBackgroundTraffic: [
    { value: 1, multiplier: '1x' },
    { value: 2, multiplier: '2x' },
    { value: 3, multiplier: '3x' },
    { value: 4, multiplier: '4x' },
  ],
  allContinuousTraffic: [
    { value: 10, throughput: '1 Mbps' },
    { value: 20, throughput: '2 Mbps' },
    { value: 30, throughput: '3 Mbps' },
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
      <ContinuousTrafficCard />
    </MixnetTrafficConfigProvider>,
  );
}

describe('ContinuousTrafficCard', () => {
  it('renders the continuous-traffic slider when poisson rate is enabled', () => {
    seed({ disablePoissonRate: false });
    renderCard();

    expect(
      screen.getByRole('slider', {
        name: /Cover traffic is always sent/,
      }),
    ).toBeInTheDocument();
  });

  it('renders the background-cover slider when poisson rate is disabled', () => {
    seed({ disablePoissonRate: true });
    renderCard();

    expect(
      screen.getByRole('slider', { name: 'Background cover traffic rate' }),
    ).toBeInTheDocument();
  });

  it('renders one switch in the card header', () => {
    seed({ disablePoissonRate: false });
    renderCard();

    expect(screen.getByRole('switch')).toBeInTheDocument();
  });

  it('updates the continuous-traffic value when a label is clicked', async () => {
    seed({ disablePoissonRate: false, messageSendingAverageDelay: 20 });
    renderCard();

    // clicking the "Low" label (index 0) selects continuousItems[0].value (10)
    await userEvent.click(screen.getByText('Low'));

    // the update lives in the card's reducer state, not the store, so assert
    // the slider now reflects the new value
    expect(
      screen.getByRole('slider', { name: /Cover traffic is always sent/ }),
    ).toHaveAttribute('aria-valuenow', '0');
  });

  it('commits the new slider value (not a stale one) on keyboard commit', async () => {
    // The slider wires `onValueCommitted={setValue}`. A keyboard commit fires
    // change + commit synchronously; if the commit forwarded a stale value the
    // card value would snap back to `before`. With the fix it advances.
    seed({ disablePoissonRate: false });
    renderCard();

    const slider = screen.getByRole('slider', {
      name: /Cover traffic is always sent/,
    });
    const before = Number(slider.getAttribute('aria-valuenow'));
    slider.focus();
    await userEvent.keyboard('{ArrowRight}');

    expect(Number(slider.getAttribute('aria-valuenow'))).toBeGreaterThan(
      before,
    );
  });
});
