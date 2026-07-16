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

  // Technique/Form filters, built from the Art registry (same split ArtGrid uses).
  let technique = $state('');
  let form = $state('');
  let search = $state('');
  let levelFilter = $state('');
  let picked = $state('');
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
        text: search,
        technique: technique || undefined,
        form: form || undefined,
        level: levelFilter === '' ? undefined : Number(levelFilter),
      },
      store.t,
    ).sort((a, b) => spellName(rs, a.id).localeCompare(spellName(rs, b.id)));
  });

  const pickedSpell = $derived(candidates.find((s) => s.id === picked) ?? null);
  // A General spell (no fixed level) needs a chosen level before it can be added.
  const isGeneral = $derived(pickedSpell != null && pickedSpell.level == null);

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

  function add() {
    if (!pickedSpell) return;
    store.addSpell(pickedSpell.id, isGeneral ? generalLevel : undefined);
    picked = '';
  }
</script>

<section class="panel spell-picker">
  {#if store.ruleset}
    <div class="spell-controls">
      <label class="field">
        <span>{store.t('filter-search-placeholder')}</span>
        <input type="search" bind:value={search} data-testid="spell-search" />
      </label>
      <label class="field">
        <span>{store.t('spell-level-label')}</span>
        <input
          type="number"
          min="1"
          step="1"
          bind:value={levelFilter}
          data-testid="spell-level-filter"
        />
      </label>
      <label class="field">
        <span>{store.t('spell-technique-label')}</span>
        <select bind:value={technique} data-testid="spell-technique-filter">
          <option value="">—</option>
          {#each techniques as t (t.id)}
            <option value={t.id}>{artLabel(store.ruleset, t.id)}</option>
          {/each}
        </select>
      </label>
      <label class="field">
        <span>{store.t('spell-form-label')}</span>
        <select bind:value={form} data-testid="spell-form-filter">
          <option value="">—</option>
          {#each forms as f (f.id)}
            <option value={f.id}>{artLabel(store.ruleset, f.id)}</option>
          {/each}
        </select>
      </label>
      <label class="field spell-select-field">
        <span>{store.t('spell-add')}</span>
        <select bind:value={picked} data-testid="spell-select">
          <option value="">{store.t('spell-none')}</option>
          {#each candidates as spell (spell.id)}
            <option value={spell.id}>{optionLabel(spell)}</option>
          {/each}
        </select>
      </label>
      {#if isGeneral}
        <label class="field">
          <span>{store.t('spell-level-label')}</span>
          <input
            type="number"
            min="1"
            step="1"
            bind:value={generalLevel}
            data-testid="spell-level-input"
          />
        </label>
      {/if}
      <button type="button" disabled={!pickedSpell} onclick={add} data-testid="spell-add">
        {store.t('spell-add')}
      </button>
    </div>

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
                −
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
            −
          </button>
        </li>
      {/each}
    </ul>
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
