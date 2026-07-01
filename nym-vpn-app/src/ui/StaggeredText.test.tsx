import { beforeAll, describe, expect, it, vi } from 'vitest';
import { render } from '@testing-library/react';
import { StaggeredText } from './StaggeredText';

// framer-motion's `useInView` relies on IntersectionObserver, which jsdom does
// not implement; provide a no-op stub so the component can mount.
class MockIntersectionObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
  takeRecords = vi.fn(() => []);
}

beforeAll(() => {
  vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);
});

describe('StaggeredText', () => {
  it('renders the full text content', () => {
    const { container } = render(<StaggeredText text="Hello" />);
    expect(container.querySelector('p')?.textContent).toBe('Hello');
  });

  it('renders one span per character', () => {
    const { container } = render(<StaggeredText text="abc" />);
    expect(container.querySelectorAll('span')).toHaveLength(3);
  });

  it('forwards a custom className to the paragraph', () => {
    const { container } = render(
      <StaggeredText text="x" className="text-lg" />,
    );
    expect(container.querySelector('p')).toHaveClass('text-lg');
  });
});
