import type { EmblaCarouselType } from 'embla-carousel';
import { describe, expect, it, vi } from 'vitest';
import { act, render, renderHook, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { DotButton, useDotButton } from './CarouselDotButton';

// A minimal Embla stub exposing only the members `useDotButton` touches.
// `useDotButton` chains `.on(...).on(...)`, so `on`/`off` return the same api.
// Embla's full type has dozens of members and overloaded event handlers, so we
// build just what the hook reads and widen through `unknown` once.
type EmblaStub = {
  scrollSnapList: () => number[];
  selectedScrollSnap: () => number;
  scrollTo: (index: number) => void;
  on: () => EmblaCarouselType;
  off: () => EmblaCarouselType;
};

function makeEmblaApi(overrides: Partial<EmblaStub> = {}): EmblaCarouselType {
  const casted = (): EmblaCarouselType => api as unknown as EmblaCarouselType;
  const api: EmblaStub = {
    scrollSnapList: () => [0, 0.5, 1],
    selectedScrollSnap: () => 1,
    scrollTo: vi.fn(),
    on: vi.fn(casted),
    off: vi.fn(casted),
    ...overrides,
  };
  return casted();
}

describe('DotButton', () => {
  it('renders a button with its children', () => {
    render(<DotButton>dot</DotButton>);

    expect(screen.getByRole('button', { name: 'dot' })).toBeInTheDocument();
  });

  it('forwards click handlers', async () => {
    const onClick = vi.fn();
    render(<DotButton onClick={onClick}>dot</DotButton>);

    await userEvent.click(screen.getByRole('button', { name: 'dot' }));

    expect(onClick).toHaveBeenCalledOnce();
  });
});

describe('useDotButton', () => {
  it('defaults to empty snaps and index 0 without an Embla api', () => {
    const { result } = renderHook(() => useDotButton(undefined));

    expect(result.current.scrollSnaps).toEqual([]);
    expect(result.current.selectedIndex).toBe(0);
  });

  it('reads snaps and the selected index from the Embla api', () => {
    const api = makeEmblaApi();
    const { result } = renderHook(() => useDotButton(api));

    expect(result.current.scrollSnaps).toEqual([0, 0.5, 1]);
    expect(result.current.selectedIndex).toBe(1);
  });

  it('delegates onDotButtonClick to Embla scrollTo', () => {
    const scrollTo = vi.fn();
    const api = makeEmblaApi({ scrollTo });
    const { result } = renderHook(() => useDotButton(api));

    act(() => {
      result.current.onDotButtonClick(2);
    });

    expect(scrollTo).toHaveBeenCalledWith(2);
  });
});
