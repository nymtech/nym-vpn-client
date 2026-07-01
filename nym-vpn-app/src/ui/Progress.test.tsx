import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import Progress from './Progress';

describe('Progress', () => {
  it('renders a progressbar reflecting the given value', () => {
    render(<Progress value={42} label="Downloading" />);
    const bar = screen.getByRole('progressbar');
    expect(bar).toHaveAttribute('aria-valuenow', '42');
    expect(bar).toHaveAttribute('aria-valuetext', '42%');
    expect(screen.getByText('Downloading')).toBeInTheDocument();
    expect(screen.getByText('42%')).toBeInTheDocument();
  });

  it('falls back to a default label when none is provided', () => {
    render(<Progress value={10} />);
    expect(screen.getByText('Progress')).toBeInTheDocument();
  });

  it('renders an indeterminate bar when value is null', () => {
    render(<Progress />);
    const bar = screen.getByRole('progressbar');
    expect(bar).not.toHaveAttribute('aria-valuenow');
    expect(bar).toHaveAttribute('aria-valuetext', 'indeterminate progress');
  });

  it('forwards a custom className to the root', () => {
    render(<Progress value={5} className="my-progress" />);
    expect(screen.getByRole('progressbar')).toHaveClass('my-progress');
  });
});
