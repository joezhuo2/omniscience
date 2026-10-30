import { ChevronDown, ChevronRight } from 'lucide-react';
import { CSS } from '@dnd-kit/utilities';
import { useSectionDrag } from './SectionDragContext';
import { useCallback, useId, useState, type ReactNode } from 'react';
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
  const {
    attributes,
    listeners,
    setNodeRef,
    setActivatorNodeRef,
    transform,
    isDragging,
    transition,
  } = useSectionDrag() ?? {};
  const cardRef = useCallback(
    (node: HTMLElement | null) => {
      setNodeRef?.(node);
      setActivatorNodeRef?.(node);
    },
    [setNodeRef, setActivatorNodeRef],
  );
  const [collapsed, setCollapsed] = useState(() => readSectionCollapsed(section));
  const toggle = () => {
    const next = !collapsed;
    setCollapsed(next);
    if (section) saveSectionCollapsed(section, next);
  };
  const Chevron = collapsed ? ChevronRight : ChevronDown;
  return (
    <section
      ref={cardRef}
      data-section={section}
      data-dragging={isDragging || undefined}
      tabIndex={attributes?.tabIndex}
      aria-describedby={attributes?.['aria-describedby']}
      aria-roledescription={attributes ? 'sortable section' : undefined}
      onMouseDown={listeners ? (event) => listeners.onMouseDown?.(event) : undefined}
      onTouchStart={listeners?.onTouchStart}
      onKeyDown={
        listeners
          ? (event) => {
              if (event.target === event.currentTarget) listeners.onKeyDown?.(event);
            }
          : undefined
      }
      style={
        attributes
          ? {
              transform: CSS.Translate.toString(transform ?? null),
              transition: isDragging ? undefined : transition,
              position: 'relative',
              zIndex: isDragging ? 1 : undefined,
              opacity: isDragging ? 0.85 : undefined,
            }
          : undefined
      }
      aria-labelledby={`${id}-heading`}
      className={`rounded-xl border border-island-faint bg-island-hover p-2.5${attributes ? ' section-sortable cursor-grab focus-visible:ring-1 focus-visible:ring-island-muted focus-visible:outline-none' : ''}`}
    >
      <header className={`${collapsed ? '' : 'mb-2 '}flex items-center justify-between gap-2`}>
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
        {collapsed ? (summary ?? aside) : aside}
      </header>
      <div id={`${id}-body`} hidden={collapsed}>
        {children}
      </div>
    </section>
  );
}
