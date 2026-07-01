import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import QuicTag from './QuicTag';

// `QuicTag` imports `MsIcon` from the `../../ui` barrel, which also loads
// `DaemonDot` (reads `window._APP.devMode`) and the Tauri OS plugin at
// module-load time; `vi.hoisted`/`vi.mock` run before the static import.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

describe('QuicTag', () => {
  it('renders the QUIC label and the package icon', () => {
    render(<QuicTag />);

    expect(screen.getByText('QUIC')).toBeInTheDocument();
    expect(screen.getByTestId('icon-package_2')).toHaveAttribute(
      'data-test-icon',
      'package_2',
    );
  });

  it('forwards a custom className onto its container', () => {
    const { container } = render(<QuicTag className="custom-class" />);

    expect(container.firstChild).toHaveClass('custom-class');
  });
});
