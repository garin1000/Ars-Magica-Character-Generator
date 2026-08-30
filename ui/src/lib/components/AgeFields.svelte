<script lang="ts">
  import { store } from '../state.svelte';

  /**
   * Show the age as a read-out instead of an input.
   *
   * An explicit prop, declared by whoever mounts the component — the same seam
   * `SpellBudgetBar`'s `readonlyBase` established (#19) — and deliberately NOT a
   * `store` lookup asking which flow or funding mode is running. One component with
   * a stated parameter, not two behaviours hidden in one file.
   */
  let { readonly = false }: { readonly?: boolean } = $props();

  const age = $derived(store.entity.age ?? null);

  function onAge(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setAge(raw === '' ? null : Number(raw));
  }
</script>

<!-- The character's age. Extracted from CharacterDetails so the guided flow can
     mount it too, and since Slice 12 (#24) the ONE editable age surface in either
     flow: the editor's Details tab and the wizard's Concept step both mount it right
     after `IdentityFields`, beside the birth year it is now linked to.

     The Experience step mounts it `readonly` instead — a magus carries two ages, and
     that step has to show the age its Gauntlet age is measured against without
     becoming a second place to change it. The age→Ability-score cap is no longer
     echoed here at all: it has one home now, beside the Ability lists it constrains. -->
<div class="detail-field">
  {#if readonly}
    <!-- A `<span>` label rather than a `<label>`: there is no form control to name,
         and a `<label>` pointing at nothing is worse than no label element at all.
         Mirrors the `confidence-readout` read-out in `CharacterDetails`. -->
    <span class="detail-label">{store.t('age-label')}</span>
    <span data-testid="age-readout">{age ?? ''}</span>
  {:else}
    <label class="field">
      <span>{store.t('age-label')}</span>
      <input
        type="number"
        min="1"
        max="4294967295"
        value={age ?? ''}
        oninput={onAge}
        data-testid="age-input"
      />
    </label>
  {/if}
</div>
