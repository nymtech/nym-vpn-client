import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { CodeDependency } from '../../../types';
import { renderWithProviders, seedStore } from '../../../test/harness';
import Legal from './Legal';

// The `../../../ui` barrel loads `DaemonDot`, which reads `window._APP.devMode`
// at module-load time; `vi.hoisted` runs before the imports so the global and
// OS plugin stub exist before that code executes.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

const navigate = vi.fn();
vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return { ...actual, useNavigate: () => navigate };
});

const openUrl = vi.fn();
vi.mock('@tauri-apps/plugin-opener', () => ({
  openUrl: (url: string) => {
    openUrl(url);
  },
}));

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

describe('Legal', () => {
  it('renders the terms of use and privacy statement entries', () => {
    seedStore({ codeDepsJs: [], codeDepsRust: [] });
    renderWithProviders(<Legal />);

    expect(screen.getByText('Terms of use')).toBeInTheDocument();
    expect(screen.getByText('Privacy statement')).toBeInTheDocument();
  });

  it('hides the licenses group when no dependencies are available', () => {
    seedStore({ codeDepsJs: [], codeDepsRust: [] });
    renderWithProviders(<Legal />);

    expect(screen.queryByText('Licenses (Rust)')).not.toBeInTheDocument();
    expect(screen.queryByText('Licenses (JS)')).not.toBeInTheDocument();
  });

  it('shows only the relevant licenses entry based on available deps', () => {
    seedStore({ codeDepsJs: [], codeDepsRust: [dep('serde')] });
    renderWithProviders(<Legal />);

    expect(screen.getByText('Licenses (Rust)')).toBeInTheDocument();
    expect(screen.queryByText('Licenses (JS)')).not.toBeInTheDocument();
  });

  it('opens the terms of use URL when clicked', async () => {
    seedStore({ codeDepsJs: [], codeDepsRust: [] });
    renderWithProviders(<Legal />);

    await userEvent.click(screen.getByText('Terms of use'));

    expect(openUrl).toHaveBeenCalledOnce();
  });

  it('navigates to the Rust licenses list when its entry is clicked', async () => {
    seedStore({ codeDepsJs: [], codeDepsRust: [dep('serde')] });
    renderWithProviders(<Legal />);

    await userEvent.click(screen.getByText('Licenses (Rust)'));

    expect(navigate).toHaveBeenCalledWith('/settings/legal/licenses-rust');
  });
});
