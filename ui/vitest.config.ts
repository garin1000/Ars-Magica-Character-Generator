import { svelte, vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vitest/config';

// Unit tests for the in-memory logic in `src/lib` (derive.ts, the AppStore in
// state.svelte.ts, …). These exercise pure logic only — no DOM, no Tauri IPC —
// so a plain Node environment is enough. E2E lives separately under `e2e/`.
//
// The Svelte plugin is loaded so `*.svelte.ts` modules (and their
// `*.svelte.test.ts` tests) get the rune transform: without it `$state` /
// `$derived` are undefined at runtime.
// `configFile: false` + `style: false` skips CSS preprocessing for the test run.
// Vitest hands the Svelte plugin only a partial Vite environment, so
// `vitePreprocess`'s CSS pass throws ("Cannot create proxy with a non-object as
// target or handler") on any component carrying a `<style>` block — which would
// make every component with scoped styles untestable (LongevityPanel, and the
// panels beside it). `configFile: false` is required because `svelte.config.js`
// applies `vitePreprocess()` again and would otherwise reinstate the CSS pass.
// The project writes plain CSS with nothing to transform, and these tests assert
// on rendered markup, never on styles; `vite.config.ts` keeps the full
// preprocessor for the real build.
export default defineConfig({
  plugins: [svelte({ configFile: false, preprocess: vitePreprocess({ style: false }) })],
  test: {
    environment: 'node',
    // `e2e/**/*.test.js` is the pure logic *inside* the e2e harness (the display
    // preflight), not the suite itself: the wdio specs are named `*.e2e.js` and
    // stay out of this run, which is why `e2e/specs/` is excluded rather than
    // the whole directory.
    include: ['src/**/*.test.ts', 'e2e/**/*.test.js'],
    exclude: ['node_modules/', 'dist/', 'e2e/specs/'],
  },
});
