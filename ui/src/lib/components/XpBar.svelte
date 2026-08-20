<script lang="ts">
  import { store } from '../state.svelte';
  import { formatSigned, generalXpAllocation, restrictedPoolLabel, totalXpSpent } from '../derive';

  // One XP summary shared by the Abilities and Arts tabs: both spend from the
  // SAME `entity.xp_pool`, so this single component drives both, differing only
  // in the data-testid prefix ('' for Abilities, 'art-' for Arts) the e2e suite
  // keys off. Pool source, used source and restricted sub-budgets are identical
  // across the two instances.
  let { prefix = '' }: { prefix?: string } = $props();

  // A life-stage plan IS the guided-funding switch (there is no stored flag), and
  // the engine makes a plan and a typed pool mutually exclusive
  // (`life_stage_xp_pool_conflict`). So under a plan the editable total must go:
  // offering it would offer a value the engine rejects. Both instances read the
  // same `entity.xp_pool`, so the Arts bar takes the identical guided shape —
  // correct, since Arts spend that very pool.
  const guided = $derived(store.entity.life_stages != null);
  // The engine's life-stage budget, or null when the plan yields none yet (no age
  // typed, or a ruleset without life-stage rules).
  const lifeStage = $derived(store.effective?.life_stage ?? null);
  // What the player typed, which under a plan should be nothing at all.
  const typedPool = $derived(store.entity.xp_pool ?? 0);
  // The general pool the `used` figure is charged against: the engine's own resolved
  // pool, in BOTH funding modes. It is the base — whichever block may fund anything
  // (apprenticeship for a magus, later life for anyone else), or the typed total —
  // plus the Skilled/Weak Parens adjustment, which no stored field holds. Reading the
  // typed total here instead was a silently wrong read-out under flat funding: a magus
  // with Skilled Parens legally spends 300 against a typed 240 and reported an
  // overspend the engine never raised. The typed figure keeps the editable field
  // below; the bonus is listed separately, so the two still close.
  const pool = $derived(store.effective?.xp_general_pool ?? typedPool);
  // The signed Virtue/Flaw contribution inside that pool (Skilled Parens +60, Weak
  // Parens -60), so the bar can name it rather than show an unexplained total.
  const bonus = $derived(store.effective?.xp_general_bonus ?? 0);
  const clearHintId = $derived(`${prefix}xp-pool-clear-hint`);
  // The engine's authoritative slice of the spend FUNDED from the general pool.
  // `restricted_xp_pools` cover the rest and are reported separately. The local
  // `totalXpSpent` fallback only covers the first frame before the effective-
  // scores call returns (no restricted grant is assumed then).
  const generalFunded = $derived(
    store.effective?.xp_general_used ??
      (store.ruleset ? totalXpSpent(store.ruleset, store.entity) : 0),
  );
  // Demand the pools cannot fund at all — the engine's overspend, the same figure
  // the `not_enough_xp` issue reports as its shortfall.
  //
  // This is load-bearing: `xp_general_used` is a max-flow value whose source edge
  // is capped by the pool, so it can NEVER exceed the pool and `pool -
  // xp_general_used` can never be negative, however far the spend overshoots. An
  // overspend only shows up as unfunded demand, so it must be added back in to get
  // the true general-pool charge.
  const overspend = $derived(
    Math.max((store.effective?.xp_total_demand ?? 0) - (store.effective?.xp_max_flow ?? 0), 0),
  );
  // What the general pool is actually being asked for. Unfundable demand is by
  // definition general demand (the general pool may fund any spend, so it is
  // exhausted before anything goes unfunded).
  const generalUsed = $derived(generalFunded + overspend);
  // Split the general spend between the base and the V/F modifier, so this bar reads
  // like the spell-levels bar the same Virtue also feeds: a positive bonus is spent
  // first, a penalty is charged to the base, and `base - baseUsed` closes against
  // `pool - used` either way.
  const alloc = $derived(generalXpAllocation(generalUsed, pool, bonus));
  // Not clamped: overspending shows a negative value in bold red (the `over`
  // class covers both the label and the number).
  const available = $derived(alloc.available);
  const restricted = $derived(store.effective?.restricted_xp_pools ?? []);

  // What a year past the Gauntlet is worth. DATA, never a literal: the 30 lives in
  // `rules/core/life_stages.json`, and the lab deduction below is read back out of
  // the engine's own points rather than recomputed from a season count.
  const pointsPerYear = $derived(
    store.ruleset?.ruleset.life_stages?.post_apprenticeship?.points_per_year ?? 0,
  );
  const labDeduction = $derived(
    lifeStage ? lifeStage.post_gauntlet_years * pointsPerYear - lifeStage.post_gauntlet_points : 0,
  );

  function onPool(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setXpPool(raw === '' ? 0 : Number(raw));
  }
</script>

