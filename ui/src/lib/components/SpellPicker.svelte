<script lang="ts">
  import { store } from '../state.svelte';
  import {
    artAbbreviation,
    artLabel,
    effectiveSpellMastery,
    filterSpells,
    groupArtsByType,
    groupSpellsByTechniqueForm,
    maxAbilityScore,
    spellMasteryXpSpent,
    spellName,
  } from '../derive';
  import type { SpellGroup } from '../derive';
  import { tooltip, type TooltipContent } from '../actions';
  import type { Art, Spell } from '../types';

  // Technique/Form/text/level-range filters live on the store, so they survive the
  // tab switch that unmounts this component (same split ArtGrid uses for the Arts).
  const filter = $derived(store.filters.spells);

  // A General spell (no fixed catalogue level) is added at this default level;
  // the level is then edited inline on its row in the selected list.
  const GENERAL_DEFAULT_LEVEL = 5;

  const techniques = $derived.by((): Art[] => {
    if (!store.ruleset) return [];
    return groupArtsByType(store.ruleset).find((g) => g.artType === 'technique')?.arts ?? [];
  });
  const forms = $derived.by((): Art[] => {
    if (!store.ruleset) return [];
    return groupArtsByType(store.ruleset).find((g) => g.artType === 'form')?.arts ?? [];
  });

  // Catalogue spells matching the Technique/Form (separate and combined), text
  // search, and inclusive level range, grouped by Technique+Form and sorted by
  // level-then-name within each group (General spells trailing).
  const groups = $derived.by((): SpellGroup[] => {
    const rs = store.ruleset;
    if (!rs) return [];
    const filtered = filterSpells(
      rs,
      Object.values(rs.ruleset.spells ?? {}),
      {
        text: filter.search,
        technique: filter.technique || undefined,
        form: filter.form || undefined,
        levelMin: filter.levelMin,
        levelMax: filter.levelMax,
      },
      store.t,
    );
    return groupSpellsByTechniqueForm(rs, filtered);
  });

  // The spell-levels budget bar: engine-authoritative used/budget, with a local
  // fallback for the first frame before effective scores arrive.
  const budget = $derived(store.effective?.spell_levels_budget ?? 0);
  const used = $derived(store.effective?.spell_levels_used ?? 0);
  // Spell levels still available to spend (used against the per-spell budget check).
  const remaining = $derived(budget - used);

  // The engine-authoritative per-Technique/Form spell-level cap (Te + Fo + Int +
  // Magic Theory + 3), keyed by the "<technique> <form>" pair. Never recomputed
  // here — the picker only reads the surfaced value.
  const capByTeFo = $derived.by((): Map<string, number> => {
    const m = new Map<string, number>();
    for (const c of store.effective?.spell_level_caps ?? [])
      m.set(`${c.technique} ${c.form}`, c.cap);
    return m;
  });
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

  // The localized group header: the two Art names composed via Fluent (never a
  // raw id) — e.g. "Creo Ignem".
  function groupHeader(group: SpellGroup): string {
    const rs = store.ruleset;
    if (!rs) return '';
    return store.t('spell-group-header', {
      technique: artLabel(rs, group.technique),
      form: artLabel(rs, group.form),
    });
  }

  // A spell's level tag for a source row: its fixed level, or the localized
  // "General" marker (the level is chosen per character). The Technique/Form is
  // already carried by the group header, so a source row shows only the level.
  function levelTag(spell: Spell): string {
    return spell.level == null ? store.t('spell-level-general') : String(spell.level);
  }

  function optionLabel(spell: Spell): string {
    const rs = store.ruleset;
    return rs ? `${spellName(rs, spell.id)} (${levelTag(spell)})` : spell.id;
  }

  // A chosen row's display: name + its TeFo tag (no grouping in the selected
  // list, so the Technique/Form stays useful here). A fixed spell shows its
  // catalogue level; a General spell shows the localized "General" marker.
  function rowLabel(spellId: string): string {
    const rs = store.ruleset;
    const cat = rs?.ruleset.spells?.[spellId];
    if (!rs || !cat) return spellId;
    const tf = `${abbr(cat.technique)}${abbr(cat.form)}`;
    const lvl = cat.level == null ? store.t('spell-level-general') : String(cat.level);
    return `${spellName(rs, spellId)} (${tf} ${lvl})`;
  }

  // The minimum level a spell can be learned at: a Ritual must be learned at 20,
  // an ordinary spell at 1 (Core Rules.md:12279-12295). Used to decide whether a
  // General spell (no fixed catalogue level) is takeable at all.
  function minLearnableLevel(spell: Spell): number {
    return spell.ritual ? 20 : 1;
  }

  // Why a source spell's add control is greyed, or null when it is takeable. A
  // fixed-level spell is tested at its catalogue level; a General spell (no fixed
  // level) is tested at its minimum learnable level — never at a nonexistent
  // catalogue level. Blocked when that level exceeds the per-spell cap or the
  // remaining spell-levels budget. The cap is the engine's surfaced value.
  function nonTakeableReason(spell: Spell): { key: string; cap: number } | null {
    const cap = capByTeFo.get(`${spell.technique} ${spell.form}`);
    const need = spell.level ?? minLearnableLevel(spell);
    if (cap != null && need > cap) return { key: 'spell-cap-reason', cap };
    if (need > remaining) return { key: 'spell-budget-reason', cap: cap ?? 0 };
    return null;
  }

  function isDisabled(spell: Spell): boolean {
    return nonTakeableReason(spell) != null;
  }

  // Clicking a source row adds the spell. A General spell (no fixed level) is
  // added at the default level and edited inline afterwards; a fixed-level spell
  // ignores the level.
  function add(spell: Spell) {
    store.addSpell(spell.id, spell.level == null ? GENERAL_DEFAULT_LEVEL : undefined);
  }

  // A chosen row is General (level editable inline) when its catalogue entry has
  // no fixed level.
  function isGeneral(spellId: string): boolean {
    return store.ruleset?.ruleset.spells?.[spellId]?.level == null;
  }

  // A source row's tooltip: the reason it is non-takeable (greyed) when blocked,
  // otherwise the spell's rules-text description. Spells carry no specialties, so
  // the tooltip is text-only.
  function sourceTip(spell: Spell): TooltipContent {
    const reason = nonTakeableReason(spell);
    if (reason) return { text: store.t(reason.key, { cap: String(reason.cap) }) };
    return { text: store.ruleset?.i18n[spell.id]?.description ?? undefined };
  }

  // A chosen (selected-list) row's tooltip: the description only.
  function tip(spellId: string): TooltipContent {
    return { text: store.ruleset?.i18n[spellId]?.description ?? undefined };
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
            <span>{store.t('spell-level-min-label')}</span>
            <input
              type="number"
              min="1"
              step="1"
              aria-label={store.t('spell-level-min-label')}
              bind:value={filter.levelMin}
              data-testid="spell-level-min-filter"
            />
          </label>
          <label class="field">
            <span>{store.t('spell-level-max-label')}</span>
            <input
              type="number"
              min="1"
              step="1"
              aria-label={store.t('spell-level-max-label')}
              bind:value={filter.levelMax}
              data-testid="spell-level-max-filter"
            />
          </label>
        </div>
        <div class="list-scroll">
          {#each groups as group (`${group.technique} ${group.form}`)}
            <h3 class="category" data-testid="spell-group-{group.technique}-{group.form}">
              {groupHeader(group)}
            </h3>
            <ul class="item-list">
              {#each group.spells as spell (spell.id)}
                <li>
                  <button
                    type="button"
                    class="pick-row"
                    disabled={isDisabled(spell)}
                    onclick={() => add(spell)}
                    use:tooltip={sourceTip(spell)}
                    data-testid="add-{spell.id}"
                  >
                    <span class="item-name">{optionLabel(spell)}</span>
                    <span class="pick-plus" aria-hidden="true">+</span>
                  </button>
                </li>
              {/each}
            </ul>
          {/each}
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
          {#each store.entity.spells ?? [] as chosen, i (`${chosen.spell}:${i}`)}
            <li use:tooltip={tip(chosen.spell)}>
              <span class="item-name">{rowLabel(chosen.spell)}</span>
              {#if isGeneral(chosen.spell)}
                <input
                  type="number"
                  min="1"
                  step="1"
                  aria-label={store.t('spell-general-level-label')}
                  value={chosen.level ?? GENERAL_DEFAULT_LEVEL}
                  oninput={(e) =>
                    store.setSpellLevelAt(i, Number((e.currentTarget as HTMLInputElement).value))}
                  data-testid="spell-level-input"
                />
              {/if}
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
