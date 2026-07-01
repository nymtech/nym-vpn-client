import type { ComponentType, ReactNode } from 'react';
import { act } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';

// `IntroAnim` plays a dotLottie animation; stub the player to a stable marker so
// the test can assert mount/unmount of the splash overlay without loading the
// real WASM-backed renderer.
vi.mock('@lottiefiles/dotlottie-react', () => ({
  DotLottieReact: () => <div data-testid="dotlottie" />,
}));

// Strip `AnimatePresence` exit-animation buffering so the overlay unmounts
// synchronously when the component removes it — otherwise `motion` keeps the
// exiting node mounted while a fake-timer clock is running and never settles.
vi.mock('motion/react', () => ({
  AnimatePresence: ({ children }: { children: ReactNode }) => <>{children}</>,
  motion: {
    div: ({ children }: { children?: ReactNode }) => <div>{children}</div>,
  },
}));

type IntroAnimProps = { theme: 'light' | 'dark' };

// The component gates its one-shot timer behind a module-level `initialized`
// flag, so each test re-imports a fresh module to exercise that first-mount path.
let IntroAnim: ComponentType<IntroAnimProps>;

beforeEach(async () => {
  vi.resetModules();
  IntroAnim = (await import('./IntroAnim')).default;
});

afterEach(() => {
  vi.useRealTimers();
});

describe('IntroAnim', () => {
  it('renders the animation overlay on mount', () => {
    render(<IntroAnim theme="light" />);

    expect(screen.getByTestId('dotlottie')).toBeInTheDocument();
  });

  it('applies the dark class on the root when the theme is dark', () => {
    const { container } = render(<IntroAnim theme="dark" />);

    expect(container.firstChild).toHaveClass('dark');
  });

  it('removes the overlay once the splash duration elapses', () => {
    vi.useFakeTimers();
    render(<IntroAnim theme="light" />);

    expect(screen.getByTestId('dotlottie')).toBeInTheDocument();

    // The one-shot `setTimeout` flips `completed`, which the mocked
    // `AnimatePresence` unmounts synchronously (no exit-animation buffering).
    act(() => {
      vi.advanceTimersByTime(1000);
    });

    expect(screen.queryByTestId('dotlottie')).not.toBeInTheDocument();
  });
});
