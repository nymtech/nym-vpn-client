import type { ReactElement } from 'react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Toast } from '@base-ui/react';
import { initialState } from '../../store/slices/createMainSlice';
import {
  mockTauriCommands,
  renderWithProviders,
  seedStore,
} from '../../test/harness';
import { ModeToggle } from './ModeToggle';

// The `../../assets` gateway icons and the store barrel are safe, but `useToast`
// needs a base-ui `Toast.Provider` around the component.
function renderToggle(ui: ReactElement) {
  return renderWithProviders(<Toast.Provider>{ui}</Toast.Provider>);
}

beforeEach(() => {
  mockTauriCommands(() => null);
});

afterEach(() => {
  seedStore({ ...initialState });
});

describe('ModeToggle', () => {
  it('renders both mode options', () => {
    seedStore({ vpnMode: 'wg' });
    renderToggle(<ModeToggle />);

    expect(screen.getByRole('button', { name: /Fast/ })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Mixnet/ })).toBeInTheDocument();
  });

  it('does not re-apply when the already-selected mode is clicked', async () => {
    seedStore({ vpnMode: 'wg' });
    const calls: string[] = [];
    mockTauriCommands((cmd) => {
      calls.push(cmd);
      return null;
    });
    const user = userEvent.setup();
    renderToggle(<ModeToggle />);

    await user.click(screen.getByRole('button', { name: /Fast/ }));

    expect(calls).not.toContain('set_vpn_mode');
  });

  it('applies the algorithm then the vpn mode when switching to mixnet', async () => {
    seedStore({ vpnMode: 'wg' });
    const calls: { cmd: string; payload?: Record<string, unknown> }[] = [];
    mockTauriCommands((cmd, payload) => {
      calls.push({ cmd, payload });
      return null;
    });
    const user = userEvent.setup();
    renderToggle(<ModeToggle />);

    await user.click(screen.getByRole('button', { name: /Mixnet/ }));

    await waitFor(() =>
      expect(calls.map((c) => c.cmd)).toContain('set_vpn_mode'),
    );
    const order = calls.map((c) => c.cmd);
    expect(order.indexOf('set_gateway_selection_algorithm')).toBeLessThan(
      order.indexOf('set_vpn_mode'),
    );
    const vpnCall = calls.find((c) => c.cmd === 'set_vpn_mode');
    expect(vpnCall?.payload).toEqual({ mode: 'mixnet' });
  });

  it('switches back to wg mode when Fast is picked from mixnet', async () => {
    seedStore({ vpnMode: 'mixnet' });
    const calls: { cmd: string; payload?: Record<string, unknown> }[] = [];
    mockTauriCommands((cmd, payload) => {
      calls.push({ cmd, payload });
      return null;
    });
    const user = userEvent.setup();
    renderToggle(<ModeToggle />);

    await user.click(screen.getByRole('button', { name: /Fast/ }));

    await waitFor(() =>
      expect(calls.map((c) => c.cmd)).toContain('set_vpn_mode'),
    );
    const vpnCall = calls.find((c) => c.cmd === 'set_vpn_mode');
    expect(vpnCall?.payload).toEqual({ mode: 'wg' });
  });
});
