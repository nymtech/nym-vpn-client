import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../test/harness';
import { SelectedNodeDisplay } from './SelectedNodeDisplay';

// The `../../ui`/`../node` barrels load `DaemonDot`, which reads
// `window._APP.devMode` at module-load time; the OS plugin is used elsewhere in
// the barrel too.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

describe('SelectedNodeDisplay', () => {
  it('renders a skeleton placeholder without a country or fastest flag', () => {
    renderWithProviders(<SelectedNodeDisplay name="loading" />);

    // No name text is shown while the skeleton placeholder is up.
    expect(screen.queryByText('loading')).not.toBeInTheDocument();
    expect(screen.queryByTestId('flag-icon-de')).not.toBeInTheDocument();
  });

  it('renders the name and sub-info once a country is known', () => {
    renderWithProviders(
      <SelectedNodeDisplay
        countryCode="de"
        name="Germany"
        subInfo="Berlin (gw-de)"
      />,
    );

    expect(screen.getByText('Germany')).toBeInTheDocument();
    expect(screen.getByText('Berlin (gw-de)')).toBeInTheDocument();
  });

  it('renders the fastest (casino) icon when no country but showFastest is set', () => {
    renderWithProviders(<SelectedNodeDisplay name="Random" showFastest />);

    expect(screen.getByText('Random')).toBeInTheDocument();
    expect(screen.getByTestId('icon-casino')).toHaveAttribute(
      'data-test-icon',
      'casino',
    );
  });

  it('shows the stream-optimized icon when requested', () => {
    renderWithProviders(
      <SelectedNodeDisplay
        countryCode="fr"
        name="France"
        showStreamOptimized
      />,
    );

    expect(screen.getByTestId('icon-smart_display')).toHaveAttribute(
      'data-test-icon',
      'smart_display',
    );
  });
});
