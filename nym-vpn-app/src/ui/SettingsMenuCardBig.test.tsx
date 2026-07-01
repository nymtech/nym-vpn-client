import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../test/harness';
import SettingsMenuCardBig from './SettingsMenuCardBig';

describe('SettingsMenuCardBig', () => {
  it('renders the header and body children', () => {
    renderWithProviders(
      <SettingsMenuCardBig header={<h2>Section</h2>}>
        <p>body content</p>
      </SettingsMenuCardBig>,
    );

    expect(
      screen.getByRole('heading', { name: 'Section' }),
    ).toBeInTheDocument();
    expect(screen.getByText('body content')).toBeInTheDocument();
  });

  it('applies the disabled opacity styling when disabled', () => {
    const { container } = renderWithProviders(
      <SettingsMenuCardBig header={<span>h</span>} disabled>
        <span>c</span>
      </SettingsMenuCardBig>,
    );

    expect(container.firstChild).toHaveClass('opacity-50');
  });
});
