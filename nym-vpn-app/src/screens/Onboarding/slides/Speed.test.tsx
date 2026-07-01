import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';

import { renderWithProviders } from '../../../test/harness';
import Speed from './Speed';

describe('Onboarding Speed slide', () => {
  it('renders the slide title', () => {
    renderWithProviders(<Speed />);

    // The title spans two lines via `whitespace-pre-line`; match its start.
    expect(
      screen.getByRole('heading', { name: /Speed when you need it/ }),
    ).toBeInTheDocument();
  });

  it('renders the asset and description copy', () => {
    const { container } = renderWithProviders(<Speed />);

    expect(container.querySelector('svg')).toBeInTheDocument();
    expect(screen.getByText(/Fast mode:/)).toBeInTheDocument();
  });
});
