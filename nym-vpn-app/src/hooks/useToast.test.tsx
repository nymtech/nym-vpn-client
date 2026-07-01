import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { Toast } from '@base-ui/react';
import useToast from './useToast';

// `useToast` is a thin, typed wrapper over base-ui's `Toast.useToastManager`.
// It requires a `Toast.Provider` in the tree, which the shared harness does not
// supply, so we wrap with one here.
function wrapper({ children }: { children: ReactNode }) {
  return <Toast.Provider>{children}</Toast.Provider>;
}

describe('useToast', () => {
  it('adds a toast and returns its id through the manager', () => {
    const { result } = renderHook(() => useToast(), { wrapper });

    let id = '';
    act(() => {
      id = result.current.add({ title: 'hello', type: 'success' });
    });

    expect(typeof id).toBe('string');
    expect(id.length).toBeGreaterThan(0);
  });

  it('surfaces the added toast on the underlying manager', () => {
    const { result } = renderHook(
      () => ({ toast: useToast(), manager: Toast.useToastManager() }),
      { wrapper },
    );

    act(() => {
      result.current.toast.add({ title: 'visible', type: 'info' });
    });

    expect(
      result.current.manager.toasts.some((t) => t.title === 'visible'),
    ).toBe(true);
  });

  it('closes a toast by id', () => {
    const { result } = renderHook(
      () => ({ toast: useToast(), manager: Toast.useToastManager() }),
      { wrapper },
    );

    let id = '';
    act(() => {
      id = result.current.toast.add({ title: 'closable', type: 'warn' });
    });
    expect(result.current.manager.toasts).toHaveLength(1);

    act(() => {
      result.current.toast.close(id);
    });

    // Once closed the toast begins its exit transition; assert it is no longer
    // reported as an active (non-ending) toast.
    const active = result.current.manager.toasts.filter(
      (t) => t.transitionStatus !== 'ending',
    );
    expect(active).toHaveLength(0);
  });

  it('keeps `add` and `close` referentially stable across renders', () => {
    const { result, rerender } = renderHook(() => useToast(), { wrapper });
    const first = result.current;
    rerender();
    expect(result.current.add).toBe(first.add);
    expect(result.current.close).toBe(first.close);
  });

  it('does not throw when closing an unknown id', () => {
    const { result } = renderHook(() => useToast(), { wrapper });
    expect(() =>
      act(() => {
        result.current.close('does-not-exist');
      }),
    ).not.toThrow();
    // sanity: the manager mock is real, not a stub
    expect(vi.isMockFunction(result.current.add)).toBe(false);
  });
});
