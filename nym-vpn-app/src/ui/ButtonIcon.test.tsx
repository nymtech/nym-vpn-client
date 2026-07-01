import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ButtonIcon from './ButtonIcon';

// `ButtonIcon` pulls `MsIcon` from the `./index` barrel, which loads modules
// that read `window._APP.devMode` and call the Tauri OS plugin's `type()` at
// module-load time. `vi.hoisted`/`vi.mock` are hoisted above the imports so
// the global exists and the plugin is stubbed before that code runs.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
}));

describe('ButtonIcon', () => {
  it('renders the given icon', () => {
    render(<ButtonIcon icon="settings" onClick={vi.fn()} />);

    const icon = screen.getByTestId('button-icon-icon');
    expect(icon).toHaveAttribute('data-test-icon', 'settings');
  });

  it('fires onClick when clicked', async () => {
    const onClick = vi.fn();
    render(<ButtonIcon icon="settings" onClick={onClick} />);

    await userEvent.click(screen.getByTestId('button-icon'));

    expect(onClick).toHaveBeenCalledOnce();
  });

  it('does not fire onClick when disabled', async () => {
    const onClick = vi.fn();
    render(<ButtonIcon icon="settings" onClick={onClick} disabled />);

    const button = screen.getByTestId('button-icon');
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('data-test-disabled', 'true');
    await userEvent.click(button);

    expect(onClick).not.toHaveBeenCalled();
  });

  it('exposes an accessible label and custom test id', () => {
    render(
      <ButtonIcon
        icon="close"
        onClick={vi.fn()}
        aria-label="Close dialog"
        data-testid="close-btn"
      />,
    );

    const button = screen.getByRole('button', { name: 'Close dialog' });
    expect(button).toHaveAttribute('data-testid', 'close-btn');
    expect(screen.getByTestId('close-btn-icon')).toHaveAttribute(
      'data-test-icon',
      'close',
    );
  });
});
