import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { i18n, renderWithProviders } from '../../../../test/harness';
import Lang from './Lang';

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

afterEach(async () => {
  await i18n.changeLanguage('en');
});

describe('Lang', () => {
  it('renders the system entry and one button per supported language', async () => {
    mockIPC(() => null);
    renderWithProviders(<Lang />);

    expect(
      await screen.findByTestId('language-button-system'),
    ).toBeInTheDocument();
    expect(screen.getByTestId('language-button-en')).toBeInTheDocument();
    expect(screen.getByText('Deutsch')).toBeInTheDocument();
    expect(screen.getByText('Français')).toBeInTheDocument();
  });

  it('marks the system entry selected when no language is stored', async () => {
    mockIPC((cmd) => (cmd === 'db_get' ? null : null));
    renderWithProviders(<Lang />);

    await waitFor(() => {
      expect(screen.getByTestId('language-button-system')).toHaveAttribute(
        'data-selected',
        'true',
      );
    });
  });

  it('marks the stored language selected and system unselected', async () => {
    mockIPC((cmd) => (cmd === 'db_get' ? 'en' : null));
    renderWithProviders(<Lang />);

    await waitFor(() => {
      expect(screen.getByTestId('language-button-system')).toHaveAttribute(
        'data-selected',
        'false',
      );
    });
    expect(screen.getByTestId('language-button-en')).toHaveAttribute(
      'data-selected',
      'true',
    );
  });

  it('persists the selected language and applies it to i18n', async () => {
    const setCalls: Record<string, unknown>[] = [];
    mockIPC((cmd, payload) => {
      if (cmd === 'db_set' && payload) {
        setCalls.push(payload as Record<string, unknown>);
      }
      return null;
    });
    renderWithProviders(<Lang />);

    await userEvent.click(await screen.findByTestId('language-button-de'));

    await waitFor(() => {
      expect(setCalls).toContainEqual({ key: 'ui-language', value: 'de' });
    });
    await waitFor(() => {
      expect(i18n.language).toBe('de');
    });
  });

  it('clears the stored preference when the system entry is selected', async () => {
    const delCalls: Record<string, unknown>[] = [];
    mockIPC((cmd, payload) => {
      if (cmd === 'db_del' && payload) {
        delCalls.push(payload as Record<string, unknown>);
      }
      return null;
    });
    renderWithProviders(<Lang />);

    await userEvent.click(await screen.findByTestId('language-button-system'));

    await waitFor(() => {
      expect(delCalls).toContainEqual({ key: 'ui-language' });
    });
  });
});
