import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../../../test/harness';
import InfoDialog from './InfoDialog';

// The `../../../ui` barrel loads modules reading `window._APP.devMode` and
// calling the Tauri OS plugin's `type()` at module-load time. `vi.hoisted`/
// `vi.mock` run before the imports so the global exists and the plugin is
// stubbed in time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

describe('InfoDialog', () => {
  it('renders the title and dismiss button when open', () => {
    renderWithProviders(<InfoDialog isOpen onClose={vi.fn()} />);

    expect(screen.getByText('Using split tunneling')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Got it!' })).toBeInTheDocument();
  });

  it('does not render its content when closed', () => {
    renderWithProviders(<InfoDialog isOpen={false} onClose={vi.fn()} />);

    expect(screen.queryByText('Using split tunneling')).not.toBeInTheDocument();
  });

  it('calls onClose when the dismiss button is clicked', async () => {
    const onClose = vi.fn();
    renderWithProviders(<InfoDialog isOpen onClose={onClose} />);

    await userEvent.click(screen.getByRole('button', { name: 'Got it!' }));

    expect(onClose).toHaveBeenCalledOnce();
  });
});
