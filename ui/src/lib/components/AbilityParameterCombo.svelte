<script lang="ts">
  import type { Snippet } from 'svelte';
  import { store } from '../state.svelte';
  import { displayName } from '../derive';
  import type { AbilityParamValue, LinkTarget } from '../types';

  // The engine-built parameter combo box (CV7, design § 6.1/§ 6.3; shared since
  // N4b): catalogue values first, then the owner's link targets, then the free-text
  // "Other…" escape — one stable order, and the component derives none of it. The
  // box and the field "Other…" opens share one wrapper, which app.css lays out as a
  // single line (try-out finding 5). Used by an Ability row, the life-stage native
  // language and a Sample Childhood slot of a catalogued Ability.
  //
  // Purely controlled: it shows `value` and reports what the player does through
  // `onchoose`/`ontext`, and never writes on its own — so mounting it cannot move
  // the document's dirty baseline.

  interface Props {
    /** Catalogue value ids, in catalogue order (`AbilityParameterOptions.catalogued`). */
    catalogued: string[];
    /** Link targets — an Ability row's own; omitted wherever no link can apply. */
    linked?: LinkTarget[];
    /** A link target's option label. Required wherever `linked` is given. */
    linkLabel?: (link: LinkTarget) => string;
    /** The stored value; absent means nothing chosen yet. */
    value: AbilityParamValue | null | undefined;
    /** The accessible name of the select and of the "Other…" field. */
    label: string;
    /** The "Other…" field's placeholder. */
    placeholder: string;
    invalid?: boolean;
    /** The id of the element saying why the value is invalid, if any. */
    describedby?: string;
    selectTestId: string;
    inputTestId: string;
    /** A list entry was chosen; `undefined` is "Other…", which clears the value. */
    onchoose: (value: AbilityParamValue | undefined) => void;
    /** Text was typed into the "Other…" field. */
    ontext: (text: string) => void;
    /** Rendered inside the wrapper after the field (an Ability row's link indicator). */
    children?: Snippet;
  }

  let {
    catalogued,
    linked = [],
    linkLabel,
    value,
    label,
    placeholder,
    invalid = false,
    describedby,
    selectTestId,
    inputTestId,
    onchoose,
    ontext,
    children,
  }: Props = $props();

  // Separator joining a link target's item id and param key into one option
  // value — mirrors `ParameterPicker.svelte`'s own `SEP` (a NUL never appears
  // in an id or a player-typed guild/craft name).
  const SEP = String.fromCharCode(0);

  /** The selected `<option>` for the stored shape — only ever a form this box writes. */
  const selected = $derived.by(() => {
    if (value == null) return 'other';
    if ('id' in value) return `cat:${value.id}`;
    if ('item' in value) return `link:${value.item}${SEP}${value.param}`;
    return 'other';
  });

  /** "Other…" is selected: no value, or typed text — the field is open. */
  const isOther = $derived(value == null || 'text' in value);
  const text = $derived(value != null && 'text' in value ? value.text : '');

  /** Choosing an entry REPLACES what was stored — there is no "keep both" (§ 6.3). */
  function onSelect(event: Event): void {
    const raw = (event.currentTarget as HTMLSelectElement).value;
    if (raw.startsWith('cat:')) {
      onchoose({ id: raw.slice('cat:'.length) });
      return;
    }
    if (raw.startsWith('link:')) {
      const [item, param] = raw.slice('link:'.length).split(SEP);
      onchoose({ item, param });
      return;
    }
    onchoose(undefined);
  }

  function optionName(id: string): string {
    return store.ruleset ? displayName(store.ruleset, id) : id;
  }
</script>

<div class="ability-param-combo">
  <select
    class="ability-param-select"
    aria-label={label}
    aria-invalid={invalid ? 'true' : undefined}
    aria-describedby={describedby}
    value={selected}
    onchange={onSelect}
    data-testid={selectTestId}
  >
    {#each catalogued as id (id)}
      <option value="cat:{id}">{optionName(id)}</option>
    {/each}
    {#each linked as link (link.item + SEP + link.param)}
      <option value="link:{link.item}{SEP}{link.param}">{linkLabel?.(link) ?? ''}</option>
    {/each}
    <option value="other">{store.t('ability-param-other')}</option>
  </select>
  {#if isOther}
    <input
      type="text"
      class="ability-param"
      aria-label={label}
      {placeholder}
      aria-invalid={invalid ? 'true' : undefined}
      aria-describedby={describedby}
      value={text}
      oninput={(e) => ontext((e.currentTarget as HTMLInputElement).value)}
      data-testid={inputTestId}
    />
  {/if}
  {@render children?.()}
</div>
