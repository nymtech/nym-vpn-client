import type { Collapsible } from '@base-ui-components/react';
import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';

import FoldButton from './FoldButton';

// `FoldButton` imports `MsIcon` from the `../../../ui` barrel, which loads
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

const state = (open: boolean) => ({ open }) as Collapsible.Root.State;

describe('FoldButton', () => {
  it('renders the chevron icon and forwards html props to the button', () => {
    render(
      <FoldButton
        html={{ 'aria-label': 'Toggle', id: 'fold-1' }}
        state={state(false)}
      />,
    );

    const button = screen.getByRole('button', { name: 'Toggle' });
    expect(button).toHaveAttribute('id', 'fold-1');
    expect(screen.getByTestId('icon-keyboard_arrow_down')).toBeInTheDocument();
  });

  it('does not rotate the chevron when collapsed', () => {
    render(<FoldButton html={{}} state={state(false)} />);

    expect(screen.getByTestId('icon-keyboard_arrow_down')).not.toHaveClass(
      'rotate-180',
    );
  });

  it('rotates the chevron when expanded', () => {
    render(<FoldButton html={{}} state={state(true)} />);

    expect(screen.getByTestId('icon-keyboard_arrow_down')).toHaveClass(
      'rotate-180',
    );
  });
});
