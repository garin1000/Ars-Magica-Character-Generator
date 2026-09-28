<script lang="ts">
  import { store } from '../state.svelte';
  import {
    abilityDisplayName,
    abilityParamDisplay,
    abilityParamKey,
    artLabel,
    artsOfType,
    canonicalizeMultiRefValue,
    displayName,
    excludeSelection,
    groupArtsByType,
    localizedSortKey,
    multiParamValue,
    paramValueUsage,
    singleParamValue,
    singleValuedParams,
    spellName,
  } from '../derive';
  import { CHARACTERISTICS, REALMS, type ParameterDef, type Selection } from '../types';

  // Two callers, two write paths. A *bought* selection lives at `index` in
  // `entity.selections` and is edited in place by the store's index-based
  // methods. A *grant pick* (House / Mythic-type open grant, an owed Warping
  // slot) lives in a choice map instead, so the caller passes `commit` and gets
  // handed the whole next Selection to store under its own choice_key.
  let {
    selection,
    params,
    index = -1,
    idSuffix,
    commit,
  }: {
    selection: Selection;
    params: ParameterDef[];
    index?: number;
    idSuffix?: string;
    commit?: (selection: Selection) => void;
  } = $props();

  // Test ids stay stable per caller: a bought row is identified by its index, a
  // grant pick by its choice_key.
  const suffix = $derived(idSuffix ?? String(index));

  // Separator joining an ability id and its instance value into one option value
  // (a NUL never appears in ids or in user-typed area/language names). Built from
  // a char code so the source file itself stays plain ASCII.
  const SEP = String.fromCharCode(0);

  /**
   * The single write sink: hand the composed params to the grant-pick `commit`,
   * or fall back to editing the bought row in place. `next` is always the FULL
   * next params object, mirroring what the index-path store method would write.
   */
  function write(next: Record<string, string | string[]>, editRow: () => void): void {
    if (commit) {
      commit({ ref: selection.ref, params: next });
      return;
    }
    editRow();
  }

  function setParam(key: string, value: string) {
    write({ ...singleValuedParams(selection.params), [key]: value }, () =>
      store.setParamAt(index, key, value),
    );
  }

  function onSelect(key: string, event: Event) {
    setParam(key, (event.currentTarget as HTMLSelectElement).value);
  }

  function onTypeText(key: string, event: Event) {
    setParam(key, (event.currentTarget as HTMLInputElement).value.trim());
  }

  // The `{ min, max }` bound a `number`-domain parameter's own `type` carries
  // (D35) — `ParameterDomain::Number` itself is the redundant half of the
  // pair and carries no bound of its own (see the engine's own doc comment),
  // so the range comes from `type`, never from `domain`.
  function numberRange(param: ParameterDef): { min: number; max: number } | undefined {
    return typeof param.type === 'object' ? param.type.number : undefined;
  }

  // Clamps on COMMIT (`change`, not `input`): a per-keystroke clamp would
  // fight a player typing a legal multi-digit value into a WIDE range (D3's
  // future truncated-apprenticeship-age reuse) by rewriting the box after the
  // first digit. `change` fires once the value is committed (blur or Enter),
  // so mid-typing keystrokes are never touched — the DOM shows exactly what
  // the player types until then, exactly as the text/select branches already
  // leave `store.setParamAt`'s write as the only source of truth for what is
  // shown. An out-of-range or non-integer value is clamped into range (a
  // blank box clears the choice, the same "not yet made" reading `onTypeText`
  // already gives an empty string).
  function onTypeNumber(
    key: string,
    range: { min: number; max: number } | undefined,
    event: Event,
  ) {
    const raw = (event.currentTarget as HTMLInputElement).value.trim();
    if (raw === '') {
      setParam(key, '');
      return;
    }
    const parsed = Number(raw);
    if (!Number.isFinite(parsed)) return;
    const whole = Math.trunc(parsed);
    const clamped = range ? Math.min(range.max, Math.max(range.min, whole)) : whole;
    setParam(key, String(clamped));
  }

  // Options for a `multi_ref` parameter — the only domain shipping today is
  // `spell` (Corrupted Spells, C5c): the character's OWN learned spells
  // (`Entity.spells`), never the whole spell catalogue — § 8's own "the
  // character's spells" reading (ArMDE:5859-5863). Deduplicated by base spell
  // id (a parameterized spell may carry several instances) and left in the
  // order the character learned them: nothing about "affect this spell" needs
  // a second sort axis, unlike the catalogue-wide lists above. A `multi_ref`
  // param currently only ever pairs with `spell` (C5a's own
  // `ParameterDomain::Spell` doc comment), so every other domain falls to the
  // empty defensive fallback rather than a branch of its own.
  function multiRefOptions(param: ParameterDef): { value: string; label: string }[] {
    if (param.domain !== 'spell') return [];
    const localized = store.ruleset;
    if (!localized) return [];
    const seen = new Set<string>();
    const options: { value: string; label: string }[] = [];
    for (const row of store.entity.spells ?? []) {
      if (seen.has(row.spell)) continue;
      seen.add(row.spell);
      options.push({ value: row.spell, label: spellName(localized, row.spell) });
    }
    return options;
  }

  // Toggles one value of a multi_ref parameter's set. Unlike a single-select's
  // onchange, there is no "current value" to read off the control that fired —
  // every OTHER checkbox in the group keeps its own independent state — so the
  // next set is this row's existing set plus/minus the one value the event
  // names, canonicalized (§ 8) before it is written so two players checking the
  // same values in different orders always write the identical array, and
  // unchecking the last one writes an empty array rather than dropping the key
  // (C5a's `missing_param`-not-`param_wrong_shape` semantics need the key
  // present).
  function onToggleMulti(key: string, value: string, event: Event): void {
    const checked = (event.currentTarget as HTMLInputElement).checked;
    const current = new Set(multiParamValue(selection.params?.[key]));
    if (checked) current.add(value);
    else current.delete(value);
    const canonical = canonicalizeMultiRefValue(current);
    write({ ...singleValuedParams(selection.params), [key]: canonical }, () =>
      store.setMultiParamAt(index, key, canonical),
    );
  }

  function abilityInstanceLabel(abilityId: string, parameter: string | null | undefined): string {
    if (!store.ruleset) return abilityId;
    return abilityDisplayName(store.ruleset, abilityId, parameter, (key) =>
      store.t('param-hint', { label: store.t(`param-label-${key}`) }),
    );
  }

  // Every catalogue Ability, plus the character's own instances of the
  // parameterized ones. Like the `art` domain below, the target need NOT already be
  // on the sheet: Puissant Ability is "choose one Ability" with no requirement that
  // a score exists (ArMDE:4814-4816), and
  // abilities are bought on a LATER step — so offering only owned rows left the
  // parameter unfillable where the Virtue is taken and deadlocked the wizard
  // (manual-testing-findings-2026-09-03 #5).
  //
  // A parameterized ability ((Area) Lore) keeps one option per owned instance AND
  // the generic catalogue entry, so a not-yet-bought area can be named too; the
  // instance input below then takes the area itself. An owned PLAIN ability needs no
  // extra option — its instance value is the bare id the catalogue entry already
  // carries, so listing it twice would only duplicate the row.
  const abilityOptions = $derived.by(() => {
    const localized = store.ruleset;
    if (!localized) return [];
    const catalogue = localized.ruleset.abilities ?? {};
    const rows = store.entity.ability_scores ?? [];
    return Object.keys(catalogue)
      .sort((a, b) => localizedSortKey(localized, a).localeCompare(localizedSortKey(localized, b)))
      .flatMap((id) => {
        const generic = { value: id, label: abilityInstanceLabel(id, undefined) };
        if (!catalogue[id]?.parameter) return [generic];
        const instances = rows
          .filter((row) => row.ability === id && abilityParamKey(row.parameter))
          .map((row) => ({
            value: `${id}${SEP}${abilityParamKey(row.parameter)}`,
            label: abilityInstanceLabel(id, abilityParamDisplay(row.parameter, localized)),
          }));
        return [generic, ...instances];
      });
  });

  // The instance key the chosen target still needs a value for ((Area) Lore →
  // `area`), or undefined for a plain target. Only meaningful once an ability is
  // chosen — and it is what keeps the generic catalogue entry from being a dead end
  // of its own, since the engine expects that key (`missing_param` otherwise).
  function abilityInstanceKey(key: string): string | undefined {
    const abilityId = singleParamValue(selection.params?.[key]);
    if (!abilityId) return undefined;
    return store.ruleset?.ruleset.abilities?.[abilityId]?.parameter ?? undefined;
  }

  // The composite value identifying this selection's current ability target, so
  // the matching <option> shows as selected.
  function abilityTargetValue(key: string): string {
    const abilityId = singleParamValue(selection.params?.[key]);
    if (!abilityId) return '';
    const instanceKey = store.ruleset?.ruleset.abilities?.[abilityId]?.parameter ?? undefined;
    const instance = instanceKey ? singleParamValue(selection.params?.[instanceKey]) : undefined;
    return instance ? `${abilityId}${SEP}${instance}` : abilityId;
  }

  function onSelectAbility(key: string, event: Event) {
    const raw = (event.currentTarget as HTMLSelectElement).value;
    const [abilityId, parameter] = raw.split(SEP);
    // Mirrors setAbilityBonusTarget: the target ability plus, for a parameterized
    // ability, the instance discriminator under that ability's own param key. Any
    // stale instance key from a previous target is dropped.
    const next: Record<string, string> = { [key]: abilityId };
    const instanceKey = store.ruleset?.ruleset.abilities?.[abilityId]?.parameter ?? undefined;
    if (instanceKey && parameter) next[instanceKey] = parameter;
    write(next, () => store.setAbilityBonusTarget(index, abilityId, parameter));
  }

  // Every catalogue Art (Techniques then Forms) — the legal Puissant Art targets.
  // Unlike abilities, an Art need not be on the sheet to be a Puissant target.
  const artOptions = $derived(
    store.ruleset
      ? groupArtsByType(store.ruleset).flatMap((group) =>
          group.arts.map((art) => ({ value: art.id, label: artLabel(store.ruleset!, art.id) })),
        )
      : [],
  );

  function onSelectArt(key: string, event: Event) {
    const artId = (event.currentTarget as HTMLSelectElement).value;
    write({ ...singleValuedParams(selection.params), [key]: artId }, () =>
      store.setArtBonusTarget(index, key, artId),
    );
  }

  // The `technique` and `form` domains are the `art` domain narrowed to one Art
  // class: the engine validates the value as an Art id AND as that class
  // (`ParameterDomain::Technique` / `Form`), so offering the other class would only
  // produce `unknown_param_value`. Options come from the shared `artsOfType`, the
  // same helper the Spells tab's Technique/Form filters and the meta-magic Vim
  // spells' target Form read — one Art picker, not three that can drift.
  const techniqueOptions = $derived(
    store.ruleset
      ? artsOfType(store.ruleset, 'technique').map((art) => ({
          value: art.id,
          label: artLabel(store.ruleset!, art.id),
        }))
      : [],
  );
  const formOptions = $derived(
    store.ruleset
      ? artsOfType(store.ruleset, 'form').map((art) => ({
          value: art.id,
          label: artLabel(store.ruleset!, art.id),
        }))
      : [],
  );

  // The `item` domain resolves against the point-item registry, so a typed string
  // could only ever be an internal slug. False Power's `virtue` target is the
  // shipped user (narrowed below by `itemOptionsFor`); the branch itself exists
  // unconditionally because the domain enum is exhaustive.
  const itemOptions = $derived(
    store.ruleset
      ? Object.keys(store.ruleset.ruleset.point_items)
          .map((id) => ({
            value: id,
            label: displayName(store.ruleset!, id, undefined, (key) =>
              store.t('param-hint', { label: store.t(`param-label-${key}`) }),
            ),
          }))
          .sort((a, b) =>
            localizedSortKey(store.ruleset!, a.value).localeCompare(
              localizedSortKey(store.ruleset!, b.value),
            ),
          )
      : [],
  );

  // `require_categories` narrows that registry to a category, the way `technique`
  // and `form` narrow the Art catalogue: the engine resolves a value outside it to
  // `unknown_param_value` (`param_value_resolves`), so offering it would only
  // offer an illegal choice. Membership categories only — `has_category` in the
  // engine — since a value names an item, not a selection of one, and
  // `index_categories` is provenance rather than membership.
  //
  // `allow_ids` (D34) is additive to `require_categories`, mirroring the
  // engine's own OR: an option is offered if EITHER test passes. False
  // Power's target is `require_categories: ['supernatural']` plus
  // `allow_ids: ['virtue.diedne_magic', 'virtue.the_gift']` — the two named
  // Virtues the category axis alone cannot reach — read straight off the
  // ruleset data, never re-derived here.
  //
  // `require_possessed` and `forbid_tainted` narrow it further, and for the
  // same reason: the engine raises `param_target_not_possessed` for a Virtue
  // nobody holds and `unknown_param_value` for a Tainted one, so offering
  // either is offering a choice the engine will refuse. Possession is the
  // grants-inclusive set the engine reads (`PrereqCtx::present_ids`), so a
  // House/warping-granted row counts exactly as a bought one does.
  function itemOptionsFor(param: ParameterDef): { value: string; label: string }[] {
    if (!store.ruleset) return itemOptions;
    const items = store.ruleset.ruleset.point_items;
    const required = param.require_categories;
    const allowed = param.allow_ids;
    let options = itemOptions;
    if (required?.length || allowed?.length) {
      options = options.filter(
        (option) =>
          required?.some((category) => items[option.value]?.categories.includes(category)) ||
          allowed?.includes(option.value),
      );
    }
    if (param.forbid_tainted) {
      options = options.filter((option) => !items[option.value]?.tainted);
    }
    if (param.require_possessed) {
      options = options.filter((option) => heldItemRefs().has(option.value));
    }
    return options;
  }

  // Every point item the character holds: bought rows plus granted ones, the
  // frontend mirror of the engine's `present_ids`.
  function heldItemRefs(): Set<string> {
    const held = new Set<string>();
    for (const s of store.entity.selections ?? []) held.add(s.ref);
    for (const s of store.effective?.granted_selections ?? []) held.add(s.ref);
    return held;
  }

  // How many other selections of this same item already claim each target, so a
  // target at max_per_target is offered no further (e.g. a Characteristic already
  // taken twice by Great Characteristic, or an ability instance already Puissant).
  const maxPerTarget = $derived(
    store.ruleset?.ruleset.point_items[selection.ref]?.max_per_target ?? 1,
  );

  // Granted rows of THIS item, folded in alongside the bought ones so a
  // House/Mythic/warping-granted copy counts against the same `max_per_target`
  // cap as a bought one (the UI half of the engine's grant-aware
  // `validate_duplicate_selections`/`validate_total_selection_cap`) — a
  // granted Puissant Ignem used to leave a bought Puissant Perdo's Art target
  // list blind to it.
  //
  // A grant pick (`commit` set, `index === -1`) is itself one of these granted
  // rows once resolved — `entity_grants`/`resolve_grants` folds a stored open
  // pick straight into `granted_selections`, unmodified — so it is excluded
  // here (by value, via `excludeSelection`) or it would count against its own
  // current target and greys out the very value it already holds. A bought
  // row being edited (`index` >= 0) carries no granted counterpart of its own,
  // so nothing is excluded in that case.
  function grantedForUsage(): Selection[] {
    const granted = store.effective?.granted_selections ?? [];
    return excludeSelection(granted, index === -1 ? selection : undefined);
  }

  // Sums two per-value usage maps (bought + granted), so counting each source
  // separately still yields one combined total per target value.
  function mergeUsage(a: Map<string, number>, b: Map<string, number>): Map<string, number> {
    const out = new Map(a);
    for (const [value, count] of b) out.set(value, (out.get(value) ?? 0) + count);
    return out;
  }

  // A grant pick passes no index (-1 excludes nothing FROM entity.selections),
  // so it reads the bought rows without excluding one of them — a grant pick is
  // not itself a bought row (see `grantedForUsage` for its own self-exclusion).
  //
  // The bought and granted counts are computed as TWO separate
  // `paramValueUsage` passes rather than one pass over a concatenated array:
  // `index` addresses a position in `entity.selections` specifically, and
  // reusing it against a combined array would misfire whenever the bought list
  // is shorter than `index` implies — the granted list would start at position
  // `index` too, so `index` would silently exclude the wrong (granted) row
  // instead of the bought one it was meant for.
  //
  // Both passes carry this row's OTHER parameter values, because a target is
  // the whole `(ref, params)` tuple — the engine's own duplicate key. Folk
  // Magic has two axes (ArMDE:3909)
  // and `ArMDE:3919` lets a second copy "align it to the same Realm as before or
  // pick a different one", so a copy sharing this row's spell category in a
  // different Realm is legal and must not grey anything out here.
  function usage(key: string): Map<string, number> {
    const siblings = selection.params;
    const bought = paramValueUsage(
      store.entity.selections ?? [],
      selection.ref,
      key,
      index,
      siblings,
    );
    const granted = paramValueUsage(grantedForUsage(), selection.ref, key, -1, siblings);
    return mergeUsage(bought, granted);
  }

  function full(usageCounts: Map<string, number>, value: string): boolean {
    return (usageCounts.get(value) ?? 0) >= maxPerTarget;
  }

  // Composite ability targets already claimed by other selections of this item
  // — bought plus granted, mirroring `usage()` above (same two-pass reasoning:
  // a granted row must never be excluded by a bought row's `index`).
  const usedAbilityTargets = $derived.by(() => {
    const counts = new Map<string, number>();
    const addFrom = (list: Selection[], exceptIndex: number): void => {
      list.forEach((s, i) => {
        if (i === exceptIndex || s.ref !== selection.ref) return;
        const abilityId = singleParamValue(s.params?.ability);
        if (!abilityId) return;
        const key = store.ruleset?.ruleset.abilities?.[abilityId]?.parameter ?? undefined;
        const instance = key ? singleParamValue(s.params?.[key]) : undefined;
        const value = instance ? `${abilityId}${SEP}${instance}` : abilityId;
        counts.set(value, (counts.get(value) ?? 0) + 1);
      });
    };
    addFrom(store.entity.selections ?? [], index);
    addFrom(grantedForUsage(), -1);
    return counts;
  });
