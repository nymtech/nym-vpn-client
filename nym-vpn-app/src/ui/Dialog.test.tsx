import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { act } from 'react';
import { renderWithProviders, seedStore } from '../test/harness';
import Dialog from './Dialog';

describe('Dialog', () => {
  it('renders its children when open', () => {
    renderWithProviders(
      <Dialog open onClose={vi.fn()}>
        <p>dialog body</p>
      </Dialog>,
    );

    expect(screen.getByText('dialog body')).toBeInTheDocument();
    expect(screen.getByTestId('dialog')).toHaveAttribute(
      'data-test-open',
      'true',
    );
  });

  it('does not render its children when closed', () => {
    renderWithProviders(
      <Dialog open={false} onClose={vi.fn()}>
        <p>dialog body</p>
      </Dialog>,
    );

    expect(screen.queryByText('dialog body')).not.toBeInTheDocument();
  });

  it('reflects the store theme on the dialog root', () => {
    act(() => {
      seedStore({ uiTheme: 'dark' });
    });

    renderWithProviders(
      <Dialog open onClose={vi.fn()}>
        <p>themed</p>
      </Dialog>,
    );

    expect(screen.getByTestId('dialog')).toHaveAttribute(
      'data-test-theme',
      'dark',
    );
  });

  it('calls onClose when the backdrop is clicked', async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();

    renderWithProviders(
      <Dialog open onClose={onClose}>
        <p>closable</p>
      </Dialog>,
    );

    await user.click(screen.getByTestId('dialog-backdrop'));

    expect(onClose).toHaveBeenCalled();
  });

  it('honours a custom data-testid', () => {
    renderWithProviders(
      <Dialog open onClose={vi.fn()} data-testid="custom-dialog">
        <p>custom</p>
      </Dialog>,
    );

    expect(screen.getByTestId('custom-dialog')).toBeInTheDocument();
    expect(screen.getByTestId('custom-dialog-panel')).toBeInTheDocument();
  });
});
