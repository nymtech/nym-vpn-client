import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import useDebounce from './useDebounce';

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('useDebounce', () => {
  it('invokes the callback only after the delay elapses', () => {
    const cb = vi.fn();
    const { result } = renderHook(() => useDebounce(cb, 200));
    act(() => result.current('a'));
    expect(cb).not.toHaveBeenCalled();
    act(() => vi.advanceTimersByTime(200));
    expect(cb).toHaveBeenCalledExactlyOnceWith('a');
  });

  it('collapses rapid calls into a single trailing invocation', () => {
    const cb = vi.fn();
    const { result } = renderHook(() => useDebounce(cb, 200));
    act(() => {
      result.current('a');
      result.current('b');
      result.current('c');
    });
    act(() => vi.advanceTimersByTime(200));
    expect(cb).toHaveBeenCalledExactlyOnceWith('c');
  });

  it('cancel() drops a pending invocation', () => {
    const cb = vi.fn();
    const { result } = renderHook(() => useDebounce(cb, 200));
    act(() => {
      result.current('a');
      result.current.cancel();
    });
    act(() => vi.advanceTimersByTime(500));
    expect(cb).not.toHaveBeenCalled();
  });

  it('clears any pending timer on unmount', () => {
    const cb = vi.fn();
    const { result, unmount } = renderHook(() => useDebounce(cb, 200));
    act(() => result.current('a'));
    unmount();
    act(() => vi.advanceTimersByTime(500));
    expect(cb).not.toHaveBeenCalled();
  });
});
