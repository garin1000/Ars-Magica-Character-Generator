import { flushSync, mount, unmount } from 'svelte';
import { describe, expect, it } from 'vitest';

import EffectProbe from './components/EffectProbe.svelte';

// Guards the client test environment itself, not any product behavior.
//
// The default vitest environment for this project is `node`, and every
// `*.test.ts` renders through `svelte/server` — server-side rendering, which
// never executes an `$effect`. That made a whole class of behavior untestable:
// the unsaved-changes guard's `update_close_guard` IPC mirror lives in an
// `$effect` keyed on `store.dirty`, so no test could observe it firing (the
// gap the full-codebase audit recorded as E2).
//
// `vitest.config.ts` maps `*.client.test.ts` to the `happy-dom` environment so
// these files get a real DOM and can `mount()` a component client-side, which
// is what makes `$effect` run. If that mapping regresses, `mount()` starts
// throwing `lifecycle_function_unavailable` (it resolves to Svelte's server
// build) and this file fails loudly instead of the effect silently never
// running — which is the failure mode worth guarding, because a test that
// silently does not execute its assertion still reports green.
describe('client test environment', () => {
  it('provides a real DOM document', () => {
    expect(typeof document).toBe('object');
    expect(document.body).toBeTruthy();
  });

  it('runs $effect on a client-mounted component, unlike the SSR default', () => {
    const target = document.createElement('div');
    document.body.appendChild(target);

    const runs: number[] = [];
    const component = mount(EffectProbe, {
      target,
      props: { onEffect: (n: number) => runs.push(n) },
    });

    // `mount()` schedules effects; `flushSync` runs them without waiting a tick.
    flushSync();
    expect(runs).toEqual([0]);

    unmount(component);
    target.remove();
  });
});
