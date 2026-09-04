<script lang="ts">
  import { store } from '../state.svelte';
  import { grantItemLabel, reputationRows, UNFILLED_REPUTATION_INDEX } from '../derive';
  import type { ReputationRow } from '../derive';
  import type { ReputationType } from '../types';

  const reputations = $derived(store.entity.reputations ?? []);
  // Reputation input is offered only for the slots a V/F grants (Core:2514) —
  // and the grant IS the row. There is no add control: an authorized Reputation
  // is already known, so it renders as a row waiting for its description rather
  // than a button that could be pressed twice into a duplicate.
  const grants = $derived(store.effective?.reputation_grants ?? []);
  // Granted slots first (matched to what is stored by the same two-pass rule the
  // engine's `validate_reputations` uses), then any stored Reputation no grant
  // covers. Purely derived — rendering the panel writes nothing to the entity,
  // so opening a character with granted Reputations does not dirty it.
  const rows = $derived(reputationRows(reputations, grants));
  // The Reputation taxonomy comes from the engine (`ReputationType::ALL` →
  // `reputation_type_order`), never re-hardcoded here.
  const kinds = $derived(store.ruleset?.ruleset.reputation_type_order ?? []);

  const kindLabel = (kind: ReputationType): string => store.t(`reputation-type-${kind}`);

  // The granting Virtue/Flaw under its rules name — never the raw id.
  const sourceLabel = (id: string): string =>
    store.ruleset ? grantItemLabel(store.ruleset, id, store.t) : id;

  // A grant that fixes no type (Famous) lets the player pick one.
  const isWildcard = (row: ReputationRow): boolean => row.grant?.kind === null;

  // Writes are lazy: the first character typed into an empty granted slot is what
  // creates the stored row, and emptying the description removes it again (an
  // undescribed Reputation records nothing the grant does not already say).
  function onContent(row: ReputationRow, event: Event): void {
    const content = (event.currentTarget as HTMLInputElement).value;
    if (row.index !== UNFILLED_REPUTATION_INDEX) {
      store.setReputationContent(row.index, content);
    } else if (content !== '' && row.kind) {
      store.addReputation(row.kind, row.score, content);
    }
  }

  // Choosing the type of a wildcard slot is itself a choice, so it persists
  // immediately — with or without a description. Clearing it back to the prompt
  // drops the row again.
  function onKind(row: ReputationRow, event: Event): void {
    const kind = (event.currentTarget as HTMLSelectElement).value as ReputationType | '';
    if (row.index === UNFILLED_REPUTATION_INDEX) {
      if (kind !== '') store.addReputation(kind, row.score, row.content);
    } else if (kind === '') {
      store.removeReputationAt(row.index);
    } else {
      store.setReputationKind(row.index, kind);
    }
  }
</script>

<!-- Reputations. Extracted from CharacterDetails alongside PersonalityTraits, so
     the wizard's `personality_reputations` step is those two and nothing else.

     Rows are keyed by their position in the derived list, not by the stored
     index, so the <input> a player is typing into keeps its DOM identity (and
     the caret) at the moment the first keystroke creates the stored row.

     Where two grants share a kind — Apostate and Senior Clergy both grant
     Ecclesiastical 4 — which row is attributed to which is arbitrary: nothing
     links a stored Reputation back to one particular grant, and `Entity::normalize`
     re-sorts them on (kind, score, content) at every save anyway. Both rows are
     always shown and always fillable; only the attribution label may swap.

     The SECTION carries its own testid because the list inside it no longer
     always exists: with no grant and nothing stored there are no rows at all,
     only the empty message. Anything asserting "the Reputations half of the
     step is mounted" must key on the section, not on the list. -->
<div class="detail-section" data-testid="reputations">
  <h3 class="detail-label">{store.t('reputations-label')}</h3>
  {#if rows.length === 0}
    <p class="empty" data-testid="reputation-empty">{store.t('reputation-empty')}</p>
  {:else}
    <ul class="reputation-list" data-testid="reputation-list">
      {#each rows as row, i (i)}
        <li>
          {#if isWildcard(row)}
            <label class="field inline">
              <span>{store.t('reputation-kind-label')}</span>
              <select
                value={row.kind ?? ''}
                onchange={(e) => onKind(row, e)}
                data-testid="reputation-kind-{i}"
              >
                <option value="">{store.t('reputation-kind-choose')}</option>
                {#each kinds as kind (kind)}
                  <option value={kind}>{kindLabel(kind)}</option>
                {/each}
              </select>
            </label>
          {:else if row.kind}
            <span class="reputation-tag">
              {kindLabel(row.kind)}
              {#if !row.grant}{row.score}{/if}
            </span>
          {/if}
          {#if row.grant}
            <span class="reputation-source" data-testid="reputation-source-{i}"
              >{store.t('reputation-granted-by', {
                score: row.score,
                source: sourceLabel(row.grant.source),
              })}</span
            >
          {/if}
          <input
            class="reputation-content"
            placeholder={store.t('reputation-content-placeholder')}
            aria-label={store.t('reputation-content-placeholder')}
            value={row.content}
            disabled={row.kind === null}
            oninput={(e) => onContent(row, e)}
            data-testid="reputation-content-{i}"
          />
          <!-- Only an ungranted row can be removed: a legacy or hand-edited save
               must be clearable, but a granted slot is not the player's to drop. -->
          {#if !row.grant}
            <button
              type="button"
              class="icon-btn"
              aria-label={store.t('remove-item', { name: row.content })}
              onclick={() => store.removeReputationAt(row.index)}
              data-testid="reputation-remove-{i}"
            >
              ×
            </button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>
