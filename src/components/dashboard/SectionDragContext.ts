import { createContext, useContext } from 'react';
import type { useSortable } from '@dnd-kit/sortable';

export const SectionDragContext = createContext<Pick<
  ReturnType<typeof useSortable>,
  | 'attributes'
  | 'listeners'
  | 'setNodeRef'
  | 'setActivatorNodeRef'
  | 'transform'
  | 'isDragging'
  | 'transition'
> | null>(null);
export const useSectionDrag = () => useContext(SectionDragContext);
