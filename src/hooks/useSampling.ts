import { useEffect } from 'react';
import { invoke } from '../lib/ipc';

/** How many mounted users want each section; several cards may share one. */
const wanted = new Map<string, number>();
let queued = false;

function flush() {
  queued = false;
  void invoke('set_sampled_sections', { sections: [...wanted.keys()] }).catch(() => {});
}

function change(sections: readonly string[], delta: 1 | -1) {
  for (const s of sections) {
    const n = (wanted.get(s) ?? 0) + delta;
    if (n > 0) wanted.set(s, n);
    else wanted.delete(s);
  }
  // One call per burst (mount, unmount), so the collector wakes once.
  if (!queued) {
    queued = true;
    queueMicrotask(flush);
  }
}

/** Asks the backend to collect `sections` while `active`. */
export function useSampling(sections: readonly string[], active: boolean) {
  const key = sections.join(',');
  useEffect(() => {
    if (!active || !key) return;
    const list = key.split(',');
    change(list, 1);
    return () => change(list, -1);
  }, [key, active]);
}
