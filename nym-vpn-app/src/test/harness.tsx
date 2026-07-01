import type { ReactElement, ReactNode } from 'react';
import {
  type RenderHookOptions,
  type RenderOptions,
  render,
  renderHook,
} from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import { MemoryRouter } from 'react-router';
import { mockIPC } from '@tauri-apps/api/mocks';
import i18n from '../i18n/config';
import { type AppStore, useAppStore } from '../store';

export type ProviderOptions = {
  /** Router history to seed a `MemoryRouter` with. Defaults to `['/']`. */
  initialEntries?: string[];
};

/**
 * Wraps children in the providers that virtually every component/hook needs:
 * the shared i18n instance (English resources are bundled synchronously) and a
 * `MemoryRouter`. The Zustand store is a module-level singleton, so it needs no
 * provider — seed it with `seedStore` instead.
 */
function AllProviders({
  children,
  initialEntries = ['/'],
}: {
  children: ReactNode;
  initialEntries?: string[];
}) {
  return (
    <I18nextProvider i18n={i18n}>
      <MemoryRouter initialEntries={initialEntries}>{children}</MemoryRouter>
    </I18nextProvider>
  );
}

/** Render a component under the shared providers. */
export function renderWithProviders(
  ui: ReactElement,
  { initialEntries, ...options }: ProviderOptions & RenderOptions = {},
) {
  return render(ui, {
    wrapper: ({ children }) => (
      <AllProviders initialEntries={initialEntries}>{children}</AllProviders>
    ),
    ...options,
  });
}

/** Render a hook under the shared providers (see `renderWithProviders`). */
export function renderHookWithProviders<Result, Props>(
  hook: (props: Props) => Result,
  {
    initialEntries,
    ...options
  }: ProviderOptions & RenderHookOptions<Props> = {},
) {
  return renderHook(hook, {
    wrapper: ({ children }) => (
      <AllProviders initialEntries={initialEntries}>{children}</AllProviders>
    ),
    ...options,
  });
}

/** Shallow-merge values into the global Zustand store for a test. */
export function seedStore(partial: Partial<AppStore>) {
  useAppStore.setState(partial);
}

/**
 * Install a Tauri IPC mock. The handler receives the command name and payload
 * and returns the mocked response (or a promise). Mocks are cleared after each
 * test by the global setup.
 */
export function mockTauriCommands(
  handler: (cmd: string, payload?: Record<string, unknown>) => unknown,
) {
  mockIPC((cmd, payload) => handler(cmd, payload as Record<string, unknown>));
}

export { i18n };
