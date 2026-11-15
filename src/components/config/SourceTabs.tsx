import { Segmented } from '../Segmented';

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
  const name = (value: boolean, text: string) => (activeAc === value ? `${text} •` : text);
  return (
    <Segmented
      role="tab"
      label={label}
      className="uppercase"
      items={[
        { value: 'ac', label: name(true, 'Plugged in') },
        { value: 'dc', label: name(false, 'Battery') },
      ]}
      value={ac ? 'ac' : 'dc'}
      onChange={(v) => onChange(v === 'ac')}
    />
  );
}
