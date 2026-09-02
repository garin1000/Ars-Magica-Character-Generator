<script lang="ts">
  import type { RestrictedXpPool } from '../types';
  import { store } from '../state.svelte';
  import { formatSigned, generalXpAllocation, restrictedPoolLabel, U32_MAX } from '../derive';
  import BudgetBonusChip from './BudgetBonusChip.svelte';

  // One XP summary shared by the Abilities and Arts tabs: both spend from the
  // SAME `entity.xp_pool`, so this single component drives both, differing only
  // in the data-testid prefix ('' for Abilities, 'art-' for Arts) the e2e suite
  // keys off. Pool source, used source and restricted sub-budgets are identical
  // across the two instances.
  let { prefix = '' }: { prefix?: string } = $props();

  // Whether the character is FUNDED by its life stages — the stored mode (schema
  // 16), not the presence of a plan. Under life-stage funding the editable total
  // must go: the pools are derived from the stages, so offering a typed figure would
  // offer one the engine never reads. Under pool funding the field belongs on screen
  // even when a plan is on file, because a preserved plan is inert and `xp_pool` is
  // the authority — reading the plan's presence here would hide the very control
  // that funds the character. Both instances read the same `entity.xp_pool`, so the
  // Arts bar takes the identical shape — correct, since Arts spend that very pool.
  const guided = $derived(store.abilityFunding === 'life_stages');
  // The engine's life-stage budget, or null when the plan yields none yet (no age
  // typed, or a ruleset without life-stage rules).
  const lifeStage = $derived(store.effective?.life_stage ?? null);
  // What the player typed. Under life-stage funding it is inert rather than
  // necessarily zero: schema 16 preserves it across a mode switch instead of zeroing
  // it, so the fallback below only ever matters before `store.effective` arrives.
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
  // The engine's authoritative slice of the spend FUNDED from the general pool.
  // `restricted_xp_pools` cover the rest and are reported separately. The `0`
  // fallback covers only the very first frame before `store.effective` arrives
  // (no restricted grant is assumed then) — it is a placeholder, not a
  // computation: pricing here would fork the engine's single evaluation path
  // and ignore Affinity and the restricted/general split, so a real figure is
  // never re-derived in TS (formerly the `totalXpSpent` helper, deleted as
  // dead weight — GF3, tmp/review/review-round-2-gerda-frontend.md — since it
  // always returned this same literal).
  const generalFunded = $derived(store.effective?.xp_general_used ?? 0);
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

  // One restricted pool per life-stage block, looked up BY BLOCK rather than by
  // index: the engine pushes them in a fixed order, but a V/F-granted pool can sit
  // among them, so an index would be a guess. Undefined means the engine formed no
  // such pool for this character — childhood's native-language block before a
  // language is named (`effective/xp.rs`), later life for anyone whose later life IS
  // the general pool.
  const childhoodNative = $derived(lifeStagePool('childhood_native_language'));
  const childhoodSpread = $derived(lifeStagePool('childhood_spread'));
  const laterLife = $derived(lifeStagePool('later_life'));
  // Only pools that are NOT a life-stage block keep a generic row of their own
  // (Educated, Warrior, Privileged Upbringing). Every block now reads as ONE chip
  // merging its derivation with its pool, so leaving the blocks in this list as well
  // would print each block's label twice — which is the whole of #14.
  const itemPools = $derived(restricted.filter((pool) => pool.origin?.kind !== 'life_stage'));

  function lifeStagePool(block: string): RestrictedXpPool | undefined {
    return restricted.find(
      (pool) => pool.origin?.kind === 'life_stage' && pool.origin.block === block,
    );
  }

  // What a year past the Gauntlet is worth. DATA, never a literal: the 30 lives in
  // `rules/core/life_stages.json`, and the lab deduction below is read back out of
  // the engine's own points rather than recomputed from a season count.
  const pointsPerYear = $derived(
    store.ruleset?.ruleset.life_stages?.post_apprenticeship?.points_per_year ?? 0,
  );
  const labDeduction = $derived(
    lifeStage ? lifeStage.post_gauntlet_years * pointsPerYear - lifeStage.post_gauntlet_points : 0,
  );
  // The ages later life spans, which is what says WHICH years these points are from:
  // it starts where childhood ends and runs to the start of apprenticeship. Both
  // terms are the character's own and neither is a literal — childhood's length is
  // ruleset data (`rules/core/life_stages.json`) and `later_life_years` is the
  // engine's own figure, already on `LifeStageBudget`.
  // Source: Ars Magica - Definitive Edition (Core Rules).md:2214, worked through over
  // ages 5 to 10 by the Darius example at `:2402`.
  const childhoodYears = $derived(store.ruleset?.ruleset.life_stages?.childhood?.years ?? 0);
  const laterLifeTo = $derived(childhoodYears + (lifeStage?.later_life_years ?? 0));

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
    <span
      class="xp-pool-used"
      class:over-value={available < 0}
      data-overspent={available < 0}
      data-testid="{prefix}xp-spent">{alloc.baseUsed}</span
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
          max={U32_MAX}
          placeholder="0"
          value={typedPool || ''}
          oninput={onPool}
          data-testid="{prefix}xp-pool"
        />
      {/if}
    </span>
  </span>
  <!-- Announced on crossing into overspend, matching the role="status" pattern
       used one line below (life-stage-no-budget): a screen reader user would
       otherwise have to re-read this bar after every purchase to notice. -->
  <span
    class="xp-available"
    class:over={available < 0}
    data-overspent={available < 0}
    role="status"
    data-testid="{prefix}xp-available"
  >
    {store.t('xp-available', { available: String(available) })}
  </span>
  <!-- A positive modifier is an extra pool of experience, spent before the base —
       so it reads used/amount exactly like a restricted pool. Gated on the number
       alone, never on the type, so no character branch enters this component. A
       penalty has no pool to draw from; it is charged to the base above, and
       reported here as the signed modifier that explains the charge. -->
  <BudgetBonusChip
    {bonus}
    testid="{prefix}xp-bonus"
    positiveText={store.t('xp-bonus-pool', {
      used: String(alloc.bonusUsed),
      amount: String(bonus),
    })}
    negativeText={store.t('xp-bonus', { bonus: formatSigned(bonus) })}
  />
  {#if guided}
    {#if lifeStage}
      <!-- THE LIFE-STAGE BLOCKS, IN THE ORDER THE CHARACTER LIVED THEM (#14): early
           childhood, later life, apprenticeship, the years after it. The rules state
           them as exactly that ordered sequence — "5. Early Childhood … 6. Later
           Life … 7. … Apprenticeship … 8. … Years after apprenticeship"
           (Ars Magica - Definitive Edition (Core Rules).md:2213-2216) — and again as
           a chronology of periods at `:2364`. The previous order put later life LAST,
           where its label read as life past the Gauntlet.

           ONE chip per block: each merges the block's derivation with the spent/total
           of the restricted pool it forms, so no block's name appears twice. Every
           chip is still gated on its own block rather than on the character type, so
           the component needs no notion of a magus. -->
      {#if childhoodNative || childhoodSpread}
        <!-- Childhood's 75 for the native language and 45 for the spread are ONE
             block under one heading (`:2378`), not two rows that read as two blocks.
             The spread-only wording covers the state before a native language is
             named, when the engine has formed no native-language pool to report. -->
        <span class="xp-life-stage" data-testid="{prefix}life-stage-early-childhood">
          {childhoodNative
            ? store.t('xp-pool-block-early-childhood', {
                nativeUsed: String(childhoodNative.used),
                nativeAmount: String(childhoodNative.amount),
                spreadUsed: String(childhoodSpread?.used ?? 0),
                spreadAmount: String(childhoodSpread?.amount ?? 0),
              })
            : store.t('xp-pool-block-early-childhood-spread-only', {
                spreadUsed: String(childhoodSpread?.used ?? 0),
                spreadAmount: String(childhoodSpread?.amount ?? 0),
              })}
        </span>
      {/if}
      <!-- Later life, with the ages it spans. It carries its own spent/total only
           when it IS a restricted pool — a magus, whose general pool is
           apprenticeship instead. For everyone else later life is the general pool
           already shown as this bar's own total, so a second spent/total for the one
           pool would be the duplication #14 exists to remove. -->
      <span class="xp-life-stage" data-testid="{prefix}life-stage-later-life">
        {laterLife
          ? store.t('xp-pool-block-later-life-restricted', {
              from: String(childhoodYears),
              to: String(laterLifeTo),
              years: String(lifeStage.later_life_years),
              rate: String(lifeStage.later_life_rate),
              xp: String(lifeStage.later_life_xp),
              used: String(laterLife.used),
              amount: String(laterLife.amount),
            })
          : store.t('xp-pool-block-later-life', {
              from: String(childhoodYears),
              to: String(laterLifeTo),
              years: String(lifeStage.later_life_years),
              rate: String(lifeStage.later_life_rate),
              xp: String(lifeStage.later_life_xp),
            })}
      </span>
      {#if lifeStage.apprenticeship_xp > 0}
        <!-- The block behind the pool total, for whoever serves an apprenticeship: its
             fifteen years and their 240 points. -->
        <span class="xp-life-stage" data-testid="{prefix}life-stage-apprenticeship">
          {store.t('xp-pool-block-apprenticeship', {
            years: String(lifeStage.apprenticeship_years),
            xp: String(lifeStage.apprenticeship_xp),
          })}
        </span>
      {/if}
      {#if lifeStage.post_gauntlet_years > 0}
        <!-- The years after apprenticeship: 30 points a year, less the charged lab
             seasons, split into experience and levels of spells. -->
        <span class="xp-life-stage" data-testid="{prefix}life-stage-post-gauntlet">
          {store.t('xp-pool-block-after-gauntlet', {
            years: String(lifeStage.post_gauntlet_years),
            rate: String(pointsPerYear),
            lab: String(labDeduction),
            points: String(lifeStage.post_gauntlet_points),
            xp: String(lifeStage.post_gauntlet_xp),
          })}
        </span>
      {/if}
    {:else}
      <!-- Announced: this row arrives in response to an edit elsewhere (the age),
           so its appearance must reach a screen reader. -->
      <span class="xp-life-stage" role="status" data-testid="{prefix}life-stage-no-budget">
        {store.t('life-stage-no-budget')}
      </span>
    {/if}
  {/if}
  {#each itemPools as restrictedPool, i (i)}
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
  <!-- There is deliberately NO "clear the typed pool" control here. One existed, and
       schema 16 (#29) removed its entire reason to exist: it was added because the
       engine reported a plan beside a typed pool as `life_stage_xp_pool_conflict`, an
       error this bar offered no field to correct, so the button was the only escape.
       That finding is retired — the funding mode is now stored explicitly instead of
       inferred from the plan's presence, so the pair is the ordinary shape of a
       character who typed a pool and then switched to the stages, and the pool is
       merely inert rather than illegal. Re-adding the button would put a one-click
       destroyer of the player's typed total next to the slice that exists to stop
       exactly that, and its old hint asserted the retired rule outright ("a pool
       entered by hand has to go back to 0"). The total stays editable on the flat
       side, so nothing is unreachable. -->
</div>
