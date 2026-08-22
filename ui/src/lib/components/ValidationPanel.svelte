<script lang="ts">
  import { store } from '../state.svelte';
  import { issuesForPhase, resolveIssueArgs } from '../derive';
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
    const all = store.result?.issues ?? [];
    const scoped = phase ? issuesForPhase(all, phase) : all;
    const seen = new Set<string>();
    const unique = [];
    for (const issue of scoped) {
      const key = `${issue.code}|${issue.context ?? ''}|${JSON.stringify(issue.args)}`;
      if (seen.has(key)) continue;
      seen.add(key);
      unique.push(issue);
    }
    return unique;
  });
</script>

<section class={docked ? 'validation-docked' : 'panel'}>
  <h2>{store.t('validation-title')}</h2>
  {#if issues.length === 0}
    <!-- Announced (role="status" implies aria-live="polite"): clearing the last
         issue is a state change worth telling a screen reader about, matching
         the app's own convention for "figure changes as you type" content
         (AgingRollCalculator.svelte, XpBar.svelte's life-stage-no-budget). -->
    <p class="muted" role="status" data-testid="no-issues">{store.t('no-issues')}</p>
  {:else}
    <!-- role="status" is polite, not assertive: a keystroke-driven revalidation
         must never interrupt the screen reader mid-sentence the way
         role="alert" would. -->
    <ul class="issue-list" role="status" data-testid="issue-list">
      {#each issues as issue (`${issue.code}|${issue.context ?? ''}|${JSON.stringify(issue.args)}`)}
        {@const rawArgs = { ...issue.args, ...(issue.context ? { context: issue.context } : {}) }}
        <li class="issue {issue.severity}" data-severity={issue.severity} data-code={issue.code}>
          <!-- Severity must not be color-only (WCAG 1.4.1). The border/tint carries
               it for sighted users who can see colour; this visible badge carries
               it for everyone else too (colourblind sighted users included), and
               doubles as the label assistive tech announces. -->
          <span class="issue-severity">{store.t(`issue-severity-${issue.severity}`)}: </span>
          {store.t(
            `issue-${issue.code}`,
            store.ruleset ? resolveIssueArgs(store.ruleset, rawArgs, store.t) : rawArgs,
          )}
        </li>
      {/each}
    </ul>
  {/if}
</section>
