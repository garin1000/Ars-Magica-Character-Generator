<script lang="ts">
  import { store } from '../state.svelte';
  import { commitStored } from '../actions';
  import Spinner from './Spinner.svelte';

  const traits = $derived(store.entity.personality_traits ?? []);

  // manual-testing-findings-2026-11-15 #9 (F9): the value is signed (|6| max,
  // S29). A `type="number"` input reads an incomplete entry — a bare "-" on the
  // way to "-4" — back as the empty string, so forwarding every keystroke
  // unconditionally (the old `Number(raw) || 0`) wrote a 0 for it, and the plain
  // `value={trait.value}` binding snapped the field back before the digit after
  // the sign could ever be typed. An empty read must never be forwarded while
  // typing; the existing `|| 0` fallback still applies on commit.
  function onValueInput(index: number, event: Event): void {
    const raw = (event.currentTarget as HTMLInputElement).value;
    if (raw === '') return;
    const typed = Number(raw);
    if (!Number.isFinite(typed) || typed < -6 || typed > 6) return;
    store.setPersonalityTraitValue(index, typed);
  }

  function commitValue(index: number, raw: string): void {
    store.setPersonalityTraitValue(index, Number(raw) || 0);
  }
</script>

<!-- Personality Traits. Extracted from CharacterDetails so the wizard's
     `personality_reputations` step can mount it (with Reputations) without the
     rest of the details panel. -->
<div class="detail-section">
  <h3 class="detail-label">{store.t('personality-label')}</h3>
  <ul class="trait-list" data-testid="personality-list">
    {#each traits as trait, i (i)}
      <li>
        <input
          class="trait-name"
          placeholder={store.t('personality-name-placeholder')}
          aria-label={store.t('personality-name-placeholder')}
          value={trait.name}
          oninput={(e) =>
            store.setPersonalityTraitName(i, (e.currentTarget as HTMLInputElement).value)}
          data-testid="personality-name-{i}"
        />
        <Spinner
          decLabel={store.t('characteristic-decrement', { name: trait.name })}
          decTestid="personality-dec-{i}"
          onDec={() => store.setPersonalityTraitValue(i, trait.value - 1)}
          incLabel={store.t('characteristic-increment', { name: trait.name })}
          incTestid="personality-inc-{i}"
          onInc={() => store.setPersonalityTraitValue(i, trait.value + 1)}
        >
          <!-- Bound number input, not just the spinner: a Personality Trait's
               |value| can reach 6 (a Personality Flaw, Core Rules), which is up to
               twelve clicks from 0 with no other way in — every other scored
               control with a wide/signed range (aging points, Talisman
               bonus/level) offers direct entry too (S29, full-audit UX). The
               range mirrors the store's own clamp (`setPersonalityTraitValue`). -->
          <input
            type="number"
            class="spinner-value-input"
            min="-6"
            max="6"
            value={trait.value}
            aria-label={store.t('personality-value-label', { name: trait.name })}
            oninput={(e) => onValueInput(i, e)}
            use:commitStored={{ read: () => trait.value, commit: (raw) => commitValue(i, raw) }}
            data-testid="personality-value-{i}"
          />
        </Spinner>
        <button
          type="button"
          class="icon-btn"
          aria-label={store.t('remove-item', { name: trait.name })}
          onclick={() => store.removePersonalityTraitAt(i)}
          data-testid="personality-remove-{i}"
        >
          ×
        </button>
      </li>
    {:else}
      <li class="empty">{store.t('personality-empty')}</li>
    {/each}
  </ul>
  <button type="button" onclick={() => store.addPersonalityTrait()} data-testid="personality-add">
    {store.t('personality-add')}
  </button>
</div>
