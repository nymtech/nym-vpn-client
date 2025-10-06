import React from 'react';
import { screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { render } from '../../test/test-utils';
import TextInput from '../TextInput';

describe('TextInput Component', () => {
  const defaultProps = {
    value: '',
    onChange: jest.fn(),
    placeholder: 'Test input',
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders with basic props', () => {
    render(<TextInput {...defaultProps} />);

    const input = screen.getByPlaceholderText('Test input');
    expect(input).toBeInTheDocument();
    expect(input).toHaveAttribute('placeholder', 'Test input');
  });

  it('calls onChange when value changes', async () => {
    const user = userEvent.setup();
    const onChange = jest.fn();

    render(<TextInput {...defaultProps} onChange={onChange} />);

    const inputValue = 'hello';
    const input = screen.getByPlaceholderText('Test input');
    await user.type(input, inputValue);

    expect(onChange).toHaveBeenCalled();
    expect(onChange).toHaveBeenLastCalledWith(inputValue.slice(-1)); // Last character typed
    expect(onChange).toHaveBeenCalledTimes(inputValue.length); // Called for each character
  });

  it('displays the provided value', () => {
    const value = 'test value';

    render(<TextInput {...defaultProps} value={value} />);

    const input = screen.getByDisplayValue(value);
    expect(input).toBeInTheDocument();
  });

  it('renders with label when provided', () => {
    const label = 'Test Label';

    render(<TextInput {...defaultProps} label={label} />);

    const labelElement = screen.getByText(label);
    expect(labelElement).toBeInTheDocument();
    expect(labelElement).toHaveTextContent(label);
  });

  it('renders with left icon when provided', () => {
    const leftIcon = 'search';

    render(<TextInput {...defaultProps} leftIcon={leftIcon} />);

    const iconElement = screen.getByTestId('text-input-left-icon');
    expect(iconElement).toBeInTheDocument();

    const input = screen.getByPlaceholderText('Test input');
    expect(input).toHaveAttribute('data-test-has-left-icon', 'true');
  });

  it('supports autoFocus', () => {
    render(<TextInput {...defaultProps} autoFocus />);

    const input = screen.getByPlaceholderText('Test input');
    expect(input).toHaveFocus();
  });

  it('accepts custom data-testid', () => {
    const customTestId = 'custom-input';

    render(<TextInput {...defaultProps} data-testid={customTestId} />);

    const input = screen.getByPlaceholderText('Test input');
    expect(input).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const customClass = 'custom-input-class';

    render(<TextInput {...defaultProps} className={customClass} />);

    const input = screen.getByPlaceholderText('Test input');
    expect(input).toHaveClass(customClass);
  });
});
