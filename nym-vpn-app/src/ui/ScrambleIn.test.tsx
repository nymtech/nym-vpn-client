import { createRef } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { act, render, screen, waitFor } from '@testing-library/react';
import { ScrambleIn, type ScrambleInHandle } from './ScrambleIn';

describe('ScrambleIn', () => {
  it('always exposes the full text to screen readers', () => {
    render(<ScrambleIn text="hello" autoStart={false} />);
    expect(screen.getByText('hello')).toHaveClass('sr-only');
  });

  it('resolves to the final text once the animation completes', async () => {
    const onComplete = vi.fn();
    const { container } = render(
      <ScrambleIn text="hi" scrambleSpeed={1} onComplete={onComplete} />,
    );
    // The animation drives itself via chained setTimeout + state updates, so
    // wait for the real timers to settle every character.
    await waitFor(() => expect(onComplete).toHaveBeenCalledTimes(1));
    // The animated (aria-hidden) copy resolves to the real text once settled.
    const animated = container.querySelector('span[aria-hidden="true"]');
    expect(animated?.textContent).toBe('hi');
  });

  it('fires onStart when the animation begins on mount', () => {
    const onStart = vi.fn();
    render(<ScrambleIn text="ab" onStart={onStart} />);
    expect(onStart).toHaveBeenCalledTimes(1);
  });

  it('can be started and reset imperatively via its ref', () => {
    const onStart = vi.fn();
    const ref = createRef<ScrambleInHandle>();
    render(
      <ScrambleIn ref={ref} text="ok" autoStart={false} onStart={onStart} />,
    );
    expect(onStart).not.toHaveBeenCalled();

    act(() => {
      ref.current?.start();
    });
    expect(onStart).toHaveBeenCalledTimes(1);

    act(() => {
      ref.current?.reset();
    });
    // After reset the sr-only text is still present.
    expect(screen.getByText('ok')).toHaveClass('sr-only');
  });
});
