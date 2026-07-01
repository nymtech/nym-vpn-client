import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import type { TunnelState } from '../../types';
import { renderWithProviders } from '../../test/harness';
import ConnectionBadge from './ConnectionBadge';

// `PulseDot` (rendered for the connecting/disconnecting states) is imported via
// the `../../ui` barrel, which loads `DaemonDot` — that reads
// `window._APP.devMode` at module-load time. `vi.hoisted`/`vi.mock` run before
// the static import below so the global exists and the OS plugin is stubbed.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

describe('ConnectionBadge', () => {
  it('renders the connected status label and reflects the state attribute', () => {
    renderWithProviders(<ConnectionBadge state="connected" />);

    const badge = screen.getByTestId('connection-badge');
    expect(badge).toHaveAttribute('data-status', 'connected');
    expect(screen.getByTestId('connection-status-text')).toHaveTextContent(
      'Tap to disconnect',
    );
  });

  const labelCases: [TunnelState, string][] = [
    ['disconnected', 'Tap to connect'],
    ['connecting', 'Tap to cancel'],
    ['disconnecting', 'Disconnecting'],
    ['error', 'Error'],
    ['offline', 'No internet'],
  ];

  it.each(labelCases)('shows the %s status text', (state, text) => {
    renderWithProviders(<ConnectionBadge state={state} />);

    expect(screen.getByTestId('connection-status-text')).toHaveTextContent(
      text,
    );
    expect(screen.getByTestId('connection-badge')).toHaveAttribute(
      'data-status',
      state,
    );
  });

  it('renders a pulse dot only while transitioning', () => {
    renderWithProviders(<ConnectionBadge state="connecting" />);
    expect(screen.getByTestId('connection-pulse-dot')).toBeInTheDocument();
  });

  it('omits the pulse dot when settled', () => {
    renderWithProviders(<ConnectionBadge state="connected" />);
    expect(
      screen.queryByTestId('connection-pulse-dot'),
    ).not.toBeInTheDocument();
  });
});
