import { invoke as tauriInvoke, type InvokeArgs } from '@tauri-apps/api/core';

const BRIDGED = ['__TAURI_INTERNALS__', '__TAURI_EVENT_PLUGIN_INTERNALS__'] as const;

export function isEmbedded(): boolean {
  return new URLSearchParams(window.location.search).get('embed') === '1';
}

// Inside the host page's iframe the Tauri globals live on the parent window.
export function initEmbed(): void {
  if (!isEmbedded()) return;
  const self = window as unknown as Record<string, unknown>;
  const parent = window.parent as unknown as Record<string, unknown>;
  for (const key of BRIDGED) self[key] ??= parent[key];
  document.documentElement.classList.add('embed');
}

export function invoke<T>(cmd: string, ...args: [args?: InvokeArgs]): Promise<T> {
  return tauriInvoke<T>(isEmbedded() ? `plugin:omniscience|${cmd}` : cmd, ...args);
}
