<script lang="ts">
  import { store } from '../state.svelte';
  import { abilityDisplayName, artLabel, groupArtsByType, paramValueUsage } from '../derive';
  import { CHARACTERISTICS, type ParameterDef, type Selection } from '../types';

  let {
    selection,
    index,
    params,
  }: { selection: Selection; index: number; params: ParameterDef[] } = $props();

  // Separator joining an ability id and its instance value into one option value
  // (NUL never appears in ids or in user-typed area/language names).
  const SEP = '\u0000';

  function onSelect(key: string, event: Event) {
    store.setParamAt(index, key, (event.currentTarget as HTMLSelectElement).value);
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
  function abilityTargetValue(): string {
    const abilityId = selection.params?.ability;
    if (!abilityId) return '';
    const key = store.ruleset?.ruleset.abilities?.[abilityId]?.parameter ?? undefined;
    const instance = key ? selection.params?.[key] : undefined;
    return instance ? `${abilityId}${SEP}${instance}` : abilityId;
  }

  function onSelectAbility(event: Event) {
    const raw = (event.currentTarget as HTMLSelectElement).value;
    const [abilityId, parameter] = raw.split(SEP);
    store.setAbilityBonusTarget(index, abilityId, parameter);
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

  function onSelectArt(event: Event) {
    store.setArtBonusTarget(index, (event.currentTarget as HTMLSelectElement).value);
  }

  // How many other selections of this same item already claim each target, so a
  // target at max_per_target is offered no further (e.g. a Characteristic already
  // taken twice by Great Characteristic, or an ability instance already Puissant).
  const maxPerTarget = $derived(
    store.ruleset?.ruleset.point_items[selection.ref]?.max_per_target ?? 1,
  );

  function usage(key: string): Map<string, number> {
    return paramValueUsage(store.entity.selections, selection.ref, key, index);
  }

  function full(usageCounts: Map<string, number>, value: string): boolean {
    return (usageCounts.get(value) ?? 0) >= maxPerTarget;
  }

  // Composite ability targets already claimed by other selections of this item.
  const usedAbilityTargets = $derived.by(() => {
    const counts = new Map<string, number>();
    store.entity.selections.forEach((s, i) => {
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
  <label class="param">
    <span>{store.t('param-prompt', { param: store.t(`param-label-${param.key}`) })}</span>
    {#if param.domain === 'characteristic'}
      <select
        value={selection.params?.[param.key] ?? ''}
        onchange={(e) => onSelect(param.key, e)}
        data-testid="param-{selection.ref}-{param.key}-{index}"
      >
        <option value="" disabled>{store.t('param-placeholder')}</option>
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
        value={abilityTargetValue()}
        onchange={onSelectAbility}
        data-testid="param-{selection.ref}-{param.key}-{index}"
      >
        <option value="" disabled>{store.t('param-placeholder')}</option>
        {#each abilityInstances as instance (instance.value)}
          <option value={instance.value} disabled={full(usedAbilityTargets, instance.value)}>
            {instance.label}
          </option>
        {/each}
      </select>
    {:else if param.domain === 'art'}
      <!-- Targets a Hermetic Art (Puissant Art). Any catalogue Art is a legal
           target; max_per_target keeps the same Art from being picked twice. -->
      <select
        value={selection.params?.[param.key] ?? ''}
        onchange={onSelectArt}
        data-testid="param-{selection.ref}-{param.key}-{index}"
      >
        <option value="" disabled>{store.t('param-placeholder')}</option>
        {#each artOptions as art (art.value)}
          <option value={art.value} disabled={full(used, art.value)}>
            {art.label}
          </option>
        {/each}
      </select>
    {:else}
      <input
        type="text"
        placeholder={store.t('param-placeholder')}
        value={selection.params?.[param.key] ?? ''}
        oninput={(e) =>
          store.setParamAt(index, param.key, (e.currentTarget as HTMLInputElement).value.trim())}
        data-testid="param-{selection.ref}-{param.key}-{index}"
      />
    {/if}
  </label>
{/each}
