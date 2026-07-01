import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DnsItemContent } from './DnsItemContent';

// `DnsItemContent` renders `ButtonIcon` from the `../../../ui` barrel, which
// loads `DaemonDot` reading `window._APP.devMode` and calls the Tauri OS
// plugin's `type()` at module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

describe('DnsItemContent', () => {
  it('renders the DNS address and the drag handle', () => {
    render(
      <DnsItemContent
        item={{ id: '1.1.1.1', dns: '1.1.1.1' }}
        dragHandle={<span data-testid="drag-handle">handle</span>}
        onDelete={vi.fn()}
      />,
    );

    expect(screen.getByText('1.1.1.1')).toBeInTheDocument();
    expect(screen.getByTestId('drag-handle')).toBeInTheDocument();
  });

  it('calls onDelete with the item id when the delete button is clicked', async () => {
    const user = userEvent.setup();
    const onDelete = vi.fn();
    render(
      <DnsItemContent
        item={{ id: '1.1.1.1', dns: '1.1.1.1' }}
        dragHandle={<span>handle</span>}
        onDelete={onDelete}
      />,
    );

    await user.click(screen.getByTestId('button-icon'));

    expect(onDelete).toHaveBeenCalledExactlyOnceWith('1.1.1.1');
  });
});
