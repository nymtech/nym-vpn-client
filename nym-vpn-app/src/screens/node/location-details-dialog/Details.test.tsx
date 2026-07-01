import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';

import { renderWithProviders } from '../../../test/harness';
import { Details } from './Details';

// `Details` imports `Link`/`MsIcon` from the `../../../ui` barrel, which loads
// `DaemonDot` (reads `window._APP.devMode`) and the Tauri OS plugin at
// module-load time; `vi.hoisted`/`vi.mock` run before the static import.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

describe('Details', () => {
  it('shows the QUIC and location sections for the entry hop', () => {
    renderWithProviders(<Details node="entry" />);

    expect(screen.getByTestId('icon-package_2')).toBeInTheDocument();
    expect(screen.getByTestId('icon-location_on')).toBeInTheDocument();
    // The streaming section is entry-only excluded.
    expect(screen.queryByTestId('icon-smart_display')).not.toBeInTheDocument();
  });

  it('shows the streaming and location sections for the exit hop', () => {
    renderWithProviders(<Details node="exit" />);

    expect(screen.getByTestId('icon-smart_display')).toBeInTheDocument();
    expect(screen.getByTestId('icon-location_on')).toBeInTheDocument();
    expect(screen.queryByTestId('icon-package_2')).not.toBeInTheDocument();
  });
});
