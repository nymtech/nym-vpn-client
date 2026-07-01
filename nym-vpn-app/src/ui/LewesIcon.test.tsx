import { describe, expect, it } from 'vitest';
import { render } from '@testing-library/react';
import { seedStore } from '../test/harness';
import { LewesIconComponent } from './LewesIcon';

function strokeColors(container: HTMLElement) {
  return Array.from(container.querySelectorAll('path')).map((p) =>
    p.getAttribute('stroke'),
  );
}

describe('LewesIconComponent', () => {
  it('renders an inline svg', () => {
    seedStore({ uiTheme: 'dark' });
    const { container } = render(<LewesIconComponent />);
    expect(container.querySelector('svg')).not.toBeNull();
  });

  it('renders the dark variant when the ui theme is dark', () => {
    seedStore({ uiTheme: 'dark' });
    const { container } = render(<LewesIconComponent />);
    const colors = strokeColors(container);
    expect(colors).toContain('#5BF0A0');
    expect(colors).not.toContain('#28C96C');
  });

  it('renders the light variant when the ui theme is light', () => {
    seedStore({ uiTheme: 'light' });
    const { container } = render(<LewesIconComponent />);
    const colors = strokeColors(container);
    expect(colors).toContain('#28C96C');
    expect(colors).not.toContain('#5BF0A0');
  });
});
