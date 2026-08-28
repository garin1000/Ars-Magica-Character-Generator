<script lang="ts">
  import { store } from '../state.svelte';
  import {
    abilityDisplayName,
    artLabel,
    artsOfType,
    displayName,
    groupArtsByType,
    localizedSortKey,
    paramValueUsage,
  } from '../derive';
  import { CHARACTERISTICS, type ParameterDef, type Selection } from '../types';

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
  function write(next: Record<string, string>, editRow: () => void): void {
    if (commit) {
      commit({ ref: selection.ref, params: next });
      return;
    }
    editRow();
  }

  function setParam(key: string, value: string) {
    write({ ...(selection.params ?? {}), [key]: value }, () => store.setParamAt(index, key, value));
  }

  function onSelect(key: string, event: Event) {
    setParam(key, (event.currentTarget as HTMLSelectElement).value);
  }

  function onTypeText(key: string, event: Event) {
    setParam(key, (event.currentTarget as HTMLInputElement).value.trim());
  }

  function abilityInstanceLabel(abilityId: string, parameter: string | null | undefined): string {
    if (!store.ruleset) return abilityId;
    return abilityDisplayName(store.ruleset, abilityId, parameter, (key) =>
      store.t('param-hint', { label: store.t(`param-label-${key}`) }),
    );
  }

  // The character's own ability instances — the only legal Puissant-style targets.
  // A parameterized ability ((Area) Lore) yields one option per instance.
  const abilityInstances = $derived(
    (store.entity.ability_scores ?? []).map((row) => ({
      value: row.parameter ? `${row.ability}${SEP}${row.parameter}` : row.ability,
      label: abilityInstanceLabel(row.ability, row.parameter),
    })),
  );

  // The composite value identifying this selection's current ability target, so
  // the matching <option> shows as selected.
  function abilityTargetValue(key: string): string {
    const abilityId = selection.params?.[key];
    if (!abilityId) return '';
    const instanceKey = store.ruleset?.ruleset.abilities?.[abilityId]?.parameter ?? undefined;
    const instance = instanceKey ? selection.params?.[instanceKey] : undefined;
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
    write({ ...(selection.params ?? {}), [key]: artId }, () =>
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
  // could only ever be an internal slug. No shipped catalogue entry declares it
  // today — the branch exists because the domain enum is exhaustive, and the whole
  // registry is the only menu the data supports (nothing narrows it further).
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

  // How many other selections of this same item already claim each target, so a
  // target at max_per_target is offered no further (e.g. a Characteristic already
  // taken twice by Great Characteristic, or an ability instance already Puissant).
  const maxPerTarget = $derived(
    store.ruleset?.ruleset.point_items[selection.ref]?.max_per_target ?? 1,
  );

  // A grant pick passes no index (-1 excludes nothing), so it reads the bought
  // rows without excluding one of them — a grant pick is not itself a bought row.
  function usage(key: string): Map<string, number> {
    return paramValueUsage(store.entity.selections ?? [], selection.ref, key, index);
  }

  function full(usageCounts: Map<string, number>, value: string): boolean {
    return (usageCounts.get(value) ?? 0) >= maxPerTarget;
  }

  // Composite ability targets already claimed by other selections of this item.
  const usedAbilityTargets = $derived.by(() => {
    const counts = new Map<string, number>();
    (store.entity.selections ?? []).forEach((s, i) => {
      if (i === index || s.ref !== selection.ref) return;
      const abilityId = s.params?.ability;
      if (!abilityId) return;
      const key = store.ruleset?.ruleset.abilities?.[abilityId]?.parameter ?? undefined;
      const instance = key ? s.params?.[key] : undefined;
      const value = instance ? `${abilityId}${SEP}${instance}` : abilityId;
      counts.set(value, (counts.get(value) ?? 0) + 1);
    });
    return counts;
  });
</script>

{#each params as param (param.key)}
  {@const used = usage(param.key)}
  <!-- The parameter type (Characteristic, Art, Language…) doubles as the empty
       prompt and the control's accessible name, so no separate label text is
       needed alongside it. -->
  {@const typeLabel = store.t(`param-label-${param.key}`)}
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
      <!-- Targets a specific ability instance the character holds; add it on the
           Abilities tab first. For (Area) Lore each area is its own target. -->
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
        {#each abilityInstances as instance, i (`${instance.value}:${i}`)}
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
      <!-- Targets another catalogue item by id. Latent: no shipped entry declares
           this domain, so the menu is the whole point-item registry — nothing in the
           data narrows it. The branch exists because the domain enum is exhaustive
           and a slug must never be typed by hand. -->
      <select
        aria-label={typeLabel}
        value={selection.params?.[param.key] ?? ''}
        onchange={(e) => onSelect(param.key, e)}
        data-testid="param-{selection.ref}-{param.key}-{suffix}"
      >
        <option value="" disabled>{typeLabel}</option>
        {#each itemOptions as option (option.value)}
          <option value={option.value} disabled={full(used, option.value)}>
            {option.label}
          </option>
        {/each}
      </select>
    {:else}
      <!-- `text` alone, and only `text`: the domain references no registry, so any
           non-empty value is legal and free text is the correct control. Every other
           variant has its own select above — the engine's `ParameterDomain` doc
           comment says as much, and #4 was exactly this fall-through catching four
           of them. -->
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
{/each}
