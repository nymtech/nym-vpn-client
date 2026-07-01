import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { Route, Routes } from 'react-router';
import { renderWithProviders } from '../../test/harness';
import SettingsRouteIndex from './SettingsRouteIndex';

describe('SettingsRouteIndex', () => {
  it('renders the matched child route through its outlet', () => {
    renderWithProviders(
      <Routes>
        <Route element={<SettingsRouteIndex />}>
          <Route path="/child" element={<span>child screen</span>} />
        </Route>
      </Routes>,
      { initialEntries: ['/child'] },
    );

    expect(screen.getByText('child screen')).toBeInTheDocument();
  });
});
