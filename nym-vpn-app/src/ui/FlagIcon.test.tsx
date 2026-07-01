import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import FlagIcon from './FlagIcon';

describe('FlagIcon', () => {
  it('renders a flag image for a valid country code', () => {
    render(<FlagIcon code="fr" alt="France" />);
    const img = screen.getByTestId('flag-icon-fr');
    expect(img).toBeInTheDocument();
    expect(img).toHaveAttribute('src', './flags/fr.svg');
    expect(img).toHaveAttribute('alt', 'France');
    expect(img).toHaveAttribute('data-test-country-code', 'fr');
    expect(screen.getByTestId('flag-icon-fr-container')).toBeInTheDocument();
  });

  it('exposes the flag via its alt text as an accessible image', () => {
    render(<FlagIcon code="de" alt="Germany" />);
    expect(screen.getByRole('img', { name: 'Germany' })).toBeInTheDocument();
  });

  it('falls back to a broken-image icon for an unknown code', () => {
    render(
      <FlagIcon
        code={'zz' as unknown as 'fr'}
        alt="Unknown"
        data-testid="flag-icon-zz"
      />,
    );
    expect(screen.getByTestId('flag-icon-zz-broken')).toBeInTheDocument();
    expect(screen.queryByRole('img')).not.toBeInTheDocument();
  });

  it('forwards a custom className to the image', () => {
    render(<FlagIcon code="us" alt="United States" className="opacity-50" />);
    expect(screen.getByTestId('flag-icon-us')).toHaveClass('opacity-50');
  });
});
