<script lang="ts">
  import { store } from '../state.svelte';
  import { tooltip, type TooltipContent } from '../actions';
  import type { SpellMasteryAbility, SpellSelection } from '../types';

  // The Spell Mastery special-ability picker for one chosen spell row: one may
  // be chosen per effective mastery level (Core:9524-9526). Extracted out of
  // `SpellTab.svelte` (G21, full-audit round) — the natural remaining cut once
  // V28 (full-audit) had already moved the eligibility helpers to `derive.ts`.
  // Self-sufficient off the global store, mirroring how `FamiliarPanel`/
  // `TalismanPanel` mount beside `MagicPossessions` with no props of their own:
  // the only state this needs that the store does not already hold is WHICH
  // row and how many mastery levels it has earned, both supplied by the caller.
  let {
    chosen,
    index,
    effMastery,
    masteryAbilityCatalogue,
  }: {
    chosen: SpellSelection;
    index: number;
    effMastery: number;
    masteryAbilityCatalogue: SpellMasteryAbility[];
  } = $props();

  const chosenAbilities = $derived(chosen.mastery_abilities ?? []);

  // A mastery ability's rules-text name — always via the i18n map, never the raw
  // id (falls back to the id only when the ruleset is not yet loaded).
  function masteryAbilityName(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }

  // A mastery ability's tooltip: its rules-text description.
  function masteryAbilityTip(id: string): TooltipContent {
    return { text: store.ruleset?.i18n[id]?.description ?? undefined };
  }
</script>

<!-- LAST IN THE ROW, and that is layout, not an afterthought
     (guided-creation-review-2026-08 #17). `.mastery-abilities` takes a
     full-width wrap line of its own (app.css), because as the score spinner's
     sibling in a content-width column it widened that column the moment
     mastery reached 1 — pushing the elastic spell name back and sliding the
     spinner sideways mid-click. A 100%-basis item claims the line it starts
     on, so the row's `×` (SpellTab's own remove button) has to come BEFORE
     this component or it would be pushed onto a third line; reordering
     visually with `order` instead would leave reading and focus order
     disagreeing with the screen. -->
<span class="mastery-abilities" data-testid="spell-mastery-abilities-{chosen.spell}-{index}">
  <span class="spinner-label">{store.t('spell-mastery-abilities-label')}</span>
  {#each chosenAbilities as abilityId, ai (`${abilityId}:${ai}`)}
    <span
      class="ability-chip"
      use:tooltip={masteryAbilityTip(abilityId)}
      data-testid="spell-mastery-ability-{chosen.spell}-{index}-{ai}"
    >
      <span class="chip-name">{masteryAbilityName(abilityId)}</span>
      <button
        type="button"
        class="icon-btn"
        aria-label={store.t('remove-item', { name: masteryAbilityName(abilityId) })}
        onclick={() => store.removeMasteryAbilityAt(index, ai)}
        data-testid="spell-mastery-ability-remove-{chosen.spell}-{index}-{ai}"
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
          store.addMasteryAbilityAt(index, sel.value);
          sel.value = '';
        }
      }}
      data-testid="spell-mastery-ability-add-{chosen.spell}-{index}"
    >
      <option value="" disabled>{store.t('spell-mastery-ability-add')}</option>
      {#each masteryAbilityCatalogue as ma (ma.id)}
        <option value={ma.id} disabled={!ma.repeatable && chosenAbilities.includes(ma.id)}
          >{masteryAbilityName(ma.id)}</option
        >
      {/each}
    </select>
  {/if}
</span>
