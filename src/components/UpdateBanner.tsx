import { useUpdaterStore } from '../updater';

/** Slim strip under the pill: "v1.2.0 available. Update? Yes / No", or a short status. */
export function UpdateBanner() {
  const banner = useUpdaterStore((s) => s.banner);
  const install = useUpdaterStore((s) => s.install);
  const dismiss = useUpdaterStore((s) => s.dismiss);
  if (!banner) return null;
  return (
    <div
      role="status"
      data-testid="update-banner"
      className="flex h-8 items-center gap-2 border-t border-slate-700/50 px-4 text-[11px] text-slate-200"
    >
      {banner.kind === 'available' && (
        <>
          <span className="flex-1 truncate">v{banner.version} available. Update?</span>
          <button
            className="rounded bg-cyan-500 px-2 py-0.5 font-semibold text-slate-950 hover:bg-cyan-400"
            onClick={() => void install()}
          >
            Yes
          </button>
          <button
            className="rounded px-2 py-0.5 text-slate-300 hover:bg-slate-700/60"
            onClick={dismiss}
          >
            No
          </button>
        </>
      )}
      {banner.kind === 'installing' && (
        <span className="flex-1 truncate">
          Installing v{banner.version}… Omniscience will restart.
        </span>
      )}
      {banner.kind === 'up_to_date' && <span className="flex-1">Omniscience is up to date.</span>}
      {banner.kind === 'error' && (
        <span className="flex-1 truncate text-red-300" title={banner.message}>
          {banner.message}
        </span>
      )}
    </div>
  );
}
