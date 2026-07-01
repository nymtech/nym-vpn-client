import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { renderWithProviders } from '../test/harness';
import ConfirmationDialog from './ConfirmationDialog';

// `ConfirmationDialog` pulls siblings from the `.` barrel, which loads
// `DaemonDot` reading `window._APP.devMode` at module-load time; `vi.hoisted`
// runs before the static import below so the global exists in time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
}));

function setup(
  overrides: Partial<React.ComponentProps<typeof ConfirmationDialog>> = {},
) {
  const onConfirm = vi.fn().mockResolvedValue(undefined);
  const onCancel = vi.fn();
  renderWithProviders(
    <ConfirmationDialog
      icon="settings"
      title="Delete account?"
      description="This cannot be undone."
      confirmButtonText="Delete"
      isOpen
      isLoading={false}
      onConfirm={onConfirm}
      onCancel={onCancel}
      {...overrides}
    />,
  );
  return { onConfirm, onCancel };
}

describe('ConfirmationDialog', () => {
  it('renders the title, description and confirm button when open', () => {
    setup();

    expect(screen.getByText('Delete account?')).toBeInTheDocument();
    expect(screen.getByText('This cannot be undone.')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
  });

  it('does not render its content when closed', () => {
    setup({ isOpen: false });

    expect(screen.queryByText('Delete account?')).not.toBeInTheDocument();
  });

  it('falls back to the translated cancel label when none is given', () => {
    setup();

    expect(screen.getByRole('button', { name: 'Cancel' })).toBeInTheDocument();
  });

  it('uses a custom cancel label when provided', () => {
    setup({ cancelButtonText: 'Keep it' });

    expect(screen.getByRole('button', { name: 'Keep it' })).toBeInTheDocument();
  });

  it('calls onConfirm when the confirm button is clicked', async () => {
    const user = userEvent.setup();
    const { onConfirm } = setup();

    await user.click(screen.getByRole('button', { name: 'Delete' }));

    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it('calls onCancel when the cancel button is clicked', async () => {
    const user = userEvent.setup();
    const { onCancel } = setup();

    await user.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('disables the confirm button while loading', () => {
    setup({ isLoading: true });

    const [confirmButton] = screen.getAllByRole('button');
    expect(confirmButton).toBeDisabled();
  });
});
