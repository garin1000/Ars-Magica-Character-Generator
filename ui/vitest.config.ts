import { svelte, vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vitest/config';

// Unit tests for the in-memory logic in `src/lib` (derive.ts, the AppStore in
// state.svelte.ts, …). These exercise pure logic only — no DOM, no Tauri IPC —
// so a plain Node environment is enough. E2E lives separately under `e2e/`.
//
// TWO ENVIRONMENTS. The `node` default above covers the great majority of tests:
// pure logic, plus components rendered through `render` from `svelte/server`.
// SSR never executes an `$effect`, so anything living in one is invisible to
// those tests — and invisible in the worst way, because the assertion simply
// never runs and the test still reports green. `environmentMatchGlobs` therefore
// routes `*.client.test.ts` to `happy-dom`, where a component can be `mount()`ed
// client-side and its effects actually fire. Pick the environment by what you
// need to observe, not by preference — see CLAUDE.md, "Frontend test
// environments", for which to reach for and why.
//
// `environmentMatchGlobs` is the mechanism available on vitest 2.x; the
// `test.projects` API that supersedes it needs vitest >= 3.2, which this project
// has not moved to yet.
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
  // NOTE: the `browser` resolve condition that client tests need is set per
  // project in `vitest.workspace.ts`, never here. Setting it globally resolves
  // the whole `svelte` package to its client build, and then any component using
  // `onMount` blows up under the SSR tests with `lifecycle_outside_component`
  // (App.test.ts's 12 tests, specifically). The two styles need different module
  // resolution, which is why they are separate projects.
  // `environment`, `include` and `exclude` are deliberately NOT set here. They
  // live in `vitest.workspace.ts`, once per project. `extends` deep-merges this
  // block into every project, so an `include` here is inherited by both and the
  // `client` project ends up running the entire SSR suite under `happy-dom` —
  // observed as the whole suite executing twice (65 files / 1600 tests) with 327
  // failures. Keeping this file to shared plugin setup only avoids that.
});
