import { createContext, useContext } from 'react';
import type { useSortable } from '@dnd-kit/sortable';

export const SectionDragContext = createContext<ReturnType<typeof useSortable> | null>(null);
export const useSectionDrag = () => useContext(SectionDragContext);
