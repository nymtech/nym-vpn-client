import { describe, expect, it } from 'vitest';
import { render } from '@testing-library/react';
import { seedStore } from '../test/harness';
import SmileyIcon from './SmileyIcon';

function fillColors(container: HTMLElement) {
  return Array.from(container.querySelectorAll('path')).map((p) =>
    p.getAttribute('fill'),
  );
}

describe('SmileyIcon', () => {
  it('renders an inline svg', () => {
    seedStore({ uiTheme: 'dark' });
    const { container } = render(<SmileyIcon />);
    expect(container.querySelector('svg')).not.toBeNull();
  });

  it('renders the dark variant when the ui theme is dark', () => {
    seedStore({ uiTheme: 'dark' });
    const { container } = render(<SmileyIcon />);
    expect(fillColors(container)).toContain('#07FF94');
  });

  it('renders the light variant when the ui theme is light', () => {
    seedStore({ uiTheme: 'light' });
    const { container } = render(<SmileyIcon />);
    expect(fillColors(container)).not.toContain('#07FF94');
  });

  it('forwards a custom className to the svg', () => {
    seedStore({ uiTheme: 'dark' });
    const { container } = render(<SmileyIcon className="h-5 w-5" />);
    const svg = container.querySelector('svg');
    expect(svg).toHaveClass('h-5');
    expect(svg).toHaveClass('w-5');
  });
});
