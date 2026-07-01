import { describe, expect, it } from 'vitest';
import { Route, Routes } from 'react-router';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../../test/harness';
import LegalRouteIndex from './LegalRouteIndex';

describe('LegalRouteIndex', () => {
  it('renders the nested route outlet', () => {
    renderWithProviders(
      <Routes>
        <Route path="/" element={<LegalRouteIndex />}>
          <Route index element={<p>Nested legal content</p>} />
        </Route>
      </Routes>,
    );

    expect(screen.getByText('Nested legal content')).toBeInTheDocument();
  });
});
