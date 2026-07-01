import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import type { MixnetTrafficDefaults } from '../../../../types';
import { renderWithProviders, seedStore } from '../../../../test/harness';
import MixnetTrafficConfigProvider from './provider';
import { useMixnetTrafficConfig } from './context';

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

function Consumer() {
  const { state, continuousItems, backgroundCoverItems, mixingDelay } =
    useMixnetTrafficConfig();
  return (
    <div>
      <span data-testid="delay">{state.averagePacketDelay}</span>
      <span data-testid="continuous-count">{continuousItems.length}</span>
      <span data-testid="background-count">{backgroundCoverItems.length}</span>
      <span data-testid="max-delay">{mixingDelay.maxValue}</span>
    </div>
  );
}

describe('MixnetTrafficConfigProvider', () => {
  it('provides derived config values built from the store defaults', () => {
    seedStore({
      mixnetTrafficConfig: {
        poissonParameterForLoopCoverStream: null,
        averagePacketDelay: null,
        messageSendingAverageDelay: null,
        disablePoissonRate: false,
        disableBackgroundCoverTraffic: false,
        minMixnodePerformance: null,
        minGatewayMixnetPerformance: null,
      },
      mixnetTrafficDefaults: defaults,
    });

    renderWithProviders(
      <MixnetTrafficConfigProvider>
        <Consumer />
      </MixnetTrafficConfigProvider>,
    );

    // null config fields fall back to the defaults
    expect(screen.getByTestId('delay')).toHaveTextContent('50');
    expect(screen.getByTestId('continuous-count')).toHaveTextContent('2');
    expect(screen.getByTestId('background-count')).toHaveTextContent('2');
    expect(screen.getByTestId('max-delay')).toHaveTextContent('200');
  });
});
