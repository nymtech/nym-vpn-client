import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Switch from './Switch';

describe('Switch', () => {
  it('renders an unchecked switch', () => {
    render(<Switch checked={false} onChange={vi.fn()} />);

    const toggle = screen.getByRole('switch');
    expect(toggle).toHaveAttribute('aria-checked', 'false');
    expect(toggle).toHaveAttribute('data-test-checked', 'false');
  });

  it('reflects the checked state', () => {
    render(<Switch checked onChange={vi.fn()} />);

    const toggle = screen.getByRole('switch');
    expect(toggle).toHaveAttribute('aria-checked', 'true');
    expect(toggle).toHaveAttribute('data-test-checked', 'true');
  });

  it('calls onChange with the toggled value when clicked', async () => {
    const onChange = vi.fn();
    render(<Switch checked={false} onChange={onChange} />);

    await userEvent.click(screen.getByRole('switch'));

    expect(onChange).toHaveBeenCalledExactlyOnceWith(true);
  });

  it('does not call onChange when disabled', async () => {
    const onChange = vi.fn();
    render(<Switch checked={false} onChange={onChange} disabled />);

    const toggle = screen.getByRole('switch');
    expect(toggle).toBeDisabled();
    await userEvent.click(toggle);

    expect(onChange).not.toHaveBeenCalled();
  });
});
