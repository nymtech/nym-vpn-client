import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../../../test/harness';
import ExcludedRegions from './ExcludedRegions';

// `ExcludedRegions` pulls UI from the `../../../../ui` barrel, which loads
// `DaemonDot` reading `window._APP.devMode` and calls the Tauri OS plugin's
// `type()` at module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

describe('ExcludedRegions', () => {
  it('renders the section title and a disabled add-region button', () => {
    renderWithProviders(<ExcludedRegions countries={[]} />);

    expect(screen.getByText('Excluded regions')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Add region/i })).toBeDisabled();
  });

  it('renders one localized row per excluded country code', () => {
    renderWithProviders(<ExcludedRegions countries={['CN', 'DE']} />);

    // `useLang().getCountryName` resolves ISO codes via Intl.DisplayNames.
    expect(screen.getByText('China')).toBeInTheDocument();
    expect(screen.getByText('Germany')).toBeInTheDocument();
  });
});
