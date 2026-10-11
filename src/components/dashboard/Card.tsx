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
    <section className="rounded-xl border border-slate-700/40 bg-slate-800/40 p-2.5">
      <header className="mb-2 flex items-center justify-between">
        <h2 className="text-[10px] font-semibold tracking-wider text-slate-400 uppercase">
          {title}
        </h2>
        {aside}
      </header>
      {children}
    </section>
  );
}
