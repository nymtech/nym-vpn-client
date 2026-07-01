import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';

import { renderWithProviders } from '../../../test/harness';
import Tracking from './Tracking';

describe('Onboarding Tracking slide', () => {
  it('renders the slide title', () => {
    renderWithProviders(<Tracking />);

    expect(
      screen.getByRole('heading', { name: 'Stop being tracked online' }),
    ).toBeInTheDocument();
  });

  it('renders the asset and description copy', () => {
    const { container } = renderWithProviders(<Tracking />);

    expect(container.querySelector('svg')).toBeInTheDocument();
    expect(screen.getByText(/no-log/)).toBeInTheDocument();
  });
});
