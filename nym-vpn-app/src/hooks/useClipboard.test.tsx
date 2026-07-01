import { afterEach, describe, expect, it, vi } from 'vitest';
import { renderHookWithProviders } from '../test/harness';
import useClipboard from './useClipboard';

const writeText = vi.fn<(text: string) => Promise<void>>();
const add = vi.fn();

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  writeText: (text: string) => writeText(text),
}));

// `useClipboard` pulls `useToast` from the hooks barrel; stub it so the toast
// manager provider isn't required and `add` calls can be asserted.
vi.mock('./index', () => ({
  useToast: () => ({ add, close: vi.fn() }),
}));

afterEach(() => {
  writeText.mockReset();
  add.mockReset();
});

describe('useClipboard', () => {
  it('writes the given text to the system clipboard', async () => {
    writeText.mockResolvedValue();
    const { result } = renderHookWithProviders(() => useClipboard());

    await result.current.copy('secret');

    expect(writeText).toHaveBeenCalledExactlyOnceWith('secret');
  });

  it('shows a success toast by default after copying', async () => {
    writeText.mockResolvedValue();
    const { result } = renderHookWithProviders(() => useClipboard());

    await result.current.copy('secret');

    expect(add).toHaveBeenCalledExactlyOnceWith({
      title: 'Copied to clipboard',
      type: 'success',
    });
  });

  it('suppresses the toast when notify is false', async () => {
    writeText.mockResolvedValue();
    const { result } = renderHookWithProviders(() => useClipboard());

    await result.current.copy('secret', false);

    expect(writeText).toHaveBeenCalledExactlyOnceWith('secret');
    expect(add).not.toHaveBeenCalled();
  });

  it('swallows a clipboard write failure without notifying', async () => {
    writeText.mockRejectedValue(new Error('denied'));
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(vi.fn());
    const { result } = renderHookWithProviders(() => useClipboard());

    await expect(result.current.copy('secret')).resolves.toBeUndefined();

    expect(add).not.toHaveBeenCalled();
    expect(errorSpy).toHaveBeenCalledOnce();
    errorSpy.mockRestore();
  });
});
