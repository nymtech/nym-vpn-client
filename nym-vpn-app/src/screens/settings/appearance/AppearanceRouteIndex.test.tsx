import { describe, expect, it } from 'vitest';
import { Route, Routes } from 'react-router';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../../test/harness';
import AppearanceRouteIndex from './AppearanceRouteIndex';

describe('AppearanceRouteIndex', () => {
  it('renders the matched child route via its outlet', () => {
    renderWithProviders(
      <Routes>
        <Route path="/" element={<AppearanceRouteIndex />}>
          <Route index element={<div>child-content</div>} />
        </Route>
      </Routes>,
    );

    expect(screen.getByText('child-content')).toBeInTheDocument();
  });
});
