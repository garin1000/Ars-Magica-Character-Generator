<script lang="ts">
  import { tick } from 'svelte';
  import { store } from '../state.svelte';
  import {
    artLabel,
    artsOfType,
    effectiveSpellMastery,
    filterSpells,
    groupSelectedSpellsByTechniqueForm,
    groupSpellsByTechniqueForm,
    invalidSelectionIds,
    isDisabled,
    maxAbilityScore,
    minLearnableLevel,
    nonTakeableReason,
    ORDINARY_SPELL_MINIMUM_LEVEL,
    RITUAL_MINIMUM_LEVEL_FALLBACK,
    usedSpellForms,
    spellCodeWithLevel,
    spellDisplayName,
    spellGroupCapTooltip,
    withinFocusAddable,
    withinFocusAndPotentFieldAddable,
    withinPotentFieldAddable,
  } from '../derive';
  import type { SelectedSpellGroup } from '../derive';
  import { commitStored, tooltip, withReason, type TooltipContent } from '../actions';
  import type {
    Art,
    Spell,
    SpellCap,
    SpellLevelCap,
    SpellMasteryAbility,
    SpellSelection,
  } from '../types';
  import SourcePicker from './SourcePicker.svelte';
  import SelectionList from './SelectionList.svelte';
  import Spinner from './Spinner.svelte';
  import SpellMasteryAbilityPicker from './SpellMasteryAbilityPicker.svelte';

  // Technique/Form/text/level-range filters live on the store, so they survive the
  // tab switch that unmounts this component (same split ArtGrid uses for the Arts).
  const filter = $derived(store.filters.spells);

  // Shared with ParameterPicker's `technique` / `form` domains, so the Forms-only
  // menu this tab already got right is the same one the V/F pickers now use.
  const techniques = $derived.by((): Art[] =>
    store.ruleset ? artsOfType(store.ruleset, 'technique') : [],
  );
  const forms = $derived.by((): Art[] => (store.ruleset ? artsOfType(store.ruleset, 'form') : []));

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
      headerTooltip: groupCapTooltip(group.technique, group.form),
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

  // D81.5: the engine-authoritative PER-SPELL level cap — each row already
  // folds that spell's own requisites and its Range (D28's beyond-Touch
  // halving), so this is what the picker greys a candidate spell BY. Never
  // recomputed here; `nonTakeableReason`/`withinFocusAddable` only read the
  // surfaced value by the candidate spell's own id.
  const capBySpell = $derived.by((): Map<string, SpellCap> => {
    const m = new Map<string, SpellCap>();
    for (const c of store.effective?.spell_caps ?? []) m.set(c.spell, c);
    return m;
  });
  // The engine-authoritative per-Technique/Form/range-class spell-level cap
  // (Te + Fo + Int + Magic Theory + 3, D1's flat lab term, halved beyond Touch
  // for Short-Ranged Magic per D28), keyed by the
  // "<technique> <form> <range_beyond_touch>" triple. Superseded as the
  // PICKER's own grey/offer logic by `capBySpell` above (D81.5: a per-spell
  // figure folding each spell's own requisites, which this Te/Fo-keyed grid
  // cannot); kept for `groupCapTooltip`'s quick at-a-glance figure on a
  // source group's header — a requisite-free BASELINE for the whole
  // Technique/Form pair, independent of any one candidate spell.
  // N1: the whole row is kept, so the tooltip can add the Magical Focus /
  // Potent Magic figures the engine surfaced beside the plain cap.
  const capByTeFo = $derived.by((): Map<string, SpellLevelCap> => {
    const m = new Map<string, SpellLevelCap>();
    for (const c of store.effective?.spell_level_caps ?? [])
      m.set(`${c.technique} ${c.form} ${c.range_beyond_touch}`, c);
    return m;
  });
  // The Ritual level floor — see `minLearnableLevel` in `derive.ts` (VA2): reads
  // the engine-surfaced `ruleset.ritual_min_level`, falling back to the fallback
  // constant only for the moment before a ruleset has loaded, when no spell
  // exists to disable anyway.
  const ritualMinLevel = $derived(
    store.ruleset?.ruleset.ritual_min_level ?? RITUAL_MINIMUM_LEVEL_FALLBACK,
  );
  // Spell-Mastery: the auto-mastery floor (Flawless Magic) drives each row's
  // effective mastery; the pool read-out itself lives in `SpellBudgetBar`.
  const masteryFloor = $derived(store.effective?.spell_mastery_floor ?? 0);
  // The Mastery Ability rises like an Ability, so its max comes from the Ability
  // advancement table (data, not a hardcoded mechanic — same path as abilities).
  const advancement = $derived(store.ruleset?.ruleset.advancement ?? []);
  const masteryMax = $derived(maxAbilityScore(advancement));

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

  // D81.5: the source group header's hover hint — the Te/Fo grid's own
  // requisite-free BASELINE cap (the near, not-beyond-Touch row), a quick
  // at-a-glance figure independent of any one candidate spell's requisites.
  // `undefined` (no `title` attribute) before the grid has any data for this
  // pair (e.g. a non-magus, where `spell_level_caps` is empty). N1: the
  // Magical Focus / Potent Magic figures follow while those Virtues are held.
  function groupCapTooltip(technique: string, form: string): string | undefined {
    return spellGroupCapTooltip(capByTeFo.get(`${technique} ${form} false`), store.t);
  }

  // A spell's level tag: its fixed level, or the localized "General" marker (the
  // level is chosen per character).
  function levelTag(spell: Spell): string {
    return spell.level == null ? store.t('spell-level-general') : String(spell.level);
  }

  // The engine's spell code (requisites included, "CrIm(Ig)") plus the level tag —
  // identical in the source and the selected list (I4).
  function codeTag(spell: Spell): string {
    return store.ruleset ? spellCodeWithLevel(store.ruleset, spell.id, levelTag(spell)) : '';
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
    return `${spellDisplayName(rs, spell.id, undefined, paramLabel)} (${codeTag(spell)})`;
  }

  // A chosen row's display: name (with the chosen target Form interpolated for a
  // parametrized spell) + the same code tag as the source row. A fixed spell shows
  // its catalogue level; a General spell shows the localized "General" marker.
  function rowLabel(chosen: SpellSelection): string {
    const rs = store.ruleset;
    const cat = rs?.ruleset.spells?.[chosen.spell];
    if (!rs || !cat) return chosen.spell;
    return `${spellDisplayName(rs, chosen.spell, chosen.parameter, paramLabel)} (${codeTag(cat)})`;
  }

  // `minLearnableLevel`, `nonTakeableReason`, `isDisabled`, and
  // `withinFocusAddable` are imported from `derive.ts` (V28, full-audit round;
  // D81.5): spell-eligibility business rules live there alongside every other
  // eligibility computation (`eligibleForConstraint`, `filterSpells`, …), not
  // inline in this component. This component only supplies the reactive state
  // (`selectedSpellIds`, `capBySpell`, `remaining`, `ritualMinLevel`) they need.

  // The Ritual floor, looked up from a chosen (selected-list) row's id rather
  // than a source-list Spell object — used by the inline level spinner so it
  // can never be scrubbed down into a Ritual's illegal range (S3). Falls back
  // to the ordinary floor for an id absent from the catalogue (defensive; a
  // selection always names a real spell in practice).
  function minLevelForChosen(spellId: string): number {
    const cat = store.ruleset?.ruleset.spells?.[spellId];
    return cat ? minLearnableLevel(cat, ritualMinLevel) : ORDINARY_SPELL_MINIMUM_LEVEL;
  }

  // Clicking a source row adds the spell. A General spell (no fixed level) is
  // added at its minimum learnable level (edited inline afterwards): the
  // ordinary floor (1) for a plain spell, but the engine's ritual_min_level
  // for a General Ritual (S3, tmp/review/review-round-2-sabine.md) — a flat
  // literal used to be applied to every General spell regardless, which put a
  // fresh Ritual pick straight into a blocking CODE_SPELL_RITUAL_LEGALITY
  // error through ordinary use. A fixed-level spell ignores the level.
  function add(spell: Spell) {
    store.addSpell(
      spell.id,
      spell.level == null ? minLearnableLevel(spell, ritualMinLevel) : undefined,
    );
  }

  // D81.5: the picker's "add within focus" action — offered only when the
  // spell's level exceeds its plain per-spell cap but fits the
  // Magical-Focus-doubled one ({@link withinFocusAddable}). Adds the spell
  // already marked `within_focus: true` in the SAME write (not a separate
  // toggle afterward), so the row is never transiently in an illegal
  // (over-cap, unmarked) state — the engine cannot match a spell to a
  // player's free-text focus on its own, so the player's click IS the claim.
  function addWithinFocus(spell: Spell) {
    store.addSpellWithinFocus(spell.id, generalAddLevel(spell));
  }

  // R3 (D83.3): the Potent Magic twins of `addWithinFocus` — the plain cap
  // never includes Potent Magic's bonus, so a spell that fits only with it is
  // added already marked within the Potent field (or with both markers, when
  // only the combined cap admits it), in the same write.
  function addWithinPotentField(spell: Spell) {
    store.addSpellWithinPotentField(spell.id, generalAddLevel(spell));
  }

  function addWithinFocusAndPotentField(spell: Spell) {
    store.addSpellWithinFocusAndPotentField(spell.id, generalAddLevel(spell));
  }

  // The level a marked add passes: a General spell's minimum learnable level,
  // nothing for a fixed-level spell (as in `add`).
  function generalAddLevel(spell: Spell): number | undefined {
    return spell.level == null ? minLearnableLevel(spell, ritualMinLevel) : undefined;
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
    const reason = nonTakeableReason(
      spell,
      selectedSpellIds,
      capBySpell,
      remaining,
      ritualMinLevel,
    );
    return withReason(
      { text: store.ruleset?.i18n[spell.id]?.description ?? undefined },
      reason ? store.t(reason.key, { cap: String(reason.cap) }) : undefined,
    );
  }

  // The "add within focus" action's own tooltip: WHY it is offered, naming
  // the focus-doubled cap (D81.5's "Its tooltip or reason text says why").
  function withinFocusTip(spell: Spell): TooltipContent {
    const cap = capBySpell.get(spell.id)?.within_focus_cap;
    return { text: store.t('spell-add-within-focus-tooltip', { cap: String(cap ?? 0) }) };
  }

  // R3 (D83.3): the Potent Magic actions' tooltips, each naming its own cap.
  function withinPotentFieldTip(spell: Spell): TooltipContent {
    const cap = capBySpell.get(spell.id)?.within_potent_field_cap;
    return { text: store.t('spell-add-within-potent-field-tooltip', { cap: String(cap ?? 0) }) };
  }

  function withinFocusAndPotentFieldTip(spell: Spell): TooltipContent {
    const cap = capBySpell.get(spell.id)?.within_focus_and_potent_field_cap;
    return {
      text: store.t('spell-add-within-focus-and-potent-field-tooltip', { cap: String(cap ?? 0) }),
    };
  }

  // A chosen (selected-list) row's tooltip: the description only.
  function tip(spellId: string): TooltipContent {
    return { text: store.ruleset?.i18n[spellId]?.description ?? undefined };
  }

  // X10c (design-x10bc-save-format.md § 3): whether this spell's own
  // (Technique, Form) cell carries a within-focus figure at all — the same
  // `within_focus != null` gate `DerivedLabCastingSection.svelte` already uses
  // for its own column. The toggle is offered only when it could matter.
  function hasFocusFigure(spell: Spell): boolean {
    return (store.derived?.casting_totals ?? []).some(
      (c) => c.technique === spell.technique && c.form === spell.form && c.within_focus != null,
    );
  }

  // D79: whether this spell's own (Technique, Form) cell carries a
  // within-Potent-Magic-field figure — independent of `hasFocusFigure` (a
  // character may hold a Magical Focus, Potent Magic, both, or neither).
  function hasPotentFieldFigure(spell: Spell): boolean {
    return (store.derived?.casting_totals ?? []).some(
      (c) =>
        c.technique === spell.technique && c.form === spell.form && c.within_potent_field != null,
    );
  }

  // W4 (finding 17): a within-focus / within-Potent-field checkbox also shows
  // while the spell is marked without the figure (a stale mark, D81.17), so
  // unticking it removes the checkbox. Keyboard focus must not fall to
  // <body>: it moves to the same row's spell name — non-destructive, unlike
  // the remove button, where a second Space press would delete the spell.
  async function setMarkKeepingFocus(
    input: HTMLInputElement,
    setMark: (checked: boolean) => void,
  ): Promise<void> {
    const hadFocus = document.activeElement === input;
    const row = input.closest('li');
    setMark(input.checked);
    await tick();
    if (!hadFocus || input.isConnected || !row) return;
    row.querySelector<HTMLElement>('.item-name')?.focus();
  }

  // The in-app Casting Total for a known spell at row `index` (X10c, D73.2;
  // D79) — a plain lookup into the engine's own per-row computation
  // (`DerivedTotals.spell_casting_totals`, index-aligned with `entity.spells`).
  // Not reconstructed client-side: the engine combines both the within-focus
  // and within-potent-field markers (and any Deficient-Art halving) directly,
  // which summing the grid's three independently-halved figures cannot do
  // correctly (`halve(a) + halve(b) != halve(a + b)`).
  function castingTotalOf(index: number): number | null {
    return store.derived?.spell_casting_totals?.[index] ?? null;
  }

  // D81.8: whether the known spell at row `index` touches one of a held
  // Incompatible Arts Flaw's two barred combinations — as its primary Arts or
  // only through a requisite ("even if one or both are requisites",
  // ArMDE:6292). A plain index lookup into the engine's own computation
  // (`DerivedTotals.spell_casting_unusable`), index-aligned with
  // `spell_casting_totals` and `entity.spells` for the same reason that field
  // is. The Casting Total figure stays visible beside this marker (unlike the
  // Lab/Casting grids, which withhold the number outright) — it still has
  // informational value here, and D81.15 separately raises a creation-time
  // ERROR for this spell on the Spells phase (surfaced through the existing
  // generic `invalidSelectionIds` row styling, not a second marker).
  function castingUnusable(index: number): boolean {
    return store.derived?.spell_casting_unusable?.[index] ?? false;
  }

  // The Spell Mastery special-ability catalogue (id order), for the per-spell
  // "add ability" picker. Empty when the ruleset ships no mastery catalogue.
  // Localized name/tooltip lookups for a catalogue entry live in
  // `SpellMasteryAbilityPicker.svelte` (G21, full-audit round) alongside the
  // rest of that picker, not here.
  const masteryAbilityCatalogue = $derived.by((): SpellMasteryAbility[] => {
    const rs = store.ruleset;
    if (!rs) return [];
    return Object.values(rs.ruleset.spell_mastery_abilities ?? {});
  });
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
        disabled={(spell: Spell) =>
          isDisabled(spell, selectedSpellIds, capBySpell, remaining, ritualMinLevel)}
        tip={(spell: Spell) => sourceTip(spell)}
      >
        {#snippet extra(spell: Spell)}
          {#if withinFocusAddable(spell, selectedSpellIds, capBySpell, remaining, ritualMinLevel)}
            <button
              type="button"
              class="within-focus-add"
              onclick={() => addWithinFocus(spell)}
              use:tooltip={withinFocusTip(spell)}
              aria-label={store.t('spell-add-within-focus-label', { name: optionLabel(spell) })}
              data-testid="add-within-focus-{spell.id}"
            >
              {store.t('spell-add-within-focus')}
            </button>
          {/if}
          {#if withinPotentFieldAddable(spell, selectedSpellIds, capBySpell, remaining, ritualMinLevel)}
            <button
              type="button"
              class="within-focus-add"
              onclick={() => addWithinPotentField(spell)}
              use:tooltip={withinPotentFieldTip(spell)}
              aria-label={store.t('spell-add-within-potent-field-label', {
                name: optionLabel(spell),
              })}
              data-testid="add-within-potent-field-{spell.id}"
            >
              {store.t('spell-add-within-potent-field')}
            </button>
          {/if}
          {#if withinFocusAndPotentFieldAddable(spell, selectedSpellIds, capBySpell, remaining, ritualMinLevel)}
            <button
              type="button"
              class="within-focus-add"
              onclick={() => addWithinFocusAndPotentField(spell)}
              use:tooltip={withinFocusAndPotentFieldTip(spell)}
              aria-label={store.t('spell-add-within-focus-and-potent-field-label', {
                name: optionLabel(spell),
              })}
              data-testid="add-within-focus-and-potent-field-{spell.id}"
            >
              {store.t('spell-add-within-focus-and-potent-field')}
            </button>
          {/if}
        {/snippet}
        {#snippet filters()}
          <input
            type="search"
            class="filter-search"
            placeholder={store.t('filter-search-placeholder')}
            aria-label={store.t('filter-search-placeholder')}
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
            <!-- commitStored-exempt: local filter state on `bind:value`; no store setter clamps it. -->
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
            <!-- commitStored-exempt: local filter state on `bind:value`; no store setter clamps it. -->
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
              {@const cat = store.ruleset?.ruleset.spells?.[chosen.spell]}
              {@const total = castingTotalOf(i)}
              <li class:invalid-selection={invalidIds.has(chosen.spell)}>
                <!-- Deliberately focusable, and NOT on the `<li>` (Sabine 3): a list
                     item is not interactive, and `use:tooltip` points
                     `aria-describedby` at its own node, which is not inherited. The
                     row's remove button made the popup open on focus by bubbling
                     while describing nothing in the tab order. Same host the
                     Available side uses — `SourcePicker` puts it on a real
                     `<button>` — and the same shape as `AbilityTab`'s chosen rows. -->
                <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
                <span
                  class="item-name"
                  tabindex="0"
                  use:tooltip={tip(chosen.spell)}
                  data-testid="spell-name-{chosen.spell}-{i}">{rowLabel(chosen)}</span
                >
                {#if total != null}
                  <span
                    class="casting-total-badge"
                    data-testid="spell-casting-total-{chosen.spell}-{i}"
                  >
                    {store.t('spell-casting-total', { total: String(total) })}
                    {#if castingUnusable(i)}
                      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
                      <span
                        class="derived-unusable"
                        tabindex="0"
                        use:tooltip={{ text: store.t('derived-unusable-tooltip') }}
                        data-testid="spell-casting-unusable-{chosen.spell}-{i}"
                      >
                        {store.t('derived-unusable')}
                      </span>
                    {/if}
                  </span>
                {/if}
                {#if cat && (hasFocusFigure(cat) || chosen.within_focus)}
                  <label class="checkbox inline within-focus-toggle">
                    <input
                      type="checkbox"
                      aria-label={store.t('spell-within-focus-label')}
                      checked={chosen.within_focus ?? false}
                      onchange={(e) =>
                        setMarkKeepingFocus(e.currentTarget as HTMLInputElement, (checked) =>
                          store.setSpellWithinFocusAt(i, checked),
                        )}
                      data-testid="spell-within-focus-{chosen.spell}-{i}"
                    /><span>{store.t('spell-within-focus-label')}</span>
                  </label>
                {/if}
                {#if cat && (hasPotentFieldFigure(cat) || chosen.within_potent_field)}
                  <!-- D79: independent of the within-focus toggle above — a
                       character may hold a Magical Focus, Potent Magic, both,
                       or neither, and the two free-text themes need not
                       coincide. -->
                  <label class="checkbox inline within-potent-field-toggle">
                    <input
                      type="checkbox"
                      aria-label={store.t('spell-within-potent-field-label')}
                      checked={chosen.within_potent_field ?? false}
                      onchange={(e) =>
                        setMarkKeepingFocus(e.currentTarget as HTMLInputElement, (checked) =>
                          store.setSpellWithinPotentFieldAt(i, checked),
                        )}
                      data-testid="spell-within-potent-field-{chosen.spell}-{i}"
                    /><span>{store.t('spell-within-potent-field-label')}</span>
                  </label>
                {/if}
                {#if isParametrized(chosen.spell)}
                  <!-- The target Form of a meta-magic Vim spell — display + identity
                     only, so the same spell can be taken once per distinct Form.
                     Same Forms-only menu ParameterPicker's `form` domain shows —
                     both read the shared `artsOfType(ruleset, 'form')`.
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
                    use:commitStored={{
                      read: () => chosen.level ?? minLevelForChosen(chosen.spell),
                    }}
                    data-testid="spell-level-input-{chosen.spell}-{i}"
                  />
                {/if}
                <!-- Spell Mastery is an Ability every magus may buy from the general
                   apprenticeship pool (ArMDE:9518), so the spinner shows whenever the
                   advancement table can price it — not only with Mastered Spells /
                   Flawless Magic. This picker is already magus-and-Spells-tab-only. -->
                {#if masteryMax > 0}
                  <Spinner
                    testid="spell-mastery-{chosen.spell}-{i}"
                    decLabel={store.t('spell-mastery-decrement', { name: rowLabel(chosen) })}
                    decTestid="spell-mastery-dec-{chosen.spell}-{i}"
                    decDisabled={(chosen.mastery ?? 0) <= 0}
                    onDec={() => store.adjustSpellMasteryAt(i, -1, masteryMax)}
                    incLabel={store.t('spell-mastery-increment', { name: rowLabel(chosen) })}
                    incTestid="spell-mastery-inc-{chosen.spell}-{i}"
                    incDisabled={(chosen.mastery ?? 0) >= masteryMax}
                    onInc={() => store.adjustSpellMasteryAt(i, 1, masteryMax)}
                  >
                    {#snippet label()}
                      <span class="spinner-label">{store.t('spell-mastery-label')}</span>
                    {/snippet}
                    {#snippet children()}
                      <span
                        class="spinner-value"
                        data-testid="spell-mastery-score-{chosen.spell}-{i}"
                      >
                        {chosen.mastery ?? 0}
                      </span>
                    {/snippet}
                    {#snippet after()}
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
                    {/snippet}
                  </Spinner>
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
                <!-- Spell Mastery special abilities: one may be chosen per effective
                   mastery level (ArMDE:9524-9526). The add-picker hides once the
                   count reaches the effective mastery; a non-repeatable ability
                   already chosen is disabled in the list, a repeatable one
                   (Precise/Quick/Quiet Casting) stays selectable again.

                   LAST IN THE ROW, and that is layout, not an afterthought
                   (guided-creation-review-2026-08 #17). `.mastery-abilities` takes a
                   full-width wrap line of its own (app.css), because as the score
                   spinner's sibling in a content-width column it widened that column
                   the moment mastery reached 1 — pushing the elastic spell name back
                   and sliding the spinner sideways mid-click. A 100%-basis item claims
                   the line it starts on, so the row's `×` has to come BEFORE it or it
                   would be pushed onto a third line; reordering visually with `order`
                   instead would leave reading and focus order disagreeing with the
                   screen. -->
                {#if masteryMax > 0 && effectiveSpellMastery(chosen.mastery, masteryFloor) > 0 && masteryAbilityCatalogue.length > 0}
                  {@const effMastery = effectiveSpellMastery(chosen.mastery, masteryFloor)}
                  <SpellMasteryAbilityPicker
                    {chosen}
                    index={i}
                    {effMastery}
                    {masteryAbilityCatalogue}
                  />
                {/if}
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
