import { beforeAll, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import type { CodeDependency } from '../../../../types';
import { renderWithProviders, seedStore } from '../../../../test/harness';
import LicenseList from './LicenseList';

// `LicenseList` calls `type()` from the OS plugin at module-load time and pulls
// `DaemonDot` (reads `window._APP.devMode`) via the `ui` barrel; both must be
// stubbed before the imports run, which `vi.hoisted`/`vi.mock` guarantee.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

// `react-window`'s virtualized `List` observes its container size; jsdom does
// not implement ResizeObserver, so provide a no-op stub.
class MockResizeObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeAll(() => {
  vi.stubGlobal('ResizeObserver', MockResizeObserver);
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const dep = (name: string): CodeDependency => ({
  name,
  version: '1.0.0',
  licenses: ['MIT'],
  authors: ['Someone'],
});

describe('LicenseList', () => {
  it('shows the empty-data message when there are no dependencies', () => {
    seedStore({ codeDepsRust: [], rootFontSize: 16 });
    renderWithProviders(<LicenseList language="rust" />);

    expect(screen.getByText('No license data available')).toBeInTheDocument();
    expect(screen.queryByRole('list')).not.toBeInTheDocument();
  });

  it('renders a virtualized list when dependencies are available', () => {
    seedStore({ codeDepsRust: [dep('serde'), dep('tokio')], rootFontSize: 16 });
    renderWithProviders(<LicenseList language="rust" />);

    expect(screen.getByRole('list')).toBeInTheDocument();
    expect(
      screen.queryByText('No license data available'),
    ).not.toBeInTheDocument();
  });

  it('reads the JS dependency list when language is js', () => {
    seedStore({
      codeDepsJs: [],
      codeDepsRust: [dep('serde')],
      rootFontSize: 16,
    });
    renderWithProviders(<LicenseList language="js" />);

    // js list is empty even though the rust list has entries
    expect(screen.getByText('No license data available')).toBeInTheDocument();
  });
});
