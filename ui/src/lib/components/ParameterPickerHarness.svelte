<script lang="ts">
  // Test-only fixture for the client test environment, not part of the app —
  // `ParameterPicker.client.test.ts` mounts this so a `store.setParamAt`
  // write (which reassigns `entity.selections` to a brand-new array of
  // brand-new objects) can be handed back into a live `ParameterPicker`
  // instance. In the real app this reactivity is free: the actual caller
  // (`VirtueFlawTab.svelte`) renders `ParameterPicker` from a template
  // `{#each store.entity.selections as selection}`, so a store write is
  // picked up on the next render for no extra work. A component mounted
  // directly with a plain (non-`$state`) props object does not get that for
  // free — mutating such an object's fields after `mount()` is not observed,
  // since Svelte's reactivity tracks a `$state` proxy, not a bare object —
  // so this fixture does that one job the real parent does, via its own
  // internal `$state` plus an exported setter the test can call.
  import { untrack } from 'svelte';
  import ParameterPicker from './ParameterPicker.svelte';
  import type { ParameterDef, Selection } from '../types';

  let { initial, index, params }: { initial: Selection; index: number; params: ParameterDef[] } =
    $props();

  // Seeds from the prop ONCE, then ignores further changes to it — later
  // updates arrive only through `setSelection` below, never through `initial`
  // itself changing (nothing re-mounts this fixture with a new one). `untrack`
  // is the compiler-endorsed way to say so (svelte.dev/e/state_referenced_locally);
  // without it `initial` reads as a forgotten reactive dependency instead of
  // a deliberate one-shot seed.
  let selection = $state(untrack(() => initial));

  export function setSelection(next: Selection): void {
    selection = next;
  }
</script>

<ParameterPicker {selection} {index} {params} />
