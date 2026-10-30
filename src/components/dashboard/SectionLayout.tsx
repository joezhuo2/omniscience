import {
  DndContext,
  KeyboardSensor,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core';
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { useRef, useState, type ReactNode } from 'react';
import {
  SECTION_LABELS,
  moveSection,
  readSectionOrder,
  saveSectionOrder,
  type SectionId,
} from '../../lib/sections';
import { SectionDragContext } from './SectionDragContext';

function SortableSection({ id, children }: { id: SectionId; children: ReactNode }) {
  const sortable = useSortable({ id, transition: null });
  return (
    <SectionDragContext.Provider
      value={{
        attributes: sortable.attributes,
        listeners: sortable.listeners,
        setNodeRef: sortable.setNodeRef,
        setActivatorNodeRef: sortable.setActivatorNodeRef,
        transform: sortable.transform,
        isDragging: sortable.isDragging,
      }}
    >
      {children}
    </SectionDragContext.Provider>
  );
}

export function SectionLayout({
  hidden = [],
  children,
}: {
  hidden?: SectionId[];
  children: (id: SectionId) => ReactNode;
}) {
  const [order, setOrder] = useState(readSectionOrder);
  const dragging = useRef(false);
  const visible = order.filter((id) => !hidden.includes(id));
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 6 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );
  function finish({ active, over }: DragEndEvent) {
    dragging.current = false;
    if (!over) return;
    const next = moveSection(order, String(active.id), String(over.id));
    if (next === order) return;
    setOrder(next);
    saveSectionOrder(next);
  }
  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={() => {
        dragging.current = true;
      }}
      onDragEnd={finish}
      onDragCancel={() => {
        dragging.current = false;
      }}
      accessibility={{
        announcements: {
          onDragStart: ({ active }) => `${SECTION_LABELS[active.id as SectionId]} picked up.`,
          onDragOver: ({ active, over }) =>
            over
              ? `${SECTION_LABELS[active.id as SectionId]} moved to position ${visible.indexOf(over.id as SectionId) + 1} of ${visible.length}.`
              : undefined,
          onDragEnd: ({ active, over }) =>
            over
              ? `${SECTION_LABELS[active.id as SectionId]} dropped at position ${visible.indexOf(over.id as SectionId) + 1}.`
              : 'Reordering cancelled.',
          onDragCancel: () => 'Reordering cancelled. The section order is unchanged.',
        },
        screenReaderInstructions: {
          draggable:
            'Press Space or Enter to pick up a section. Use the up and down arrow keys to move it. Press Space or Enter to drop it, or Escape to cancel.',
        },
      }}
    >
      <div
        className="contents"
        onKeyDownCapture={(event) => {
          // Let the sensor cancel the drag before the island handles Escape.
          if (dragging.current && event.key === 'Escape') event.preventDefault();
        }}
      >
        <SortableContext items={visible} strategy={verticalListSortingStrategy}>
          {visible.map((id) => (
            <SortableSection key={id} id={id}>
              {children(id)}
            </SortableSection>
          ))}
        </SortableContext>
      </div>
    </DndContext>
  );
}
