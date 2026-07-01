import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../test/harness';
import SettingsMenuCard from './SettingsMenuCard';

describe('SettingsMenuCard', () => {
  it('renders the title', () => {
    renderWithProviders(<SettingsMenuCard title="Appearance" />);

    expect(screen.getByText('Appearance')).toBeInTheDocument();
  });

  it('renders an optional description', () => {
    renderWithProviders(
      <SettingsMenuCard title="Language" description="English" />,
    );

    expect(screen.getByText('English')).toBeInTheDocument();
  });

  it('renders leading and trailing icons', () => {
    renderWithProviders(
      <SettingsMenuCard
        title="DNS"
        leadingIcon="dns"
        trailingIcon="keyboard_arrow_right"
      />,
    );

    expect(screen.getByTestId('icon-dns')).toBeInTheDocument();
    expect(screen.getByTestId('icon-keyboard_arrow_right')).toBeInTheDocument();
  });

  it('calls onClick when clicked', async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();

    renderWithProviders(<SettingsMenuCard title="Account" onClick={onClick} />);

    await user.click(screen.getByRole('button', { name: /Account/ }));

    expect(onClick).toHaveBeenCalledOnce();
  });

  it('calls onClick on Enter keydown', async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();

    renderWithProviders(<SettingsMenuCard title="Support" onClick={onClick} />);

    screen.getByRole('button', { name: /Support/ }).focus();
    await user.keyboard('{Enter}');

    expect(onClick).toHaveBeenCalledOnce();
  });

  it('is not focusable when disabled', () => {
    renderWithProviders(<SettingsMenuCard title="Legal" disabled />);

    expect(screen.getByRole('button', { name: /Legal/ })).toHaveAttribute(
      'tabindex',
      '-1',
    );
  });
});
