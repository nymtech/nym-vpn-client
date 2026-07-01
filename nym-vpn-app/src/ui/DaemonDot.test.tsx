import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import DaemonDot from './DaemonDot';

// `DaemonDot` reads `window._APP.devMode` at module-load time, so the global
// must exist before the component module is imported. `vi.hoisted` is lifted
// above all imports at runtime, so the assignment happens first.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

describe('DaemonDot', () => {
  it('renders nothing when the daemon status is auth-denied', () => {
    const { container } = render(<DaemonDot status="auth-denied" />);
    expect(container).toBeEmptyDOMElement();
  });

  it('renders a success-colored dot for an ok status (dev mode)', () => {
    render(<DaemonDot status="ok" />);
    const dot = screen.getByTestId('daemon-dot');
    expect(dot).toBeInTheDocument();
    expect(dot).toHaveAttribute('data-test-status', 'ok');
    expect(dot).toHaveClass('animate-pulse');
    expect(screen.getByTestId('daemon-dot-indicator')).toHaveClass(
      'bg-status-success',
    );
  });

  it('renders a warning-colored fast-pulsing dot for non-compat', () => {
    render(<DaemonDot status="non-compat" />);
    const dot = screen.getByTestId('daemon-dot');
    expect(dot).toHaveClass('animate-pulse-fast');
    expect(screen.getByTestId('daemon-dot-indicator')).toHaveClass(
      'bg-status-warning',
    );
  });

  it('renders an error-colored dot for a down status', () => {
    render(<DaemonDot status="down" />);
    expect(screen.getByTestId('daemon-dot-indicator')).toHaveClass(
      'bg-status-error',
    );
  });

  it('honors a custom data-testid for both wrapper and indicator', () => {
    render(<DaemonDot status="down" data-testid="my-dot" />);
    expect(screen.getByTestId('my-dot')).toBeInTheDocument();
    expect(screen.getByTestId('my-dot-indicator')).toBeInTheDocument();
  });
});
