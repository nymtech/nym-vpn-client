import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import StartupError from './StartupError';

// `StartupError` imports `Button`/`MsIcon` from the `../ui` barrel, which loads
// `DaemonDot` (reads `window._APP.devMode`) and the Tauri OS plugin at
// module-load time; the hoisted global + OS mock satisfy both before import.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const exit = vi.fn<(code: number) => void>();
vi.mock('@tauri-apps/plugin-process', () => ({
  exit: (code: number) => exit(code),
}));

const show = vi.fn().mockResolvedValue(undefined);
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ label: 'error', show }),
}));

const dbLockedError: StartupError = { key: 'db-locked', detail: null };

beforeEach(() => {
  show.mockClear();
  exit.mockClear();
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe('StartupError', () => {
  it('renders the problem heading and close button', () => {
    render(<StartupError error={dbLockedError} theme="light" />);

    expect(screen.getByTestId('startup-error-title')).toHaveTextContent(
      'Problem detected',
    );
    expect(screen.getByRole('button', { name: 'Close' })).toBeInTheDocument();
  });

  it('shows the db-locked message for the db-locked key', () => {
    render(<StartupError error={dbLockedError} theme="light" />);

    expect(screen.getByTestId('startup-error-message')).toHaveTextContent(
      'The application is likely already running.',
    );
  });

  it('shows the internal-error message for the internal key', () => {
    render(
      <StartupError error={{ key: 'internal', detail: null }} theme="dark" />,
    );

    expect(screen.getByTestId('startup-error-message')).toHaveTextContent(
      'Internal error.',
    );
  });

  it('reflects the theme and renders the detail block when a detail is present', () => {
    render(
      <StartupError
        error={{ key: 'db-open', detail: 'sled: io error' }}
        theme="dark"
      />,
    );

    expect(screen.getByTestId('startup-error-container')).toHaveClass('dark');
    expect(screen.getByTestId('startup-error-details')).toHaveTextContent(
      'sled: io error',
    );
  });

  it('omits the detail block when there is no detail', () => {
    render(<StartupError error={dbLockedError} theme="light" />);

    expect(
      screen.queryByTestId('startup-error-details'),
    ).not.toBeInTheDocument();
  });

  it('exits the app when the close button is clicked', async () => {
    render(<StartupError error={dbLockedError} theme="light" />);

    await userEvent.click(screen.getByRole('button', { name: 'Close' }));

    expect(exit).toHaveBeenCalledWith(0);
  });
});
