<script lang="ts">
  import { store } from '../state.svelte';
  import {
    childhoodEntryPreview,
    childhoodSlotFault,
    childhoodSlots,
    displayName,
    localizedSortKey,
    paramHint,
    resolveIssueArgs,
    type ChildhoodSlotFault,
  } from '../derive';

  // Sample Childhoods: ready-made Ability packages a player may take instead of
  // dividing early childhood's spread by hand.
  //
  // "The following Ability packages can be taken to speed up character generation.
  // Each represents a particular sort of childhood. Note that you can spend the 45
  // experience points for yourself, as well."
  // Source: Ars Magica - Definitive Edition (Core Rules).md:2380-2388

  // A ruleset shipping no packages has no choice to offer, so the whole picker is
  // absent rather than an empty dropdown.
  const packages = $derived.by(() => {
    const localized = store.ruleset;
    if (!localized) return [];
    return Object.values(localized.ruleset.childhoods ?? {}).sort((a, b) =>
      localizedSortKey(localized, a.id).localeCompare(localizedSortKey(localized, b.id)),
    );
  });

  /** A package's localized rules name — never its id. */
  function packageName(id: string): string {
    const localized = store.ruleset;
    return localized ? displayName(localized, id, undefined, paramHint(store.t)) : id;
  }

  // The RECORD: the package this character actually took, which lives on the plan.
  // History, not a form — so it is read-only text and it never pre-fills the select
  // below. Slot answers are UI-only draft state and are gone after a reload, so a
  // recorded package restored as a draft would show spurious empty-slot faults.
  const taken = $derived(store.entity.life_stages?.childhood_package ?? null);

  // The DRAFT: what the player is considering right now (store, UI-only).
  const draft = $derived.by(() => {
    const id = store.childhoodDraft.packageId;
    if (!id) return null;
    return store.ruleset?.ruleset.childhoods?.[id] ?? null;
  });

  const slots = $derived(
    draft && store.ruleset ? childhoodSlots(store.ruleset, draft, store.t) : [],
  );

  const preview = $derived(
    draft && store.ruleset
      ? childhoodEntryPreview(
          store.ruleset,
          draft,
          store.entity.life_stages,
          store.childhoodDraft.slots,
          store.t,
        )
      : [],
  );

  /** Each slot's locally decidable fault, keyed by slot, so nothing is computed twice. */
  const faults = $derived.by(() => {
    const found = new Map<string, ChildhoodSlotFault | null>();
    if (!draft || !store.ruleset) return found;
    for (const slot of slots) {
      found.set(
        slot.slot,
        childhoodSlotFault(
          store.ruleset,
          draft,
          slot.slot,
          store.childhoodDraft.slots,
          store.entity.life_stages,
        ),
      );
    }
    return found;
  });

  // Apply is refused while any answer is faulty, and the FIRST fault in slot order
  // is the reason given — announced through `aria-describedby`, never a grey button
  // with no explanation.
  const blockingFault = $derived(
    slots.map((slot) => faults.get(slot.slot)).find((fault) => !!fault) ?? null,
  );

  const APPLY_REASON_ID = 'childhood-apply-reason';

  function reasonId(slot: string): string {
    return `childhood-slot-${slot}-reason`;
  }

  function faultReason(fault: ChildhoodSlotFault): string {
    return store.t(`childhood-slot-${fault}-reason`);
  }

  function onPackage(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value;
    store.setChildhoodDraftPackage(value === '' ? null : value);
  }

  function onSlot(slot: string, event: Event) {
    store.setChildhoodDraftSlot(slot, (event.currentTarget as HTMLInputElement).value);
  }
</script>

{#if packages.length > 0}
  <div class="childhood-picker">
    {#if taken}
      <span class="childhood-taken" data-testid="childhood-taken">
        {store.t('childhood-taken', { name: packageName(taken) })}
      </span>
    {/if}

    <label class="field inline">
      <span>{store.t('childhood-label')}</span>
      <select
        value={store.childhoodDraft.packageId ?? ''}
        onchange={onPackage}
        data-testid="childhood-package-select"
      >
        <!-- Dividing the experience yourself is a first-class choice, not an
             opt-out — the rulebook offers it in the same breath as the packages. -->
        <option value="">{store.t('childhood-choose-prompt')}</option>
        {#each packages as pkg (pkg.id)}
          <option value={pkg.id}>{packageName(pkg.id)}</option>
        {/each}
      </select>
    </label>

    {#if draft}
      <p class="childhood-preview-label">{store.t('childhood-preview-label')}</p>
      <ul class="childhood-preview" data-testid="childhood-package-preview">
        {#each preview as row, i (i)}
          <li>{row}</li>
        {/each}
      </ul>

      {#if slots.length > 0}
        <div class="childhood-slots">
          {#each slots as slot (slot.slot)}
            {@const fault = faults.get(slot.slot) ?? null}
            <div class="childhood-slot">
              <label class="field inline">
                <!-- The label is the Ability, ordinal-disambiguated where one
                     Ability holds two slots; the slot key is a machine key and is
                     never shown. -->
                <span>{slot.label}</span>
                <input
                  type="text"
                  value={store.childhoodDraft.slots[slot.slot] ?? ''}
                  aria-invalid={fault ? 'true' : undefined}
                  aria-describedby={fault ? reasonId(slot.slot) : undefined}
                  oninput={(event) => onSlot(slot.slot, event)}
                  data-testid="childhood-slot-{slot.slot}"
                />
              </label>
              {#if fault}
                <span
                  class="childhood-reason"
                  id={reasonId(slot.slot)}
                  data-testid={reasonId(slot.slot)}
                >
                  {faultReason(fault)}
                </span>
              {/if}
            </div>
          {/each}
        </div>
      {/if}

      <div class="childhood-apply">
        <button
          type="button"
          disabled={!!blockingFault}
          aria-describedby={blockingFault ? APPLY_REASON_ID : undefined}
          onclick={() => store.applyChildhoodPackage()}
          data-testid="childhood-apply"
        >
          {store.t('childhood-apply')}
        </button>
        {#if blockingFault}
          <span class="childhood-reason" id={APPLY_REASON_ID} data-testid={APPLY_REASON_ID}>
            {faultReason(blockingFault)}
          </span>
        {/if}
      </div>
    {/if}

    {#if store.childhoodRejections.length > 0}
      <!-- Once Apply has been pressed the engine is the authority, and its findings
           localize through the same `issue-<code>` + resolved-args path
           ValidationPanel uses, so one wording serves both surfaces. -->
      <ul class="issue-list" data-testid="childhood-rejections">
        {#each store.childhoodRejections as issue, i (i)}
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
  </div>
{/if}
