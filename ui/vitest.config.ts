import { svelte, vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vitest/config';

// Unit tests for the in-memory logic in `src/lib` (derive.ts, the AppStore in
// state.svelte.ts, …). These exercise pure logic only — no DOM, no Tauri IPC —
// so a plain Node environment is enough. E2E lives separately under `e2e/`.
//
// The Svelte plugin is loaded so `*.svelte.ts` modules (and their
// `*.svelte.test.ts` tests) get the rune transform: without it `$state` /
// `$derived` are undefined at runtime.
export default defineConfig({
  plugins: [svelte({ preprocess: vitePreprocess() })],
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
    exclude: ['node_modules/', 'dist/', 'e2e/'],
  },
});
