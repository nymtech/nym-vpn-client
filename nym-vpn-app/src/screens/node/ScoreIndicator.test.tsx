import { describe, expect, it } from 'vitest';
import { render } from '@testing-library/react';
import type { Score } from '../../types';
import { ScoreIndicator } from './ScoreIndicator';

// Each signal-bars SVG uses a distinct fill colour on its bars, so the variant
// can be identified from the rendered markup without relying on a test id.
const fillFor: Record<Exclude<Score, never>, string> = {
  high: '#76FFB1',
  medium: '#FFB400',
  low: '#ED5060',
  offline: '#3A3A3C',
};

function firstFill(container: HTMLElement): string | null {
  return container.querySelector('path')?.getAttribute('fill') ?? null;
}

describe('ScoreIndicator', () => {
  it('renders an svg sized via the size-5 utility', () => {
    const { container } = render(<ScoreIndicator score="high" />);

    const svg = container.querySelector('svg');
    expect(svg).not.toBeNull();
    expect(svg).toHaveClass('size-5');
  });

  it.each<Score>(['offline', 'low', 'medium', 'high'])(
    'renders the %s signal bars variant',
    (score) => {
      const { container } = render(<ScoreIndicator score={score} />);

      expect(firstFill(container)).toBe(fillFor[score]);
    },
  );

  it('defaults to the good (high) variant when no score is given', () => {
    const { container } = render(<ScoreIndicator />);

    expect(firstFill(container)).toBe(fillFor.high);
  });
});
