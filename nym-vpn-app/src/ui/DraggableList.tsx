import { ReactNode } from 'react';
import {
  DndContext,
  DragEndEvent,
  KeyboardSensor,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import {
  SortableContext,
  arrayMove,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import clsx from 'clsx';
import { CSS } from '@dnd-kit/utilities';
import MsIcon from './MsIcon';

export type DraggableListItem = {
  id: string;
};

type SortableItemProps<T extends DraggableListItem> = {
  item: T;
  renderItem: (item: T, dragHandle: ReactNode) => ReactNode;
  dragHandleClassName?: string;
};

function SortableItem<T extends DraggableListItem>({
  item,
  renderItem,
  dragHandleClassName,
}: SortableItemProps<T>) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: item.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  const dragHandle = (
    <button
      type="button"
      {...attributes}
      {...listeners}
      className={
        dragHandleClassName ?? 'cursor-grab touch-none active:cursor-grabbing'
      }
    >
      <MsIcon icon="drag_indicator" className="text-text-secondary" />
    </button>
  );

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={clsx(
        'border-bombay dark:border-iron border-t',
        isDragging ? 'z-10 opacity-50' : '',
      )}
    >
      {renderItem(item, dragHandle)}
    </div>
  );
}

export type DraggableListProps<T extends DraggableListItem> = {
  items: T[];
  onReorder: (items: T[]) => void;
  renderItem: (item: T, dragHandle: ReactNode) => ReactNode;
  dragHandleClassName?: string;
};

function DraggableList<T extends DraggableListItem>({
  items,
  onReorder,
  renderItem,
  dragHandleClassName,
}: DraggableListProps<T>) {
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (over && active.id !== over.id) {
      const oldIndex = items.findIndex((item) => item.id === active.id);
      const newIndex = items.findIndex((item) => item.id === over.id);
      onReorder(arrayMove(items, oldIndex, newIndex));
    }
  };

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragEnd={handleDragEnd}
    >
      <SortableContext
        items={items.map((item) => item.id)}
        strategy={verticalListSortingStrategy}
      >
        <div className="border-bombay dark:border-iron flex flex-col border-b">
          {items.map((item) => (
            <SortableItem
              key={item.id}
              item={item}
              renderItem={renderItem}
              dragHandleClassName={dragHandleClassName}
            />
          ))}
        </div>
      </SortableContext>
    </DndContext>
  );
}

export default DraggableList;
