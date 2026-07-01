import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../../../test/harness';
import LaunchConfirmDialog from './LaunchConfirmDialog';

// The `../../../ui` barrel loads modules reading `window._APP.devMode` and
// calling the Tauri OS plugin's `type()` at module-load time. `vi.hoisted`/
// `vi.mock` run before the imports.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

function setup(
  overrides: Partial<React.ComponentProps<typeof LaunchConfirmDialog>> = {},
) {
  const onConfirm = vi.fn();
  const onCancel = vi.fn();
  renderWithProviders(
    <LaunchConfirmDialog
      isOpen
      appName="Firefox"
      onConfirm={onConfirm}
      onCancel={onCancel}
      {...overrides}
    />,
  );
  return { onConfirm, onCancel };
}

describe('LaunchConfirmDialog', () => {
  it('renders the interpolated title and action buttons when open', () => {
    setup();

    expect(screen.getByText('Launching Firefox')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Launch' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeInTheDocument();
  });

  it('does not render its content when closed', () => {
    setup({ isOpen: false });

    expect(screen.queryByText('Launching Firefox')).not.toBeInTheDocument();
  });

  it('calls onConfirm when the launch button is clicked', async () => {
    const { onConfirm } = setup();

    await userEvent.click(screen.getByRole('button', { name: 'Launch' }));

    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it('calls onCancel when the cancel button is clicked', async () => {
    const { onCancel } = setup();

    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(onCancel).toHaveBeenCalledOnce();
  });
});
