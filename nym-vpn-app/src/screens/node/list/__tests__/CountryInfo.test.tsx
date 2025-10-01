import React from 'react';
import { render, screen } from '@testing-library/react';
import { useTranslation } from 'react-i18next';
import CountryInfo from '../CountryInfo';
import { UiCountry } from '../../../../contexts';

describe('CountryInfo', () => {
  const mockCountry: UiCountry = {
    code: 'US',
    name: 'United States',
    isSelected: false,
  };

  it('renders country name and flag', () => {
    render(
      <CountryInfo country={mockCountry} name="United States" gwCount={5} />,
    );

    expect(screen.getByTestId('country-info-US')).toBeInTheDocument();
    expect(screen.getByTestId('country-flag-US')).toBeInTheDocument();
    expect(screen.getByTestId('country-name-US')).toHaveTextContent(
      'United States',
    );
  });

  it('renders server count with correct pluralization', () => {
    render(
      <CountryInfo country={mockCountry} name="United States" gwCount={5} />,
    );

    const serverCount = screen.getByTestId('country-server-count-US');
    expect(serverCount).toBeInTheDocument();
    expect(serverCount).toHaveTextContent('5');
  });

  it('renders single server count correctly', () => {
    render(
      <CountryInfo country={mockCountry} name="United States" gwCount={1} />,
    );

    const serverCount = screen.getByTestId('country-server-count-US');
    expect(serverCount).toBeInTheDocument();
    expect(serverCount).toHaveTextContent('1');
  });

  it('renders zero server count correctly', () => {
    render(
      <CountryInfo country={mockCountry} name="United States" gwCount={0} />,
    );

    const serverCount = screen.getByTestId('country-server-count-US');
    expect(serverCount).toBeInTheDocument();
    expect(serverCount).toHaveTextContent('0');
  });

  it('handles different country codes', () => {
    const germanyCountry: UiCountry = {
      code: 'DE',
      name: 'Germany',
      isSelected: false,
    };
    render(
      <CountryInfo country={germanyCountry} name="Deutschland" gwCount={3} />,
    );

    expect(screen.getByTestId('country-info-DE')).toBeInTheDocument();
    expect(screen.getByTestId('country-flag-DE')).toBeInTheDocument();
    expect(screen.getByTestId('country-name-DE')).toHaveTextContent(
      'Deutschland',
    );
  });

  it('truncates long country names', () => {
    const longName = 'Very Long Country Name That Should Be Truncated';
    render(<CountryInfo country={mockCountry} name={longName} gwCount={2} />);

    const nameElement = screen.getByTestId('country-name-US');
    expect(nameElement).toHaveClass('truncate');
    expect(nameElement).toHaveTextContent(longName);
  });
});
