import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../test/harness';
import BetaPill from './BetaPill';

describe('BetaPill', () => {
  it('renders the translated beta label', () => {
    renderWithProviders(<BetaPill />);
    expect(screen.getByText('Beta')).toBeInTheDocument();
  });

  it('applies a passed className alongside its base classes', () => {
    renderWithProviders(<BetaPill className="custom-pill" />);
    const pill = screen.getByText('Beta');
    expect(pill).toHaveClass('custom-pill');
    expect(pill).toHaveClass('rounded-full');
  });
});
