<script lang="ts">
  import { store } from '../state.svelte';
  import {
    formatSigned,
    maxAbilityScore,
    spellLevelAllocation,
    spellMasteryXpSpent,
  } from '../derive';

  // Whether the base is READ-ONLY at this mount. The guided wizard passes `true`;
  // the editor leaves it editable (guided-creation-review-2026-08 #19, DECIDED).
  //
  // An explicit prop, and deliberately NOT a `store` lookup of which flow is
  // running: reading the flow here would make the component's behaviour depend on
  // global state its own caller cannot see, and every mount would have to be traced
  // to know what it renders. The mount already declares its own differences through
  // `WizardStep`'s `barProps` seam (which carries `XpBar`'s testid prefix the same
  // way), so the divergence is visible at the mount site.
  //
  // Why the wizard may withhold it: the 120 is a fixed rules grant — "Take 120
  // levels of spells" (Ars Magica - Definitive Edition (Core Rules).md:2215) — and
  // every legitimate in-rules variation of the budget already arrives somewhere else
  // on this bar: Skilled/Weak Parens as the `bonus` entry, and the levels bought past
  // the Gauntlet as the `lifeStage` entry. The editor keeps the field because direct
  // entry exists precisely to record a character the rules-as-written did not build.
  let { readonlyBase = false }: { readonlyBase?: boolean } = $props();

  // The Spells tab's budget status line. Sits ABOVE the Available/Selected lists
  // as a sibling status bar — the same placement (and the same markup, classes and
  // overspend behavior) as the shared XP bar on the Abilities/Arts tabs, so both
  // budgets read identically: label, used figure, total, then Available. It was
  // previously rendered inside the selected list's column header, which put a
  // whole-character budget inside one of the two lists.
  const budget = $derived(store.effective?.spell_levels_budget ?? 0);
  const used = $derived(store.effective?.spell_levels_used ?? 0);
  // The type profile's base budget (120 for a magus) is the base field's
  // placeholder, so the default stays data-driven (never a Svelte literal).
  const profileBase = $derived(store.effective?.spell_levels_profile_base ?? 0);
  // The V/F contribution on its own (Skilled Parens +30, Weak Parens -30).
  const bonus = $derived(store.effective?.spell_levels_bonus ?? 0);
  // The levels the magus's years past its Gauntlet bought — the slice of its
  // 30-points-a-year it took as spells rather than experience. There is no input
  // for it here on purpose: that one number defines both the experience pool and
  // this budget, and the magus phase order is abilities, arts, spells, so editing
  // it on the Spells step would retroactively shrink an xp pool already spent two
  // steps earlier. The Abilities step owns the choice; this step shows what it did.
  const lifeStage = $derived(store.effective?.spell_levels_life_stage ?? 0);
  // Split the spend between the base, the V/F modifier and the post-Gauntlet levels
  // so this bar reads exactly like the XP bar: a positive bonus is spent first (as
  // the engine drains restricted pools before the general one), a penalty is
  // charged to the base, and the already-earned post-Gauntlet levels sit on the
  // base's side.
  //
  // The figures close against `alloc.denominator` — the whole unconditional side,
  // `base + lifeStage` — and NOT against the bracketed base, which they do only
  // while `lifeStage === 0`. Claiming otherwise here is what let #18 stand: see
  // `spellLevelAllocation` in derive.ts for the honest statement of the invariant.
  const alloc = $derived(spellLevelAllocation(used, budget, bonus, lifeStage));
  // Not clamped: overspending shows a negative value in bold red (`over`) with the
  // used figure in plain red (`over-value`) — exactly as the XP pool does.
  const available = $derived(alloc.available);

  // Spell-Mastery: XP pool (Mastered Spells) + auto-mastery floor (Flawless Magic)
  // + whether Flawless Magic doubles advancement (halving each mastery point's XP).
  const masteryXp = $derived(store.effective?.spell_mastery_xp ?? 0);
  const masteryFloor = $derived(store.effective?.spell_mastery_floor ?? 0);
  const masteryDoubled = $derived(store.effective?.spell_mastery_advancement_doubled ?? false);
  // The Mastery Ability rises like an Ability, so it is priced from the Ability
  // advancement table (data, not a hardcoded mechanic — same path as abilities).
  const advancement = $derived(store.ruleset?.ruleset.advancement ?? []);
  // Charged like the engine: only mastery above the free floor, halved when
  // Flawless Magic doubles advancement totals.
  const masteryUsed = $derived(
    spellMasteryXpSpent(advancement, store.entity.spells ?? [], masteryFloor, masteryDoubled),
  );
  // Referenced so the mastery read-out only claims a pool the table can price.
  const masteryMax = $derived(maxAbilityScore(advancement));

  function onOverride(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setSpellLevelsOverride(raw === '' ? null : Number(raw));
  }
