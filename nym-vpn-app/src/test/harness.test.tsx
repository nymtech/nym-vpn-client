import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { useTranslation } from 'react-i18next';
import { Link } from 'react-router';
import { renderWithProviders } from './harness';

describe('renderWithProviders', () => {
  it('renders a trivial component', () => {
    renderWithProviders(<div>hello harness</div>);
    expect(screen.getByText('hello harness')).toBeInTheDocument();
  });

  it('provides an initialized i18n instance', () => {
    function Translated() {
      const { i18n } = useTranslation();
      return <span>{i18n.language ? 'i18n-ready' : 'no-i18n'}</span>;
    }
    renderWithProviders(<Translated />);
    expect(screen.getByText('i18n-ready')).toBeInTheDocument();
  });

  it('provides a router so <Link> renders', () => {
    renderWithProviders(<Link to="/somewhere">go</Link>);
    const link = screen.getByRole('link', { name: 'go' });
    expect(link).toHaveAttribute('href', '/somewhere');
  });
});
