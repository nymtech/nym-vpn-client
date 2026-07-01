import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/react';
import useClickAway from './useClickAway';

function Fixture({ on, disabled }: { on: () => void; disabled?: boolean }) {
  const ref = useClickAway<HTMLDivElement>({ on, disabled });
  return (
    <div>
      <div ref={ref} data-testid="inside">
        inside
      </div>
      <button data-testid="outside">outside</button>
    </div>
  );
}

describe('useClickAway', () => {
  it('fires when a mousedown lands outside the ref element', () => {
    const on = vi.fn();
    render(<Fixture on={on} />);
    fireEvent.mouseDown(screen.getByTestId('outside'));
    expect(on).toHaveBeenCalledOnce();
  });

  it('does not fire when the mousedown is inside the ref element', () => {
    const on = vi.fn();
    render(<Fixture on={on} />);
    fireEvent.mouseDown(screen.getByTestId('inside'));
    expect(on).not.toHaveBeenCalled();
  });

  it('does nothing while disabled', () => {
    const on = vi.fn();
    render(<Fixture on={on} disabled />);
    fireEvent.mouseDown(screen.getByTestId('outside'));
    expect(on).not.toHaveBeenCalled();
  });
});
