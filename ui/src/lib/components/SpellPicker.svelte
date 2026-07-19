<script lang="ts">
  import { store } from '../state.svelte';
  import {
    artAbbreviation,
    artLabel,
    effectiveSpellMastery,
    filterSpells,
    groupArtsByType,
    maxAbilityScore,
    spellMasteryXpSpent,
    spellName,
  } from '../derive';
  import type { Art, Spell } from '../types';

  // Technique/Form/text/level filters live on the store, so they survive the tab
  // switch that unmounts this component (same split ArtGrid uses for the Arts).
  // `generalLevel` is transient add-a-spell state, so it stays local.
  const filter = $derived(store.filters.spells);
  let generalLevel = $state(15);

  const techniques = $derived.by((): Art[] => {
    if (!store.ruleset) return [];
    return groupArtsByType(store.ruleset).find((g) => g.artType === 'technique')?.arts ?? [];
  });
  const forms = $derived.by((): Art[] => {
    if (!store.ruleset) return [];
    return groupArtsByType(store.ruleset).find((g) => g.artType === 'form')?.arts ?? [];
  });

  // Catalogue spells matching the Technique/Form (separate and combined), text
  // search, and level filters, sorted by name.
  const candidates = $derived.by((): Spell[] => {
    const rs = store.ruleset;
    if (!rs) return [];
    return filterSpells(
      rs,
      Object.values(rs.ruleset.spells ?? {}),
      {
        text: filter.search,
        technique: filter.technique || undefined,
        form: filter.form || undefined,
        level: filter.level,
      },
      store.t,
    ).sort((a, b) => spellName(rs, a.id).localeCompare(spellName(rs, b.id)));
  });

  // The spell-levels budget bar: engine-authoritative used/budget, with a local
  // fallback for the first frame before effective scores arrive.
  const budget = $derived(store.effective?.spell_levels_budget ?? 0);
  const used = $derived(store.effective?.spell_levels_used ?? 0);
  // Spell-Mastery: XP pool (Mastered Spells) + auto-mastery floor (Flawless Magic).
  const masteryXp = $derived(store.effective?.spell_mastery_xp ?? 0);
  const masteryFloor = $derived(store.effective?.spell_mastery_floor ?? 0);
  // The Mastery Ability rises like an Ability, so it is priced from the Ability
  // advancement table (data, not a hardcoded mechanic — same path as abilities).
  const advancement = $derived(store.ruleset?.ruleset.advancement ?? []);
  const masteryMax = $derived(maxAbilityScore(advancement));
  const masteryUsed = $derived(spellMasteryXpSpent(advancement, store.entity.spells ?? []));

  function abbr(artId: string): string {
    return store.ruleset ? artAbbreviation(store.ruleset, artId) : '';
  }

  // "CrIg 20" / "ReVi Gen" — the Technique+Form tag plus level, appended to names.
  function tag(spell: Spell): string {
    const tf = `${abbr(spell.technique)}${abbr(spell.form)}`;
    return spell.level == null ? `${tf} Gen` : `${tf} ${spell.level}`;
  }

  function optionLabel(spell: Spell): string {
    const rs = store.ruleset;
    return rs ? `${spellName(rs, spell.id)} (${tag(spell)})` : spell.id;
  }

  // A chosen row's display: name + its resolved TeFo/level tag.
  function rowLabel(spellId: string, level: number | null | undefined): string {
    const rs = store.ruleset;
    const cat = rs?.ruleset.spells?.[spellId];
    if (!rs || !cat) return spellId;
    const lvl = cat.level ?? level ?? null;
    const tf = `${abbr(cat.technique)}${abbr(cat.form)}`;
    return `${spellName(rs, spellId)} (${tf}${lvl == null ? ' Gen' : ` ${lvl}`})`;
  }

  // Clicking a source row adds the spell. A General spell (no fixed level) takes
  // the level from the always-visible General-level input; a fixed-level spell
  // ignores it.
  function add(spell: Spell) {
    store.addSpell(spell.id, spell.level == null ? generalLevel : undefined);
  }
</script>

