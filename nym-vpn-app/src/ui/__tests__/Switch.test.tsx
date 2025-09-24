import { screen, fireEvent } from '@testing-library/react';
import { render } from '../../test/test-utils';
import Switch from '../Switch';

describe('Switch Component', () => {
  const defaultProps = {
    checked: false,
    onChange: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders correctly', () => {
    render(<Switch {...defaultProps} />);

    const switchElement = screen.getByTestId('switch');
    expect(switchElement).toBeInTheDocument();

    const thumb = screen.getByTestId('switch-thumb');
    expect(thumb).toBeInTheDocument();
  });

  it('calls onChange when clicked', () => {
    const onChange = jest.fn();
    render(<Switch {...defaultProps} onChange={onChange} />);

    const switchElement = screen.getByTestId('switch');
    fireEvent.click(switchElement);

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it('reflects checked state in data attributes', () => {
    const { rerender } = render(<Switch {...defaultProps} checked={false} />);

    let switchElement = screen.getByTestId('switch');
    expect(switchElement).toHaveAttribute('data-test-checked', 'false');

    rerender(<Switch {...defaultProps} checked={true} />);

    switchElement = screen.getByTestId('switch');
    expect(switchElement).toHaveAttribute('data-test-checked', 'true');
  });

  it('can be disabled', () => {
    const onChange = jest.fn();
    render(<Switch {...defaultProps} onChange={onChange} disabled={true} />);

    const switchElement = screen.getByTestId('switch');
    expect(switchElement).toHaveAttribute('data-test-disabled', 'true');

    fireEvent.click(switchElement);
    expect(onChange).not.toHaveBeenCalled();
  });

  it('applies correct CSS classes based on checked state', () => {
    const { rerender } = render(<Switch {...defaultProps} checked={false} />);

    let switchElement = screen.getByTestId('switch');
    expect(switchElement).toHaveClass('bg-bombay/60');

    rerender(<Switch {...defaultProps} checked={true} />);

    switchElement = screen.getByTestId('switch');
    expect(switchElement).toHaveClass('bg-malachite');
  });

  it('accepts custom data-testid', () => {
    const customTestId = 'custom-switch';

    render(<Switch {...defaultProps} data-testid={customTestId} />);

    const switchElement = screen.getByTestId(customTestId);
    expect(switchElement).toBeInTheDocument();

    const thumb = screen.getByTestId('custom-switch-thumb');
    expect(thumb).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const customClass = 'custom-switch-class';

    render(<Switch {...defaultProps} className={customClass} />);

    const switchElement = screen.getByTestId('switch');
    expect(switchElement).toHaveClass(customClass);
  });
});
