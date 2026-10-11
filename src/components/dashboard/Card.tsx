import type { ReactNode } from 'react';

export function Card({
  title,
  aside,
  children,
}: {
  title: string;
  aside?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section className="rounded-xl border border-island-faint bg-island-hover p-2.5">
      <header className="mb-2 flex items-center justify-between">
        <h2 className="text-[10px] font-semibold tracking-wider text-island-muted uppercase">
          {title}
        </h2>
        {aside}
      </header>
      {children}
    </section>
  );
}