<div class="xp-summary">
  <span class="xp-pool">
    <span class="xp-pool-label">{store.t('xp-pool')}</span>
    <!-- The experience charged to the BASE (a positive V/F bonus is spent first and
         reported in its own entry; a penalty is charged here), so this figure and
         Available always close against the bracketed total beside them. -->
    <span class="xp-pool-used" class:over-value={available < 0} data-testid="{prefix}xp-spent"
      >{alloc.baseUsed}</span
    >
    <span class="xp-pool-total">
      {#if guided}
        <!-- Read-only text, not a disabled input: assistive tech must not announce
             a control the player cannot use. The life-stage block's own base, with
             any V/F contribution listed separately below. -->
        <span data-testid="{prefix}xp-pool-total">{alloc.base}</span>
      {:else}
        <input
          type="number"
          min="0"
          max="4294967295"
          placeholder="0"
          value={typedPool || ''}
          oninput={onPool}
          data-testid="{prefix}xp-pool"
        />
      {/if}
    </span>
  </span>
  <span class="xp-available" class:over={available < 0} data-testid="{prefix}xp-available">
    {store.t('xp-available', { available: String(available) })}
  </span>
  {#if bonus > 0}
    <!-- A positive modifier is an extra pool of experience, spent before the base —
         so it reads used/amount exactly like a restricted pool. Gated on the number
         alone, never on the type, so no character branch enters this component. -->
    <span class="xp-restricted" data-testid="{prefix}xp-bonus">
      {store.t('xp-bonus-pool', {
        used: String(alloc.bonusUsed),
        amount: String(bonus),
      })}
    </span>
  {:else if bonus < 0}
    <!-- A penalty has no pool to draw from; it is charged to the base above, and
         reported here as the signed modifier that explains the charge. -->
    <span class="xp-restricted over" data-testid="{prefix}xp-bonus">
      {store.t('xp-bonus', { bonus: formatSigned(bonus) })}
    </span>
  {/if}
  {#if guided}
    {#if lifeStage}
      {#if lifeStage.apprenticeship_xp > 0}
        <!-- The block behind the pool total, for whoever serves an apprenticeship: its
             fifteen years and their 240 points. Gated on the block, not on the type, so
             the component needs no notion of a magus and the bar of a character who
             serves none is unchanged. -->
        <span class="xp-life-stage" data-testid="{prefix}life-stage-apprenticeship">
          {store.t('life-stage-apprenticeship', {
            years: String(lifeStage.apprenticeship_years),
            xp: String(lifeStage.apprenticeship_xp),
          })}
        </span>
      {/if}
      {#if lifeStage.post_gauntlet_years > 0}
        <!-- Life as a magus after the Gauntlet: 30 points a year, less the charged lab
             seasons, split into experience and levels of spells. Gated on the years the
             block covers, not on the type — the same reason the apprenticeship line
             above needs no notion of a magus. -->
        <span class="xp-life-stage" data-testid="{prefix}life-stage-post-gauntlet">
          {store.t('life-stage-post-gauntlet', {
            years: String(lifeStage.post_gauntlet_years),
            rate: String(pointsPerYear),
            lab: String(labDeduction),
            points: String(lifeStage.post_gauntlet_points),
            xp: String(lifeStage.post_gauntlet_xp),
          })}
        </span>
      {/if}
      <span class="xp-life-stage" data-testid="{prefix}life-stage-later-life">
        {store.t('life-stage-later-life', {
          years: String(lifeStage.later_life_years),
          rate: String(lifeStage.later_life_rate),
          xp: String(lifeStage.later_life_xp),
        })}
      </span>
    {:else}
      <!-- Announced: this row arrives in response to an edit elsewhere (the age),
           so its appearance must reach a screen reader. -->
      <span class="xp-life-stage" role="status" data-testid="{prefix}life-stage-no-budget">
        {store.t('life-stage-no-budget')}
      </span>
    {/if}
  {/if}
  {#each restricted as restrictedPool, i (i)}
    <span
      class="xp-restricted"
      class:over={restrictedPool.used > restrictedPool.amount}
      data-testid="{prefix}restricted-xp-{i}"
    >
      {store.ruleset
        ? store.t('restricted-xp-pool', {
            eligibility: restrictedPoolLabel(store.ruleset, restrictedPool, store.t),
            used: String(restrictedPool.used),
            amount: String(restrictedPool.amount),
          })
        : ''}
    </span>
  {/each}
  {#if guided && typedPool > 0}
    <!-- Escape hatch for a hand-edited save that carries both a plan and a typed
         pool: guided mode shows no field to correct one, so without this the
         engine's conflict error would be inescapable from the UI. -->
    <button
      type="button"
      class="xp-pool-clear"
      aria-describedby={clearHintId}
      onclick={() => store.setXpPool(0)}
      data-testid="{prefix}xp-pool-clear"
    >
      {store.t('xp-pool-clear')}
    </button>
    <span class="xp-pool-clear-hint" id={clearHintId} data-testid="{prefix}xp-pool-clear-hint">
      {store.t('xp-pool-clear-hint')}
    </span>
  {/if}
</div>
