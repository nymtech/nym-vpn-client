import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { PanelContent } from './NodeListPanelContent';

describe('PanelContent', () => {
  it('renders its children', () => {
    render(
      <PanelContent>
        <span>panel body</span>
      </PanelContent>,
    );

    expect(screen.getByText('panel body')).toBeInTheDocument();
  });

  it('renders children when animation is enabled', () => {
    render(
      <PanelContent animate>
        <span>animated body</span>
      </PanelContent>,
    );

    expect(screen.getByText('animated body')).toBeInTheDocument();
  });
});
