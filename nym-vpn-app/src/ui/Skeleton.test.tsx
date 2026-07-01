import { describe, expect, it } from 'vitest';
import { render } from '@testing-library/react';
import Skeleton from './Skeleton';

function firstChild(container: HTMLElement) {
  const el = container.firstElementChild;
  if (!el) throw new Error('Skeleton rendered nothing');
  return el;
}

describe('Skeleton', () => {
  it('renders a pulsing placeholder with default (small) rounding', () => {
    const { container } = render(<Skeleton />);
    const el = firstChild(container);
    expect(el).toHaveClass('animate-pulse');
    expect(el).toHaveClass('rounded');
    expect(el).not.toHaveClass('rounded-full');
  });

  it('applies full rounding when rounded="full"', () => {
    const { container } = render(<Skeleton rounded="full" />);
    const el = firstChild(container);
    expect(el).toHaveClass('rounded-full');
    expect(el).not.toHaveClass('rounded');
  });

  it('applies no rounding class when rounded is false', () => {
    const { container } = render(<Skeleton rounded={false} />);
    const el = firstChild(container);
    expect(el).not.toHaveClass('rounded');
    expect(el).not.toHaveClass('rounded-full');
  });

  it('forwards a custom className', () => {
    const { container } = render(<Skeleton className="h-4 w-20" />);
    const el = firstChild(container);
    expect(el).toHaveClass('h-4');
    expect(el).toHaveClass('w-20');
  });
});
