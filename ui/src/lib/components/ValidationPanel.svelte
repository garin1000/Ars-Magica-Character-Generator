<script lang="ts">
  import { store } from '../state.svelte';

  const issues = $derived(store.result?.issues ?? []);
</script>

<section class="panel">
  <h2>{store.t('validation-title')}</h2>
  {#if issues.length === 0}
    <p class="muted" data-testid="no-issues">{store.t('no-issues')}</p>
  {:else}
    <ul class="issue-list" data-testid="issue-list">
      {#each issues as issue (issue.code + (issue.context ?? ''))}
        <li class="issue {issue.severity}" data-severity={issue.severity}>
          {store.t(`issue-${issue.code}`, issue.context ? { context: issue.context } : undefined)}
        </li>
      {/each}
    </ul>
  {/if}
</section>
