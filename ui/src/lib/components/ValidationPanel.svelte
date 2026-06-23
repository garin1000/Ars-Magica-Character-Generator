<script lang="ts">
  import { store } from '../state.svelte';

  // `compact` drops the boxed panel chrome so the validation summary can sit in
  // the header, right-bounded above the separator line.
  let { compact = false }: { compact?: boolean } = $props();

  const issues = $derived(store.result?.issues ?? []);
</script>

<section class={compact ? 'validation-compact' : 'panel'}>
  <h2>{store.t('validation-title')}</h2>
  {#if issues.length === 0}
    <p class="muted" data-testid="no-issues">{store.t('no-issues')}</p>
  {:else}
    <ul class="issue-list" data-testid="issue-list">
      {#each issues as issue (issue.code + (issue.context ?? ''))}
        <li class="issue {issue.severity}" data-severity={issue.severity}>
          {store.t(`issue-${issue.code}`, {
            ...issue.args,
            ...(issue.context ? { context: issue.context } : {}),
          })}
        </li>
      {/each}
    </ul>
  {/if}
</section>
