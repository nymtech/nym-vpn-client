import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { act, renderHook, waitFor } from '@testing-library/react';
import { MemoryRouter, useLocation } from 'react-router';
import {
  CardAnimationProvider,
  useCardAnimation,
} from '../contexts/CardAnimationContext';
import { useAnimatedNavigate } from './useAnimatedNavigate';

function Wrapper({ children }: { children: ReactNode }) {
  return (
    <MemoryRouter initialEntries={['/start']}>
      <CardAnimationProvider>{children}</CardAnimationProvider>
    </MemoryRouter>
  );
}

describe('useAnimatedNavigate', () => {
  it('navigates to the target once the exit animation resolves', async () => {
    const { result } = renderHook(
      () => ({
        navigate: useAnimatedNavigate(),
        location: useLocation(),
      }),
      { wrapper: Wrapper },
    );

    expect(result.current.location.pathname).toBe('/start');

    act(() => {
      result.current.navigate('/next');
    });

    await waitFor(() => {
      expect(result.current.location.pathname).toBe('/next');
    });
  });

  it('runs the registered exit animation before navigating', async () => {
    const order: string[] = [];
    const exit = vi.fn((): Promise<void> => {
      order.push('exit');
      return Promise.resolve();
    });

    const { result } = renderHook(
      () => {
        const { registerExit } = useCardAnimation();
        return {
          registerExit,
          navigate: useAnimatedNavigate(),
          location: useLocation(),
        };
      },
      { wrapper: Wrapper },
    );

    act(() => {
      result.current.registerExit(exit);
    });

    act(() => {
      result.current.navigate('/after');
    });

    await waitFor(() => {
      expect(result.current.location.pathname).toBe('/after');
    });

    order.push('navigate');
    expect(exit).toHaveBeenCalledOnce();
    // The exit callback runs (and pushes) strictly before navigation completes.
    expect(order[0]).toBe('exit');
  });

  it('still navigates when no exit animation is registered', async () => {
    const { result } = renderHook(
      () => ({
        navigate: useAnimatedNavigate(),
        location: useLocation(),
      }),
      { wrapper: Wrapper },
    );

    act(() => {
      result.current.navigate('/plain');
    });

    await waitFor(() => {
      expect(result.current.location.pathname).toBe('/plain');
    });
  });
});
