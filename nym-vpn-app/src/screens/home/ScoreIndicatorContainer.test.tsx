import { afterEach, describe, expect, it } from 'vitest';
import type { GatewaySelectionAlgorithm } from '../../types';
import { initialState } from '../../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../../test/harness';
import { ScoreIndicatorContainer } from './ScoreIndicatorContainer';

function seedAlgo(algorithm: GatewaySelectionAlgorithm) {
  seedStore({
    gatewaySelectionAlgorithmConfig: {
      enableGeoLocation: true,
      gatewaySelectionAlgorithm: algorithm,
    },
  });
}

afterEach(() => {
  seedStore({ ...initialState });
});

describe('ScoreIndicatorContainer', () => {
  it('renders an svg score indicator for the given score', () => {
    seedAlgo('explicit');
    seedStore({ state: 'disconnected' });

    const { container } = renderWithProviders(
      <ScoreIndicatorContainer score="low" />,
    );

    expect(container.querySelector('svg')).not.toBeNull();
  });

  it('forces a high score in auto mode while not connected', () => {
    // Reference: an explicit "high" score rendering.
    seedAlgo('explicit');
    seedStore({ state: 'disconnected' });
    const high = renderWithProviders(<ScoreIndicatorContainer score="high" />);
    const highHtml = high.container.innerHTML;
    high.unmount();

    // auto + not-connected ignores the passed "low" score and renders "high".
    seedAlgo('auto');
    seedStore({ state: 'disconnected' });
    const forced = renderWithProviders(<ScoreIndicatorContainer score="low" />);

    expect(forced.container.innerHTML).toBe(highHtml);
  });

  it('respects the passed score in auto mode once connected', () => {
    // Reference: an explicit "low" score rendering.
    seedAlgo('explicit');
    seedStore({ state: 'disconnected' });
    const low = renderWithProviders(<ScoreIndicatorContainer score="low" />);
    const lowHtml = low.container.innerHTML;
    low.unmount();

    // Once connected the auto branch no longer overrides — "low" is honoured.
    seedAlgo('auto');
    seedStore({ state: 'connected' });
    const connected = renderWithProviders(
      <ScoreIndicatorContainer score="low" />,
    );

    expect(connected.container.innerHTML).toBe(lowHtml);
  });
});
