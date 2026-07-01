import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { renderWithProviders } from '../../../test/harness';
import LocationDetailsDialog from './LocationDetailsDialog';

// The dialog pulls `Button`/`Dialog`/`MsIcon` from the `../../../ui` barrel,
// which loads `DaemonDot` (reads `window._APP.devMode`) and the Tauri OS plugin
// at module-load time; `vi.hoisted`/`vi.mock` run before the static import.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

describe('LocationDetailsDialog', () => {
  it('renders the entry title and details when open', () => {
    renderWithProviders(
      <LocationDetailsDialog isOpen onClose={vi.fn()} node="entry" />,
    );

    expect(screen.getByTestId('location-details-title')).toHaveTextContent(
      'Choosing entry servers',
    );
    expect(
      screen.getByTestId('location-details-info-icon'),
    ).toBeInTheDocument();
    // Entry hop shows the QUIC section.
    expect(screen.getByTestId('icon-package_2')).toBeInTheDocument();
  });

  it('renders the exit title when opened for the exit hop', () => {
    renderWithProviders(
      <LocationDetailsDialog isOpen onClose={vi.fn()} node="exit" />,
    );

    expect(screen.getByTestId('location-details-title')).toHaveTextContent(
      'Choosing exit locations',
    );
  });

  it('does not render its content when closed', () => {
    renderWithProviders(
      <LocationDetailsDialog isOpen={false} onClose={vi.fn()} node="entry" />,
    );

    expect(
      screen.queryByTestId('location-details-title'),
    ).not.toBeInTheDocument();
  });

  it('calls onClose when the OK button is clicked', async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    renderWithProviders(
      <LocationDetailsDialog isOpen onClose={onClose} node="entry" />,
    );

    await user.click(screen.getByRole('button', { name: 'Ok' }));

    expect(onClose).toHaveBeenCalledOnce();
  });
});
