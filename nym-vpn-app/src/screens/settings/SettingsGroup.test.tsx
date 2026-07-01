import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../../test/harness';
import SettingsGroup from './SettingsGroup';

describe('SettingsGroup', () => {
  it('renders a row per settings item with title and description', () => {
    renderWithProviders(
      <SettingsGroup
        settings={[
          { title: 'DNS', desc: 'Custom DNS servers' },
          { title: 'Legal' },
        ]}
      />,
    );

    expect(screen.getByText('DNS')).toBeInTheDocument();
    expect(screen.getByText('Custom DNS servers')).toBeInTheDocument();
    expect(screen.getByText('Legal')).toBeInTheDocument();
  });

  it('skips falsy entries', () => {
    renderWithProviders(
      <SettingsGroup settings={[{ title: 'Visible' }, false, undefined]} />,
    );

    expect(screen.getByText('Visible')).toBeInTheDocument();
    expect(screen.getAllByRole('button')).toHaveLength(1);
  });

  it('calls the item onClick when clicked', async () => {
    const onClick = vi.fn();
    renderWithProviders(
      <SettingsGroup settings={[{ title: 'Support', onClick }]} />,
    );

    await userEvent.click(screen.getByText('Support'));

    expect(onClick).toHaveBeenCalledOnce();
  });

  it('renders leading and trailing icons', () => {
    renderWithProviders(
      <SettingsGroup
        settings={[
          { title: 'DNS', leadingIcon: 'dns', trailingIcon: 'open_in_new' },
        ]}
      />,
    );

    expect(screen.getByText('dns')).toBeInTheDocument();
    expect(screen.getByText('open_in_new')).toBeInTheDocument();
  });

  it('renders a custom trailing node', () => {
    renderWithProviders(
      <SettingsGroup
        settings={[{ title: 'Toggle', trailing: <span>trailing-node</span> }]}
      />,
    );

    expect(screen.getByText('trailing-node')).toBeInTheDocument();
  });
});
