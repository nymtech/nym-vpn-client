import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import CardSwitch from './CardSwitch';

describe('CardSwitch', () => {
  it('renders the header and subheader', () => {
    render(
      <CardSwitch
        header="Auto-connect"
        subheader="Connect on launch"
        checked={false}
        onClick={vi.fn()}
      />,
    );

    expect(screen.getByText('Auto-connect')).toBeInTheDocument();
    expect(screen.getByText('Connect on launch')).toBeInTheDocument();
  });

  it('reflects the checked state on the inner switch', () => {
    render(<CardSwitch header="Auto-connect" checked onClick={vi.fn()} />);

    expect(screen.getByRole('switch')).toHaveAttribute('aria-checked', 'true');
  });

  it('calls onClick when the card is clicked', async () => {
    const onClick = vi.fn();
    render(
      <CardSwitch header="Auto-connect" checked={false} onClick={onClick} />,
    );

    await userEvent.click(screen.getByText('Auto-connect'));

    expect(onClick).toHaveBeenCalled();
  });

  it('calls onClick when the inner switch is toggled', async () => {
    const onClick = vi.fn();
    render(
      <CardSwitch header="Auto-connect" checked={false} onClick={onClick} />,
    );

    await userEvent.click(screen.getByRole('switch'));

    expect(onClick).toHaveBeenCalled();
  });

  it('disables the switch and takes the card out of the tab order when disabled', async () => {
    const onClick = vi.fn();
    render(
      <CardSwitch
        header="Auto-connect"
        checked={false}
        onClick={onClick}
        disabled
      />,
    );

    expect(screen.getByRole('switch')).toBeDisabled();
    expect(screen.getByRole('button')).toHaveAttribute('tabindex', '-1');

    await userEvent.click(screen.getByRole('switch'));

    expect(onClick).not.toHaveBeenCalled();
  });
});
