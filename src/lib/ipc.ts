import { invoke as tauriInvoke, type InvokeArgs } from '@tauri-apps/api/core';
import { listen as tauriListen, type EventCallback, type UnlistenFn } from '@tauri-apps/api/event';

const BRIDGED = ['__TAURI_INTERNALS__', '__TAURI_EVENT_PLUGIN_INTERNALS__'] as const;

export function isEmbedded(): boolean {
  return new URLSearchParams(window.location.search).get('embed') === '1';
}

export function isWidget(): boolean {
  return new URLSearchParams(window.location.search).get('widget') === '1';
}

// Inside the host page's iframe the Tauri globals live on the parent window.
export function initEmbed(): void {
  if (isWidget()) document.documentElement.classList.add('embed', 'widget');
  if (!isEmbedded()) return;
  const self = window as unknown as Record<string, unknown>;
  const parent = window.parent as unknown as Record<string, unknown>;
  for (const key of BRIDGED) self[key] ??= parent[key];
  document.documentElement.classList.add('embed');
}

export function invoke<T>(cmd: string, ...args: [args?: InvokeArgs]): Promise<T> {
  return tauriInvoke<T>(isEmbedded() ? `plugin:omniscience|${cmd}` : cmd, ...args);
}

const unlisteners = new Set<UnlistenFn>();

// The host removes the iframe without running React cleanup, which would leave listeners on its window.
if (isEmbedded()) window.addEventListener('pagehide', () => unlisteners.forEach((f) => f()));

export function listen<T>(event: string, handler: EventCallback<T>): Promise<UnlistenFn> {
  const pending = tauriListen<T>(event, handler);
  if (isEmbedded()) void pending.then((f) => unlisteners.add(f));
  return pending;
}
