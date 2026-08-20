<script lang="ts">
  import type { Component } from 'svelte';
  import type { CreationPhase } from '../types';
  import { store } from '../state.svelte';
  import { wizardGuidance } from '../derive';

  import AbilityTab from './AbilityTab.svelte';
  import AgingStep from './AgingStep.svelte';
  import ArtGrid from './ArtGrid.svelte';
  import BalanceBar from './BalanceBar.svelte';
  import CharacteristicPicker from './CharacteristicPicker.svelte';
  import HouseSelector from './HouseSelector.svelte';
  import IdentityFields from './IdentityFields.svelte';
  import MythicCompanionTypeSelector from './MythicCompanionTypeSelector.svelte';
  import PersonalityReputationsStep from './PersonalityReputationsStep.svelte';
  import SpellTab from './SpellTab.svelte';
  import TypeStep from './TypeStep.svelte';
  import VirtueFlawTab from './VirtueFlawTab.svelte';
  import WizardReview from './WizardReview.svelte';
  import XpBar from './XpBar.svelte';

  let { phase }: { phase: CreationPhase } = $props();

  interface StepDef {
    /** The input surface for this phase — the very component the editor's tab mounts. */
    component: Component;
    /** A budget bar above it, where App.svelte mounts one for the matching tab. */
    bar?: Component;
    /** Props for the bar (only XpBar's testid/label prefix needs any). */
    barProps?: Record<string, string>;
    /** Long, self-contained panels need the scrolling wrapper or they clip. */
    scroll?: boolean;
  }

  // Every phase has exactly one entry, checked by `satisfies`: a new CreationPhase
  // is a type error here until it has a step, rather than a silently blank one.
  //
  // The wizard reuses the direct-entry components as they are, so its steps and
  // the editor's tabs can never drift apart. SpellTab mounts its own budget bar,
  // which is why `spells` declares none.
  const STEPS = {
    concept: { component: IdentityFields, scroll: true },
    type: { component: TypeStep, scroll: true },
    characteristics: { component: CharacteristicPicker },
    virtues_flaws: { component: VirtueFlawTab, bar: BalanceBar },
    abilities: { component: AbilityTab, bar: XpBar },
    arts: { component: ArtGrid, bar: XpBar, barProps: { prefix: 'art-' } },
    spells: { component: SpellTab },
    house_specialisation: { component: HouseSelector, scroll: true },
    mythic_type: { component: MythicCompanionTypeSelector },
    personality_reputations: { component: PersonalityReputationsStep, scroll: true },
    aging: { component: AgingStep, scroll: true },
    review: { component: WizardReview, scroll: true },
  } satisfies Record<CreationPhase, StepDef>;

  const step = $derived<StepDef>(STEPS[phase]);
  const Body = $derived(step.component);
  const Bar = $derived(step.bar);

  // What this stage of character creation is, in the rules' own terms. Its
  // numbers come from the loaded ruleset, so nothing here restates a rule value.
  const guidance = $derived(
    wizardGuidance(phase, {
      profile: store.ruleset?.ruleset.type_profiles[store.entity.type_id],
      characteristicRules: store.ruleset?.ruleset.characteristic_rules,
    }),
  );
</script>

<!-- Reproduces the editor's height chain verbatim — `.tab-content > .vf-tab
     [> .tab-scroll]` (app.css) — because `.region-row`'s Available/Selected
     columns collapse without it. The step's title lives in the rail, never here:
     each region component already ships its own "Available"/"Selected" headings,
     and a second heading above them would nest. -->
<div class="vf-tab">
  <!-- A note, not a heading: it sits where the step's bar sits, inside the same
       flex column, so it costs the input surface below only its own height. -->
  {#if guidance}
    <p class="hint wizard-guidance" data-testid="wizard-guidance">
      {store.t(guidance.key, guidance.args)}
    </p>
  {/if}
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
