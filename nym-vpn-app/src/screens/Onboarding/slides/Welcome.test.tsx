import { describe, expect, it, vi } from 'vitest';
import { act, screen } from '@testing-library/react';

import { renderWithProviders } from '../../../test/harness';
import Welcome from './Welcome';

describe('Onboarding Welcome slide', () => {
  it('renders the slide title', () => {
    renderWithProviders(<Welcome />);

    expect(
      screen.getByRole('heading', { name: 'Welcome to NymVPN' }),
    ).toBeInTheDocument();
  });

  it('reveals the asset after the mount delay', () => {
    vi.useFakeTimers();
    try {
      const { container } = renderWithProviders(<Welcome />);

      // The asset is gated behind a 200ms timeout on mount.
      expect(container.querySelector('svg')).not.toBeInTheDocument();

      act(() => {
        vi.advanceTimersByTime(250);
      });

      expect(container.querySelector('svg')).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });
});