</script>

{#each params as param (param.key)}
  {@const used = usage(param.key)}
  <!-- The parameter type (Characteristic, Art, Language…) doubles as the empty
       prompt and the control's accessible name, so no separate label text is
       needed alongside it. -->
  {@const typeLabel = store.t(`param-label-${param.key}`)}
  <!-- The instance key an ability target still owes a value for ((Area) Lore →
       `area`), rendered as a control of its own below rather than inside the same
       label, so each control keeps exactly one label. -->
  {@const instanceKey = param.domain === 'ability' ? abilityInstanceKey(param.key) : undefined}
  <!-- B4/Q-51: a `required_if`-gated parameter (Magical Blood's `characteristic`,
       meaningless for Magic Animal/Spirit/Thing) is hidden entirely while its
       gate does not hold — the engine's own `missing_param` relaxation
       (`validation/selections.rs`) means there is nothing to fill, so showing
       the control would ask for a choice the character never uses. -->
  {@const gateHolds =
    !param.required_if ||
    singleParamValue(selection.params?.[param.required_if.param]) === param.required_if.equals}
  {#if gateHolds}
    {#if param.type === 'multi_ref'}
      <!-- The multi-select control for a MultiRef parameter (D9 part 3, C5b) —
         a checkbox GROUP, so its container is a `<fieldset>`/`<legend>`, never
         a `<label>` (a label wraps ONE control, not several). Checked FIRST,
         ahead of every domain branch below, so a future multi_ref param on the
         `ability`/`art` domain (C5c's Corrupted Abilities/Arts) renders here
         rather than falling into that domain's single-select branch. Follows
         the same checkbox-list markup `LivingConditionsPicker.svelte` already
         uses (`.checkbox.inline`, a `<ul>` of `<li>` rows) — one accessible
         multi-select pattern, not two that could drift. -->
      {@const options = multiRefOptions(param)}
      <fieldset class="param multi-ref" data-testid="param-{selection.ref}-{param.key}-{suffix}">
        <legend>{typeLabel}</legend>
        {#if options.length === 0}
          <p class="hint">{store.t('param-multi-ref-empty')}</p>
        {/if}
        <ul>
          {#each options as option (option.value)}
            <li>
              <label class="checkbox inline">
                <input
                  type="checkbox"
                  checked={multiParamValue(selection.params?.[param.key]).includes(option.value)}
                  onchange={(e) => onToggleMulti(param.key, option.value, e)}
                  data-testid="param-{selection.ref}-{param.key}-{suffix}-{option.value}"
                />
                <span>{option.label}</span>
              </label>
            </li>
          {/each}
        </ul>
      </fieldset>
    {:else}
      <label class="param">
        {#if param.domain === 'characteristic'}
          <select
            aria-label={typeLabel}
            value={selection.params?.[param.key] ?? ''}
            onchange={(e) => onSelect(param.key, e)}
            data-testid="param-{selection.ref}-{param.key}-{suffix}"
          >
            <option value="" disabled>{typeLabel}</option>
            {#each CHARACTERISTICS as characteristic (characteristic)}
              <option
                value="characteristic.{characteristic}"
                disabled={full(used, `characteristic.${characteristic}`)}
              >
                {store.t(`characteristic-${characteristic}`)}
              </option>
            {/each}
          </select>
        {:else if param.domain === 'ability'}
          <!-- Targets an Ability from the catalogue — it need not be on the sheet yet,
           since abilities are bought on a later step. For (Area) Lore each area is
           its own target, so the character's own areas are listed alongside the
           generic entry (which the instance input below completes). -->
          <select
            aria-label={typeLabel}
            value={abilityTargetValue(param.key)}
            onchange={(e) => onSelectAbility(param.key, e)}
            data-testid="param-{selection.ref}-{param.key}-{suffix}"
          >
            <option value="" disabled>{typeLabel}</option>
            <!-- Keyed by index as well as value: two instances of a parameterized
             ability whose parameter is still unset share the same bare id, and a
             duplicate key throws `each_key_duplicate` (in production too), which
             would kill this whole tab's render. -->
            {#each abilityOptions as instance, i (`${instance.value}:${i}`)}
              <option value={instance.value} disabled={full(usedAbilityTargets, instance.value)}>
                {instance.label}
              </option>
            {/each}
          </select>
        {:else if param.domain === 'art' || param.domain === 'technique' || param.domain === 'form'}
          <!-- Targets a Hermetic Art. `art` accepts either class; `technique` and `form`
           are the same control narrowed to one, because the engine validates those
           two as an Art id PLUS the right ArtType and raises unknown_param_value
           otherwise. One branch for all three, so the narrowed domains cannot fall
           through to a text input where only an internal slug would ever pass.
           `max_per_target` keeps the same Art from being picked twice. The param key
           is unrelated to the domain — Affinity with (Art) keys on `art`, Deft (Form)
           on `form`. -->
          {@const artChoices =
            param.domain === 'technique'
              ? techniqueOptions
              : param.domain === 'form'
                ? formOptions
                : artOptions}
          <select
            aria-label={typeLabel}
            value={selection.params?.[param.key] ?? ''}
            onchange={(e) => onSelectArt(param.key, e)}
            data-testid="param-{selection.ref}-{param.key}-{suffix}"
          >
            <option value="" disabled>{typeLabel}</option>
            {#each artChoices as art (art.value)}
              <option value={art.value} disabled={full(used, art.value)}>
                {art.label}
              </option>
            {/each}
          </select>
        {:else if param.domain === 'item'}
          <!-- Targets another catalogue item by id — False Power's `virtue` target is
           the shipped user. The menu is the point-item registry, narrowed to the
           parameter's `require_categories`/`allow_ids` (D34) when it declares
           either. The branch exists unconditionally because the domain enum is
           exhaustive and a slug must never be typed by hand. -->
          <select
            aria-label={typeLabel}
            value={selection.params?.[param.key] ?? ''}
            onchange={(e) => onSelect(param.key, e)}
            data-testid="param-{selection.ref}-{param.key}-{suffix}"
          >
            <option value="" disabled>{typeLabel}</option>
            {#each itemOptionsFor(param) as option (option.value)}
              <option value={option.value} disabled={full(used, option.value)}>
                {option.label}
              </option>
            {/each}
          </select>
        {:else if param.domain === 'enumerated'}
          <!-- The parameter carries its own closed list, so the menu is the DATA's
           `values` and nothing narrows a catalogue: Folk Magic's four spell
           categories, the (Beings) classes. Each id is mapped through the rules
           i18n here — options do NOT get localized for free (`resolveIssueArgValue`
           localizes validation-issue arguments, not picker options), and rendering
           the slug would be the same violation as hardcoding a string.
           `max_per_target` greys out a value another copy already holds, which is
           what caps Folk Magic at one copy per category. -->
          <select
            aria-label={typeLabel}
            value={selection.params?.[param.key] ?? ''}
            onchange={(e) => onSelect(param.key, e)}
            data-testid="param-{selection.ref}-{param.key}-{suffix}"
          >
            <option value="" disabled>{typeLabel}</option>
            {#each param.values ?? [] as value (value)}
              <option {value} disabled={full(used, value)}>
                {store.ruleset
                  ? displayName(store.ruleset, value, undefined, (key) =>
                      store.t('param-hint', { label: store.t(`param-label-${key}`) }),
                    )
                  : value}
              </option>
            {/each}
          </select>
        {:else if param.domain === 'category'}
          <!-- The item is "taken as" one of its OWN listed categories — Sufi
           (ArMDE:5083) "either as a
           Minor Social Status Virtue or a Minor Supernatural Virtue". The menu
           is the item's own declared `values` (a subset of its `categories`,
           enforced at load), labelled through the same `category-<id>` Fluent
           family the V/F badge and the category filter already use — NOT
           `displayName`, which resolves rules ids and would find no i18n entry
           for a bare category slug. `max_per_target` (default 1, and every
           `taken_as` item's `max_total` is forced to 1 too) greys out a
           reading another copy already holds. -->
          <select
            aria-label={typeLabel}
            value={selection.params?.[param.key] ?? ''}
            onchange={(e) => onSelect(param.key, e)}
            data-testid="param-{selection.ref}-{param.key}-{suffix}"
          >
            <option value="" disabled>{typeLabel}</option>
            {#each param.values ?? [] as value (value)}
              <option {value} disabled={full(used, value)}>
                {store.t(`category-${value}`)}
              </option>
            {/each}
          </select>
        {:else if param.domain === 'realm'}
          <!-- The supernatural realm the item is aligned to — Folk Magic
           (ArMDE:3909) "The choice of
           (Realm) Lore also determines which supernatural realm his magic is
           aligned to". The four Realms are a closed engine taxonomy, so the menu
           is `REALMS` (the single frontend source of that order, never
           re-hardcoded) and the labels are the `realm-<id>` Fluent family the
           Might picker already reads — NOT `displayName`, which resolves rules
           ids and would find no i18n entry for a bare realm slug.
           Deliberately NOT greyed by `full()`: `ArMDE:3919` says a further copy "can
           align it to the same Realm as before or pick a different one", so a
           Realm another copy holds stays offered. What may not be repeated is
           the whole target, which `usage()` now judges across every axis. -->
          <select
            aria-label={typeLabel}
            value={selection.params?.[param.key] ?? ''}
            onchange={(e) => onSelect(param.key, e)}
            data-testid="param-{selection.ref}-{param.key}-{suffix}"
          >
            <option value="" disabled>{typeLabel}</option>
            {#each REALMS as realm (realm)}
              <option value="realm.{realm}">{store.t(`realm-${realm}`)}</option>
            {/each}
          </select>
        {:else if param.domain === 'number'}
          <!-- D35: a bounded integer count (Simple Student's finished-years, 1-2)
           — the first numeric parameter, so the first `<input type="number">`
           control. `min`/`max` come from the parameter's own `type` (the
           bound lives there, not on `domain` — see `numberRange`); the
           browser's own spinner/validity affordance guides the *keyboard*,
           but the actual clamp is enforced in `onTypeNumber` on commit, since
           an out-of-range value would otherwise round-trip through
           `setParamAt` unclamped and only be caught by the engine's
           `unknown_param_value` after a revalidate. -->
          {@const range = numberRange(param)}
          <input
            type="number"
            aria-label={typeLabel}
            min={range?.min}
            max={range?.max}
            value={selection.params?.[param.key] ?? ''}
            onchange={(e) => onTypeNumber(param.key, range, e)}
            data-testid="param-{selection.ref}-{param.key}-{suffix}"
          />
        {:else}
          <!-- `text` alone, and only `text`: the domain references no registry, so any
           value with non-whitespace content is legal and free text is the correct
           control. That "non-empty" claim is now ENFORCED rather than merely
           asserted here: `onTypeText` trims (as does `store.setParamAt`, and the
           engine again on load), and a value that trims to nothing raises the
           engine's `missing_param` naming this box — so a blank descriptor no
           longer validates clean, and 'Wolf Shape ' is the same power as
           'Wolf Shape'. Case is deliberately untouched. `require_power` does NOT
           turn this into a select over the character's own powers, unlike the
           way `require_possessed` narrows the `item` menu: powers are entered on
           the Review step, AFTER Virtues and Flaws, so a select would be empty
           and unfillable where the Flaw is taken — the deadlock
           `ability_bonus_dangling_target` already taught us. The engine reports
           `power_dangling_target` on Review instead, where the fix lives.
           Every other domain has its
           own control above (a select, or `number`'s bounded input) — the
           engine's `ParameterDomain` doc comment says as much, and #4 was
           exactly this fall-through catching four of them. Keep that true:
           the domain union is closed, so a new variant belongs in a branch of
           its own, never here. -->
          <input
            type="text"
            aria-label={typeLabel}
            placeholder={typeLabel}
            value={selection.params?.[param.key] ?? ''}
            oninput={(e) => onTypeText(param.key, e)}
            data-testid="param-{selection.ref}-{param.key}-{suffix}"
          />
        {/if}
      </label>
    {/if}
  {/if}
  {#if instanceKey}
    <!-- The area/language the chosen parameterized target names. Free text, exactly
         as on the Abilities tab: the value is the player's own, backed by no
         registry. The engine expects this key whenever the target ability is
         parameterized, so without this control the generic catalogue entry above
         would leave a `missing_param` nothing on screen could clear. -->
    {@const instanceLabel = store.t(`param-label-${instanceKey}`)}
    <label class="param">
      <input
        type="text"
        aria-label={instanceLabel}
        placeholder={instanceLabel}
        value={selection.params?.[instanceKey] ?? ''}
        oninput={(e) => onTypeText(instanceKey, e)}
        data-testid="param-{selection.ref}-{instanceKey}-{suffix}"
      />
    </label>
  {/if}
{/each}
