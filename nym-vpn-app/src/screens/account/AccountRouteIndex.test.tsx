import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { Route, Routes } from 'react-router';

import { renderWithProviders } from '../../test/harness';
import AccountRouteIndex from './AccountRouteIndex';

describe('AccountRouteIndex', () => {
  it('renders the matched child route through its Outlet', () => {
    renderWithProviders(
      <Routes>
        <Route path="/account" element={<AccountRouteIndex />}>
          <Route index element={<div>account-child</div>} />
        </Route>
      </Routes>,
      { initialEntries: ['/account'] },
    );

    expect(screen.getByText('account-child')).toBeInTheDocument();
  });

  it('renders nothing extra when no child route matches', () => {
    const { container } = renderWithProviders(
      <Routes>
        <Route path="/account" element={<AccountRouteIndex />} />
      </Routes>,
      { initialEntries: ['/account'] },
    );

    expect(container).toBeEmptyDOMElement();
  });
});
