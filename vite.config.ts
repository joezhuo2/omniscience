/// <reference types="vitest/config" />
import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;
const vendored = fileURLToPath(new URL('./src/vendor/crystal/', import.meta.url));
// Crystal packages not on npm are copied into src/vendor/crystal; the two published ones resolve from node_modules.
const alias = [
  {
    find: /^@crystalflux\/(?!color-picker$)([\w-]+)$/,
    replacement: `${vendored}$1/index.ts`,
  },
];

// https://v2.tauri.app/start/frontend/vite/
export default defineConfig({
  base: './',
  plugins: [react(), tailwindcss()],
  resolve: { alias },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'oxc' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
    server: { deps: { inline: [/@crystalflux\//] } },
  },
});
