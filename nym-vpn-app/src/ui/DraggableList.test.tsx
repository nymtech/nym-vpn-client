import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../test/harness';
import DraggableList, { type DraggableListItem } from './DraggableList';

type Row = DraggableListItem & { label: string };

const items: Row[] = [
  { id: 'a', label: 'Alpha' },
  { id: 'b', label: 'Beta' },
  { id: 'c', label: 'Gamma' },
];

function renderList(onReorder = vi.fn()) {
  renderWithProviders(
    <DraggableList
      items={items}
      onReorder={onReorder}
      renderItem={(item: Row, dragHandle: ReactNode) => (
        <div>
          <span>{item.label}</span>
          {dragHandle}
        </div>
      )}
    />,
  );
  return { onReorder };
}

describe('DraggableList', () => {
  it('renders every item via renderItem', () => {
    renderList();

    expect(screen.getByText('Alpha')).toBeInTheDocument();
    expect(screen.getByText('Beta')).toBeInTheDocument();
    expect(screen.getByText('Gamma')).toBeInTheDocument();
  });

  it('renders a drag handle for each item', () => {
    renderList();

    expect(screen.getAllByTestId('icon-drag_indicator')).toHaveLength(
      items.length,
    );
  });

  it('wires each item into a sortable draggable via dnd-kit', () => {
    renderList();

    // dnd-kit augments each draggable handle with sortable ARIA metadata; its
    // presence proves the SortableContext/useSortable wiring is in place. A
    // full pointer/keyboard drag cannot be reliably simulated in jsdom because
    // it has no layout, so we assert the wiring rather than a synthetic drag.
    const handles = screen.getAllByRole('button');
    expect(handles).toHaveLength(items.length);
    handles.forEach((handle) => {
      expect(handle).toHaveAttribute('aria-roledescription', 'sortable');
      expect(handle).toHaveAttribute('aria-disabled', 'false');
    });
  });
});
