<script lang="ts">
  import { store } from '../state.svelte';
  import { resolveIssueArgs } from '../derive';

  // `docked` drops the boxed panel chrome so the validation summary can sit at
  // the bottom of the Selected region as a fixed-height, scrollable box.
  let { docked = false }: { docked?: boolean } = $props();

  // Collapse identical issues (same code, context and args) to a single line:
  // several unfilled instances of the same parameterized virtue each emit an
  // identical `missing_param`, and rendering them under one key would crash the
  // keyed list. Dedup keeps the panel correct and quiet.
  const issues = $derived.by(() => {
    const seen = new Set<string>();
    const unique = [];
    for (const issue of store.result?.issues ?? []) {
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
    <p class="muted" data-testid="no-issues">{store.t('no-issues')}</p>
  {:else}
    <ul class="issue-list" data-testid="issue-list">
      {#each issues as issue (`${issue.code}|${issue.context ?? ''}|${JSON.stringify(issue.args)}`)}
        {@const rawArgs = { ...issue.args, ...(issue.context ? { context: issue.context } : {}) }}
        <li class="issue {issue.severity}" data-severity={issue.severity} data-code={issue.code}>
          {store.t(
            `issue-${issue.code}`,
            store.ruleset ? resolveIssueArgs(store.ruleset, rawArgs, store.t) : rawArgs,
          )}
        </li>
      {/each}
    </ul>
  {/if}
</section>
