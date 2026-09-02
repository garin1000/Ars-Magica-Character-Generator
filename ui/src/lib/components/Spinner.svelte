<script lang="ts">
  import type { Snippet } from 'svelte';

  // The increment/decrement control hand-reimplemented independently across
  // AbilityTab, ArtGrid, CharacteristicPicker, FamiliarPanel, PersonalityTraits
  // and SpellTab (V29, full-audit round). One shared primitive; every host still
  // supplies its own translated aria-labels, disabled predicates and testids —
  // this component owns only the repeated `-`/`+` button pair and wrapper.
  //
  // `testid`/`label`/`after` are optional because only one host (the Spell
  // Mastery spinner) nests a label inside the wrapper and follows the buttons
  // with an effective-score badge; every other host places those as SIBLINGS
  // outside the spinner instead, so leaving the props unset there reproduces
  // the exact prior markup rather than adding a wrapper nothing used before.
  let {
    testid,
    label,
    decLabel,
    decTestid,
    decDisabled = false,
    onDec,
    incLabel,
    incTestid,
    incDisabled = false,
    onInc,
    after,
    children,
  }: {
    testid?: string;
    label?: Snippet;
    decLabel: string;
    decTestid: string;
    decDisabled?: boolean;
    onDec: () => void;
    incLabel: string;
    incTestid: string;
    incDisabled?: boolean;
    onInc: () => void;
    after?: Snippet;
    children: Snippet;
  } = $props();
</script>

<span class="spinner" data-testid={testid}>
  {#if label}{@render label()}{/if}
  <button
    type="button"
    class="icon-btn"
    aria-label={decLabel}
    disabled={decDisabled}
    onclick={onDec}
    data-testid={decTestid}
  >
    -
  </button>
  {@render children()}
  <button
    type="button"
    class="icon-btn"
    aria-label={incLabel}
    disabled={incDisabled}
    onclick={onInc}
    data-testid={incTestid}
  >
    +
  </button>
  {#if after}{@render after()}{/if}
</span>
