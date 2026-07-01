import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Button from './Button';

// The loading state renders `Spinner`, which calls `type()` from the Tauri OS
// plugin; stub it so no injected Tauri global is required.
vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
}));

describe('Button', () => {
  it('renders its children', () => {
    render(<Button>Connect</Button>);
    expect(screen.getByRole('button', { name: 'Connect' })).toBeInTheDocument();
  });

  it('fires onClick when clicked', async () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Connect</Button>);

    await userEvent.click(screen.getByRole('button', { name: 'Connect' }));

    expect(onClick).toHaveBeenCalledOnce();
  });

  it('does not fire onClick when disabled', async () => {
    const onClick = vi.fn();
    render(
      <Button onClick={onClick} disabled>
        Connect
      </Button>,
    );

    const button = screen.getByRole('button', { name: 'Connect' });
    expect(button).toBeDisabled();
    await userEvent.click(button);

    expect(onClick).not.toHaveBeenCalled();
  });

  it('renders a spinner instead of children while loading and disables the button', () => {
    render(<Button loading>Connect</Button>);

    expect(screen.getByTestId('button-spinner')).toBeInTheDocument();
    expect(screen.queryByText('Connect')).not.toBeInTheDocument();
    expect(screen.getByRole('button')).toBeDisabled();
  });

  it('applies a custom className', () => {
    render(<Button className="custom-class">Connect</Button>);
    expect(screen.getByRole('button', { name: 'Connect' })).toHaveClass(
      'custom-class',
    );
  });
});
