import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../../../test/harness';
import { CustomDnsServers } from './CustomDnsServers';
import type { DnsItem } from './DnsItemContent';

// `CustomDnsServers` renders `TextInput`/`Button` from the `../../../ui`
// barrel, which loads `DaemonDot` reading `window._APP.devMode` and calls the
// Tauri OS plugin's `type()` at module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

function renderServers(overrides: {
  customDnsList?: DnsItem[];
  hasUnsavedChanges?: boolean;
  onApplyDns?: (dnsList: string[]) => Promise<void>;
  onListChange?: (dnsList: DnsItem[]) => void;
}) {
  const onApplyDns =
    overrides.onApplyDns ?? vi.fn().mockResolvedValue(undefined);
  const onListChange = overrides.onListChange ?? vi.fn();
  renderWithProviders(
    <CustomDnsServers
      customDnsList={overrides.customDnsList ?? []}
      hasUnsavedChanges={overrides.hasUnsavedChanges ?? false}
      onApplyDns={onApplyDns}
      onListChange={onListChange}
    />,
  );
  return { onApplyDns, onListChange };
}

describe('CustomDnsServers', () => {
  it('renders the list header and entries when the list is non-empty', () => {
    renderServers({
      customDnsList: [{ id: '1.1.1.1', dns: '1.1.1.1' }],
    });

    expect(screen.getByText(/Custom DNS servers \(1\/5\)/)).toBeInTheDocument();
    expect(screen.getByText('1.1.1.1')).toBeInTheDocument();
  });

  it('hides the input row once the maximum number of servers is reached', () => {
    const full: DnsItem[] = [
      '1.1.1.1',
      '1.0.0.1',
      '8.8.8.8',
      '8.8.4.4',
      '9.9.9.9',
    ].map((dns) => ({ id: dns, dns }));
    renderServers({ customDnsList: full });

    expect(
      screen.queryByRole('button', { name: 'Add' }),
    ).not.toBeInTheDocument();
    expect(screen.getByText(/Custom DNS servers \(5\/5\)/)).toBeInTheDocument();
  });

  it('appends a valid entry through onListChange', async () => {
    const user = userEvent.setup();
    const { onListChange } = renderServers({ customDnsList: [] });

    await user.type(screen.getByPlaceholderText(/address/i), '8.8.8.8');
    await user.click(screen.getByRole('button', { name: 'Add' }));

    expect(onListChange).toHaveBeenCalledExactlyOnceWith([
      { id: '8.8.8.8', dns: '8.8.8.8' },
    ]);
  });

  it('rejects a duplicate entry with an error and no list change', async () => {
    const user = userEvent.setup();
    const { onListChange } = renderServers({
      customDnsList: [{ id: '8.8.8.8', dns: '8.8.8.8' }],
    });

    await user.type(screen.getByPlaceholderText(/address/i), '8.8.8.8');
    await user.click(screen.getByRole('button', { name: 'Add' }));

    expect(screen.getByText('Duplicate DNS address')).toBeInTheDocument();
    expect(onListChange).not.toHaveBeenCalled();
  });

  it('rejects an invalid address with an error and no list change', async () => {
    const user = userEvent.setup();
    const { onListChange } = renderServers({ customDnsList: [] });

    await user.type(screen.getByPlaceholderText(/address/i), '999.999.999.999');
    await user.click(screen.getByRole('button', { name: 'Add' }));

    expect(screen.getByText('Invalid DNS address format')).toBeInTheDocument();
    expect(onListChange).not.toHaveBeenCalled();
  });

  it('disables the apply button unless there are unsaved changes', () => {
    renderServers({ customDnsList: [], hasUnsavedChanges: false });

    expect(screen.getByRole('button', { name: 'Save changes' })).toBeDisabled();
  });

  it('invokes onApplyDns with the current list when applied', async () => {
    const user = userEvent.setup();
    const onApplyDns = vi.fn().mockResolvedValue(undefined);
    renderServers({
      customDnsList: [{ id: '1.1.1.1', dns: '1.1.1.1' }],
      hasUnsavedChanges: true,
      onApplyDns,
    });

    await user.click(screen.getByRole('button', { name: 'Save changes' }));

    expect(onApplyDns).toHaveBeenCalledExactlyOnceWith(['1.1.1.1']);
  });
});
