import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { initialState } from '../../../../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../../../../test/harness';
import UiScaler from './UiScaler';

// The `../../../../ui` barrel loads `DaemonDot`, which reads
// `window._APP.devMode` at module-load time and calls the Tauri OS plugin.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

afterEach(() => {
  seedStore({ ...initialState });
});

describe('UiScaler', () => {
  it('renders the current font size from the store', () => {
    seedStore({ rootFontSize: 16 });
    renderWithProviders(<UiScaler />);

    expect(screen.getByTestId('ui-scaler-value')).toHaveTextContent('16');
  });

  it('exposes a slider with the store font size as its value', () => {
    seedStore({ rootFontSize: 12 });
    renderWithProviders(<UiScaler />);

    expect(screen.getByRole('slider')).toHaveAttribute('aria-valuenow', '12');
  });

  it('reflects the new value when the slider is moved', async () => {
    mockIPC(() => undefined);
    seedStore({ rootFontSize: 14 });
    renderWithProviders(<UiScaler />);

    // base-ui renders the slider as a native `<input type="range">` (exposed
    // with role "slider"); a change event drives its onValueChange handler,
    // which updates the displayed size via the component's local state.
    const slider = screen.getByRole('slider');
    fireEvent.change(slider, { target: { value: '15' } });

    await waitFor(() => {
      expect(screen.getByTestId('ui-scaler-value')).toHaveTextContent('15');
    });
  });

  it('persists the committed font size (not a stale value) on commit', async () => {
    // Regression: a keyboard commit fires change + commit synchronously. The
    // committed handler must receive the NEW size (15), not the previous one.
    const dbSetCalls: (number | undefined)[] = [];
    mockIPC((cmd, payload) => {
      if (cmd === 'db_set') {
        dbSetCalls.push((payload as { value?: number }).value);
      }
      return undefined;
    });
    seedStore({ rootFontSize: 14 });
    renderWithProviders(<UiScaler />);

    const slider = screen.getByRole('slider');
    slider.focus();
    await userEvent.keyboard('{ArrowRight}'); // 14 -> 15, commits synchronously

    await waitFor(() => {
      expect(document.documentElement.style.fontSize).toBe('15px');
    });
    expect(dbSetCalls).toContain(15);
    expect(dbSetCalls).not.toContain(14);
  });
});
