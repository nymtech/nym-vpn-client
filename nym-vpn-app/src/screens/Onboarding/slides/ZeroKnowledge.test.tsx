import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';

import { renderWithProviders } from '../../../test/harness';
import ZeroKnowledge from './ZeroKnowledge';

describe('Onboarding ZeroKnowledge slide', () => {
  it('renders the slide title', () => {
    renderWithProviders(<ZeroKnowledge />);

    expect(
      screen.getByRole('heading', { name: 'Anonymity from payment to web' }),
    ).toBeInTheDocument();
  });

  it('renders the asset and description copy', () => {
    const { container } = renderWithProviders(<ZeroKnowledge />);

    expect(container.querySelector('svg')).toBeInTheDocument();
    expect(screen.getByText(/Zero-knowledge proofs/)).toBeInTheDocument();
  });
});
