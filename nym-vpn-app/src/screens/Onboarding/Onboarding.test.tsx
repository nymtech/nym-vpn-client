import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { renderWithProviders } from '../../test/harness';
import Onboarding from './Onboarding';

// Embla reaches for `window.matchMedia`, `IntersectionObserver`, and
// `ResizeObserver` when it mounts; jsdom implements none of them, so stub each
// before the carousel activates.
class MockObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
  takeRecords = vi.fn(() => []);
}

beforeAll(() => {
  vi.stubGlobal('IntersectionObserver', MockObserver);
  vi.stubGlobal('ResizeObserver', MockObserver);
  vi.stubGlobal('matchMedia', (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  }));
});

// `Onboarding` pulls `Button`/`ButtonIconNew` from the `../../ui` barrel, which
// loads `DaemonDot` reading `window._APP.devMode` at module-load time;
// `vi.hoisted` runs before the static imports below so the global exists.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const navigate = vi.fn();
vi.mock('../../hooks/useAnimatedNavigate', () => ({
  useAnimatedNavigate: () => navigate,
}));

// The individual slides are covered by their own tests; stub them to keep the
// carousel shell isolated from their assets and timers.
vi.mock('./slides', () => ({
  Welcome: () => <div>slide-welcome</div>,
  Speed: () => <div>slide-speed</div>,
  Tracking: () => <div>slide-tracking</div>,
  ZeroKnowledge: () => <div>slide-zero-knowledge</div>,
}));

describe('Onboarding', () => {
  beforeEach(() => {
    navigate.mockReset();
  });

  it('renders all four slides and the get-started control', () => {
    renderWithProviders(<Onboarding />);

    expect(screen.getByText('slide-welcome')).toBeInTheDocument();
    expect(screen.getByText('slide-speed')).toBeInTheDocument();
    expect(screen.getByText('slide-tracking')).toBeInTheDocument();
    expect(screen.getByText('slide-zero-knowledge')).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Get Started' }),
    ).toBeInTheDocument();
  });

  it('navigates to the welcome screen on get-started', async () => {
    renderWithProviders(<Onboarding />);

    await userEvent.click(screen.getByRole('button', { name: 'Get Started' }));

    expect(navigate).toHaveBeenCalledWith('/welcome');
  });

  it('navigates to root when the close button is clicked', async () => {
    renderWithProviders(<Onboarding />);

    // `ButtonIconNew` renders the icon glyph ("close") as its inline text.
    await userEvent.click(screen.getByRole('button', { name: 'close' }));

    expect(navigate).toHaveBeenCalledWith('/home');
  });
});
