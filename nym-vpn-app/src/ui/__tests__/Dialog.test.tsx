import { screen, fireEvent } from '@testing-library/react';
import { render, mockDialogProps } from '../../test/test-utils';
import Dialog from '../Dialog';
import { useMainState } from '../../contexts';

// Mock the useMainState hook
const mockUseMainState = useMainState as jest.MockedFunction<
  typeof useMainState
>;

describe('Dialog Component', () => {
  const defaultProps = mockDialogProps();

  beforeEach(() => {
    jest.clearAllMocks();
    // Reset to default mock state
    mockUseMainState.mockReturnValue({
      uiTheme: 'light' as const,
    } as any);
  });

  describe('Rendering', () => {
    it('renders when open is true', () => {
      const props = mockDialogProps({ open: true });
      render(<Dialog {...props} />);

      const dialog = screen.getByTestId('dialog');
      expect(dialog).toBeInTheDocument();
      expect(dialog).toHaveAttribute('data-test-open', 'true');
    });

    it('renders with default test id', () => {
      const props = mockDialogProps({ open: true });
      render(<Dialog {...props} />);

      const dialog = screen.getByTestId('dialog');
      expect(dialog).toBeInTheDocument();
    });

    it('accepts custom data-testid', () => {
      const customTestId = 'custom-dialog';
      const props = mockDialogProps({
        open: true,
        'data-testid': customTestId,
      });

      render(<Dialog {...props} />);

      const dialog = screen.getByTestId(customTestId);
      expect(dialog).toBeInTheDocument();

      // Check that related elements use the custom test id
      expect(
        screen.getByTestId(`${customTestId}-backdrop`),
      ).toBeInTheDocument();
      expect(
        screen.getByTestId(`${customTestId}-container`),
      ).toBeInTheDocument();
      expect(screen.getByTestId(`${customTestId}-wrapper`)).toBeInTheDocument();
      expect(screen.getByTestId(`${customTestId}-panel`)).toBeInTheDocument();
    });

    it('renders dialog elements with correct structure', () => {
      const props = mockDialogProps({ open: true });
      render(<Dialog {...props} />);

      // Check all dialog elements are present
      expect(screen.getByTestId('dialog-backdrop')).toBeInTheDocument();
      expect(screen.getByTestId('dialog-container')).toBeInTheDocument();
      expect(screen.getByTestId('dialog-wrapper')).toBeInTheDocument();
      expect(screen.getByTestId('dialog-panel')).toBeInTheDocument();
    });

    it('renders children content', () => {
      const childContent = 'Custom dialog content';
      const props = mockDialogProps({ open: true, children: childContent });

      render(<Dialog {...props} />);

      expect(screen.getByText(childContent)).toBeInTheDocument();
    });

    it('renders complex children content', () => {
      const complexChildren = (
        <div>
          <h2>Dialog Title</h2>
          <p>Dialog description</p>
          <button>Action Button</button>
        </div>
      );
      const props = mockDialogProps({ open: true, children: complexChildren });

      render(<Dialog {...props} />);

      expect(screen.getByText('Dialog Title')).toBeInTheDocument();
      expect(screen.getByText('Dialog description')).toBeInTheDocument();
      expect(screen.getByText('Action Button')).toBeInTheDocument();
    });
  });

  describe('Props and State', () => {
    it('reflects open state in data attributes', () => {
      // When closed, dialog should not be in DOM
      const { rerender } = render(<Dialog {...defaultProps} open={false} />);

      const dialog = screen.queryByTestId('dialog');
      expect(dialog).not.toBeInTheDocument();

      // When open, dialog should be in DOM with correct attribute
      rerender(<Dialog {...defaultProps} open={true} />);

      const openDialog = screen.getByTestId('dialog');
      expect(openDialog).toBeInTheDocument();
      expect(openDialog).toHaveAttribute('data-test-open', 'true');
    });

    it('applies custom className to dialog panel', () => {
      const customClass = 'custom-dialog-class';
      const props = mockDialogProps({ open: true, className: customClass });

      render(<Dialog {...props} />);

      const panel = screen.getByTestId('dialog-panel');
      expect(panel).toHaveClass(customClass);
    });
  });

  describe('Theme Integration', () => {
    it('applies light theme by default', () => {
      mockUseMainState.mockReturnValue({
        uiTheme: 'light' as const,
      } as any);

      const props = mockDialogProps({ open: true });
      render(<Dialog {...props} />);

      const dialog = screen.getByTestId('dialog');
      expect(dialog).toHaveAttribute('data-test-theme', 'light');
      expect(dialog).not.toHaveClass('dark');
    });

    it('applies dark theme when uiTheme is dark', () => {
      mockUseMainState.mockReturnValue({
        uiTheme: 'dark' as const,
      } as any);

      const props = mockDialogProps({ open: true });
      render(<Dialog {...props} />);

      const dialog = screen.getByTestId('dialog');
      expect(dialog).toHaveAttribute('data-test-theme', 'dark');
      expect(dialog).toHaveClass('dark');
    });

    it('updates theme when uiTheme changes', () => {
      mockUseMainState.mockReturnValue({
        uiTheme: 'light' as const,
      } as any);

      const props = mockDialogProps({ open: true });
      const { rerender } = render(<Dialog {...props} />);

      let dialog = screen.getByTestId('dialog');
      expect(dialog).toHaveAttribute('data-test-theme', 'light');
      expect(dialog).not.toHaveClass('dark');

      // Change theme to dark
      mockUseMainState.mockReturnValue({
        uiTheme: 'dark' as const,
      } as any);

      rerender(<Dialog {...props} />);

      dialog = screen.getByTestId('dialog');
      expect(dialog).toHaveAttribute('data-test-theme', 'dark');
      expect(dialog).toHaveClass('dark');
    });
  });

  describe('Accessibility', () => {
    it('has proper dialog role', () => {
      const props = mockDialogProps({ open: true });
      render(<Dialog {...props} />);

      const dialog = screen.getByRole('dialog');
      expect(dialog).toBeInTheDocument();
    });

    it('is properly structured for screen readers', () => {
      const props = mockDialogProps({
        open: true,
        children: (
          <div>
            <h2 id="dialog-title">Dialog Title</h2>
            <p id="dialog-description">Dialog content description</p>
          </div>
        ),
      });

      render(<Dialog {...props} />);

      expect(screen.getByText('Dialog Title')).toBeInTheDocument();
      expect(
        screen.getByText('Dialog content description'),
      ).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('handles undefined children gracefully', () => {
      const props = mockDialogProps({ open: true, children: undefined });

      expect(() => render(<Dialog {...props} />)).not.toThrow();

      const dialog = screen.getByTestId('dialog');
      expect(dialog).toBeInTheDocument();
    });

    it('handles null children gracefully', () => {
      const props = mockDialogProps({ open: true, children: null });

      expect(() => render(<Dialog {...props} />)).not.toThrow();

      const dialog = screen.getByTestId('dialog');
      expect(dialog).toBeInTheDocument();
    });

    it('handles empty string children', () => {
      const props = mockDialogProps({ open: true, children: '' });

      render(<Dialog {...props} />);

      const dialog = screen.getByTestId('dialog');
      expect(dialog).toBeInTheDocument();
    });
  });
});
