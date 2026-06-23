<script lang="ts">
  import { store } from '../state.svelte';

  // `docked` drops the boxed panel chrome so the validation summary can sit at
  // the bottom of the Selected region as a fixed-height, scrollable box.
  let { docked = false }: { docked?: boolean } = $props();

  const issues = $derived(store.result?.issues ?? []);
</script>

<section class={docked ? 'validation-docked' : 'panel'}>
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
