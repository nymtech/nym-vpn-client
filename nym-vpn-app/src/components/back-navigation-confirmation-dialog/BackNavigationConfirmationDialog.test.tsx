import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { TopBarProvider, useTopBar } from '../../contexts/topbar';
import { renderWithProviders } from '../../test/harness';
import BackNavigationConfirmationDialog from './BackNavigationConfirmationDialog';

// The component pulls `ConfirmationDialog` from the `../../ui` barrel, which
// loads `DaemonDot` reading `window._APP.devMode` at module-load time;
// `vi.hoisted` runs before the static import below so the global exists in time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
}));

// The component registers a custom left-nav handler on the TopBar context and
// only opens its dialog when that handler runs. This trigger surfaces the
// registered handler so a test can invoke it like the TopBar back button would.
function BackTrigger() {
  const { customLeftNavHandler } = useTopBar();
  return (
    <button
      type="button"
      onClick={() => customLeftNavHandler?.()}
      data-testid="fire-back"
    >
      back
    </button>
  );
}

function setup(
  props: Partial<
    React.ComponentProps<typeof BackNavigationConfirmationDialog>
  > = {},
) {
  const onConfirm = vi.fn().mockResolvedValue(undefined);
  const onCancel = vi.fn();
  const tree: ReactNode = (
    <TopBarProvider>
      <BackTrigger />
      <BackNavigationConfirmationDialog
        hasUnsavedChanges
        onConfirm={onConfirm}
        onCancel={onCancel}
        {...props}
      />
    </TopBarProvider>
  );
  renderWithProviders(tree);
  return { onConfirm, onCancel };
}

describe('BackNavigationConfirmationDialog', () => {
  it('keeps the dialog closed initially', () => {
    setup();
    expect(screen.queryByText('Save changes?')).not.toBeInTheDocument();
  });

  it('opens the dialog when navigating back with unsaved changes', async () => {
    const user = userEvent.setup();
    setup({ hasUnsavedChanges: true });

    await user.click(screen.getByTestId('fire-back'));

    expect(screen.getByText('Save changes?')).toBeInTheDocument();
    expect(screen.getByText('You have unsaved changes.')).toBeInTheDocument();
  });

  it('does not open the dialog when there are no unsaved changes', async () => {
    const user = userEvent.setup();
    setup({ hasUnsavedChanges: false });

    await user.click(screen.getByTestId('fire-back'));

    expect(screen.queryByText('Save changes?')).not.toBeInTheDocument();
  });

  it('calls onConfirm when the save button is clicked', async () => {
    const user = userEvent.setup();
    const { onConfirm } = setup({ hasUnsavedChanges: true });

    await user.click(screen.getByTestId('fire-back'));
    await user.click(screen.getByRole('button', { name: 'Save changes' }));

    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it('closes the dialog and calls onCancel when cancelled', async () => {
    const user = userEvent.setup();
    const { onCancel } = setup({ hasUnsavedChanges: true });

    await user.click(screen.getByTestId('fire-back'));
    await user.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(onCancel).toHaveBeenCalledOnce();
    // The dialog plays a leave transition, so wait for its title to be removed.
    await waitFor(() =>
      expect(screen.queryByText('Save changes?')).not.toBeInTheDocument(),
    );
  });
});
