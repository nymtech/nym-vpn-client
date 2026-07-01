import type { ReactElement } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Toast } from '@base-ui/react';
import { initialState } from '../../store/slices/createMainSlice';
import {
  mockTauriCommands,
  renderWithProviders,
  seedStore,
} from '../../test/harness';
import HopSelect from './HopSelect';

// `../../ui` barrel loads `DaemonDot` (`window._APP.devMode`) at module load.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

// `useActionToast` (fired on disabled clicks) needs a base-ui `Toast.Provider`.
function renderHop(ui: ReactElement) {
  return renderWithProviders(<Toast.Provider>{ui}</Toast.Provider>);
}

beforeEach(() => {
  mockTauriCommands(() => null);
});

afterEach(() => {
  seedStore({ ...initialState });
});

describe('HopSelect', () => {
  it('labels the entry hop', () => {
    renderHop(
      <HopSelect
        nodeHop="entry"
        node="random"
        gatewayId={null}
        onClick={vi.fn()}
      />,
    );

    expect(screen.getByText('Entry')).toBeInTheDocument();
    // A 'random' selection renders the localized "Random" name.
    expect(screen.getByText('Random')).toBeInTheDocument();
  });

  it('labels the exit hop', () => {
    renderHop(
      <HopSelect
        nodeHop="exit"
        node="random"
        gatewayId={null}
        onClick={vi.fn()}
      />,
    );

    expect(screen.getByText('Exit')).toBeInTheDocument();
  });

  it('fires onClick when enabled and clicked', async () => {
    const onClick = vi.fn();
    const user = userEvent.setup();
    renderHop(
      <HopSelect
        nodeHop="entry"
        node="random"
        gatewayId={null}
        onClick={onClick}
      />,
    );

    await user.click(screen.getByText('Random'));

    expect(onClick).toHaveBeenCalledOnce();
  });

  it('does not fire onClick when disabled', async () => {
    // A non-idle state makes the disabled click surface a toast instead.
    seedStore({ state: 'connected' });
    const onClick = vi.fn();
    const user = userEvent.setup();
    renderHop(
      <HopSelect
        nodeHop="entry"
        node="random"
        gatewayId={null}
        onClick={onClick}
        disabled
      />,
    );

    await user.click(screen.getByText('Random'));

    expect(onClick).not.toHaveBeenCalled();
  });
});
