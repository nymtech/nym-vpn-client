import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import Error from './Error';

type RouteError = { statusText: string; message: string };

// `Error` reads the active route error via `useRouteError`; mock it so we can
// drive both the `statusText` and `message` branches without a full data router.
const routeError = vi.fn<() => RouteError>();
vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return { ...actual, useRouteError: () => routeError() };
});

describe('Error', () => {
  it('renders the generic error copy', () => {
    routeError.mockReturnValue({ statusText: '', message: '' });

    render(<Error />);

    expect(screen.getByText('Oops!')).toBeInTheDocument();
    expect(
      screen.getByText('Sorry, an unexpected error has occurred.'),
    ).toBeInTheDocument();
  });

  it('prefers the status text when present', () => {
    routeError.mockReturnValue({ statusText: 'Not Found', message: 'boom' });

    render(<Error />);

    expect(screen.getByText('Not Found')).toBeInTheDocument();
  });

  it('falls back to the message when there is no status text', () => {
    routeError.mockReturnValue({ statusText: '', message: 'boom' });

    render(<Error />);

    expect(screen.getByText('boom')).toBeInTheDocument();
  });
});
