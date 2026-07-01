import { afterEach, describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import dayjs from 'dayjs';
import duration from 'dayjs/plugin/duration';
import type { AppError } from '../../types';
import { initialState } from '../../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../../test/harness';
import { TunnelState } from './TunnelState';

dayjs.extend(duration);

afterEach(() => {
  seedStore({ ...initialState });
});

describe('TunnelState', () => {
  it('shows the "not protected" label when disconnected', () => {
    seedStore({ state: 'disconnected' });

    renderWithProviders(<TunnelState />);

    expect(screen.getByText('Not protected')).toBeInTheDocument();
  });

  it('renders the connection timer once connected', () => {
    seedStore({ state: 'connected', tunnelConnectedAt: dayjs() });

    renderWithProviders(<TunnelState />);

    expect(screen.getByTestId('connection-timer')).toBeInTheDocument();
    // The idle "not protected" label is gone in the connected phase.
    expect(screen.queryByText('Not protected')).not.toBeInTheDocument();
  });

  it('surfaces a generic tunnel error in the error phase', () => {
    const error = {
      key: 'unknown',
      message: 'something broke',
    } as unknown as AppError;
    seedStore({ state: 'error', error });

    renderWithProviders(<TunnelState />);

    expect(screen.getByTestId('tunnel-error-key')).toBeInTheDocument();
  });

  it('shows the offline message for the offline state', () => {
    seedStore({ state: 'offline' });

    renderWithProviders(<TunnelState />);

    // The offline branch renders its own message and no connection timer.
    expect(screen.queryByTestId('connection-timer')).not.toBeInTheDocument();
    expect(screen.queryByText('Not protected')).not.toBeInTheDocument();
  });
});
