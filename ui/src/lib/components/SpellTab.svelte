<script lang="ts">
  import { store } from '../state.svelte';
  import {
    artAbbreviation,
    artLabel,
    effectiveSpellMastery,
    filterSpells,
    groupArtsByType,
    groupSelectedSpellsByTechniqueForm,
    groupSpellsByTechniqueForm,
    invalidSelectionIds,
    maxAbilityScore,
    usedSpellForms,
    spellDisplayName,
  } from '../derive';
  import type { SelectedSpellGroup } from '../derive';
  import { tooltip, withReason, type TooltipContent } from '../actions';
  import type { Art, Spell, SpellMasteryAbility, SpellSelection } from '../types';
  import SourcePicker from './SourcePicker.svelte';
  import SelectionList from './SelectionList.svelte';

  // Technique/Form/text/level-range filters live on the store, so they survive the
  // tab switch that unmounts this component (same split ArtGrid uses for the Arts).
  const filter = $derived(store.filters.spells);

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
  const sourceGroups = $derived.by(() => {
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
    return groupSpellsByTechniqueForm(rs, filtered).map((group) => ({
      key: `${group.technique} ${group.form}`,
      header: groupHeader(group.technique, group.form),
      headerTestid: `spell-group-${group.technique}-${group.form}`,
      items: group.spells,
    }));
  });

  // The selected spells grouped by Technique+Form — the same grouping and order
  // as the available list, so the selected side carries category headers like the
  // Abilities and V/F tabs do. Each entry carries its ORIGINAL index into
  // `entity.spells` so the index-addressed row mutations still target the right
  // row — the display order is never written back to the entity.
  const selectedGroups = $derived.by((): SelectedSpellGroup[] => {
    const rs = store.ruleset;
    if (!rs) return [];
    return groupSelectedSpellsByTechniqueForm(rs, store.entity.spells ?? []);
  });

  const selectedColumns = $derived([
    {
      key: 'spells',
      empty: (store.entity.spells ?? []).length === 0,
      emptyText: store.t('empty-selections-side'),
      emptyClass: 'empty',
      groups: selectedGroups.map((g, gi) => ({
        key: `${g.technique} ${g.form}`,
        // A group whose Te/Fo is unknown (spell absent from the catalogue) has no
        // localizable header, so it renders header-less rather than showing a slug.
        header: g.technique && g.form ? groupHeader(g.technique, g.form) : undefined,
        listClass: 'spell-list',
        // The e2e suite keys off `spell-list`; keep it on the first group so the
        // testid stays unique now that the list is split per Technique/Form.
        ulTestid: gi === 0 ? 'spell-list' : undefined,
        rows: g.entries.map((s) => ({
          key: `${s.selection.spell}:${s.selection.parameter ?? ''}:${s.index}`,
          item: s,
        })),
      })),
    },
  ]);

  // Spells an error-severity issue points at (e.g. a level now above its Te/Fo cap
  // after the Arts were lowered, or an illegal Ritual) — their rows render red.
  const invalidIds = $derived(invalidSelectionIds(store.result));

  // The selected spell ids, for the "already selected" grey-out of ordinary
  // fixed-level spells (see `nonTakeableReason`). A General spell (learnable at
  // several levels) or a parameterized spell (takeable once per Form) is excluded
  // there and stays re-takeable — matching how Abilities/Virtues grey out.
  const selectedSpellIds = $derived(new Set((store.entity.spells ?? []).map((s) => s.spell)));

  // Spell levels still available to spend — the per-spell budget check behind the
  // greying of source rows. The budget figures are engine-authoritative; the bar
  // that displays them lives in `SpellBudgetBar`.
  const budget = $derived(store.effective?.spell_levels_budget ?? 0);
  const used = $derived(store.effective?.spell_levels_used ?? 0);
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
  // Spell-Mastery: the auto-mastery floor (Flawless Magic) drives each row's
  // effective mastery; the pool read-out itself lives in `SpellBudgetBar`.
  const masteryFloor = $derived(store.effective?.spell_mastery_floor ?? 0);
  // The Mastery Ability rises like an Ability, so its max comes from the Ability
  // advancement table (data, not a hardcoded mechanic — same path as abilities).
  const advancement = $derived(store.ruleset?.ruleset.advancement ?? []);
  const masteryMax = $derived(maxAbilityScore(advancement));

  function abbr(artId: string): string {
    return store.ruleset ? artAbbreviation(store.ruleset, artId) : '';
  }

  // The localized group header: the two Art names composed via Fluent (never a
  // raw id) — e.g. "Creo Ignem". Shared by the available and selected lists.
  function groupHeader(technique: string, form: string): string {
    const rs = store.ruleset;
    if (!rs) return '';
    return store.t('spell-group-header', {
      technique: artLabel(rs, technique),
      form: artLabel(rs, form),
    });
  }

  // A spell's level tag for a source row: its fixed level, or the localized
  // "General" marker (the level is chosen per character). The Technique/Form is
  // already carried by the group header, so a source row shows only the level.
  function levelTag(spell: Spell): string {
    return spell.level == null ? store.t('spell-level-general') : String(spell.level);
  }

  // The localized param label ("Form"), used as the unchosen-parameter hint. The
  // spell-name template supplies the literal parens ("… ({form})"), so the hint
  // is the plain label — a source candidate reads "Wizard's Boost (Form)".
  function paramLabel(key: string): string {
    return store.t(`param-label-${key}`);
  }

  // Whether a spell takes a selection parameter (a meta-magic Vim spell whose
  // target Form is chosen per instance).
  function isParametrized(spellId: string): boolean {
    return (store.ruleset?.ruleset.spells?.[spellId]?.parameters?.length ?? 0) > 0;
  }

  function optionLabel(spell: Spell): string {
    const rs = store.ruleset;
    if (!rs) return spell.id;
    return `${spellDisplayName(rs, spell.id, undefined, paramLabel)} (${levelTag(spell)})`;
  }

  // A chosen row's display: name (with the chosen target Form interpolated for a
  // parametrized spell) + its TeFo tag (no grouping in the selected list, so the
  // Technique/Form stays useful here). A fixed spell shows its catalogue level; a
  // General spell shows the localized "General" marker.
  function rowLabel(chosen: SpellSelection): string {
    const rs = store.ruleset;
    const cat = rs?.ruleset.spells?.[chosen.spell];
    if (!rs || !cat) return chosen.spell;
    const tf = `${abbr(cat.technique)}${abbr(cat.form)}`;
    const lvl = cat.level == null ? store.t('spell-level-general') : String(cat.level);
    return `${spellDisplayName(rs, chosen.spell, chosen.parameter, paramLabel)} (${tf} ${lvl})`;
  }

  // The minimum level a spell can be learned at: a Ritual must be learned at
  // the engine's `ritual_min_level` (Ars Magica - Definitive Edition (Core
  // Rules).md:12293, "Ritual spells are always at least level 20"), an
  // ordinary spell at 1 — the latter is not a book-stated floor, just the
  // lowest level a spell can exist at, so it stays a local constant.
  //
  // VA2 (tmp/review/review-round-1-viktor-app.md), CLOSED: the Ritual floor
  // used to be a bare literal duplicating the engine's own check
  // (`crates/arm-rules/src/ruleset.rs`'s `validate_spell`, and
  // `crates/arm-rules/src/validation/magus.rs`). It now reads
  // `ruleset.ritual_min_level`, derived from `spell::RITUAL_MIN_LEVEL` — see
  // `crates/arm-rules/src/spell.rs`. The fallback below only covers the moment
  // before a ruleset has loaded, when no spell exists to disable anyway.
  const ORDINARY_MINIMUM_LEVEL = 1;
  const RITUAL_MINIMUM_LEVEL_FALLBACK = 20;
  function minLearnableLevel(spell: Spell): number {
    if (!spell.ritual) return ORDINARY_MINIMUM_LEVEL;
    return store.ruleset?.ruleset.ritual_min_level ?? RITUAL_MINIMUM_LEVEL_FALLBACK;
  }

  // The same floor, looked up from a chosen (selected-list) row's id rather
  // than a source-list Spell object — used by the inline level spinner so it
  // can never be scrubbed down into a Ritual's illegal range (S3). Falls back
  // to the ordinary floor for an id absent from the catalogue (defensive; a
  // selection always names a real spell in practice).
  function minLevelForChosen(spellId: string): number {
    const cat = store.ruleset?.ruleset.spells?.[spellId];
    return cat ? minLearnableLevel(cat) : ORDINARY_MINIMUM_LEVEL;
  }

  // Why a source spell's add control is greyed, or null when it is takeable. A
  // fixed-level spell is tested at its catalogue level; a General spell (no fixed
  // level) is tested at its minimum learnable level — never at a nonexistent
  // catalogue level. Blocked when that level exceeds the per-spell cap or the
  // remaining spell-levels budget. The cap is the engine's surfaced value.
  function nonTakeableReason(spell: Spell): { key: string; cap: number } | null {
    // An ordinary fixed-level spell is taken only once, so grey it once selected.
    // General spells (multiple learnable levels) and parameterized spells (once
    // per Form) stay re-takeable and are excluded from this test.
    if (spell.level != null && !isParametrized(spell.id) && selectedSpellIds.has(spell.id))
      return { key: 'spell-already-taken-reason', cap: 0 };
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
  // added at its minimum learnable level (edited inline afterwards): the
  // ordinary floor (1) for a plain spell, but the engine's ritual_min_level
  // for a General Ritual (S3, tmp/review/review-round-2-sabine.md) — a flat
  // literal used to be applied to every General spell regardless, which put a
  // fresh Ritual pick straight into a blocking CODE_SPELL_RITUAL_LEGALITY
  // error through ordinary use. A fixed-level spell ignores the level.
  function add(spell: Spell) {
    store.addSpell(spell.id, spell.level == null ? minLearnableLevel(spell) : undefined);
  }

  // A chosen row is General (level editable inline) when its catalogue entry has
  // no fixed level.
  function isGeneral(spellId: string): boolean {
    return store.ruleset?.ruleset.spells?.[spellId]?.level == null;
  }

  // A source row's tooltip: always the spell's rules-text description, prefixed
  // by the reason it is non-takeable (greyed) when blocked, so a blocked spell
  // shows WHY plus its description rather than the reason replacing it. Spells
  // carry no specialties, so the tooltip is otherwise text-only.
  function sourceTip(spell: Spell): TooltipContent {
    const reason = nonTakeableReason(spell);
    return withReason(
      { text: store.ruleset?.i18n[spell.id]?.description ?? undefined },
      reason ? store.t(reason.key, { cap: String(reason.cap) }) : undefined,
    );
  }

  // A chosen (selected-list) row's tooltip: the description only.
  function tip(spellId: string): TooltipContent {
    return { text: store.ruleset?.i18n[spellId]?.description ?? undefined };
  }

  // The Spell Mastery special-ability catalogue (id order), for the per-spell
  // "add ability" picker. Empty when the ruleset ships no mastery catalogue.
  const masteryAbilityCatalogue = $derived.by((): SpellMasteryAbility[] => {
    const rs = store.ruleset;
    if (!rs) return [];
    return Object.values(rs.ruleset.spell_mastery_abilities ?? {});
  });

  // A mastery ability's rules-text name — always via the i18n map, never the raw
  // id (falls back to the id only when the ruleset is not yet loaded, mirroring
  // rowLabel's defensive fallback).
  function masteryAbilityName(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }

  // A mastery ability's tooltip: its rules-text description.
  function masteryAbilityTip(id: string): TooltipContent {
    return { text: store.ruleset?.i18n[id]?.description ?? undefined };
  }
</script>

{#if store.ruleset}
  <!-- The spell-levels budget + Mastery read-out (`SpellBudgetBar`) is NOT mounted
       here: like `BalanceBar` and `XpBar` it is a whole-character budget, so it
       belongs to whoever mounts this surface — `App.svelte`'s Spells tab and
       `WizardStep`'s phase table — as a sibling status line above both lists. -->
  <div class="region-row">
    <section class="region region-source">
      <h2 class="region-title" data-testid="available-title">{store.t('available-title')}</h2>
      <SourcePicker
        groups={sourceGroups}
        getId={(spell: Spell) => spell.id}
        onAdd={(spell: Spell) => add(spell)}
        disabled={(spell: Spell) => isDisabled(spell)}
        tip={(spell: Spell) => sourceTip(spell)}
      >
        {#snippet filters()}
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
              <option value={t.id}>{artLabel(store.ruleset!, t.id)}</option>
            {/each}
          </select>
          <select
            bind:value={filter.form}
            aria-label={store.t('spell-form-label')}
            data-testid="spell-form-filter"
          >
            <option value="">{store.t('spell-form-label')}</option>
            {#each forms as f (f.id)}
              <option value={f.id}>{artLabel(store.ruleset!, f.id)}</option>
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
        {/snippet}
        {#snippet row(spell: Spell)}
          <span class="item-name">{optionLabel(spell)}</span>
        {/snippet}
      </SourcePicker>
    </section>

    <section class="region region-selected">
      <h2 class="region-title">{store.t('selections-title')}</h2>
      <div class="selected-frame">
        <!-- The frame carries the border and its padding; this inner box does the
             scrolling, so the padding stays a gap the rows cannot scroll into. -->
        <div class="selected-scroll">
          <SelectionList columns={selectedColumns}>
            {#snippet row(item: { selection: SpellSelection; index: number })}
              {@const chosen = item.selection}
              {@const i = item.index}
              <li
                class:invalid-selection={invalidIds.has(chosen.spell)}
                use:tooltip={tip(chosen.spell)}
              >
                <span class="item-name" data-testid="spell-name-{chosen.spell}-{i}"
                  >{rowLabel(chosen)}</span
                >
                {#if isParametrized(chosen.spell)}
                  <!-- The target Form of a meta-magic Vim spell — display + identity
                     only, so the same spell can be taken once per distinct Form.
                     Mirrors the ParameterPicker Art <select> (groupArtsByType).
                     Forms already used by this spell at this level are greyed so
                     each (spell, level, Form) is takeable only once. -->
                  {@const usedForms = usedSpellForms(
                    store.entity.spells ?? [],
                    chosen.spell,
                    chosen.level,
                    i,
                  )}
                  <select
                    aria-label={store.t('param-label-form')}
                    value={chosen.parameter ?? ''}
                    onchange={(e) =>
                      store.setSpellParameterAt(
                        i,
                        (e.currentTarget as HTMLSelectElement).value || null,
                      )}
                    data-testid="spell-param-{chosen.spell}-{i}"
                  >
                    <option value="" disabled>{store.t('param-label-form')}</option>
                    {#each forms as f (f.id)}
                      <option value={f.id} disabled={usedForms.has(f.id)}
                        >{artLabel(store.ruleset!, f.id)}</option
                      >
                    {/each}
                  </select>
                {/if}
                {#if isGeneral(chosen.spell)}
                  <input
                    type="number"
                    min={minLevelForChosen(chosen.spell)}
                    max="255"
                    step="1"
                    aria-label={store.t('spell-general-level-label')}
                    value={chosen.level ?? minLevelForChosen(chosen.spell)}
                    oninput={(e) =>
                      store.setSpellLevelAt(i, Number((e.currentTarget as HTMLInputElement).value))}
                    data-testid="spell-level-input-{chosen.spell}-{i}"
                  />
                {/if}
                <!-- Spell Mastery is an Ability every magus may buy from the general
                   apprenticeship pool (Core:9518), so the spinner shows whenever the
                   advancement table can price it — not only with Mastered Spells /
                   Flawless Magic. This picker is already magus-and-Spells-tab-only.
                   The score spinner and the special-ability picker share ONE stacked
                   column: side by side the two would squeeze the (typically long)
                   spell name down to a word per line. -->
                {#if masteryMax > 0}
                  <span
                    class="spell-mastery-block"
                    data-testid="spell-mastery-block-{chosen.spell}-{i}"
                  >
                    <span class="spinner" data-testid="spell-mastery-{chosen.spell}-{i}">
                      <span class="spinner-label">{store.t('spell-mastery-label')}</span>
                      <button
                        type="button"
                        class="icon-btn"
                        aria-label={store.t('spell-mastery-decrement', { name: rowLabel(chosen) })}
                        disabled={(chosen.mastery ?? 0) <= 0}
                        onclick={() => store.adjustSpellMasteryAt(i, -1, masteryMax)}
                        data-testid="spell-mastery-dec-{chosen.spell}-{i}"
                      >
                        -
                      </button>
                      <span
                        class="spinner-value"
                        data-testid="spell-mastery-score-{chosen.spell}-{i}"
                      >
                        {chosen.mastery ?? 0}
                      </span>
                      <button
                        type="button"
                        class="icon-btn"
                        aria-label={store.t('spell-mastery-increment', { name: rowLabel(chosen) })}
                        disabled={(chosen.mastery ?? 0) >= masteryMax}
                        onclick={() => store.adjustSpellMasteryAt(i, 1, masteryMax)}
                        data-testid="spell-mastery-inc-{chosen.spell}-{i}"
                      >
                        +
                      </button>
                      {#if effectiveSpellMastery(chosen.mastery, masteryFloor) !== (chosen.mastery ?? 0)}
                        <span class="eff-slot">
                          <span
                            class="eff-badge"
                            data-testid="spell-mastery-eff-{chosen.spell}-{i}"
                          >
                            {store.t('effective-score', {
                              score: String(effectiveSpellMastery(chosen.mastery, masteryFloor)),
                            })}
                          </span>
                        </span>
                      {/if}
                    </span>
                    <!-- Spell Mastery special abilities: one may be chosen per effective
                       mastery level (Core:9524-9526). The add-picker hides once the
                       count reaches the effective mastery; a non-repeatable ability
                       already chosen is disabled in the list, a repeatable one
                       (Precise/Quick/Quiet Casting) stays selectable again. -->
                    {#if effectiveSpellMastery(chosen.mastery, masteryFloor) > 0 && masteryAbilityCatalogue.length > 0}
                      {@const effMastery = effectiveSpellMastery(chosen.mastery, masteryFloor)}
                      {@const chosenAbilities = chosen.mastery_abilities ?? []}
                      <span
                        class="mastery-abilities"
                        data-testid="spell-mastery-abilities-{chosen.spell}-{i}"
                      >
                        <span class="spinner-label">{store.t('spell-mastery-abilities-label')}</span
                        >
                        {#each chosenAbilities as abilityId, ai (`${abilityId}:${ai}`)}
                          <span
                            class="ability-chip"
                            use:tooltip={masteryAbilityTip(abilityId)}
                            data-testid="spell-mastery-ability-{chosen.spell}-{i}-{ai}"
                          >
                            <span class="chip-name">{masteryAbilityName(abilityId)}</span>
                            <button
                              type="button"
                              class="icon-btn"
                              aria-label={store.t('remove-item', {
                                name: masteryAbilityName(abilityId),
                              })}
                              onclick={() => store.removeMasteryAbilityAt(i, ai)}
                              data-testid="spell-mastery-ability-remove-{chosen.spell}-{i}-{ai}"
                            >
                              ×
                            </button>
                          </span>
                        {/each}
                        {#if chosenAbilities.length < effMastery}
                          <select
                            aria-label={store.t('spell-mastery-ability-add')}
                            value=""
                            onchange={(e) => {
                              const sel = e.currentTarget as HTMLSelectElement;
                              if (sel.value) {
                                store.addMasteryAbilityAt(i, sel.value);
                                sel.value = '';
                              }
                            }}
                            data-testid="spell-mastery-ability-add-{chosen.spell}-{i}"
                          >
                            <option value="" disabled>{store.t('spell-mastery-ability-add')}</option
                            >
                            {#each masteryAbilityCatalogue as ma (ma.id)}
                              <option
                                value={ma.id}
                                disabled={!ma.repeatable && chosenAbilities.includes(ma.id)}
                                >{masteryAbilityName(ma.id)}</option
                              >
                            {/each}
                          </select>
                        {/if}
                      </span>
                    {/if}
                  </span>
                {/if}
                <button
                  type="button"
                  class="icon-btn"
                  aria-label={store.t('remove-item', { name: rowLabel(chosen) })}
                  onclick={() => store.removeSpellAt(i)}
                  data-testid="spell-remove-{chosen.spell}-{i}"
                >
                  ×
                </button>
              </li>
            {/snippet}
          </SelectionList>
        </div>
      </div>
    </section>
  </div>
{:else}
  <p>{store.t('loading')}</p>
{/if}
