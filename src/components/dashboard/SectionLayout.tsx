import {
  DndContext,
  KeyboardSensor,
  MouseSensor,
  TouchSensor,
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

function SortableSection({
  id,
  animate,
  children,
}: {
  id: SectionId;
  animate: boolean;
  children: ReactNode;
}) {
  const sortable = useSortable({
    id,
    transition: animate ? { duration: 180, easing: 'cubic-bezier(0.32, 0.72, 0, 1)' } : null,
  });
  return (
    <SectionDragContext.Provider
      value={{
        attributes: sortable.attributes,
        listeners: sortable.listeners,
        setNodeRef: sortable.setNodeRef,
        setActivatorNodeRef: sortable.setActivatorNodeRef,
        transform: sortable.transform,
        isDragging: sortable.isDragging,
        transition: sortable.transition,
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
  const [animate, setAnimate] = useState(true);
  const visible = order.filter((id) => !hidden.includes(id));
  const sensors = useSensors(
    useSensor(MouseSensor, { activationConstraint: { distance: 6 } }),
    useSensor(TouchSensor, { activationConstraint: { delay: 200, tolerance: 5 } }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
      scrollBehavior: 'auto',
    }),
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
      onDragStart={(event) => {
        dragging.current = true;
        setAnimate(!(event.activatorEvent instanceof KeyboardEvent));
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
            'Focus a card, then press Space or Enter to pick it up. Use the up and down arrow keys to move it. Press Space or Enter to drop it, or Escape to cancel.',
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
            <SortableSection key={id} id={id} animate={animate}>
              {children(id)}
            </SortableSection>
          ))}
        </SortableContext>
      </div>
    </DndContext>
  );
}
