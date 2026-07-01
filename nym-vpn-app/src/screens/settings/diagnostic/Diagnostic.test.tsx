import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { Toast } from '@base-ui/react';
import { renderWithProviders } from '../../../test/harness';
import Diagnostic from './Diagnostic';

// `Diagnostic` pulls UI from the `../../../ui` barrel, which loads `DaemonDot`
// reading `window._APP.devMode` and calls the Tauri OS plugin's `type()` at
// module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

describe('Diagnostic', () => {
  it('renders the run button and hides the report before a run', () => {
    renderWithProviders(
      <Toast.Provider>
        <Diagnostic />
      </Toast.Provider>,
    );

    expect(
      screen.getByRole('button', { name: 'Run Diagnostic' }),
    ).toBeInTheDocument();
    expect(screen.queryByText('Diagnostic report')).not.toBeInTheDocument();
  });

  it('runs the diagnostic and displays the serialized report', async () => {
    const user = userEvent.setup();
    mockIPC((cmd) => {
      if (cmd === 'run_diagnostic') return { status: 'ok', checks: 3 };
      return undefined;
    });
    renderWithProviders(
      <Toast.Provider>
        <Diagnostic />
      </Toast.Provider>,
    );

    await user.click(screen.getByRole('button', { name: 'Run Diagnostic' }));

    expect(await screen.findByText('Diagnostic report')).toBeInTheDocument();
    expect(screen.getByText(/"status": "ok"/)).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Share report' }),
    ).toBeInTheDocument();
  });

  it('shares the report via share_diagnostic once one exists', async () => {
    const user = userEvent.setup();
    const calls: { cmd: string; payload?: unknown }[] = [];
    mockIPC((cmd, payload) => {
      calls.push({ cmd, payload });
      if (cmd === 'run_diagnostic') return { status: 'ok' };
      return undefined;
    });
    renderWithProviders(
      <Toast.Provider>
        <Diagnostic />
      </Toast.Provider>,
    );

    await user.click(screen.getByRole('button', { name: 'Run Diagnostic' }));
    await user.click(
      await screen.findByRole('button', { name: 'Share report' }),
    );

    expect(calls).toContainEqual({
      cmd: 'share_diagnostic',
      payload: { report: { status: 'ok' } },
    });
  });
});
