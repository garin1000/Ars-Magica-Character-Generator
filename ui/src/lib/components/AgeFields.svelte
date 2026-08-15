<script lang="ts">
  import { store } from '../state.svelte';

  const age = $derived(store.entity.age ?? null);
  // The engine's age→max-Ability-score cap, echoed read-only beside the age.
  const ageCap = $derived(store.effective?.age_ability_cap ?? null);

  function onAge(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setAge(raw === '' ? null : Number(raw));
  }
</script>

<!-- The character's age and the Ability cap it implies. Extracted from
     CharacterDetails so the guided flow can mount it on the aging step: under flat
     (pool) funding there is no age field anywhere in the wizard —
     `life-stage-age-input` renders only for life-stage funding — and the age is
     the aging schedule's only input. Every surface reads and writes the one
     `entity.age`, so they cannot diverge. -->
<div class="detail-field">
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
  {#if ageCap != null}
    <span class="age-cap" data-testid="age-cap-note">
      {store.t('age-cap-note', { cap: String(ageCap) })}
    </span>
  {/if}
</div>
