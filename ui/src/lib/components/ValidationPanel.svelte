<script lang="ts">
  import { store } from '../state.svelte';
  import { issuesForStep, phaseSelectedItemIds, resolveIssueArgs } from '../derive';
  import type { CreationPhase } from '../types';

  // `docked` drops the boxed panel chrome so the validation summary can sit at
  // the bottom of the Selected region as a fixed-height, scrollable box.
  //
  // `phase` narrows the panel to one creation phase, which is what makes a
  // wizard step's footer about that step. Omitted — as the editor mounts it — the
  // panel shows every finding for the whole character, and so does the wizard's
  // terminal Review step, which is the only place the findings no phase owns can
  // be seen at all.
  let { docked = false, phase = undefined }: { docked?: boolean; phase?: CreationPhase } = $props();

  // Collapse identical issues (same code, context and args) to a single line:
  // several unfilled instances of the same parameterized virtue each emit an
  // identical `missing_param`, and rendering them under one key would crash the
  // keyed list. Dedup keeps the panel correct and quiet.
  const issues = $derived.by(() => {
    // `sagaIssues` rides on the same list as the engine's own findings. It is not a
    // reading of the entity — `validate` cannot emit it, because the saga year it
    // compares against is app state that never reaches the engine as entity data —
    // but it is a finding about the character in front of the user, carries a phase
    // like any other, and localizes through the same `issue-<code>` catalogue, so
    // showing it anywhere else would just be a second findings panel.
    const all = [...(store.result?.issues ?? []), ...store.sagaIssues];
    // A step also shows the findings filed on ANOTHER phase whose `context` names
    // an item chosen right here (manual-testing-findings #4a/#4b) — otherwise the
    // step where a Great/Poor Characteristic Virtue was just taken said nothing at
    // all about it. Each such entry carries the phase that owns the fix, which the
    // row then names: it reads as an error while Next stays enabled, so without
    // that sentence the gate looks broken.
    const scoped = issuesForStep(all, phase, phaseSelectedItemIds(store.entity.selections, phase));
    const seen = new Set<string>();
    const unique = [];
    for (const entry of scoped) {
      const key = `${entry.issue.code}|${entry.issue.context ?? ''}|${JSON.stringify(entry.issue.args)}`;
      if (seen.has(key)) continue;
      seen.add(key);
      unique.push(entry);
    }
    return unique;
  });
</script>

<section class={docked ? 'validation-docked' : 'panel'}>
  <h2>{store.t('validation-title')}</h2>
  <!-- One box for both branches, so the panel does not JUMP when the last issue
       clears. The empty state and the issue list are different elements of
       different natural heights; giving them a shared parent means app.css can
       floor that parent once (`.validation-body`, min-height = one issue row)
       instead of trying to keep two unrelated boxes in agreement. -->
  <div class="validation-body" data-testid="validation-body">
    {#if issues.length === 0}
      <!-- Announced (role="status" implies aria-live="polite"): clearing the last
           issue is a state change worth telling a screen reader about, matching
           the app's own convention for "figure changes as you type" content
           (AgingRollCalculator.svelte, XpBar.svelte's life-stage-no-budget). -->
      <p class="muted" role="status" data-testid="no-issues">{store.t('no-issues')}</p>
    {:else}
      <!-- role="status" is polite, not assertive: a keystroke-driven revalidation
           must never interrupt the screen reader mid-sentence the way
           role="alert" would. It lives on this wrapping div, never on the <ul>
           itself — role="status" on a <ul> overrides its native list role, so
           assistive tech loses list navigation (item count, "list with N
           items"). The wrapper carries the announcement; the list stays a list. -->
      <div role="status">
        <ul class="issue-list" data-testid="issue-list">
          {#each issues as entry (`${entry.issue.code}|${entry.issue.context ?? ''}|${JSON.stringify(entry.issue.args)}`)}
            {@const issue = entry.issue}
            {@const rawArgs = {
              ...issue.args,
              ...(issue.context ? { context: issue.context } : {}),
            }}
            <li
              class="issue {issue.severity}"
              class:issue-elsewhere={entry.elsewhere !== undefined}
              data-severity={issue.severity}
              data-code={issue.code}
              data-elsewhere={entry.elsewhere}
            >
              <!-- Severity must not be color-only (WCAG 1.4.1). The border/tint carries
                   it for sighted users who can see colour; this visible badge carries
                   it for everyone else too (colourblind sighted users included), and
                   doubles as the label assistive tech announces. -->
              <span class="issue-severity">{store.t(`issue-severity-${issue.severity}`)}: </span>
              {store.t(
                `issue-${issue.code}`,
                store.ruleset ? resolveIssueArgs(store.ruleset, rawArgs, store.t) : rawArgs,
              )}
              <!-- A finding this step did not cause the gate for: it is filed on
                   another phase, so `canAdvance` ignores it and Next stays enabled.
                   Said VISIBLY (not `.sr-only`), because a sighted user faced with an
                   Error whose Next button still works needs the same explanation a
                   screen-reader user gets — and said through the `phase-<slug>` key,
                   never the slug. -->
              {#if entry.elsewhere}
                <span class="issue-elsewhere-note" data-testid="issue-elsewhere-{issue.code}">
                  {store.t('issue-other-step', { step: store.t(`phase-${entry.elsewhere}`) })}
                </span>
              {/if}
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  </div>
</section>
