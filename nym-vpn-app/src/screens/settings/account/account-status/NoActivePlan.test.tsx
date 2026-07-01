import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../../../test/harness';
import { NoActivePlan } from './NoActivePlan';

// The `../../../../ui` barrel loads `DaemonDot`, which reads
// `window._APP.devMode` at module-load time and calls the Tauri OS plugin.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

describe('NoActivePlan', () => {
  it('renders the no-plan message', () => {
    renderWithProviders(<NoActivePlan />);

    expect(screen.getByText('No active plan')).toBeInTheDocument();
  });

  it('renders the placeholder icon', () => {
    renderWithProviders(<NoActivePlan />);

    expect(screen.getByTestId('icon-remove_moderator')).toBeInTheDocument();
  });
});
