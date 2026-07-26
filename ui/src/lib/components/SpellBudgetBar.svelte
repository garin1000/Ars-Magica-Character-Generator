<script lang="ts">
  import { store } from '../state.svelte';
  import { formatSigned, maxAbilityScore, spellMasteryXpSpent } from '../derive';

  // The Spells tab's budget status line. Sits ABOVE the Available/Selected lists
  // as a sibling status bar — the same placement (and the same markup, classes and
  // overspend behavior) as the shared XP bar on the Abilities/Arts tabs, so both
  // budgets read identically: label, used figure, bracketed editable total, then
  // Available. It was previously rendered inside the selected list's column
  // header, which put a whole-character budget inside one of the two lists.
  const budget = $derived(store.effective?.spell_levels_budget ?? 0);
  const used = $derived(store.effective?.spell_levels_used ?? 0);
  // The type profile's base budget (120 for a magus) is the base field's
  // placeholder, so the default stays data-driven (never a Svelte literal).
  const profileBase = $derived(store.effective?.spell_levels_profile_base ?? 0);
  // The V/F contribution on its own (Skilled Parens +30, Weak Parens -30). Shown
  // as a separate signed entry beside the editable base — the same shape the XP
  // bar uses for its restricted pools — so the bracketed total stays the number
  // the player actually edits. `base + bonus === budget` (engine-computed).
  const bonus = $derived(store.effective?.spell_levels_bonus ?? 0);
  // Not clamped: overspending shows a negative value in bold red (`over`) with the
  // used figure in plain red (`over-value`) — exactly as the XP pool does.
  const available = $derived(budget - used);

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
    <span class="xp-pool-used" class:over-value={available < 0} data-testid="spell-levels-used"
      >{used}</span
    >
    <!-- The bracketed, editable BASE — the direct counterpart of the XP pool's
         editable total. Empty falls back to the type profile's base (the
         placeholder). Any V/F contribution is listed separately, so this stays the
         one number the player edits. -->
    <span class="xp-pool-total">
      <input
        type="number"
        min="1"
        step="1"
        aria-label={store.t('spell-levels-base-label')}
        placeholder={String(profileBase)}
        value={store.entity.spell_levels_override ?? ''}
        oninput={onOverride}
        data-testid="spell-levels-base"
      />
    </span>
  </span>
  <span class="xp-available" class:over={available < 0} data-testid="spell-levels-available">
    {store.t('spell-levels-available', { available: String(available) })}
  </span>
  {#if bonus !== 0}
    <span class="xp-restricted" data-testid="spell-levels-bonus">
      {store.t('spell-levels-bonus', { bonus: formatSigned(bonus) })}
    </span>
  {/if}
  {#if masteryXp > 0 || (masteryFloor > 0 && masteryMax > 0)}
    <span
      class="xp-restricted"
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
    </span>
  {/if}
</div>
