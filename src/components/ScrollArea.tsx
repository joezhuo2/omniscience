import { useSmoothScroll } from '@crystalflux/smooth-scroll';
import { useRef, type HTMLAttributes } from 'react';

export function ScrollArea(props: HTMLAttributes<HTMLDivElement>) {
  const ref = useRef<HTMLDivElement>(null);
  useSmoothScroll(ref);
  return <div ref={ref} {...props} />;
}
