import { Collapsible } from '@base-ui-components/react';
import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { UiCountry, UiRegion } from '../../../types/node';

import { renderWithProviders } from '../../../test/harness';
import RowHeader, { type RowHeaderProps } from './RowHeader';

// `RowHeader` transitively imports the `../../../ui` barrel via its children,
// which loads `DaemonDot` (reads `window._APP.devMode`) and the Tauri OS plugin
// at module-load time; `vi.hoisted`/`vi.mock` run before the static import.
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

function setup(overrides: Partial<RowHeaderProps> = {}) {
  const props: RowHeaderProps = {
    hop: 'entry',
    isSelected: false,
    node: country,
    i18n: 'Germany',
    onClick: vi.fn(),
    open: false,
    ...overrides,
  };
  renderWithProviders(
    <Collapsible.Root>
      <RowHeader {...props} />
    </Collapsible.Root>,
  );
  return props;
}

describe('RowHeader', () => {
  it('renders the localized country name and a flag for a country row', () => {
    setup();

    expect(screen.getByTestId('country-name-DE')).toHaveTextContent('Germany');
    expect(screen.getByTestId('country-flag-DE')).toBeInTheDocument();
  });

  it('renders the region name without a flag for a sub row', () => {
    setup({ node: region, i18n: 'United States', sub: true });

    expect(screen.getByTestId('country-name-US')).toHaveTextContent('West');
    expect(screen.queryByTestId('country-flag-US')).not.toBeInTheDocument();
  });

  it('invokes onClick with the node when the row body is clicked', async () => {
    const user = userEvent.setup();
    const { onClick } = setup();

    await user.click(screen.getByTestId('country-name-DE'));

    expect(onClick).toHaveBeenCalledWith(country);
  });

  it('renders the collapsible fold trigger', () => {
    setup();

    expect(screen.getByTestId('icon-keyboard_arrow_down')).toBeInTheDocument();
  });
});
