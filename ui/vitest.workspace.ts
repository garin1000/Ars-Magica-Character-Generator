import { defineWorkspace } from 'vitest/config';

// Two test projects, because SSR tests and client tests need *different module
// resolution* and cannot share one config.
//
// `ssr` is the long-standing default and holds almost everything: pure logic
// plus components rendered through `render` from `svelte/server`. It is fast and
// needs no DOM.
//
// `client` exists because SSR never executes an `$effect`. That left a whole
// class of behavior untestable — most importantly the unsaved-changes guard,
// whose `update_close_guard` IPC mirror lives in an `$effect` keyed on
// `store.dirty` (the audit finding E2). Worse than untested: an assertion placed
// after an effect that never fires still reports green, so the gap was invisible.
//
// The split is not stylistic. A client test needs `mount()`, which lives only in
// Svelte's client build, and reaching it requires the `browser` resolve
// condition. Applying that condition globally resolves the whole `svelte`
// package to its client build, and then every component using `onMount` fails
// under the SSR renderer with `lifecycle_outside_component`. So the condition is
// scoped to this project alone — which is exactly what a project boundary is
// for, and why `environmentMatchGlobs` (a per-file *environment* switch) is not
// sufficient on its own: it changes the DOM, not the module resolution.
//
// Adding a test: default to `ssr`. Name a file `*.client.test.ts` only when you
// must observe something that requires a live component instance — an `$effect`
// firing, focus management, an event listener, a lifecycle hook. See CLAUDE.md,
// "Frontend test environments".
export default defineWorkspace([
  {
    extends: './vitest.config.ts',
    test: {
      name: 'ssr',
      environment: 'node',
      // `e2e/**/*.test.js` is the pure logic *inside* the e2e harness (the
      // display preflight), not the suite itself: the wdio specs are named
      // `*.e2e.js` and stay out of this run, which is why `e2e/specs/` is
      // excluded rather than the whole directory.
      include: ['src/**/*.test.ts', 'e2e/**/*.test.js'],
      // `*.client.test.ts` is excluded here and owned by the `client` project
      // below; without this exclusion every client test would also run under
      // `node`, where `mount()` throws.
      exclude: ['node_modules/', 'dist/', 'e2e/specs/', 'src/**/*.client.test.ts'],
    },
  },
  {
    extends: './vitest.config.ts',
    resolve: { conditions: ['browser'] },
    test: {
      name: 'client',
      environment: 'happy-dom',
      include: ['src/**/*.client.test.ts'],
      exclude: ['node_modules/', 'dist/', 'e2e/specs/'],
    },
  },
]);