{#if store.ruleset}
  <div class="region-row">
    <section class="region region-source">
      <h2 class="region-title">{store.t('available-title')}</h2>
      <section class="panel">
        <div class="filter-bar">
          <input
            type="search"
            class="filter-search"
            placeholder={store.t('filter-search-placeholder')}
            bind:value={filter.search}
            data-testid="spell-search"
          />
          <select
            bind:value={filter.technique}
            aria-label={store.t('spell-technique-label')}
            data-testid="spell-technique-filter"
          >
            <option value="">{store.t('spell-technique-label')}</option>
            {#each techniques as t (t.id)}
              <option value={t.id}>{artLabel(store.ruleset, t.id)}</option>
            {/each}
          </select>
          <select
            bind:value={filter.form}
            aria-label={store.t('spell-form-label')}
            data-testid="spell-form-filter"
          >
            <option value="">{store.t('spell-form-label')}</option>
            {#each forms as f (f.id)}
              <option value={f.id}>{artLabel(store.ruleset, f.id)}</option>
            {/each}
          </select>
          <label class="field">
            <span>{store.t('spell-level-label')}</span>
            <input
              type="number"
              min="1"
              step="1"
              bind:value={filter.level}
              data-testid="spell-level-filter"
            />
          </label>
          <label class="field">
            <span>{store.t('spell-general-level-label')}</span>
            <input
              type="number"
              min="1"
              step="1"
              bind:value={generalLevel}
              data-testid="spell-level-input"
            />
          </label>
        </div>
        <div class="list-scroll">
          <ul class="item-list">
            {#each candidates as spell (spell.id)}
              <li>
                <button
                  type="button"
                  class="pick-row"
                  onclick={() => add(spell)}
                  data-testid="add-{spell.id}"
                >
                  <span class="item-name">{optionLabel(spell)}</span>
                  <span class="pick-plus" aria-hidden="true">+</span>
                </button>
              </li>
            {/each}
          </ul>
        </div>
      </section>
    </section>

    <section class="region region-selected">
      <h2 class="region-title">{store.t('selections-title')}</h2>
      <div class="selected-frame">
        <p class="spell-levels" class:over={used > budget} data-testid="spell-levels-used">
          {store.t('spell-levels-used', { used: String(used), budget: String(budget) })}
        </p>

        {#if masteryXp > 0 || masteryFloor > 0}
          <p
            class="spell-mastery"
            class:over={masteryUsed > masteryXp}
            data-testid="spell-mastery-info"
          >
            {#if masteryXp > 0}{store.t('spell-mastery-pool', {
                used: String(masteryUsed),
                pool: String(masteryXp),
              })}{/if}{#if masteryXp > 0 && masteryFloor > 0}
              ·
            {/if}{#if masteryFloor > 0}{store.t('spell-mastery-floor', {
                score: String(masteryFloor),
              })}{/if}
          </p>
        {/if}

        <ul class="spell-list" data-testid="spell-list">
          {#each store.entity.spells ?? [] as chosen, i (`${chosen.spell}:${chosen.level ?? ''}:${i}`)}
            <li>
              <span class="item-name">{rowLabel(chosen.spell, chosen.level)}</span>
              {#if masteryXp > 0 || masteryFloor > 0}
                <span class="spinner" data-testid="spell-mastery-{chosen.spell}-{i}">
                  <span class="spinner-label">{store.t('spell-mastery-label')}</span>
                  <button
                    type="button"
                    class="icon-btn"
                    aria-label={store.t('spell-mastery-decrement')}
                    disabled={(chosen.mastery ?? 0) <= 0}
                    onclick={() => store.adjustSpellMasteryAt(i, -1, masteryMax)}
                    data-testid="spell-mastery-dec-{chosen.spell}-{i}"
                  >
                    -
                  </button>
                  <span class="spinner-value" data-testid="spell-mastery-score-{chosen.spell}-{i}">
                    {chosen.mastery ?? 0}
                  </span>
                  <button
                    type="button"
                    class="icon-btn"
                    aria-label={store.t('spell-mastery-increment')}
                    disabled={(chosen.mastery ?? 0) >= masteryMax}
                    onclick={() => store.adjustSpellMasteryAt(i, 1, masteryMax)}
                    data-testid="spell-mastery-inc-{chosen.spell}-{i}"
                  >
                    +
                  </button>
                  {#if effectiveSpellMastery(chosen.mastery, masteryFloor) !== (chosen.mastery ?? 0)}
                    <span class="eff-badge" data-testid="spell-mastery-eff-{chosen.spell}-{i}">
                      {store.t('effective-score', {
                        score: String(effectiveSpellMastery(chosen.mastery, masteryFloor)),
                      })}
                    </span>
                  {/if}
                </span>
              {/if}
              <button
                type="button"
                class="icon-btn"
                aria-label={store.t('spell-remove')}
                onclick={() => store.removeSpellAt(i)}
                data-testid="spell-remove-{chosen.spell}-{i}"
              >
                -
              </button>
            </li>
          {/each}
        </ul>
      </div>
    </section>
  </div>
{:else}
  <p>{store.t('loading')}</p>
{/if}
