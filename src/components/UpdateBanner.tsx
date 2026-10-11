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
      className="flex items-center gap-2 pt-2 text-[12px] text-island-text"
    >
      {banner.kind === 'available' && (
        <>
          <span className="flex-1 truncate">v{banner.version} available. Update?</span>
          <button
            className="rounded-full bg-allow-bg px-3 py-1 font-semibold text-allow-text hover:opacity-90"
            onClick={() => void install()}
          >
            Yes
          </button>
          <button
            className="rounded-full bg-deny-bg px-3 py-1 text-deny-text hover:opacity-90"
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
        <span className="flex-1 truncate text-island-crit" title={banner.message}>
          {banner.message}
        </span>
      )}
    </div>
  );
}
