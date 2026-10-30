import { ChevronDown, ChevronRight, GripVertical } from 'lucide-react';
import { CSS } from '@dnd-kit/utilities';
import { useSectionDrag } from './SectionDragContext';
import { useId, useState, type ReactNode } from 'react';
import { readSectionCollapsed, saveSectionCollapsed, type SectionId } from '../../lib/sections';

export function Card({
  title,
  section,
  aside,
  summary,
  children,
}: {
  title: string;
  section?: SectionId;
  aside?: ReactNode;
  summary?: ReactNode;
  children: ReactNode;
}) {
  const id = useId();
  const drag = useSectionDrag();
  const [collapsed, setCollapsed] = useState(() => readSectionCollapsed(section));
  const toggle = () => {
    const next = !collapsed;
    setCollapsed(next);
    if (section) saveSectionCollapsed(section, next);
  };
  const Chevron = collapsed ? ChevronRight : ChevronDown;
  return (
    <section
      ref={drag?.setNodeRef}
      data-section={section}
      style={
        drag
          ? {
              transform: CSS.Translate.toString(drag.transform),
              position: 'relative',
              zIndex: drag.isDragging ? 1 : undefined,
              opacity: drag.isDragging ? 0.65 : undefined,
            }
          : undefined
      }
      aria-labelledby={`${id}-heading`}
      className="rounded-xl border border-island-faint bg-island-hover p-2.5"
    >
      <header className={`${collapsed ? '' : 'mb-2 '}flex items-center justify-between gap-2`}>
        <div className="flex min-w-0 items-center gap-1">
          {drag && (
            <button
              ref={drag.setActivatorNodeRef}
              type="button"
              {...drag.attributes}
              {...drag.listeners}
              aria-label={`Reorder ${title}`}
              title="Drag to reorder, or press Space and use arrow keys"
              className="-ml-1 flex h-5 w-4 shrink-0 touch-none cursor-grab items-center justify-center rounded text-island-muted hover:text-island-text focus-visible:ring-1 focus-visible:ring-island-muted focus-visible:outline-none active:cursor-grabbing"
            >
              <GripVertical size={12} aria-hidden />
            </button>
          )}
          <h2
            id={`${id}-heading`}
            className="min-w-0 text-[10px] font-semibold tracking-wider text-island-muted uppercase"
          >
            {section ? (
              <button
                type="button"
                aria-expanded={!collapsed}
                aria-controls={`${id}-body`}
                title={`${collapsed ? 'Expand' : 'Minimize'} ${title}`}
                onClick={toggle}
                className="-m-1 flex items-center gap-1 rounded p-1 text-left hover:text-island-text focus-visible:ring-1 focus-visible:ring-island-muted focus-visible:outline-none"
              >
                <Chevron size={12} aria-hidden className="shrink-0" />
                {title}
              </button>
            ) : (
              title
            )}
          </h2>
        </div>
        {collapsed ? (summary ?? aside) : aside}
      </header>
      <div id={`${id}-body`} hidden={collapsed}>
        {children}
      </div>
    </section>
  );
}
