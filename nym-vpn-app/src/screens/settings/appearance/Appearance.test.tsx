import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../../../test/harness';
import Appearance from './Appearance';

// The `../../../ui` barrel loads `DaemonDot`, which reads `window._APP.devMode`
// at module-load time and calls the Tauri OS plugin; stub both before import.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const navigate = vi.fn();

vi.mock('react-router', async () => {
  const actual =
    await vi.importActual<typeof import('react-router')>('react-router');
  return { ...actual, useNavigate: () => navigate };
});

describe('Appearance', () => {
  it('renders the language and display entries', () => {
    renderWithProviders(<Appearance />);

    expect(screen.getByText('Language')).toBeInTheDocument();
    expect(screen.getByText('Display mode')).toBeInTheDocument();
  });

  it('navigates to the language screen when the language entry is clicked', async () => {
    renderWithProviders(<Appearance />);

    await userEvent.click(screen.getByRole('button', { name: /Language/ }));

    expect(navigate).toHaveBeenCalledExactlyOnceWith(
      '/settings/appearance/lang',
    );
  });

  it('navigates to the display screen when the display entry is clicked', async () => {
    renderWithProviders(<Appearance />);

    await userEvent.click(screen.getByRole('button', { name: /Display mode/ }));

    expect(navigate).toHaveBeenCalledExactlyOnceWith(
      '/settings/appearance/display',
    );
  });
});
