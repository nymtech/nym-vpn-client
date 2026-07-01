import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import type { UiCountry, UiRegion } from '../../../types/node';

import LocationInfo from './LocationInfo';

// `LocationInfo` imports `FlagIcon` from the `../../../ui` barrel, which loads
// `DaemonDot` (reads `window._APP.devMode`) and the Tauri OS plugin at
// module-load time; `vi.hoisted`/`vi.mock` run before the static import.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const country: UiCountry = {
  nodeType: 'country',
  isSelected: false,
  name: 'Germany',
  code: 'DE',
};

const region: UiRegion = {
  nodeType: 'region',
  isSelected: false,
  name: 'West',
  country: { name: 'United States', code: 'US' },
  gateways: [],
  type: 'wg',
  quic: false,
};

describe('LocationInfo', () => {
  it('renders the name and the country flag', () => {
    render(<LocationInfo node={country} name="Germany" />);

    expect(screen.getByTestId('country-name-DE')).toHaveTextContent('Germany');
    expect(screen.getByTestId('country-info-DE')).toBeInTheDocument();
    expect(screen.getByTestId('country-flag-DE')).toBeInTheDocument();
  });

  it('resolves the country from a region node', () => {
    render(<LocationInfo node={region} name="West" hideFlag />);

    // The region's underlying country code (US) drives the test ids.
    expect(screen.getByTestId('country-name-US')).toHaveTextContent('West');
  });

  it('hides the flag when hideFlag is set', () => {
    render(<LocationInfo node={country} name="Germany" hideFlag />);

    expect(screen.queryByTestId('country-flag-DE')).not.toBeInTheDocument();
  });
});
