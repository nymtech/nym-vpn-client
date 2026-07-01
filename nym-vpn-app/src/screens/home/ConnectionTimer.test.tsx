import { afterEach, describe, expect, it, vi } from 'vitest';
import { act, screen } from '@testing-library/react';
import dayjs from 'dayjs';
import duration from 'dayjs/plugin/duration';
import { initialState } from '../../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../../test/harness';
import ConnectionTimer from './ConnectionTimer';

dayjs.extend(duration);

afterEach(() => {
  seedStore({ ...initialState });
  vi.useRealTimers();
});

describe('ConnectionTimer', () => {
  it('renders nothing until the tunnel is connected', () => {
    seedStore({ state: 'connecting', tunnelConnectedAt: dayjs() });

    renderWithProviders(<ConnectionTimer />);

    expect(screen.queryByTestId('connection-timer')).not.toBeInTheDocument();
  });

  it('renders the elapsed timer once connected', () => {
    seedStore({ state: 'connected', tunnelConnectedAt: dayjs() });

    renderWithProviders(<ConnectionTimer />);

    expect(screen.getByTestId('connection-timer')).toBeInTheDocument();
    expect(screen.getByTestId('connection-time-value')).toHaveTextContent(
      /\d{2}:\d{2}:\d{2}/,
    );
  });

  it('ticks the elapsed value forward on its interval', () => {
    vi.useFakeTimers();
    // Anchor "now" so the elapsed formatting is deterministic.
    vi.setSystemTime(new Date('2024-01-01T00:00:00Z'));
    seedStore({
      state: 'connected',
      tunnelConnectedAt: dayjs('2024-01-01T00:00:00Z'),
    });

    renderWithProviders(<ConnectionTimer />);
    expect(screen.getByTestId('connection-time-value')).toHaveTextContent(
      '00:00:00',
    );

    act(() => {
      vi.advanceTimersByTime(2000);
    });

    expect(screen.getByTestId('connection-time-value')).toHaveTextContent(
      '00:00:02',
    );
  });
});
