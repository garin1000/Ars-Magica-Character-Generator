<script lang="ts">
  import { store } from '../state.svelte';
  import { formatSigned } from '../derive';

  // The die-independent half of the character's aging, as the engine computes it.
  // Null when the ruleset ships no aging table at all — the subsystem is then stood
  // down and the panel renders nothing rather than an empty read-out.
  const aging = $derived(store.effective?.aging ?? null);

  // The owed years, first and last. Read off the engine's schedule; the panel never
  // derives the threshold or counts the set itself.
  const first = $derived(aging?.schedule[0] ?? null);
  const last = $derived(aging?.schedule[aging.schedule.length - 1] ?? null);
  // Calendar years exist only for a character with a birth year.
  const showYears = $derived(first?.year != null && last?.year != null);

  // AGING TOTAL: stress die (no botch) + age/10 (round up) - Living Conditions
  // modifier - Longevity Ritual modifier.
  // Source: Ars Magica - Definitive Edition (Core Rules).md:16567-16569
  //
  // Both modifiers are SUBTRACTED, so a stored +7 ritual bonus lowers the total by
  // 7 — the formula shows each term as it acts on the total, which is what the
  // player adds the die to. `fixed_total` is the engine's own sum of all of them,
  // so nothing is re-derived here.
  //
  // FOUR terms, not the book's three. The Virtue/Flaw aging-roll modifiers are a
  // separate quantity, ADDED with their stored sign — "You are resistant to aging,
  // and get -1 to all aging rolls" (Faerie Blood).
  // Source: Ars Magica - Definitive Edition (Core Rules).md:3801
  // — and `fixed_total` has always counted them. Naming only the book's three left
  // a read-out that contradicted itself for any character holding such a Virtue or
  // Flaw ("+4 (age) 0 (living conditions) 0 (Longevity Ritual) = stress die +3"),
  // so the term is named too, shown as 0 like the others when it is zero. The
  // sibling `aging-total-parts` in AgingRollCalculator has always carried it.
  const formula = $derived(
    aging
      ? {
          age: formatSigned(aging.age_modifier),
          conditions: formatSigned(-aging.living_conditions_modifier),
          longevity: formatSigned(-aging.longevity_modifier),
          traits: formatSigned(aging.trait_modifier),
          fixed: formatSigned(aging.fixed_total),
        }
      : null,
  );
</script>

{#if aging && formula}
  <!-- Announced: every figure here changes the moment an age is typed or a
       condition picked, so the change must reach a screen reader. -->
  <div class="detail-section aging-schedule" role="status" data-testid="aging-schedule">
    <h3 class="detail-label">{store.t('aging-schedule-label')}</h3>
    <p data-testid="aging-first-roll-age">
      {store.t('aging-first-roll-age', {
        begins: String(aging.begins_after_age),
        first: String(aging.first_roll_age),
      })}
    </p>
    {#if aging.rolls_owed > 0 && first && last}
      <p data-testid="aging-rolls-owed">
        {store.t('aging-rolls-owed', {
          count: aging.rolls_owed,
          from: String(first.age),
          to: String(last.age),
        })}
      </p>
      {#if showYears}
        <p data-testid="aging-rolls-years">
          {store.t('aging-rolls-years', {
            count: aging.rolls_owed,
            from: String(first.year),
            to: String(last.year),
          })}
        </p>
      {/if}
      <p data-testid="aging-rolls-recorded">
        {store.t('aging-rolls-recorded', {
          recorded: String(aging.rolls_recorded),
          owed: String(aging.rolls_owed),
        })}
      </p>
    {:else}
      <p data-testid="aging-rolls-none">{store.t('aging-rolls-none')}</p>
    {/if}
    <p class="aging-formula" data-testid="aging-total-formula">
      {store.t('aging-total-formula', formula)}
    </p>
  </div>
{/if}
