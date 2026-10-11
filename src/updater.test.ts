import type { Update } from '@tauri-apps/plugin-updater';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useUpdaterStore, type UpdaterDeps } from './updater';

function fakeUpdate(version: string, install = vi.fn(async () => {})): Update {
  return { version, downloadAndInstall: install } as unknown as Update;
}

function deps(update: Update | null | Error): UpdaterDeps & { relaunch: ReturnType<typeof vi.fn> } {
  return {
    check: vi.fn(async () => {
      if (update instanceof Error) throw update;
      return update;
    }),
    relaunch: vi.fn(async () => {}),
  };
}

describe('updater store', () => {
  beforeEach(() => useUpdaterStore.setState({ banner: null }));

  it('offers an available update', async () => {
    await useUpdaterStore.getState().checkNow(false, deps(fakeUpdate('1.2.0')));
    expect(useUpdaterStore.getState().banner).toEqual({ kind: 'available', version: '1.2.0' });
  });

  it('stays silent on a launch check with no update or a network error', async () => {
    await useUpdaterStore.getState().checkNow(false, deps(null));
    await useUpdaterStore.getState().checkNow(false, deps(new Error('offline')));
    expect(useUpdaterStore.getState().banner).toBeNull();
  });

  it('reports up to date and errors on a manual check', async () => {
    await useUpdaterStore.getState().checkNow(true, deps(null));
    expect(useUpdaterStore.getState().banner).toEqual({ kind: 'up_to_date' });
    await useUpdaterStore.getState().checkNow(true, deps(new Error('offline')));
    expect(useUpdaterStore.getState().banner).toMatchObject({ kind: 'error' });
  });

  it('installs and relaunches on Yes', async () => {
    const install = vi.fn(async () => {});
    const d = deps(fakeUpdate('1.2.0', install));
    await useUpdaterStore.getState().checkNow(false, d);
    await useUpdaterStore.getState().install(d);
    expect(install).toHaveBeenCalledOnce();
    expect(d.relaunch).toHaveBeenCalledOnce();
  });

  it('shows the error and does not relaunch when install fails', async () => {
    const d = deps(
      fakeUpdate(
        '1.2.0',
        vi.fn(async () => {
          throw new Error('bad signature');
        }),
      ),
    );
    await useUpdaterStore.getState().checkNow(false, d);
    await useUpdaterStore.getState().install(d);
    expect(d.relaunch).not.toHaveBeenCalled();
    expect(useUpdaterStore.getState().banner).toMatchObject({ kind: 'error' });
  });

  it('dismisses on No', async () => {
    await useUpdaterStore.getState().checkNow(false, deps(fakeUpdate('1.2.0')));
    useUpdaterStore.getState().dismiss();
    expect(useUpdaterStore.getState().banner).toBeNull();
  });
});