</script>

<div class="xp-summary">
  <span class="xp-pool">
    <span class="xp-pool-label">{store.t('spell-levels-pool')}</span>
    <!-- The levels charged to the UNCONDITIONAL side — the base plus whatever the
         years past the Gauntlet bought. (A positive V/F bonus is spent first and
         reported in its own entry; a penalty is charged here.) -->
    <span
      class="xp-pool-used"
      class:over-value={available < 0}
      data-overspent={available < 0}
      data-testid="spell-levels-used">{alloc.baseUsed}</span
    >
    <!-- The figure that used figure is charged against, so the pair CLOSES:
         `alloc.denominator - alloc.baseUsed === alloc.available`, always
         (`spellLevelAllocation` in derive.ts). It is `base + lifeStage`, not the
         base alone — pairing the used figure with the base showed "150 / 120
         Available: 0" for a magus whose spend the engine considers exactly balanced
         (guided-creation-review-2026-08 #18, D2 answer (a)).

         READ-ONLY text, which is why the editable base is no longer here: the two
         are different numbers whenever the magus has lived past its Gauntlet, and a
         field showing 120 where the denominator is 150 would restate the same defect
         as an input. The base has its own entry after Available. -->
    <span class="xp-pool-total">
      <span data-testid="spell-levels-total">{alloc.denominator}</span>
    </span>
  </span>
  <span
    class="xp-available"
    class:over={available < 0}
    data-overspent={available < 0}
    data-testid="spell-levels-available"
  >
    {store.t('spell-levels-available', { available: String(available) })}
  </span>
  <!-- The BASE as an entry of its own, beside the bonus and post-Gauntlet entries it
       sums with. Empty falls back to the type profile's base (the placeholder), so
       the default stays data-driven. Read-only in the wizard (#19) — and rendered as
       plain text rather than a disabled input, so assistive tech does not announce a
       control the player cannot use, exactly as the XP bar's guided total does. -->
  <span class="xp-pool">
    <span class="xp-pool-label">{store.t('spell-levels-base-entry')}</span>
    <span class="xp-pool-total">
      {#if readonlyBase}
        <span data-testid="spell-levels-base">{alloc.base}</span>
      {:else}
        <input
          type="number"
          min="1"
          max="4294967295"
          step="1"
          aria-label={store.t('spell-levels-base-label')}
          placeholder={String(profileBase)}
          value={store.entity.spell_levels_override ?? ''}
          oninput={onOverride}
          data-testid="spell-levels-base"
        />
      {/if}
    </span>
  </span>
  {#if bonus > 0}
    <!-- A positive modifier is an extra pool of levels, spent before the base — so
         it reads used/amount exactly like a restricted XP pool. -->
    <span class="xp-restricted" data-testid="spell-levels-bonus">
      {store.t('spell-levels-bonus-pool', {
        used: String(alloc.bonusUsed),
        amount: String(bonus),
      })}
    </span>
  {:else if bonus < 0}
    <!-- A penalty has no pool to draw from; it is charged to the base above, and
         reported here as the signed modifier that explains the charge. -->
    <span class="xp-restricted over" data-testid="spell-levels-bonus">
      {store.t('spell-levels-bonus', { bonus: formatSigned(bonus) })}
    </span>
  {/if}
  {#if lifeStage > 0}
    <!-- Read-only: levels the years past the Gauntlet already earned, listed so the
         budget is not an unexplained total. Gated on the number alone — no is_magus
         test, no type branch — exactly like the XP bar's life-stage lines, so a
         magus standing at its Gauntlet and every non-magus show nothing. Muted
         (`xp-life-stage`) rather than a pool entry, because there is nothing to
         spend against: these levels are counted in Available above. -->
    <span class="xp-life-stage" data-testid="spell-levels-post-gauntlet">
      {store.t('spell-levels-post-gauntlet', { levels: String(lifeStage) })}
    </span>
  {/if}
  {#if masteryXp > 0 || (masteryFloor > 0 && masteryMax > 0)}
    <span
      class="xp-restricted"
      class:over={masteryUsed > masteryXp}
      data-overspent={masteryUsed > masteryXp}
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
    </span>
  {/if}
</div>
