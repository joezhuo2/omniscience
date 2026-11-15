import { GlassTooltip } from '@crystalflux/tooltip-glass';
import type { ReactNode } from 'react';

export function Tip({ content, children }: { content?: ReactNode; children: ReactNode }) {
  if (!content) return <>{children}</>;
  return (
    <span data-no-drag className="contents">
      <GlassTooltip content={content}>{children}</GlassTooltip>
    </span>
  );
}
