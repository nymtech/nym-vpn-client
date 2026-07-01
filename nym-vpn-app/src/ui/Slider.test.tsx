import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../test/harness';
import Slider from './Slider';

describe('Slider', () => {
  it('renders a slider reflecting the current value', () => {
    renderWithProviders(
      <Slider value={30} min={0} max={100} step={10} ariaLabel="Delay" />,
    );

    const slider = screen.getByRole('slider', { name: 'Delay' });
    expect(slider).toHaveAttribute('aria-valuenow', '30');
    expect(slider).toHaveAttribute('min', '0');
    expect(slider).toHaveAttribute('max', '100');
  });

  it('moves and calls onChange when using the keyboard', async () => {
    const onChange = vi.fn();
    renderWithProviders(
      <Slider
        value={30}
        min={0}
        max={100}
        step={10}
        onChange={onChange}
        ariaLabel="Delay"
      />,
    );

    const slider = screen.getByRole('slider', { name: 'Delay' });
    slider.focus();
    await userEvent.keyboard('{ArrowRight}');

    expect(onChange).toHaveBeenCalledWith(40);
    expect(slider).toHaveAttribute('aria-valuenow', '40');
  });

  it('commits the new value on a synchronous keyboard commit', async () => {
    // Keyboard interaction fires change + commit in the same event tick, before
    // React re-renders — so the commit must forward the value base-ui provides,
    // not a stale closed-over state value.
    const onValueCommitted = vi.fn();
    renderWithProviders(
      <Slider
        value={30}
        min={0}
        max={100}
        step={10}
        onValueCommitted={onValueCommitted}
        ariaLabel="Delay"
      />,
    );

    const slider = screen.getByRole('slider', { name: 'Delay' });
    slider.focus();
    await userEvent.keyboard('{ArrowRight}');

    expect(onValueCommitted).toHaveBeenCalledWith(40);
  });

  it('commits the final value after successive keyboard steps', async () => {
    const onValueCommitted = vi.fn();
    renderWithProviders(
      <Slider
        value={30}
        min={0}
        max={100}
        step={10}
        onValueCommitted={onValueCommitted}
        ariaLabel="Delay"
      />,
    );

    const slider = screen.getByRole('slider', { name: 'Delay' });
    slider.focus();
    await userEvent.keyboard('{ArrowRight}{ArrowRight}');

    expect(onValueCommitted).toHaveBeenLastCalledWith(50);
  });

  it('renders the provided labels', () => {
    renderWithProviders(
      <Slider
        value={1}
        min={0}
        max={2}
        step={1}
        ariaLabel="Level"
        labels={[<span key="low">Low</span>, <span key="high">High</span>]}
      />,
    );

    expect(screen.getByText('Low')).toBeInTheDocument();
    expect(screen.getByText('High')).toBeInTheDocument();
  });
});
