<script lang="ts">
  // Test fixture for the client test environment, not part of the app.
  // `client-env.client.test.ts` mounts this to prove `$effect` actually runs
  // under `happy-dom`; nothing in the product imports it.
  let { onEffect }: { onEffect: (count: number) => void } = $props();

  let count = $state(0);

  $effect(() => {
    onEffect(count);
  });
</script>

<span data-testid="effect-probe">{count}</span>
