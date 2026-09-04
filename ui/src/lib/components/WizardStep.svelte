<script lang="ts">
  import type { Component } from 'svelte';
  import type { CreationPhase } from '../types';
  import AbilityTab from './AbilityTab.svelte';
  import AgingStep from './AgingStep.svelte';
  import ArtGrid from './ArtGrid.svelte';
  import BalanceBar from './BalanceBar.svelte';
  import CharacteristicPicker from './CharacteristicPicker.svelte';
  import ConceptStep from './ConceptStep.svelte';
  import ExperienceStep from './ExperienceStep.svelte';
  import HouseSelector from './HouseSelector.svelte';
  import MythicCompanionTypeSelector from './MythicCompanionTypeSelector.svelte';
  import PersonalityReputationsStep from './PersonalityReputationsStep.svelte';
  import SpellBudgetBar from './SpellBudgetBar.svelte';
  import SpellTab from './SpellTab.svelte';
  import VirtueFlawTab from './VirtueFlawTab.svelte';
  import WizardReview from './WizardReview.svelte';
  import XpBar from './XpBar.svelte';

  let { phase }: { phase: CreationPhase } = $props();

  interface StepDef {
    /** The input surface for this phase — the very component the editor's tab mounts. */
    component: Component;
    /** A budget bar above it, where App.svelte mounts one for the matching tab. */
    bar?: Component;
    /**
     * Props for the bar: the per-mount differences this step declares out loud.
     * `XpBar`'s testid/label prefix, and `SpellBudgetBar`'s `readonlyBase` (#19).
     */
    barProps?: Record<string, string | boolean>;
    /** Long, self-contained panels need the scrolling wrapper or they clip. */
    scroll?: boolean;
  }

  // Every phase has exactly one entry, checked by `satisfies`: a new CreationPhase
  // is a type error here until it has a step, rather than a silently blank one.
  //
  // The wizard reuses the direct-entry components THEMSELVES, so its steps and the
  // editor's tabs cannot drift apart as separate implementations. Every budget bar is
  // declared here rather than mounted by the input surface below it, `spells`
  // included: a budget belongs to the whole character, so the step (and, in the
  // editor, the tab) owns it and the picker stays a picker.
  //
  // Where a step must behave differently from the matching tab, the difference is a
  // PROP passed through `barProps`, declared right here in the table — never a
  // `store` lookup inside the component asking which flow is running. That is the
  // difference between one component with a stated parameter and two behaviours
  // hidden inside one file. `spells` is the first such divergence: the spell-levels
  // base is read-only in the wizard because it is a fixed rules grant
  // (guided-creation-review-2026-08 #19), and editable in the editor because direct
  // entry exists to record characters the rules-as-written did not build. So "no
  // drift" now means "no divergence that is not declared in this table", not "no
  // divergence at all".
  //
  // Four steps are compositions of their own (`ConceptStep`, `ExperienceStep`,
  // `AgingStep`, `PersonalityReputationsStep`) rather than a single editor leaf. The
  // editor's tab list now mirrors this phase list (#28), so `App.svelte` mounts the
  // very same compositions — one component, two mounts, nothing to drift. `concept`
  // is the exception and says so out loud: it mounts `IdentityFields` + `AgeFields`
  // in the editor's own order, plus the app-level saga-year setting, which the
  // editor's Details tab deliberately does not carry (#24 leaves it unchanged).
  const STEPS = {
    concept: { component: ConceptStep, scroll: true },
    characteristics: { component: CharacteristicPicker },
    virtues_flaws: { component: VirtueFlawTab, bar: BalanceBar },
    // The bar is the step's own input, not just a read-out: under flat funding the
    // pool total lives in `XpBar`, so without it the step asks where a character's
    // experience comes from while offering no way to answer for one of the two
    // modes. It stays on `abilities` as well — there you spend against the total,
    // here you set it — and #14's life-stage chips belong on this step too.
    experience: { component: ExperienceStep, scroll: true, bar: XpBar },
    abilities: { component: AbilityTab, bar: XpBar },
    arts: { component: ArtGrid, bar: XpBar, barProps: { prefix: 'art-' } },
    spells: { component: SpellTab, bar: SpellBudgetBar, barProps: { readonlyBase: true } },
    house_specialisation: { component: HouseSelector, scroll: true },
    mythic_type: { component: MythicCompanionTypeSelector },
    personality_reputations: { component: PersonalityReputationsStep, scroll: true },
    aging: { component: AgingStep, scroll: true },
    review: { component: WizardReview, scroll: true },
  } satisfies Record<CreationPhase, StepDef>;

  const step = $derived<StepDef>(STEPS[phase]);
  const Body = $derived(step.component);
  const Bar = $derived(step.bar);
</script>

<!-- Reproduces the editor's height chain verbatim — `.tab-content > .vf-tab
     [> .tab-scroll]` (app.css) — because `.region-row`'s Available/Selected
     columns collapse without it. The step's title lives in the rail, never here:
     each region component already ships its own "Available"/"Selected" headings,
     and a second heading above them would nest. -->
<div class="vf-tab">
  <!-- No per-step guidance paragraph (manual-testing-findings #21): the player has
       the rulebook open, and the sentences cost every step a paragraph of height
       above its input surface. Only the bar — the step's own budget, which is
       content and not teaching — sits above the body now. -->
  {#if Bar}
    <Bar {...step.barProps ?? {}} />
  {/if}
  {#if step.scroll}
    <div class="tab-scroll">
      <Body />
    </div>
  {:else}
    <Body />
  {/if}
</div>
