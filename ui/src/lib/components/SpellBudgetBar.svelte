<script lang="ts">
  import { store } from '../state.svelte';
  import { maxAbilityScore, spellMasteryXpSpent } from '../derive';

  // The Spells tab's budget status line. Sits ABOVE the Available/Selected lists
  // as a sibling status bar — the same placement (and the same markup, classes and
  // overspend behavior) as the shared XP bar on the Abilities/Arts tabs, so both
  // budgets read identically: label, used figure, bracketed editable total, then
  // Available. It was previously rendered inside the selected list's column
  // header, which put a whole-character budget inside one of the two lists.
  const budget = $derived(store.effective?.spell_levels_budget ?? 0);
  const used = $derived(store.effective?.spell_levels_used ?? 0);
  // The type profile's base budget (120 for a magus) is the override field's
  // placeholder, so the default stays data-driven (never a Svelte literal).
  const profileBase = $derived(store.effective?.spell_levels_profile_base ?? 0);
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
    <!-- Read-only, unlike the XP pool's editable total: the effective budget is
         the base (profile default or the override below) PLUS virtue bonuses
         (Skilled Parens +30), so the engine owns the number. The editable part is
         the base override, which gets its own labeled field. -->
    <span class="xp-pool-total" data-testid="spell-levels-budget">{budget}</span>
  </span>
  <span class="xp-available" class:over={available < 0} data-testid="spell-levels-available">
    {store.t('spell-levels-available', { available: String(available) })}
  </span>
  <label class="field">
    <span>{store.t('spell-levels-override-label')}</span>
    <input
      type="number"
      min="1"
      step="1"
      placeholder={String(profileBase)}
      value={store.entity.spell_levels_override ?? ''}
      oninput={onOverride}
      data-testid="spell-levels-override"
    />
  </label>
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
