import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import MsIcon from './MsIcon';

describe('MsIcon', () => {
  it('renders the icon glyph name as its text content', () => {
    render(<MsIcon icon="settings" />);
    const icon = screen.getByTestId('icon-settings');
    expect(icon).toHaveTextContent('settings');
    expect(icon).toHaveAttribute('data-test-icon', 'settings');
  });

  it('is unfilled by default (no FILL variation style)', () => {
    render(<MsIcon icon="home" />);
    expect(screen.getByTestId('icon-home')).not.toHaveStyle({
      fontVariationSettings: "'FILL' 1",
    });
  });

  it('applies the filled font variation when filled', () => {
    render(<MsIcon icon="home" filled />);
    expect(screen.getByTestId('icon-home')).toHaveStyle({
      fontVariationSettings: "'FILL' 1",
    });
  });

  it('forwards a custom className and data-testid', () => {
    render(<MsIcon icon="close" className="text-red" data-testid="close-x" />);
    const icon = screen.getByTestId('close-x');
    expect(icon).toHaveClass('text-red');
    expect(icon).toHaveClass('font-icon');
  });
});
