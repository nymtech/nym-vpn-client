import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';

import { renderWithProviders } from '../../test/harness';
import { ToastIcon } from './ToastIcon';

// `ToastIcon` pulls `MsIcon` from the `../../ui` barrel, which loads `DaemonDot`
// reading `window._APP.devMode` at module-load time; `vi.hoisted` runs before
// the static import below so the global exists in time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
}));

describe('ToastIcon', () => {
  it('renders the error icon for the error type', () => {
    renderWithProviders(<ToastIcon type="error" />);
    expect(screen.getByTestId('icon-error')).toBeInTheDocument();
  });

  it('renders the warning icon for the warn type', () => {
    renderWithProviders(<ToastIcon type="warn" />);
    expect(screen.getByTestId('icon-fmd_bad')).toBeInTheDocument();
  });

  it('renders the info icon for the info type', () => {
    renderWithProviders(<ToastIcon type="info" />);
    expect(screen.getByTestId('icon-info')).toBeInTheDocument();
  });

  it('renders the success icon for the success type', () => {
    renderWithProviders(<ToastIcon type="success" />);
    expect(screen.getByTestId('icon-check_circle')).toBeInTheDocument();
  });

  it('falls back to the info icon when no type is given', () => {
    renderWithProviders(<ToastIcon />);
    expect(screen.getByTestId('icon-info')).toBeInTheDocument();
  });
});
