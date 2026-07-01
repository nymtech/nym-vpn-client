import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { mockTauriCommands, renderWithProviders } from '../test/harness';
import TopBar from './TopBar';

// `TopBar` (via its barrel imports) loads `DaemonDot`, which reads
// `window._APP.devMode` at module-load time; `vi.hoisted` runs before the
// static import below so the global exists in time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
}));

// `StaggeredText` (rendered for string titles) uses framer-motion's
// `useInView`, which relies on `IntersectionObserver`; jsdom lacks it.
class MockIntersectionObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
  takeRecords = vi.fn(() => []);
}

describe('TopBar', () => {
  beforeAll(() => {
    vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);
  });

  beforeEach(() => {
    // `useSystemTheme` queries the webview theme on mount via IPC.
    mockTauriCommands(() => null);
  });

  it('renders the nav bar for a settings route with its title', () => {
    renderWithProviders(<TopBar />, { initialEntries: ['/settings'] });

    const topBar = screen.getByTestId('top-bar');
    expect(topBar).toBeInTheDocument();
    expect(topBar).toHaveAttribute('data-test-route', '/settings');
    // The title renders through `StaggeredText`, which splits the label into
    // per-letter spans; assert on the container's combined text content.
    expect(screen.getByTestId('top-bar-title-container')).toHaveTextContent(
      'Settings',
    );
  });

  it('shows a left navigation button on settings sub-routes', () => {
    renderWithProviders(<TopBar />, { initialEntries: ['/settings/legal'] });

    // `ButtonIconNew` renders the icon glyph as inline text, not an MsIcon.
    expect(
      screen.getByTestId('top-bar-left-button-container'),
    ).toHaveTextContent('keyboard_arrow_left');
  });

  it('renders a right-side settings button on the home route', () => {
    renderWithProviders(<TopBar />, { initialEntries: ['/home'] });

    expect(
      screen.getByTestId('top-bar-right-button-container'),
    ).toHaveTextContent('settings');
  });

  it('marks routes flagged as having no background', () => {
    renderWithProviders(<TopBar />, { initialEntries: ['/home'] });

    expect(screen.getByTestId('top-bar')).toHaveAttribute(
      'data-test-no-background',
      'true',
    );
  });

  it('lets the user activate the left navigation button', async () => {
    const user = userEvent.setup();
    renderWithProviders(<TopBar />, { initialEntries: ['/settings'] });

    // No assertion on navigation itself (MemoryRouter), but the button must be
    // interactive without throwing.
    const leftButton = screen
      .getByTestId('top-bar-left-button-container')
      .querySelector('button');
    expect(leftButton).not.toBeNull();
    await user.click(leftButton!);
  });
});
