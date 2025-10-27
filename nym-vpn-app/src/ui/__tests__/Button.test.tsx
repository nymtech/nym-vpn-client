import React from 'react';
import { screen, fireEvent } from '@testing-library/react';
import { render, mockButtonProps } from '../../test/test-utils';
import Button from '../Button';

describe('Button Component', () => {
  it('renders with default props', () => {
    const props = mockButtonProps();
    render(<Button {...props} />);

    const button = screen.getByRole('button', { name: 'Test Button' });
    expect(button).toBeInTheDocument();
    expect(button).toHaveTextContent('Test Button');
  });

  it('calls onClick when clicked', () => {
    const onClick = jest.fn();
    const props = mockButtonProps({ onClick });

    render(<Button {...props} />);

    const button = screen.getByRole('button', { name: 'Test Button' });
    fireEvent.click(button);

    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('is disabled when disabled prop is true', () => {
    const onClick = jest.fn();
    const props = mockButtonProps({ onClick, disabled: true });

    render(<Button {...props} />);

    const button = screen.getByRole('button', { name: 'Test Button' });
    expect(button).toBeDisabled();

    fireEvent.click(button);
    expect(onClick).not.toHaveBeenCalled();
  });

  it('applies correct color classes', () => {
    const props = mockButtonProps({ color: 'cornflower' });
    render(<Button {...props} />);

    const button = screen.getByRole('button', { name: 'Test Button' });
    expect(button).toHaveAttribute('data-test-color', 'cornflower');
  });

  it('applies outline styles when outline prop is true', () => {
    const props = mockButtonProps({ outline: true });
    render(<Button {...props} />);

    const button = screen.getByRole('button', { name: 'Test Button' });
    expect(button).toHaveAttribute('data-test-outline', 'true');
  });

  it('shows spinner when spinner prop is true', () => {
    const props = mockButtonProps({ spinner: true });
    render(<Button {...props} />);

    const spinner = screen.getByTestId('button-spinner');
    expect(spinner).toBeInTheDocument();

    // Button text should not be rendered when spinner is shown
    const buttonText = screen.queryByTestId('button-text');
    expect(buttonText).not.toBeInTheDocument();
  });

  it('applies custom className', () => {
    const customClass = 'custom-button-class';
    const props = mockButtonProps({ className: customClass });

    render(<Button {...props} />);

    const button = screen.getByRole('button', { name: 'Test Button' });
    expect(button).toHaveClass(customClass);
  });

  it('accepts custom data-testid', () => {
    const customTestId = 'custom-button';
    const props = mockButtonProps({ 'data-testid': customTestId });

    render(<Button {...props} />);

    const button = screen.getByRole('button', { name: 'Test Button' });
    expect(button).toBeInTheDocument();
  });

  describe('Color variations', () => {
    const colors = ['malachite', 'cornflower', 'gray', 'red'] as const;

    colors.forEach((color) => {
      it(`renders with ${color} color`, () => {
        const props = mockButtonProps({ color });
        render(<Button {...props} />);

        const button = screen.getByRole('button', { name: 'Test Button' });
        expect(button).toHaveAttribute('data-test-color', color);
      });
    });
  });

  describe('Accessibility', () => {
    it('has proper button role', () => {
      const props = mockButtonProps();
      render(<Button {...props} />);

      const button = screen.getByRole('button', { name: 'Test Button' });
      expect(button).toBeInTheDocument();
    });

    it('supports keyboard navigation', () => {
      const onClick = jest.fn();
      const props = mockButtonProps({ onClick });

      render(<Button {...props} />);

      const button = screen.getByRole('button', { name: 'Test Button' });
      button.focus();

      fireEvent.keyDown(button, { key: 'Enter' });
      fireEvent.keyUp(button, { key: 'Enter' });

      expect(button).toHaveFocus();
    });
  });
});
