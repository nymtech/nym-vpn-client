import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../test/harness';
import PageAnim from './PageAnim';

describe('PageAnim', () => {
  it('renders its children', () => {
    renderWithProviders(
      <PageAnim>
        <p>page content</p>
      </PageAnim>,
    );

    expect(screen.getByText('page content')).toBeInTheDocument();
  });

  it('uses the default test id', () => {
    renderWithProviders(<PageAnim>content</PageAnim>);

    expect(screen.getByTestId('page-animation')).toBeInTheDocument();
  });

  it('honours a custom data-testid', () => {
    renderWithProviders(<PageAnim data-testid="settings-page">x</PageAnim>);

    expect(screen.getByTestId('settings-page')).toBeInTheDocument();
  });

  it('records an explicit slide origin', () => {
    renderWithProviders(<PageAnim slideOrigin="right">x</PageAnim>);

    expect(screen.getByTestId('page-animation')).toHaveAttribute(
      'data-test-slide-origin',
      'right',
    );
  });
});
