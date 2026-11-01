export function SourceTabs({
  label,
  ac,
  activeAc,
  onChange,
}: {
  label: string;
  ac: boolean;
  activeAc: boolean | null | undefined;
  onChange: (ac: boolean) => void;
}) {
  return (
    <div
      role="tablist"
      aria-label={label}
      className="mb-2 flex w-fit rounded-md bg-island-hover p-0.5 text-[10px]"
    >
      {[
        { value: true, name: 'Plugged in' },
        { value: false, name: 'Battery' },
      ].map(({ value, name }) => (
        <button
          key={name}
          type="button"
          role="tab"
          aria-selected={ac === value}
          className={`rounded px-1.5 py-0.5 font-medium uppercase transition-colors ${
            ac === value
              ? 'bg-island-track text-island-text'
              : 'text-island-muted hover:text-island-text'
          }`}
          onClick={() => onChange(value)}
        >
          {name}
          {activeAc === value && (
            <span
              title="Current power source"
              className="ml-1 inline-block h-1 w-1 rounded-full bg-current align-middle"
            />
          )}
        </button>
      ))}
    </div>
  );
}
