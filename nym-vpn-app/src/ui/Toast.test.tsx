import type { ReactElement } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import {
  Provider as ToastProvider,
  Viewport as ToastViewport,
} from '@radix-ui/react-toast';
import { renderWithProviders } from '../test/harness';
import Toast from './Toast';

// The radix `Toast.Root` used inside `Toast` requires a `ToastProvider` (and a
// `Viewport` for the toast collection to mount into), which the shared harness
// does not supply.
function renderToast(ui: ReactElement) {
  return renderWithProviders(
    <ToastProvider>
      {ui}
      <ToastViewport />
    </ToastProvider>,
  );
}

describe('Toast', () => {
  it('renders its message when open by default', () => {
    renderToast(<Toast message="Connected" />);

    expect(screen.getByText('Connected')).toBeInTheDocument();
    expect(screen.getByTestId('toast')).toHaveAttribute(
      'data-test-type',
      'info',
    );
  });

  it('does not render when the controlled open prop is false', () => {
    renderToast(<Toast message="hidden" open={false} />);

    expect(screen.queryByText('hidden')).not.toBeInTheDocument();
  });

  it('renders custom content over title/message', () => {
    renderToast(<Toast content={<span>custom node</span>} message="ignored" />);

    expect(screen.getByText('custom node')).toBeInTheDocument();
    expect(screen.queryByText('ignored')).not.toBeInTheDocument();
  });

  it('reflects the error type', () => {
    renderToast(<Toast message="oops" type="error" />);

    expect(screen.getByTestId('toast')).toHaveAttribute(
      'data-test-type',
      'error',
    );
  });

  it('closes and fires onOpenChange when the close button is clicked', async () => {
    const onOpenChange = vi.fn();

    renderToast(
      <Toast message="dismiss me" close onOpenChange={onOpenChange} />,
    );

    // `fireEvent.click` (vs `userEvent`) avoids radix's pointer-capture path,
    // which jsdom does not implement.
    fireEvent.click(screen.getByTestId('toast-close-button'));

    expect(onOpenChange).toHaveBeenCalledWith(false);
    await waitFor(() =>
      expect(screen.queryByText('dismiss me')).not.toBeInTheDocument(),
    );
  });
});
