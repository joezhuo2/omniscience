export function Num({
  value,
  format,
}: {
  value: number | null | undefined;
  format: (value: number | null | undefined) => string;
}) {
  // Plain text: the WebView renders in software, so a tween repaints the window every frame.
  return <>{format(value)}</>;
}
