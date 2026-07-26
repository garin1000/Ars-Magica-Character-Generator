<script lang="ts">
  import { store } from '../state.svelte';

  const attunements = $derived(store.entity.talisman_attunements ?? []);

  function num(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value);
  }
</script>

<div class="detail-section">
  <h3 class="detail-label">{store.t('talisman-label')}</h3>
  <ul class="talisman-list" data-testid="talisman-list">
    {#each attunements as attunement, i (i)}
      <li>
        <input
          class="talisman-desc"
          placeholder={store.t('talisman-desc-placeholder')}
          value={attunement.description}
          oninput={(e) =>
            store.setTalismanDescription(i, (e.currentTarget as HTMLInputElement).value)}
          data-testid="talisman-desc-{i}"
        />
        <label class="field inline">
          <span>{store.t('talisman-bonus-label')}</span>
          <input
            type="number"
            value={attunement.bonus}
            oninput={(e) => store.setTalismanBonus(i, num(e))}
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
  <button type="button" onclick={() => store.addTalismanAttunement()} data-testid="talisman-add">
    {store.t('talisman-add')}
  </button>
</div>

<style>
  /* `.detail-section` (spacing within a section) and `.field.inline` (label
     beside its input) are shared globals in app.css. */

  /* Section-level action buttons (Add attunement) are direct children of a
     `.detail-section`, which is a column flex with the default
     `align-items: stretch` — so without this they stretch to the full panel
     width. Shrink them to their label. Row-level icon buttons (the × on a list
     row) live inside an `<li>`, not as direct children, so are unaffected. */
  .detail-section > button {
    align-self: flex-start;
  }

  /* Attunement rows: a growing description input, an inline number field and the
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
