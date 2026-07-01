import type { ReactElement } from 'react';
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { CardAnimationProvider } from '../../contexts/CardAnimationContext';
import { renderWithProviders } from '../../test/harness';
import { InteractiveCard } from './InteractiveCard';

// `InteractiveCard` reads `useCardAnimation` from a React context; the default
// context value is a no-op, but wrapping in the real provider exercises the
// `registerExit` handshake without stubbing.
function renderCard(ui: ReactElement) {
  return renderWithProviders(
    <CardAnimationProvider>{ui}</CardAnimationProvider>,
  );
}

describe('InteractiveCard', () => {
  it('renders its children', () => {
    renderCard(
      <InteractiveCard>
        <span>card body</span>
      </InteractiveCard>,
    );

    expect(screen.getByText('card body')).toBeInTheDocument();
  });

  it('forwards a custom className to the animated container', () => {
    const { container } = renderCard(
      <InteractiveCard className="custom-card">
        <span>content</span>
      </InteractiveCard>,
    );

    expect(container.querySelector('.custom-card')).not.toBeNull();
  });

  it('mounts and unmounts cleanly, removing its content', () => {
    const { unmount } = renderCard(
      <InteractiveCard>
        <span>tracked</span>
      </InteractiveCard>,
    );

    expect(screen.getByText('tracked')).toBeInTheDocument();
    unmount();
    expect(screen.queryByText('tracked')).not.toBeInTheDocument();
  });
});
