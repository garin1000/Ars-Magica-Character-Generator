<script lang="ts">
  import { store } from '../state.svelte';
  import { commitStored } from '../actions';

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

  // manual-testing-findings-2026-11-15 #9 (F9): the birth-year pattern
  // (`IdentityFields.svelte::onBirthYearInput`). An empty read forwards as `null`
  // (a genuine clear is legitimate), but a non-empty one only reaches the store
  // once it is within [1, ageInputMax] — otherwise every keystroke past the bound
  // reached `setAge` unconditionally, clamped, and the plain `value={age ?? ''}`
  // binding snapped the field to the clamped number before typing finished.
  function onAgeInput(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    if (raw === '') {
      store.setAge(null);
      return;
    }
    const typed = Number(raw);
    if (!Number.isFinite(typed) || typed < 1 || typed > store.ageInputMax) return;
    store.setAge(typed);
  }

  // Commit (blur or Enter): clamps the full typed text, exactly as `onAgeInput`
  // would have had it stayed in bounds the whole time.
  function commitAge(raw: string) {
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
    <!-- manual-testing-findings #10: a blank read-out beside the label read as a
         broken field rather than a character with no recorded age. -->
    <span data-testid="age-readout">{age ?? store.t('age-unknown')}</span>
  {:else}
    <label class="field">
      <span>{store.t('age-label')}</span>
      <input
        type="number"
        min="1"
        max={store.ageInputMax}
        value={age ?? ''}
        oninput={onAgeInput}
        use:commitStored={{ read: () => store.entity.age, commit: commitAge }}
        data-testid="age-input"
      />
    </label>
  {/if}
</div>
