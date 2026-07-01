import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders, seedStore } from '../test/harness';
import ThemeSetter from './ThemeSetter';

describe('ThemeSetter', () => {
  it('renders its children', () => {
    renderWithProviders(
      <ThemeSetter>
        <span>content</span>
      </ThemeSetter>,
    );

    expect(screen.getByText('content')).toBeInTheDocument();
  });

  it('reflects the light theme from the store', () => {
    seedStore({ uiTheme: 'light' });
    renderWithProviders(
      <ThemeSetter>
        <span>content</span>
      </ThemeSetter>,
    );

    const wrapper = screen.getByTestId('theme-setter');
    expect(wrapper).toHaveAttribute('data-test-theme', 'light');
    expect(wrapper).not.toHaveClass('dark');
  });

  it('applies the dark class when the store theme is dark', () => {
    seedStore({ uiTheme: 'dark' });
    renderWithProviders(
      <ThemeSetter>
        <span>content</span>
      </ThemeSetter>,
    );

    const wrapper = screen.getByTestId('theme-setter');
    expect(wrapper).toHaveAttribute('data-test-theme', 'dark');
    expect(wrapper).toHaveClass('dark');
  });
});
