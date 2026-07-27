<script lang="ts">
  import { store } from '../state.svelte';

  const talisman = $derived(store.entity.talisman ?? null);
  const attunements = $derived(talisman?.attunements ?? []);
  const effects = $derived(talisman?.effects ?? []);

  // The capacity is whatever the ENGINE computed — never recomputed here. It reads
  // the magus's effective Art scores (Puissant Art folded in), so deriving it in TS
  // would fork the single evaluation path (the `itemBudget`/`itemUsed` pattern).
  const capacity = $derived(store.derived?.talisman_capacity ?? null);

  // Localized display name for a catalogue id (here: the two contributing Arts),
  // from the rules i18n map — never the raw slug as a label.
  function name(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }

  function num(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value);
  }

  function text(event: Event): string {
    return (event.currentTarget as HTMLInputElement).value;
  }
</script>

<div class="detail-section">
  <h3 class="detail-label">{store.t('talisman-label')}</h3>
  {#if talisman}
    <label class="field">
      <span>{store.t('talisman-description-label')}</span>
      <input
        type="text"
        value={talisman.description ?? ''}
        placeholder={store.t('talisman-description-placeholder')}
        oninput={(e) => store.setTalismanDescription(text(e))}
        data-testid="talisman-description"
      />
    </label>

    <!-- Capacity is guidance, not a budget: the rules cap the pawns of Vim vis used
         to prepare the item, and the model holds no vis stock to spend against it.
         Instilled effects are therefore charged against nothing at all. -->
    {#if capacity}
      <!-- `pawns` goes in as a NUMBER: the Fluent message pluralizes it with a
           `$pawns ->` selector, which a stringified argument cannot drive. -->
      <p class="budget-readout" data-testid="talisman-capacity">
        {store.t('talisman-capacity', { pawns: capacity.pawns })}
      </p>
      <!-- Both Arts are named (through the rules i18n, never as a raw slug) so the
           player can check the sum against the Arts on their sheet. -->
      <p class="hint" data-testid="talisman-capacity-note">
        {store.t('talisman-capacity-note', {
          technique: name(capacity.technique),
          techniqueScore: String(capacity.technique_score),
          form: name(capacity.form),
          formScore: String(capacity.form_score),
        })}
      </p>
    {/if}

    <h4 class="detail-label">{store.t('talisman-attunements-label')}</h4>
    <ul class="talisman-list" data-testid="talisman-list">
      {#each attunements as attunement, i (i)}
        <li>
          <input
            class="talisman-desc"
            placeholder={store.t('talisman-desc-placeholder')}
            value={attunement.description}
            oninput={(e) => store.setTalismanAttunementDescription(i, text(e))}
            data-testid="talisman-desc-{i}"
          />
          <label class="field inline">
            <span>{store.t('talisman-bonus-label')}</span>
            <input
              type="number"
              min="-128"
              max="127"
              value={attunement.bonus}
              oninput={(e) => store.setTalismanAttunementBonus(i, num(e))}
              data-testid="talisman-bonus-{i}"
            />
          </label>
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('spell-remove')}
            onclick={() => store.removeTalismanAttunementAt(i)}
            data-testid="talisman-remove-{i}"
          >
            ×
          </button>
        </li>
      {:else}
        <li class="empty">{store.t('talisman-empty')}</li>
      {/each}
    </ul>
    <button
      type="button"
      onclick={() => store.addTalismanAttunement()}
      data-testid="talisman-attunement-add"
    >
      {store.t('talisman-attunement-add')}
    </button>

    <h4 class="detail-label">{store.t('talisman-effects-label')}</h4>
    <ul class="talisman-list" data-testid="talisman-effect-list">
      {#each effects as effect, i (i)}
        <li>
          <input
            class="talisman-desc"
            placeholder={store.t('talisman-effect-name-placeholder')}
            value={effect.name}
            oninput={(e) => store.setTalismanEffectName(i, text(e))}
            data-testid="talisman-effect-name-{i}"
          />
          <label class="field inline">
            <span>{store.t('talisman-effect-level-label')}</span>
            <input
              type="number"
              min="0"
              max="65535"
              value={effect.level}
              oninput={(e) => store.setTalismanEffectLevel(i, num(e))}
              data-testid="talisman-effect-level-{i}"
            />
          </label>
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('spell-remove')}
            onclick={() => store.removeTalismanEffectAt(i)}
            data-testid="talisman-effect-remove-{i}"
          >
            ×
          </button>
        </li>
      {:else}
        <li class="empty">{store.t('talisman-effects-empty')}</li>
      {/each}
    </ul>
    <button
      type="button"
      onclick={() => store.addTalismanEffect()}
      data-testid="talisman-effect-add"
    >
      {store.t('talisman-effect-add')}
    </button>

    <button type="button" onclick={() => store.removeTalisman()} data-testid="talisman-remove-item">
      {store.t('talisman-remove-item')}
    </button>
  {:else}
    <p class="empty" data-testid="talisman-empty-item">{store.t('talisman-empty-item')}</p>
    <button type="button" onclick={() => store.addTalisman()} data-testid="talisman-add">
      {store.t('talisman-add')}
    </button>
  {/if}
</div>

<style>
  /* `.detail-section` (spacing within a section), `.field` / `.field.inline` (label
     above vs. beside its input) and `.hint` (muted advisory text) are shared globals
     in app.css. */

  /* Section-level action buttons (Add/Remove talisman, Add attunement, Add effect)
     are direct children of a `.detail-section`, which is a column flex with the
     default `align-items: stretch` — so without this they stretch to the full panel
     width. Shrink them to their label. Row-level icon buttons (the × on a list row)
     live inside an `<li>`, not as direct children, so are unaffected. */
  .detail-section > button {
    align-self: flex-start;
  }

  /* The capacity read-out mirrors the item-level budget line above it, so the two
     numbers on the tab read alike even though only one is a budget. */
  .budget-readout {
    margin: 0;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  /* Attunement and effect rows: a growing text input, an inline number field and the
     remove button, all baseline-aligned on one line. */
  .talisman-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .talisman-list li {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .talisman-desc {
    flex: 1;
    min-width: 0;
  }

  .talisman-list li.empty {
    color: var(--muted);
    display: block;
  }

  .empty {
    color: var(--muted);
  }
</style>
