import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ButtonText from './ButtonText';

describe('ButtonText', () => {
  it('renders its children', () => {
    render(<ButtonText>Learn more</ButtonText>);
    expect(
      screen.getByRole('button', { name: 'Learn more' }),
    ).toBeInTheDocument();
  });

  it('fires onClick when clicked', async () => {
    const onClick = vi.fn();
    render(<ButtonText onClick={onClick}>Learn more</ButtonText>);

    await userEvent.click(screen.getByRole('button', { name: 'Learn more' }));

    expect(onClick).toHaveBeenCalledOnce();
  });

  it('fires onDoubleClick when double-clicked', async () => {
    const onDoubleClick = vi.fn();
    render(<ButtonText onDoubleClick={onDoubleClick}>Learn more</ButtonText>);

    await userEvent.dblClick(
      screen.getByRole('button', { name: 'Learn more' }),
    );

    expect(onDoubleClick).toHaveBeenCalledOnce();
  });

  it('does not fire onClick when disabled', async () => {
    const onClick = vi.fn();
    render(
      <ButtonText onClick={onClick} disabled>
        Learn more
      </ButtonText>,
    );

    const button = screen.getByRole('button', { name: 'Learn more' });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('data-test-disabled', 'true');
    await userEvent.click(button);

    expect(onClick).not.toHaveBeenCalled();
  });

  it('marks the content as truncated when truncate is set', () => {
    render(
      <ButtonText truncate data-testid="trunc-btn">
        A very long label
      </ButtonText>,
    );

    const button = screen.getByTestId('trunc-btn');
    expect(button).toHaveAttribute('data-test-truncate', 'true');
    expect(screen.getByTestId('trunc-btn-content')).toHaveClass('truncate');
  });
});
