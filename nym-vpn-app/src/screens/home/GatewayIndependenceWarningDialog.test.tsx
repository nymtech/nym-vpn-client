import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { GwIndependenceWarningContext } from '../../contexts/gatewayIndependence';
import { renderWithProviders } from '../../test/harness';
import GatewayIndependenceWarningDialog from './GatewayIndependenceWarningDialog';

// The dialog reads its open/accept/cancel handlers from a React context that
// the shared harness does not provide; mock the hook so each test can drive it.
const accept = vi.fn();
const cancel = vi.fn();
const useGwIndependenceWarning = vi.fn<() => GwIndependenceWarningContext>();
vi.mock('../../contexts/gatewayIndependence', () => ({
  useGwIndependenceWarning: () => useGwIndependenceWarning(),
}));

// `Dialog` pulls the `../../ui` barrel which loads `DaemonDot`
// (`window._APP.devMode`) at module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

function seedContext(isOpen: boolean) {
  useGwIndependenceWarning.mockReturnValue({
    isOpen,
    requestConfirmation: vi.fn(),
    accept,
    cancel,
  });
}

describe('GatewayIndependenceWarningDialog', () => {
  it('renders the warning content when open', () => {
    seedContext(true);
    renderWithProviders(<GatewayIndependenceWarningDialog />);

    expect(
      screen.getByText('The selected servers are in the same operator family!'),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Connect anyway' }),
    ).toBeInTheDocument();
  });

  it('stays closed when the context reports it is not open', () => {
    seedContext(false);
    renderWithProviders(<GatewayIndependenceWarningDialog />);

    expect(
      screen.queryByText(
        'The selected servers are in the same operator family!',
      ),
    ).not.toBeInTheDocument();
  });

  it('invokes accept when the user confirms', async () => {
    seedContext(true);
    const user = userEvent.setup();
    renderWithProviders(<GatewayIndependenceWarningDialog />);

    await user.click(screen.getByRole('button', { name: 'Connect anyway' }));

    expect(accept).toHaveBeenCalledOnce();
  });

  it('invokes cancel when the user dismisses', async () => {
    seedContext(true);
    const user = userEvent.setup();
    renderWithProviders(<GatewayIndependenceWarningDialog />);

    await user.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(cancel).toHaveBeenCalledOnce();
  });
});
