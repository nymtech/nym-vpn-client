import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import type { CodeDependency } from '../../../../types';
import { renderWithProviders } from '../../../../test/harness';
import LicenseDetails from './LicenseDetails';

// The `../../../../ui` barrel loads `DaemonDot`, which reads
// `window._APP.devMode` at module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const license: CodeDependency = {
  name: 'serde',
  version: '1.0.0',
  licenses: ['MIT', 'Apache-2.0'],
  repository: 'https://github.com/serde-rs/serde',
  authors: ['David Tolnay'],
};

function renderWithState(state: unknown) {
  // MemoryRouter accepts a location object as an entry, but the harness narrows
  // `initialEntries` to `string[]`; a minimal cast carries the router state.
  return renderWithProviders(<LicenseDetails />, {
    initialEntries: [{ pathname: '/', state } as unknown as string],
  });
}

describe('LicenseDetails', () => {
  it('renders the license fields from the router location state', () => {
    renderWithState({ license, language: 'rust' });

    expect(screen.getByTestId('license-details-name-value')).toHaveTextContent(
      'serde',
    );
    expect(
      screen.getByTestId('license-details-version-value'),
    ).toHaveTextContent('1.0.0');
    expect(
      screen.getByTestId('license-details-repository-link'),
    ).toHaveAttribute('href', license.repository);
  });

  it('lists every license entry', () => {
    renderWithState({ license, language: 'rust' });

    const list = screen.getByTestId('license-details-licenses-list');
    expect(list.querySelectorAll('li')).toHaveLength(2);
    expect(screen.getByText('MIT')).toBeInTheDocument();
    expect(screen.getByText('Apache-2.0')).toBeInTheDocument();
  });

  it('labels the language as JavaScript when language is js', () => {
    renderWithState({ license, language: 'js' });

    expect(
      screen.getByTestId('license-details-language-value'),
    ).toHaveTextContent('JavaScript');
  });

  it('shows the no-data fallback when there is no license in state', () => {
    renderWithState({ language: 'rust' });

    expect(screen.getByTestId('license-details-no-data')).toBeInTheDocument();
    expect(
      screen.queryByTestId('license-details-content'),
    ).not.toBeInTheDocument();
  });
});
