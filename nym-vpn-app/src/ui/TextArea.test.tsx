import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import TextArea from './TextArea';

describe('TextArea', () => {
  it('renders a textbox with the given value', () => {
    render(<TextArea value="content" onChange={vi.fn()} />);

    expect(screen.getByRole('textbox')).toHaveValue('content');
  });

  it('renders a label when provided', () => {
    render(<TextArea value="" onChange={vi.fn()} label="Notes" />);

    expect(screen.getByText('Notes')).toBeInTheDocument();
  });

  it('calls onChange with each typed character', async () => {
    const onChange = vi.fn();
    render(<TextArea value="" onChange={onChange} />);

    await userEvent.type(screen.getByRole('textbox'), 'hi');

    expect(onChange).toHaveBeenCalledTimes(2);
    expect(onChange).toHaveBeenNthCalledWith(1, 'h');
    expect(onChange).toHaveBeenNthCalledWith(2, 'i');
  });

  it('applies the rows and resize props', () => {
    render(<TextArea value="" onChange={vi.fn()} rows={5} resize="none" />);

    const textarea = screen.getByRole('textbox');
    expect(textarea).toHaveAttribute('rows', '5');
    expect(textarea).toHaveAttribute('data-test-resize', 'none');
    expect(textarea).toHaveClass('resize-none');
  });
});
