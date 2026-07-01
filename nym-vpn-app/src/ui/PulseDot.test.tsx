import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import PulseDot from './PulseDot';

describe('PulseDot', () => {
  it('renders a cornflower (info) dot with default test ids', () => {
    render(<PulseDot color="cornflower" />);
    const wrapper = screen.getByTestId('pulse-dot-cornflower');
    expect(wrapper).toBeInTheDocument();
    expect(wrapper).toHaveAttribute('data-test-color', 'cornflower');
    expect(screen.getByTestId('pulse-dot-cornflower-ping')).toHaveClass(
      'bg-status-info',
    );
    expect(screen.getByTestId('pulse-dot-cornflower-dot')).toHaveClass(
      'bg-status-info',
    );
  });

  it('renders an error-colored dot for the red variant', () => {
    render(<PulseDot color="red" />);
    expect(screen.getByTestId('pulse-dot-red-dot')).toHaveClass(
      'bg-status-error',
    );
  });

  it('renders a warning-colored dot for the yellow variant', () => {
    render(<PulseDot color="yellow" />);
    const dot = screen.getByTestId('pulse-dot-yellow-dot');
    expect(dot).toHaveClass('dark:bg-status-warning');
  });

  it('honors a custom data-testid across ping and dot', () => {
    render(<PulseDot color="red" data-testid="beacon" />);
    expect(screen.getByTestId('beacon')).toBeInTheDocument();
    expect(screen.getByTestId('beacon-ping')).toBeInTheDocument();
    expect(screen.getByTestId('beacon-dot')).toBeInTheDocument();
  });
});
