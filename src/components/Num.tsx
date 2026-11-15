import { AnimatedNumber } from '@crystalflux/animated-number';

export function Num({
  value,
  format,
}: {
  value: number | null | undefined;
  format: (value: number | null | undefined) => string;
}) {
  if (value == null || !Number.isFinite(value)) return <>{format(value)}</>;
  return <AnimatedNumber value={value} format={format} duration={500} />;
}
