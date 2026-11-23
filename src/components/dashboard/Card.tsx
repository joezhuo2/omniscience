import { CSS } from '@dnd-kit/utilities';
import { GlassAccordion, GlassAccordionItem } from '@crystalflux/accordion-glass';
import { GlassSwitch } from '@crystalflux/switch-glass';
import { useSectionDrag } from './SectionDragContext';
import {
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type MouseEvent,
  type ReactNode,
} from 'react';
import { useSampling } from '../../hooks/useSampling';
import { InfoTabVisible } from './infoTabContext';
import { useSectionToggle } from '../config/sectionToggleContext';
import {
  DEFAULT_SECTION_ORDER,
  readSectionCollapsed,
  saveSectionCollapsed,
} from '../../lib/sections';

const INTERACTIVE =
  'button, a, input, select, textarea, label, [role=slider], [role=tab], [contenteditable], [data-no-drag]';
const DRAG_DISTANCE = 6;

export function Card({
  title,
  section,
  aside,
  summary,
  onOpenChange,
  samples,
  children,
}: {
  title: string;
  section?: string;
  aside?: ReactNode;
  summary?: ReactNode;
  /** Called with whether the card's content is shown, on mount and on every change. */
  onOpenChange?: (open: boolean) => void;
  /** Backend sections to collect while the card is open; defaults to `section`. */
  samples?: readonly string[];
  children: ReactNode;
}) {
  const power = useSectionToggle();
  const off = power?.enabled === false;
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
  const press = useRef<{ x: number; y: number } | null>(null);
  const dragged = useRef(false);
  useEffect(() => {
    if (isDragging) dragged.current = true;
  }, [isDragging]);
  const open = !off && (!section || !collapsed);
  useEffect(() => onOpenChange?.(open), [open, onOpenChange]);
  const infoVisible = useContext(InfoTabVisible);
  const dashboard = (DEFAULT_SECTION_ORDER as readonly string[]).includes(section ?? '');
  useSampling(samples ?? (dashboard && section ? [section] : []), open && infoVisible);
  const setOpen = (open: boolean) => {
    setCollapsed(!open);
    if (section) saveSectionCollapsed(section, !open);
  };
  const surfaceClick = (event: MouseEvent<HTMLElement>) => {
    if (!section || (event.target as Element).closest(INTERACTIVE)) return;
    const from = press.current;
    if (dragged.current || isDragging) return;
    if (from && Math.hypot(event.clientX - from.x, event.clientY - from.y) > DRAG_DISTANCE) return;
    if (window.getSelection()?.toString()) return;
    setOpen(collapsed);
  };
  const noDrag = (target: EventTarget) => (target as Element).closest('[data-no-drag]') != null;
  return (
    <section
      ref={cardRef}
      onClick={surfaceClick}
      data-section={section}
      data-collapsed={collapsed}
      data-dragging={isDragging || undefined}
      tabIndex={attributes?.tabIndex}
      aria-describedby={attributes?.['aria-describedby']}
      aria-roledescription={attributes ? 'sortable section' : undefined}
      role="article"
      aria-label={title}
      onMouseDownCapture={(event) => {
        press.current = { x: event.clientX, y: event.clientY };
        dragged.current = false;
      }}
      onTouchStartCapture={() => {
        press.current = null;
        dragged.current = false;
      }}
      onMouseDown={
        listeners
          ? (event) => {
              if (!noDrag(event.target)) listeners.onMouseDown?.(event);
            }
          : undefined
      }
      onTouchStart={
        listeners
          ? (event) => {
              if (!noDrag(event.target)) listeners.onTouchStart?.(event);
            }
          : undefined
      }
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
      className={`card-accordion relative min-w-0 shrink-0 rounded-xl ${attributes ? 'section-sortable cursor-grab focus-visible:ring-1 focus-visible:ring-island-muted focus-visible:outline-none' : ''}`}
    >
      <GlassAccordion radius={12} className="card-surface">
        <GlassAccordionItem
          title={title}
          open={open}
          onOpenChange={section && !off ? setOpen : () => {}}
          aside={
            <div className="flex shrink-0 items-center gap-2">
              {!off && (collapsed ? (summary ?? aside) : aside)}
              {power && (
                <GlassSwitch
                  checked={!off}
                  onChange={(on) => power.setEnabled(on)}
                  label={<span className="sr-only">{`${title} enabled`}</span>}
                />
              )}
            </div>
          }
        >
          {!off && children}
        </GlassAccordionItem>
      </GlassAccordion>
    </section>
  );
}
