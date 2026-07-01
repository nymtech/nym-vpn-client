import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import InfoBanner from './InfoBanner';

describe('InfoBanner', () => {
  it('renders the message text and icon glyph', () => {
    render(<InfoBanner text="Heads up" icon="info" variant="info" />);
    expect(screen.getByText('Heads up')).toBeInTheDocument();
    expect(screen.getByTestId('icon-info')).toHaveTextContent('info');
  });

  it('applies info coloring for the info variant', () => {
    render(<InfoBanner text="msg" icon="info" variant="info" />);
    expect(screen.getByText('msg')).toHaveClass('text-status-info');
    expect(screen.getByTestId('icon-info')).toHaveClass('text-status-info');
  });

  it('applies warning coloring for the warning variant', () => {
    render(<InfoBanner text="careful" icon="warning" variant="warning" />);
    expect(screen.getByText('careful')).toHaveClass('text-status-warning');
    expect(screen.getByTestId('icon-warning')).toHaveClass(
      'text-status-warning',
    );
  });

  it('applies error coloring for the error variant', () => {
    render(<InfoBanner text="failed" icon="error" variant="error" />);
    expect(screen.getByText('failed')).toHaveClass('text-status-error');
    expect(screen.getByTestId('icon-error')).toHaveClass('text-status-error');
  });
});
