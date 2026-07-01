import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import IntroSplash from './IntroSplash';

// `IntroSplash` renders the `NymSplash` SVG asset; stub it to a stable marker so
// the test asserts the splash chrome without depending on the SVG's internals.
vi.mock('../assets', () => ({
  NymSplash: ({ className }: { className?: string }) => (
    <svg data-testid="nym-splash" className={className} />
  ),
}));

describe('IntroSplash', () => {
  it('renders the splash logo', () => {
    render(<IntroSplash theme="light" />);

    expect(screen.getByTestId('nym-splash')).toBeInTheDocument();
  });

  it('applies the dark class on the root when the theme is dark', () => {
    const { container } = render(<IntroSplash theme="dark" />);

    expect(container.firstChild).toHaveClass('dark');
  });

  it('does not apply the dark class when the theme is light', () => {
    const { container } = render(<IntroSplash theme="light" />);

    expect(container.firstChild).not.toHaveClass('dark');
  });
});
