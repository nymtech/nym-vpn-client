import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  ContactSupportUrl,
  DiscordInviteUrl,
  FaqUrl,
  GitHubIssuesUrl,
} from '../../../constants';
import { renderWithProviders } from '../../../test/harness';
import Support from './Support';

// `Support` pulls `PageAnim` from the `../../../ui` barrel, which loads
// `DaemonDot` reading `window._APP.devMode` and calls the Tauri OS plugin's
// `type()` at module-load time. `vi.hoisted`/`vi.mock` run before the imports.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const openUrl = vi.fn<(url: string) => Promise<void>>();

vi.mock('@tauri-apps/plugin-opener', () => ({
  openUrl: (url: string) => openUrl(url),
}));

afterEach(() => {
  openUrl.mockReset();
});

describe('Support', () => {
  it('renders the scam-protection intro and support links', () => {
    renderWithProviders(<Support />);

    expect(
      screen.getByText('⚠️ Protect yourself from scams'),
    ).toBeInTheDocument();
    expect(screen.getByText('Check the FAQ')).toBeInTheDocument();
    expect(screen.getByText('Open a GitHub issue')).toBeInTheDocument();
  });

  it('opens the FAQ url when the FAQ item is clicked', async () => {
    openUrl.mockResolvedValue();
    renderWithProviders(<Support />);

    await userEvent.click(screen.getByText('Check the FAQ'));

    expect(openUrl).toHaveBeenCalledExactlyOnceWith(FaqUrl);
  });

  it('opens the GitHub issues url when the GitHub item is clicked', async () => {
    openUrl.mockResolvedValue();
    renderWithProviders(<Support />);

    await userEvent.click(screen.getByText('Open a GitHub issue'));

    expect(openUrl).toHaveBeenCalledExactlyOnceWith(GitHubIssuesUrl);
  });

  it('opens distinct urls for the contact and discord items', async () => {
    openUrl.mockResolvedValue();
    renderWithProviders(<Support />);

    await userEvent.click(screen.getByText('Contact us'));
    await userEvent.click(screen.getByText('Join us on Discord'));

    expect(openUrl).toHaveBeenCalledWith(ContactSupportUrl);
    expect(openUrl).toHaveBeenCalledWith(DiscordInviteUrl);
  });
});
