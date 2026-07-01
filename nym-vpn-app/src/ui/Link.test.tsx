import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { type Routes } from '../types';
import { renderWithProviders } from '../test/harness';
import Link from './Link';

const settingsRoute = '/settings' as Routes;

const openUrl = vi.fn<(url: string) => Promise<void>>();
const navigate = vi.fn();

vi.mock('@tauri-apps/plugin-opener', () => ({
  openUrl: (url: string) => openUrl(url),
}));

vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return { ...actual, useNavigate: () => navigate };
});

afterEach(() => {
  openUrl.mockReset();
  navigate.mockReset();
});

describe('Link', () => {
  it('renders the given text', () => {
    renderWithProviders(<Link text="Nym docs" />);

    expect(screen.getByText('Nym docs')).toBeInTheDocument();
  });

  it('renders children over text when provided', () => {
    renderWithProviders(<Link text="fallback">custom child</Link>);

    expect(screen.getByText('custom child')).toBeInTheDocument();
    expect(screen.queryByText('fallback')).not.toBeInTheDocument();
  });

  it('renders an icon when icon is set', () => {
    renderWithProviders(<Link text="Nym docs" icon />);

    expect(screen.getByTestId('link-nym-docs-icon')).toHaveAttribute(
      'data-test-icon',
      'open_in_new',
    );
  });

  it('opens an external url on click', async () => {
    openUrl.mockResolvedValue();
    renderWithProviders(<Link text="Nym docs" url="https://nym.com" />);

    await userEvent.click(screen.getByTestId('link-nym-docs'));

    expect(openUrl).toHaveBeenCalledExactlyOnceWith('https://nym.com');
    expect(navigate).not.toHaveBeenCalled();
  });

  it('navigates to an internal route on click', async () => {
    renderWithProviders(<Link text="Settings" to={settingsRoute} />);

    await userEvent.click(screen.getByTestId('link-settings'));

    expect(navigate).toHaveBeenCalledExactlyOnceWith(settingsRoute);
    expect(openUrl).not.toHaveBeenCalled();
  });
});
