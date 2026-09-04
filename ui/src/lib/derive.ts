// Pure helpers deriving display data from the loaded ruleset + entity. Kept out
// of components so they can be unit-tested and reused.

import type {
  Ability,
  AbilityBonus,
  AbilityCategory,
  AbilityFloor,
  AbilityScore,
  Addend,
  Art,
  ArtType,
  ChildhoodEntry,
  ChildhoodPackage,
  CreationPhase,
  EntityTypeProfile,
  EquipmentSlot,
  Grant,
  GrantConstraint,
  ItemKind,
  LifeStagePlan,
  LocalizedRuleset,
  Magnitude,
  PointItem,
  Prereq,
  RestrictedXpPool,
  Selection,
  Spell,
  SpellSelection,
  ValidationIssue,
  ValidationMode,
  ValidationResult,
} from './types';

/**
 * Case- and diacritic-insensitive search normalization, so "Übernatürlich"
 * matches "ubernaturlich" and "Größe" matches "grosse"-ish. Strips combining
 * marks (NFD) and lowercases; the search text and haystack pass through the same
 * normalizer, so a filter is language-agnostic.
 */
function normalizeSearch(s: string): string {
  return s.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase().trim();
}

/**
 * Formats a number with an explicit sign for display: `+` for positive, an ASCII
 * hyphen-minus `-` (U+002D) for negative, and a plain unsigned value for zero.
 * Single source of truth so signed modifiers render one consistent sign. Uses
 * the hyphen-minus (not the mathematical minus U+2212) so values stay ASCII —
 * copy-paste clean and read correctly by assistive tech.
 */
export function formatSigned(n: number): string {
  return n > 0 ? `+${n}` : `${n}`;
}

/**
 * The "no constraint" sentinel bounds for an i32/u32 rules field with no
 * ruleset-supplied range yet (a payload predating the field, or the moment
 * before `store.ruleset` has loaded). Single source of truth so every field's
 * `min`/`max` fallback — and every `<input>` sharing its Rust field width —
 * states the same bound instead of retyping the literal at each call site
 * (G19, full-audit round).
 */
export const I32_MIN = -2147483648;
export const I32_MAX = 2147483647;
export const U32_MAX = 4294967295;

/** A `store.t`-shaped translator, threaded in so search can index rendered labels. */
export type Translate = (key: string, args?: Record<string, string>) => string;

/** The standard "(Label)" placeholder hint for an unfilled `{param}` token. */
export function paramHint(t: Translate): (key: string) => string {
  return (key) => t('param-hint', { label: t(`param-label-${key}`) });
}

/** A labelled addend's display name (stable slug → Fluent `derived-addend-<label>`). */
function addendLabel(a: Addend, t: Translate): string {
  return t(`derived-addend-${a.label}`);
}

/**
 * The hover-tooltip breakdown text for a list of signed addends — e.g. a Lab
 * Total's "Technique +5, Form +3, Intelligence +2". Shared by every Derived
 * Totals section that carries one (Lab/Casting, Magic Resistance, Soak) rather
 * than reimplemented per section (V26, full-audit round).
 */
export function addendBreakdown(addends: Addend[], t: Translate): string {
  return addends
    .map((a) => `${addendLabel(a, t)} ${formatSigned(a.value)}`)
    .join(`${t('derived-addend-list-separator')} `);
}

/**
 * The searchable text of an item id, normalized. Folds three things so a filter
 * matches what the user actually sees: the raw template with braces stripped (so
 * an unresolved token like `language` still matches), the fully rendered label
 * with its param hint resolved (so the visible "(Language)"/"(Sprache)" is
 * indexed in the active language), and the summary. Without a translator only
 * the template + summary are indexed (the pre-render fallback).
 */
function searchHaystack(localized: LocalizedRuleset, id: string, t?: Translate): string {
  const entry = localized.i18n[id];
  const template = (entry?.name ?? id).replace(/[{}]/g, '');
  const rendered = t ? displayName(localized, id, undefined, paramHint(t)) : template;
  return normalizeSearch(`${template} ${rendered} ${entry?.summary ?? ''}`);
}

/** Facets a Virtue/Flaw list can be filtered by. All optional; omitted = no constraint. */
export interface ItemFilter {
  text?: string;
  categories?: string[];
  magnitudes?: Magnitude[];
  tainted?: boolean;
}

/**
 * Virtues/Flaws matching every supplied facet (facets AND together): free-text
 * over the localized name/summary, category, magnitude, and the Tainted tag.
 * Order is preserved, so callers group/sort afterward as before.
 */
export function filterItems(
  localized: LocalizedRuleset,
  items: PointItem[],
  filter: ItemFilter,
  t?: Translate,
): PointItem[] {
  const text = filter.text ? normalizeSearch(filter.text) : '';
  return items.filter((it) => {
    if (text && !searchHaystack(localized, it.id, t).includes(text)) return false;
    // Membership, not display: an item whose descriptor names two categories
    // matches on either (virtue.sufi is "Social Status, Supernatural"), exactly
    // as the engine's `PointItem::has_category` reads it.
    if (filter.categories?.length && !filter.categories.some((c) => it.categories.includes(c)))
      return false;
    if (filter.magnitudes?.length && !filter.magnitudes.includes(it.magnitude)) return false;
    if (filter.tainted && !it.tainted) return false;
    return true;
  });
}

/** Facets an Ability list can be filtered by. */
export interface AbilityFilter {
  text?: string;
  categories?: AbilityCategory[];
}

/** Abilities matching the text (localized name) and category facets. */
export function filterAbilities(
  localized: LocalizedRuleset,
  abilities: Ability[],
  filter: AbilityFilter,
  t?: Translate,
): Ability[] {
  const text = filter.text ? normalizeSearch(filter.text) : '';
  return abilities.filter((a) => {
    if (text && !searchHaystack(localized, a.id, t).includes(text)) return false;
    if (filter.categories?.length && !filter.categories.includes(a.category)) return false;
    return true;
  });
}

/** Facets a Spell list can be filtered by (Technique/Form separate and combined). */
export interface SpellFilter {
  text?: string;
  technique?: string;
  form?: string;
  /** Inclusive lower bound on a fixed spell's level (null/undefined = open). */
  levelMin?: number | null;
  /** Inclusive upper bound on a fixed spell's level (null/undefined = open). */
  levelMax?: number | null;
}

/** A finite numeric bound, or `undefined` when the input is empty/NaN (open). */
function finiteBound(value: number | null | undefined): number | undefined {
  return value != null && Number.isFinite(value) ? value : undefined;
}

/**
 * Spells matching the text (localized name), Technique and Form (each optional,
 * so they filter separately or combined), and an inclusive level range. Each
 * range bound is optional; a null/empty/NaN bound is open (an emptied number
 * input binds to `null`, which must widen the range rather than match level 0).
 *
 * A **General** spell (catalogue level `null`) has no fixed level to test — its
 * learned level is chosen per character — so it is **always shown** regardless of
 * the bounds. Only a fixed-level spell is range-tested. This mirrors the
 * grouping decision (`groupSpellsByTechniqueForm` buckets General spells as a
 * trailing entry within their Te/Fo group).
 */
export function filterSpells(
  localized: LocalizedRuleset,
  spells: Spell[],
  filter: SpellFilter,
  t?: Translate,
): Spell[] {
  const text = filter.text ? normalizeSearch(filter.text) : '';
  const min = finiteBound(filter.levelMin);
  const max = finiteBound(filter.levelMax);
  return spells.filter((s) => {
    // A parametrized spell's name carries a `{param}` placeholder (e.g.
    // "Wizard's Boost ({form})"); a source candidate has no chosen value, so the
    // hint-aware path renders the localized "(Form)" hint into the searchable
    // label. Plain spells have no token and render as their bare name.
    const name = t
      ? displayName(localized, s.id, undefined, paramHint(t))
      : spellName(localized, s.id);
    if (text && !normalizeSearch(name).includes(text)) return false;
    if (filter.technique && s.technique !== filter.technique) return false;
    if (filter.form && s.form !== filter.form) return false;
    // General spells (level null) always pass the range; only fixed levels test.
    if (s.level != null) {
      if (min !== undefined && s.level < min) return false;
      if (max !== undefined && s.level > max) return false;
    }
    return true;
  });
}

/** The three equipment catalogue kinds, in display order. */
export type EquipmentKind = 'weapons' | 'shields' | 'armor';

const EQUIPMENT_KINDS: EquipmentKind[] = ['weapons', 'shields', 'armor'];

/** Facets an equipment catalogue can be filtered by. */
export interface EquipmentFilter {
  text?: string;
  kind?: EquipmentKind;
}

/** A filtered, name-sorted group of equipment ids of one kind. */
export interface EquipmentGroup {
  kind: EquipmentKind;
  ids: string[];
}

/**
 * Equipment is stored as three separate catalogue maps (weapons/shields/armor)
 * with no shared `kind` field, so this returns per-kind groups (in display
 * order, empty groups dropped) rather than a flat list. Each group's ids are
 * free-text filtered over the localized name and sorted by it. Equipment ids
 * carry no `{param}` placeholder, so `searchHaystack` matches the plain name.
 */
export function filterEquipment(
  localized: LocalizedRuleset,
  catalogue: Record<EquipmentKind, Record<string, { id: string }>>,
  filter: EquipmentFilter,
  t?: Translate,
): EquipmentGroup[] {
  const text = filter.text ? normalizeSearch(filter.text) : '';
  const nameOf = (id: string) => localized.i18n[id]?.name ?? id;
  return EQUIPMENT_KINDS.filter((kind) => !filter.kind || filter.kind === kind)
    .map((kind) => ({
      kind,
      ids: Object.values(catalogue[kind] ?? {})
        .map((it) => it.id)
        .filter((id) => !text || searchHaystack(localized, id, t).includes(text))
        .sort((a, b) => nameOf(a).localeCompare(nameOf(b))),
    }))
    .filter((group) => group.ids.length > 0);
}

/**
 * Rules display name for an item, substituting any `{param}` placeholders.
 *
 * A placeholder with no value (e.g. an unfilled parameter in the picker) falls
 * back to `placeholderLabel(key)` when given — used to show a localized hint
 * like "(Ability)" instead of the raw `{ability}` token — or to `{key}` if not.
 *
 * A present value is a raw ref slug (e.g. `characteristic.per`); pass
 * `resolveValue` to turn it into a display label ("Perception") so the result
 * reads "Great Perception" rather than "Great characteristic.per".
 *
 * An entry may **opt out** of the hint for the wholly-unfilled case by declaring
 * `name_unfilled` (see `I18nEntry`). That is for the one template shape whose hint
 * doubles: a `{token}` next to a parenthetical literal, where "{language} (Dead
 * Language)" plus the "(Language)" hint reads "(Language) (Dead Language)". It is
 * deliberately opt-in per entry and NOT blanket suppression of the hint — the
 * ~36 templates whose token is the head of the name or sits mid-phrase would
 * degrade to "Puissant", "Affinity with" or "Ways Of The" (and to dangling
 * inflected adjectives in German), so they keep the hint. See the guard test
 * `an unfilled template without name_unfilled still renders the param hint`.
 */
export function displayName(
  localized: LocalizedRuleset,
  ref: string,
  params?: Record<string, string>,
  placeholderLabel?: (key: string) => string,
  resolveValue?: (key: string, value: string) => string,
): string {
  const entry = localized.i18n[ref];
  const raw = entry?.name ?? ref;
  const filled = (key: string) => {
    const value = params?.[key];
    return value !== undefined && value !== '';
  };
  const tokens = [...raw.matchAll(/\{(\w+)\}/g)].map((m) => m[1]);
  if (entry?.name_unfilled && tokens.length > 0 && !tokens.some(filled)) {
    return entry.name_unfilled;
  }
  return raw.replace(/\{(\w+)\}/g, (_match, key: string) => {
    const value = params?.[key];
    if (filled(key)) {
      return resolveValue ? resolveValue(key, value!) : value!;
    }
    return placeholderLabel?.(key) ?? `{${key}}`;
  });
}

/**
 * How many *other* selections of `itemRef` already use each parameter value, for
 * the param `key`. Used to gray out a target that has hit the item's
 * `max_per_target` cap (e.g. Perception, once Great Characteristic was taken for
 * it twice). The selection at `exceptIndex` is excluded so its own current value
 * always stays selectable.
 */
export function paramValueUsage(
  selections: { ref: string; params?: Record<string, string> }[],
  itemRef: string,
  key: string,
  exceptIndex: number,
): Map<string, number> {
  const counts = new Map<string, number>();
  selections.forEach((selection, i) => {
    if (i === exceptIndex || selection.ref !== itemRef) return;
    const value = selection.params?.[key];
    if (!value) return;
    counts.set(value, (counts.get(value) ?? 0) + 1);
  });
  return counts;
}

export interface CategoryGroup {
  category: string;
  items: PointItem[];
}

/**
 * Sort key for an item/ability: its localized display name, so lists order
 * alphabetically in the active language (German names sort as German words).
 * Parameter placeholders are unwrapped (`{area} Lore` → `area Lore`) so a
 * parameterized entry sorts by its visible word, not the `{` glyph.
 */
export function localizedSortKey(localized: LocalizedRuleset, id: string): string {
  return (localized.i18n[id]?.name ?? id).replace(/[{}]/g, '');
}

/**
 * Display label for a granted item or a grant option: the localized name with
 * any `{param}` token filled — an unfilled token becomes the localized hint
 * ("(Ability)"), a chosen ref resolves to its own localized name ("Puissant
 * Ignem", never "Puissant art.ignem"). Shared by every grant picker (House,
 * Mythic type, Warping-owed) so no picker renders a raw slug or a raw brace.
 */
export function grantItemLabel(
  localized: LocalizedRuleset,
  ref: string,
  t: Translate,
  params: Record<string, string> = {},
): string {
  return displayName(
    localized,
    ref,
    params,
    paramHint(t),
    (_key, value) => localized.i18n[value]?.name ?? value,
  );
}

/** Extra filtering an open-grant picker may need beyond the engine constraint. */
export interface EligibilityOptions {
  /**
   * Drop items carrying a `warping_grant` effect. The Warping-owed slots need
   * this recursion guard — a fill must never re-feed the Warping Score that
   * decides how many V/F are owed (the engine rejects it as
   * `warping_fill_ineligible`). No other grant has that constraint.
   */
  excludeWarpingSources?: boolean;
}

/**
 * How deeply a prerequisite expression may nest. Mirrors the engine's
 * `PREREQ_MAX_DEPTH` (`crates/arm-rules/src/types.rs`), which rejects a deeper
 * tree at load — so this is defence in depth for a ruleset that somehow reached
 * the frontend unvalidated, not a limit the UI enforces on its own.
 */
const PREREQ_MAX_DEPTH = 32;

/**
 * Whether `prereq` is DEFINITELY unsatisfiable for a character in `house`,
 * judging `house` leaves alone and treating every other leaf as undecided.
 *
 * The exact mirror of the engine's `Prereq::conflicts_with_house`
 * (`crates/arm-rules/src/types.rs`) — a deliberate matched pair, since the menu
 * this filters and the validation that would otherwise catch the pick must agree
 * on what is offerable. Returns `undefined` for "undecided", which is why an
 * unknown House and a non-`house` prerequisite both leave an item on the menu.
 */
function houseOnlyValue(prereq: Prereq, house: string | null, depth: number): boolean | undefined {
  if (depth > PREREQ_MAX_DEPTH) return undefined;

  // AND: one false child sinks it; all-true makes it true.
  // OR:  one true child carries it; all-false makes it false.
  // NOR: one true child sinks it; all-false makes it true.
  const fold = (
    children: Prereq[],
    trigger: boolean,
    shortCircuit: boolean,
    allKnown: boolean,
  ): boolean | undefined => {
    let sawUndecided = false;
    for (const child of children) {
      const value = houseOnlyValue(child, house, depth + 1);
      if (value === trigger) return shortCircuit;
      if (value === undefined) sawUndecided = true;
    }
    return sawUndecided ? undefined : allKnown;
  };

  switch (prereq.kind) {
    case 'all':
      return fold(prereq.value, false, false, true);
    case 'any':
      return fold(prereq.value, true, true, false);
    case 'none':
      return fold(prereq.value, true, false, true);
    case 'house':
      return house === null ? undefined : house === prereq.value;
    default:
      // `has`, `ability_min`, `art_min`, `is_magus`: outside this question's
      // remit, so they can neither exclude an item nor rescue one.
      return undefined;
  }
}

/**
 * Point items an open grant admits: matching kind, matching magnitude (when the
 * constraint fixes one), inside any required-category allow-list and outside the
 * forbid-list, and not demanding a House other than `house` — mirroring the
 * engine's `open_pick_satisfies`, so a picker offers exactly the legal choices
 * and nothing more. Both category lists are matched
 * against EVERY category the item carries (`require_categories` needs a non-empty
 * intersection, `forbid_categories` an empty one), so a descriptor's secondary
 * category both admits a pick and rules one out. The rules name no fixed menu for a
 * Warping-owed slot (the pick is storyguide judgement, Core:16553-16561), so the
 * constraint is the only filter.
 *
 * `house` is the character's own Hermetic House (`store.entity.house`), or `null`
 * when there is none. It is a required argument rather than an option so that
 * every picker must state what it knows: an open grant pick is never
 * prerequisite-checked by the engine, so a menu that quietly forgot the House
 * would be the only thing standing between a Jerbiton magus and a Heartbeast.
 *
 * Sorted by localized name, with `{param}` braces unwrapped, so a parameterized
 * entry sorts by its visible word instead of clustering under "{".
 */
export function eligibleForConstraint(
  localized: LocalizedRuleset,
  constraint: GrantConstraint,
  house: string | null,
  opts: EligibilityOptions = {},
): PointItem[] {
  const items = Object.values(localized.ruleset.point_items ?? {});
  return items
    .filter(
      (it) =>
        it.kind === constraint.kind &&
        (!constraint.magnitude || it.magnitude === constraint.magnitude) &&
        (!constraint.require_categories?.length ||
          constraint.require_categories.some((c) => it.categories.includes(c))) &&
        !(constraint.forbid_categories ?? []).some((c) => it.categories.includes(c)) &&
        !(opts.excludeWarpingSources && (it.effects ?? []).some((e) => e.type === 'warping_grant')),
    )
    .filter((it) => !it.prerequisites || houseOnlyValue(it.prerequisites, house, 1) !== false)
    .sort((a, b) =>
      localizedSortKey(localized, a.id).localeCompare(localizedSortKey(localized, b.id)),
    );
}

/** One owed-Warping slot: the grant's stable key plus what it admits. */
export interface WarpingSlot {
  choice_key: string;
  constraint: GrantConstraint;
}

/**
 * A run of owed-Warping slots that expect the same thing, with the Fluent keys
 * naming it: `labelKey` labels one slot ("Minor Flaw"), `countKey` heads the
 * group ("You owe 2 Minor Flaws").
 */
export interface WarpingSlotGroup {
  labelKey: string;
  countKey: string;
  grants: WarpingSlot[];
}

/**
 * Groups the engine's owed-Warping OPEN grants by what each slot expects, so the
 * UI can label them instead of showing a flat row of identical <select>s. The
 * bucket is read off the grant's `constraint` (kind + magnitude) — never parsed
 * out of the `choice_key` slug, which is an opaque identifier.
 *
 * The engine emits the slots already ordered (Minor Flaws, supernatural Minor
 * Virtues, Major Flaws), so this preserves order and drops empty groups; a
 * non-open grant (none today, by construction) is skipped.
 */
export function groupWarpingOwedGrants(grants: Grant[]): WarpingSlotGroup[] {
  // A Map keyed by the label preserves insertion order, so the engine's slot
  // order carries through while repeated kinds collapse into one group.
  const groups = new Map<string, WarpingSlotGroup>();
  for (const grant of grants) {
    if (grant.kind !== 'open') continue;
    const { labelKey, countKey } = warpingSlotKeys(grant.constraint);
    let group = groups.get(labelKey);
    if (!group) {
      group = { labelKey, countKey, grants: [] };
      groups.set(labelKey, group);
    }
    group.grants.push({ choice_key: grant.choice_key, constraint: grant.constraint });
  }
  return [...groups.values()];
}

/** The Fluent label/count keys for one owed-Warping slot's constraint. */
function warpingSlotKeys(constraint: GrantConstraint): { labelKey: string; countKey: string } {
  if (constraint.kind === 'virtue') {
    return {
      labelKey: 'warping-slot-supernatural-virtue',
      countKey: 'warping-owed-supernatural-virtues',
    };
  }
  if (constraint.magnitude === 'major') {
    return { labelKey: 'warping-slot-major-flaw', countKey: 'warping-owed-major-flaws' };
  }
  return { labelKey: 'warping-slot-minor-flaw', countKey: 'warping-owed-minor-flaws' };
}

/**
 * The category a *single-bucket* list files an item under: the one its rulebook
 * descriptor names first. This is a tie-break, not a rule — the book has no
 * notion of a primary category (see `groupByCategory`) — so it is used only where
 * exactly one bucket is structurally required, i.e. `groupSelectionsByCategory`.
 * Mirrors the engine's `PointItem::first_listed_category`. The engine rejects a
 * categoryless item at load, so the empty fallback is unreachable through a real
 * ruleset.
 */
function firstListedCategory(item: PointItem): string {
  return item.categories[0] ?? '';
}

/**
 * Point items grouped by category; groups by category id, items alphabetically
 * by localized name within each group. When `kinds` is given, only items whose
 * `kind` is in it are kept (used to split the picker into separate Virtue and
 * Flaw lists).
 *
 * An item appears under EVERY category it carries, because that is what the
 * rulebook itself does: its Virtue/Flaw indexes list each dual-category item
 * twice, once per category, with no "primary" among them. Sufi is at
 * `Ars Magica - Definitive Edition (Core Rules).md:3179` under
 * "### Supernatural, Minor" (:3135) and again at :3230 under
 * "### Social Status, Minor" (:3187); likewise Suppressed Gift (:5301 Hermetic,
 * Major / :5369 Story, Major), Raised from the Dead (:5365 Story, Major / :5399
 * Supernatural, Major) and Visions (:5517 Story, Minor / :5561 Supernatural,
 * Minor). Filing such an item under one heading only hid it from a player
 * browsing the other category — the very category that may be the one making it
 * legal for their character (`validation/selections.rs` permits on ANY category).
 *
 * This also makes the grouping the complete source of category names, which
 * `VirtueFlawTab.categoriesFor` relies on for the filter dropdown's options.
 *
 * The Selected list does NOT mirror this; see `groupSelectionsByCategory`.
 */
export function groupByCategory(localized: LocalizedRuleset, kinds?: ItemKind[]): CategoryGroup[] {
  const groups = new Map<string, PointItem[]>();
  for (const item of Object.values(localized.ruleset.point_items)) {
    if (kinds && !kinds.includes(item.kind)) continue;
    for (const key of item.categories) {
      const list = groups.get(key) ?? [];
      list.push(item);
      groups.set(key, list);
    }
  }
  return [...groups.entries()]
    .map(([category, items]) => ({
      category,
      items: items.sort((a, b) =>
        localizedSortKey(localized, a.id).localeCompare(localizedSortKey(localized, b.id)),
      ),
    }))
    .sort((a, b) => a.category.localeCompare(b.category));
}

/** A chosen selection paired with its original index in `entity.selections`. */
export interface IndexedSelection {
  selection: Selection;
  index: number;
}

/**
 * One row of a grouped V/F list: either a selection the player bought, carrying
 * its original `entity.selections` index for the index-addressed mutators, or a
 * read-only engine-granted one, carrying its position in the `granted` list it
 * was passed in (per side, since the caller splits the grants by side first).
 *
 * The granted position is load-bearing, not decoration: the engine concatenates
 * House, Mythic Companion, `grants_selection` and warping grants WITHOUT dedup
 * (`effective.rs` `entity_grants`), so the same `ref` can arrive twice and only
 * the position tells the two rows apart.
 */
export type SelectionRow =
  | { kind: 'sel'; selection: Selection; index: number }
  | { kind: 'granted'; selection: Selection; grantIndex: number };

export interface SelectionGroup {
  category: string;
  rows: SelectionRow[];
}

/**
 * Virtue/Flaw rows grouped by category and alpha-sorted by localized name within
 * each group, using the same category headings and the same ordering (by category
 * id) as the source picker (`groupByCategory`) — but placing each row in exactly
 * one of them, which is where the two deliberately differ (see below). Rows whose
 * item ref is unknown are dropped.
 *
 * Bought and `granted` rows are grouped and sorted TOGETHER, each under its own
 * item's category (guided-creation-review-2026-08 #9). Granted rows used to be
 * appended as a header-less group of their own, which put them under whichever
 * category heading happened to sort last — a Hermetic granted Virtue read as
 * Supernatural. A granted row is ordered like any other row; only its marker and
 * the absent remove button distinguish it, exactly as for a `Required` row.
 * Equal-name ties keep bought before granted (the sort is stable).
 *
 * Each row lands in exactly ONE group, keyed on its item's first-listed category
 * (`categories[0]`). This is the one place that deliberately DIVERGES from the
 * source picker, which lists a dual-category item under both of its headings:
 * a bought row carries its `entity.selections` index and `VirtueFlawTab` removes
 * by that index, so a row repeated under a second heading would show the player
 * two apparently independent rows that delete each other — and would make one
 * selection look like two against the point budget. The descriptor's own order
 * picks the bucket because it is the only ordering the data carries, and it keeps
 * the heading agreeing with the row's FIRST category badge (the invariant
 * `houses.e2e.js` asserts).
 */
export function groupSelectionsByCategory(
  localized: LocalizedRuleset,
  entries: IndexedSelection[],
  granted: Selection[] = [],
): SelectionGroup[] {
  const groups = new Map<string, SelectionRow[]>();
  const add = (row: SelectionRow): void => {
    const item = localized.ruleset.point_items[row.selection.ref];
    if (!item) return;
    const key = firstListedCategory(item);
    const list = groups.get(key) ?? [];
    list.push(row);
    groups.set(key, list);
  };
  for (const entry of entries) {
    add({ kind: 'sel', selection: entry.selection, index: entry.index });
  }
  granted.forEach((selection, grantIndex) => add({ kind: 'granted', selection, grantIndex }));
  return [...groups.entries()]
    .map(([category, rows]) => ({
      category,
      rows: rows.sort((a, b) =>
        localizedSortKey(localized, a.selection.ref).localeCompare(
          localizedSortKey(localized, b.selection.ref),
        ),
      ),
    }))
    .sort((a, b) => a.category.localeCompare(b.category));
}

/**
 * Item ids the given type profile mandates: its `required_traits` plus its
 * `gift_id` when the Gift is `required`. These are the free, profile-declared
 * traits the store auto-selects (The Gift, Hermetic Magus for a magus) and the
 * V/F list shows non-removable — data-driven, so no id is hardcoded in the UI.
 */
export function mandatoryTraitRefs(profile: EntityTypeProfile | undefined): Set<string> {
  const refs = new Set<string>();
  if (!profile) return refs;
  for (const ref of profile.required_traits ?? []) refs.add(ref);
  if (profile.gift_policy === 'required' && profile.gift_id) refs.add(profile.gift_id);
  return refs;
}

/**
 * Item ids that may not be added because a currently selected item excludes
 * them, each mapped to the selected item responsible (so a greyed row can say
 * WHY). Built from every selected item's `incompatible_with`. Covers both
 * magnitude-variant pairs (Major/Minor Magical Focus, Ambitious Major/Minor) and
 * hand-authored exclusion cliques (Gentle vs Blatant Gift; Dwarf / Small Frame /
 * Giant Blood / Large), since both are expressed through the same data field.
 *
 * Only `enforced` mode blocks: `advisory` and `silent` leave every option
 * takeable and let the engine's `incompatible` issue report the violation
 * instead. Declared incompatibilities are symmetric (the engine rejects a
 * ruleset where they are not), so walking the selected side alone is complete.
 *
 * Mirrors `validate_incompatibilities`, which likewise tests bought selections
 * only: a House-granted item never blocks a pick.
 */
export function incompatibleRefs(
  localized: LocalizedRuleset,
  selections: Selection[],
  mode: ValidationMode,
): Map<string, string> {
  const blocked = new Map<string, string>();
  if (mode !== 'enforced') return blocked;
  for (const selection of selections) {
    const item = localized.ruleset.point_items[selection.ref];
    for (const ref of item?.incompatible_with ?? []) {
      // First blocker wins: with several selected excluders the reason names one,
      // and removing it re-derives the map against whatever still blocks.
      if (!blocked.has(ref)) blocked.set(ref, selection.ref);
    }
  }
  return blocked;
}

/**
 * Whether two selections are the same instance: same item ref and same
 * parameter values (order-independent). Used when auto-seeding/removing a Mythic
 * Companion type's required package, where a parameterized requirement (Great
 * Characteristic, Puissant Guile) must match on both ref and params.
 */
export function sameSelection(a: Selection, b: Selection): boolean {
  if (a.ref !== b.ref) return false;
  const pa = a.params ?? {};
  const pb = b.params ?? {};
  const ka = Object.keys(pa).sort();
  const kb = Object.keys(pb).sort();
  if (ka.length !== kb.length) return false;
  return ka.every((k, i) => k === kb[i] && pa[k] === pb[k]);
}

/**
 * Engine-granted selections that belong on one V/F side (virtue/boon vs
 * flaw/hook), by resolving each grant's kind against the ruleset. Grants whose
 * item is unknown are dropped. Rendered read-only, but grouped and ordered like
 * any other row — {@link groupSelectionsByCategory} takes these as its third
 * argument. Order is preserved, because a grant's POSITION is what tells two
 * grants of the same ref apart.
 */
export function grantedSelectionsForSide(
  localized: LocalizedRuleset,
  granted: Selection[] | undefined,
  side: 'virtue' | 'flaw',
): Selection[] {
  const kinds: ItemKind[] = side === 'virtue' ? ['virtue', 'boon'] : ['flaw', 'hook'];
  return (granted ?? []).filter((sel) => {
    const item = localized.ruleset.point_items[sel.ref];
    return item ? kinds.includes(item.kind) : false;
  });
}

/**
 * Total XP committed to per-spell Spell Mastery Abilities. A spell's Mastery rises
 * like an Ability, so it is priced from the same advancement table; unmastered
 * spells (0/null) cost nothing. Two Flawless-Magic reductions mirror the engine's
 * charge (`xp_allocation`): a granted `floor` (auto-mastery at 1) is free, so only
 * the table cost *above* the floor is charged; and when advancement is `doubled`
 * that remainder is halved (rounded up). Spent from the Mastered-Spells pool plus
 * the general pool. Source: Core Rules.md:3887-3889, :4471-4474.
 *
 * KNOWN DRIFT RISK (GD4, tmp/review/review-round-2-gerda-derived.md): this
 * duplicates the mastery-spend leg of `crates/arm-rules/src/effective/xp.rs`'s
 * `build_spends` as a second, independent implementation — the same class of
 * problem `characteristicPointsUsed` above documents, pending an engine-surfaced
 * mastery-pool `used` figure on `EffectiveScores` (the value is already computed
 * inside `xp_allocation` but deliberately not surfaced, per the "the mastery
 * pool is flow-only" comment at `effective/xp.rs:568-570`). The `doubled`
 * boolean below is ALSO NOT a general Affinity reduction: the engine's
 * `charged_cost(payable, affinity)` handles any `(num, den)` ratio, but this
 * collapses it to `Math.ceil(payable / 2)`, correct only for a 2/1 ratio.
 * Verified against the shipped ruleset (2026-08 round 2): exactly one item
 * grants Spell Mastery (`rules/core/virtues_flaws.json`'s Flawless Magic entry,
 * `advancement_num: 2, advancement_den: 1`), so no live call site can hit a
 * different ratio today — but the engine's `Effect::GrantsSpellMastery` already
 * supports an arbitrary ratio (`effective/spell.rs:311-326`), so a future
 * Virtue/Flaw with a different one would silently diverge here while the
 * engine-computed (validated/exported) total updated correctly. The test below
 * is pinned to the exact same worked example as
 * `effective.rs::flawless_magic_floors_first_mastery_free_and_halves_the_rest`,
 * so a change to either side's arithmetic without the other fails a test on
 * both. The correct long-term fix is the same shape as
 * `characteristicPointsUsed`'s: surface the mastery pool's `used` amount on
 * `EffectiveScores` and have `SpellBudgetBar.svelte` read it from
 * `store.effective` instead of calling this function.
 *
 * V2 (full-audit round) re-confirmed this is dormant, not live: option (a)
 * (surface the engine-computed total) needs `crates/arm-rules/src/effective/xp.rs`
 * and `crates/arm-app/src/ruleset_io.rs` (`EffectiveScores`) changes outside this
 * fix's file set, so it is NOT done here — flagged as the recommended follow-up,
 * unchanged from the paragraph above. Option (b) — explicitly scoping this
 * function to 2:1-only — is what this fix adds: the name keeps its established
 * call-site spelling (`SpellBudgetBar.svelte`'s only caller is also outside this
 * fix's file set, so renaming here would leave that import broken), but the
 * 2:1-only constraint is now enforced by a **data-integrity tripwire**, not just
 * this docstring: `derive.test.ts`'s "spellMasteryXpSpent — 2:1-only, guarded
 * against silent drift (V2)" describe block reads the SHIPPED
 * `rules/core/virtues_flaws.json` and fails the moment any `grants_spell_mastery`
 * effect ships a genuine reduction ratio other than 2/1 — so a future Virtue/Flaw
 * introducing one cannot land silently.
 */
export function spellMasteryXpSpent(
  advancement: { score: number; total_xp: number }[] | undefined,
  spells: { mastery?: number | null }[] | undefined,
  floor = 0,
  doubled = false,
): number {
  if (!advancement) return 0;
  const tableFor = (score: number): number | undefined =>
    advancement.find((r) => r.score === score)?.total_xp;
  const floorTable = floor > 0 ? (tableFor(floor) ?? 0) : 0;
  let total = 0;
  for (const { mastery } of spells ?? []) {
    const bought = mastery ?? 0;
    if (bought <= 0) continue;
    const table = tableFor(bought);
    if (table === undefined) continue;
    const payable = Math.max(0, table - floorTable);
    total += doubled ? Math.ceil(payable / 2) : payable;
  }
  return total;
}

/**
 * A spell's effective Spell Mastery score: the higher of its bought mastery and
 * the granted floor (Flawless Magic auto-masters every spell at 1). Mirrors the
 * engine's `effective_spell_mastery`.
 */
export function effectiveSpellMastery(bought: number | null | undefined, floor: number): number {
  return Math.max(bought ?? 0, floor);
}

/**
 * The lowest level an ORDINARY spell can be learned at — not a book-stated
 * floor, just the lowest level a spell can exist at (a Ritual's floor is
 * `ritual_min_level` instead; see {@link minLearnableLevel}).
 */
export const ORDINARY_SPELL_MINIMUM_LEVEL = 1;

/**
 * Fallback Ritual floor for the moment before a ruleset has loaded, when no
 * spell exists yet to disable anyway. Mirrors `crates/arm-rules/src/spell.rs`'s
 * `RITUAL_MIN_LEVEL`, which `ruleset.ritual_min_level` normally carries.
 */
export const RITUAL_MINIMUM_LEVEL_FALLBACK = 20;

/**
 * The minimum level a spell can be learned at: a Ritual must be learned at the
 * ruleset's `ritual_min_level` (Ars Magica - Definitive Edition (Core
 * Rules).md:12293, "Ritual spells are always at least level 20"), an ordinary
 * spell at 1.
 *
 * VA2 (tmp/review/review-round-1-viktor-app.md): the Ritual floor used to be a
 * bare literal duplicating the engine's own check
 * (`crates/arm-rules/src/ruleset.rs`'s `validate_spell`, and
 * `crates/arm-rules/src/validation/magus.rs`). It now takes the engine-surfaced
 * `ruleset.ritual_min_level` (derived from `spell::RITUAL_MIN_LEVEL`) as an
 * argument rather than reading it itself, so this function stays pure.
 */
export function minLearnableLevel(
  spell: Spell,
  ritualMinLevel: number = RITUAL_MINIMUM_LEVEL_FALLBACK,
): number {
  return spell.ritual ? ritualMinLevel : ORDINARY_SPELL_MINIMUM_LEVEL;
}

/**
 * Why a source spell's add control is greyed, or `null` when it is takeable. A
 * fixed-level spell is tested at its catalogue level; a General spell (no fixed
 * level) or a parameterized spell (takeable once per Form) is tested at its
 * minimum learnable level — never at a nonexistent catalogue level. Blocked
 * when that level exceeds the per-spell cap or the remaining spell-levels
 * budget. `capByTeFo` and `remaining` are engine-authoritative figures, never
 * recomputed here.
 *
 * `selectedSpellIds` greys an ordinary fixed-level spell once selected; a
 * General spell (multiple learnable levels) or a parameterized spell (once per
 * Form) stays re-takeable and is excluded from that check — matching how
 * Abilities/Virtues grey out.
 */
export function nonTakeableReason(
  spell: Spell,
  selectedSpellIds: Set<string>,
  capByTeFo: Map<string, number>,
  remaining: number,
  ritualMinLevel: number = RITUAL_MINIMUM_LEVEL_FALLBACK,
): { key: string; cap: number } | null {
  const isParametrized = (spell.parameters?.length ?? 0) > 0;
  if (spell.level != null && !isParametrized && selectedSpellIds.has(spell.id)) {
    return { key: 'spell-already-taken-reason', cap: 0 };
  }
  const cap = capByTeFo.get(`${spell.technique} ${spell.form}`);
  const need = spell.level ?? minLearnableLevel(spell, ritualMinLevel);
  if (cap != null && need > cap) return { key: 'spell-cap-reason', cap };
  if (need > remaining) return { key: 'spell-budget-reason', cap: cap ?? 0 };
  return null;
}

/** Whether a source spell's add control should be greyed — {@link nonTakeableReason} != null. */
export function isDisabled(
  spell: Spell,
  selectedSpellIds: Set<string>,
  capByTeFo: Map<string, number>,
  remaining: number,
  ritualMinLevel: number = RITUAL_MINIMUM_LEVEL_FALLBACK,
): boolean {
  return nonTakeableReason(spell, selectedSpellIds, capByTeFo, remaining, ritualMinLevel) != null;
}

/** Highest whole score the advancement table can price (the spinner ceiling). */
export function maxAbilityScore(advancement: { score: number }[] | undefined): number {
  if (!advancement || advancement.length === 0) return 0;
  return advancement.reduce((m, r) => Math.max(m, r.score), 0);
}

/**
 * Localized ability name with its parameter interpolated. For a parameterized
 * ability the i18n name is a template ("{area} Lore" / "{area}-Kunde"); the
 * `{param}` token is filled with `value`, or with a localized hint like "(Area)"
 * when empty. Plain abilities have no token, so the name is returned as-is.
 */
export function abilityDisplayName(
  localized: LocalizedRuleset,
  abilityId: string,
  value: string | null | undefined,
  placeholderLabel: (key: string) => string,
): string {
  const paramKey = localized.ruleset.abilities?.[abilityId]?.parameter ?? undefined;
  const params = paramKey && value ? { [paramKey]: value } : undefined;
  return displayName(localized, abilityId, params, placeholderLabel);
}

/**
 * The i18n key prefix a requirement's `exemplar` slug resolves through.
 *
 * The mechanics files may carry no translatable string, so `rules/core/` states only
 * the language-neutral slug (`"exemplar": "latin"`) and the text lives in
 * `rules/i18n/<lang>/` under `exemplar.<slug>`. Prefixed rather than used bare so the
 * slug can never collide with — or be mistaken for — a catalogue id.
 */
const EXEMPLAR_I18N_PREFIX = 'exemplar.';

/**
 * The localized name of a requirement's exemplar, or `null` when there is none.
 *
 * `null` for an absent slug **and** for one the i18n layer does not know: an
 * unresolvable exemplar is dropped rather than printed, because rendering the slug
 * would put a raw id on screen. Callers therefore fall back to the plain requirement.
 */
export function exemplarLabel(
  localized: LocalizedRuleset,
  exemplar: string | null | undefined,
): string | null {
  if (!exemplar) return null;
  return localized.i18n[`${EXEMPLAR_I18N_PREFIX}${exemplar}`]?.name ?? null;
}

/**
 * An Ability requirement's label — the example the rules themselves name, where they
 * name one.
 *
 * The Core Rules demand "Latin 1" of every magus (Core Rules `:2437`), but
 * `ability.dead_language` takes a **free-text** instance — a troupe decides which
 * languages exist and which are dead — so the engine can only enforce "any Dead
 * Language ≥ N". That widening is permanent (see `crates/arm-rules/RULES.md`), so the
 * honest presentation is to enforce the wide check and *say* what the rules mean.
 *
 * The label is the **exemplar alone** so the score can follow it directly and the
 * sentence reads "Latin 1", as the rulebook states it. It used to read "Dead Language
 * (e.g. Latin)", which put the example between the Ability and its score — "below Dead
 * Language (e.g. Latin) 1" reads as though "e.g. Latin" were being scored, and buries
 * the demand. The widening itself is not dropped: it trails the score as
 * [`requirementExemplarNote`]. The bought instance is deliberately ignored when the
 * rules name an exemplar — the requirement is "Latin 1" whichever dead language the
 * character happens to hold, and the score in the same sentence says what they hold.
 *
 * The single label path for both surfaces that show such a requirement — the magus
 * minimums checklist and the `issue-magus_minimum_ability` /
 * `issue-magus_recommended_ability` /
 * `issue-academic_ability_without_scholarly_language` messages — so the two can never
 * word the same demand differently.
 */
export function requirementAbilityLabel(
  localized: LocalizedRuleset,
  abilityId: string,
  instance: string | null | undefined,
  exemplar: string | null | undefined,
  t: Translate,
): string {
  const example = exemplarLabel(localized, exemplar);
  if (example) return example;
  return abilityDisplayName(localized, abilityId, instance, paramHint(t));
}

/**
 * The note that trails a widened requirement's score — " (any Dead Language)" — or
 * the empty string when the requirement names no exemplar.
 *
 * The companion of [`requirementAbilityLabel`]: that one names the rules' example so
 * "Latin 1" reads as one phrase, this one says what the engine actually enforces, in
 * the one place where it cannot be mistaken for part of the score. It names the
 * **general** Ability with no instance filled in, so it stays true of every dead
 * language a troupe invents.
 *
 * Empty rather than absent, because every message interpolating it does so
 * unconditionally and Fluent throws on a variable the args map does not carry.
 */
export function requirementExemplarNote(
  localized: LocalizedRuleset,
  abilityId: string,
  exemplar: string | null | undefined,
  t: Translate,
): string {
  if (!exemplarLabel(localized, exemplar)) return '';
  return t('requirement-exemplar', {
    ability: abilityDisplayName(localized, abilityId, null, paramHint(t)),
  });
}

/**
 * Localized ability name with a trailing marker (the rulebook's `*`) appended
 * for "asterisked" abilities — those that cannot be used without at least one
 * experience point in it (the ability's `requires_training` flag). This spans
 * General, Academic, Arcane, and all Supernatural abilities, so it is driven by
 * the per-ability flag, not by category. The marker text is passed in (from the
 * `ability-requires-training-marker` Fluent string) so no glyph is hardcoded
 * here. Abilities usable untrained are returned unmarked.
 */
export function abilityLabel(
  localized: LocalizedRuleset,
  abilityId: string,
  value: string | null | undefined,
  placeholderLabel: (key: string) => string,
  trainingMarker: string,
): string {
  const name = abilityDisplayName(localized, abilityId, value, placeholderLabel);
  const requiresTraining = localized.ruleset.abilities?.[abilityId]?.requires_training ?? false;
  return requiresTraining ? `${name}${trainingMarker}` : name;
}

/** One parameter value a Sample Childhood package must be told before it can be taken. */
export interface ChildhoodSlot {
  /** The package's own machine key for the slot (`area_a`, `language`) — never shown. */
  slot: string;
  /** The Ability the answer parameterizes. */
  ability: string;
  /** The field's user-facing name: the localized Ability, ordinal-disambiguated. */
  label: string;
}

/**
 * The parameter values a Sample Childhood package must be told, in the package's
 * own entry order.
 *
 * A slot's label is its Ability's localized name with the parameter hint
 * ("(Area) Lore"), carrying a 1-based ordinal **only** where one Ability holds two
 * or more slots — Traveling Childhood's two Area Lores are indistinguishable
 * without one, while Exploring Childhood's single Area Lore must not gain a
 * pointless "(1)".
 *
 * The slot key itself is never rendered: keys are per-package *data*, so a Fluent
 * key per slot would make the catalogue's shape code, and printing the slug raw
 * would render an id as a label. Both are forbidden.
 *
 * Source: Ars Magica - Definitive Edition (Core Rules).md:2384-2388.
 */
export function childhoodSlots(
  localized: LocalizedRuleset,
  pkg: ChildhoodPackage,
  t: Translate,
): ChildhoodSlot[] {
  const slotted = pkg.entries.filter(
    (entry): entry is ChildhoodEntry & { slot: string } => !!entry.slot,
  );
  const total = new Map<string, number>();
  for (const entry of slotted) total.set(entry.ability, (total.get(entry.ability) ?? 0) + 1);

  const seen = new Map<string, number>();
  return slotted.map((entry) => {
    const name = abilityDisplayName(localized, entry.ability, undefined, paramHint(t));
    const ordinal = (seen.get(entry.ability) ?? 0) + 1;
    seen.set(entry.ability, ordinal);
    const label =
      (total.get(entry.ability) ?? 0) > 1
        ? t('childhood-slot-label-nth', { name, index: String(ordinal) })
        : t('childhood-slot-label', { name });
    return { slot: entry.slot, ability: entry.ability, label };
  });
}

/**
 * The rows a Sample Childhood package would buy, as readable lines — the preview a
 * player decides on before taking it. One line per entry, in package order.
 *
 * Every Ability is named as the sheet will show it: the package's native-language
 * entry reads the plan's chosen language ("German 5", never the `{language}` token
 * and never the Ability id), a slot already answered reads its answer ("Rhine Lore
 * 1"), and an unanswered one falls back to the localized parameter hint.
 *
 * Source: Ars Magica - Definitive Edition (Core Rules).md:2384-2388.
 */
export function childhoodEntryPreview(
  localized: LocalizedRuleset,
  pkg: ChildhoodPackage,
  plan: LifeStagePlan | null | undefined,
  slots: Record<string, string>,
  t: Translate,
): string[] {
  const nativeLanguage = plan?.native_language?.trim() ?? '';
  return pkg.entries.map((entry) => {
    const value = entry.native ? nativeLanguage : entry.slot ? slots[entry.slot] : undefined;
    const name = abilityDisplayName(localized, entry.ability, value, paramHint(t));
    return t('childhood-entry', { name, score: String(entry.score) });
  });
}

/** What is wrong with one drafted childhood slot answer, as far as the UI can tell. */
export type ChildhoodSlotFault = 'empty' | 'duplicate' | 'native';

/**
 * The locally decidable fault in one drafted slot answer, or `null` when there is
 * none. Lets the picker disable Apply with a reason instead of submitting a form
 * the engine is bound to reject; the engine stays the authority once it is taken.
 *
 * Three faults, in the engine's own order of precedence (`childhood::apply_package`):
 *  - `empty` — nothing answered, so there is no value to buy the Ability under;
 *  - `native` — a childhood *language* repeating the native language, which the
 *    spread may not buy ("Living Language (other than the character's native
 *    language)"). Only the childhood's own language Ability is restricted, so an
 *    Area Lore named after the native language is fine; and with no native language
 *    chosen yet there is nothing to collide with.
 *  - `duplicate` — the same answer as another slot **of the same Ability**, which
 *    would merge into a single row and waste the other entry's experience.
 *
 * Reported symmetrically for a duplicate: both answers need looking at, and either
 * one is a legitimate thing to change.
 *
 * Source: Ars Magica - Definitive Edition (Core Rules).md:2378, :2384-2388.
 */
export function childhoodSlotFault(
  localized: LocalizedRuleset,
  pkg: ChildhoodPackage,
  slot: string,
  slots: Record<string, string>,
  plan: LifeStagePlan | null | undefined,
): ChildhoodSlotFault | null {
  const entry = pkg.entries.find((candidate) => candidate.slot === slot);
  if (!entry) return null;

  const value = (slots[slot] ?? '').trim();
  if (!value) return 'empty';

  const nativeLanguage = plan?.native_language?.trim() ?? '';
  const languageAbility = localized.ruleset.life_stages?.childhood.native_language_ability;
  if (nativeLanguage && entry.ability === languageAbility && value === nativeLanguage) {
    return 'native';
  }

  for (const other of pkg.entries) {
    if (!other.slot || other.slot === slot || other.ability !== entry.ability) continue;
    if ((slots[other.slot] ?? '').trim() === value) return 'duplicate';
  }
  return null;
}

/**
 * Human-readable eligibility label for a restricted XP pool: its eligible
 * ability names (localized) and category names (via `ability-category-<id>`
 * Fluent keys), separator-joined. Never renders a raw category slug — categories
 * go through `t`, abilities through their i18n name (with the usual id fallback).
 */
export function restrictedPoolLabel(
  localized: LocalizedRuleset,
  pool: RestrictedXpPool,
  t: (key: string, args?: Record<string, string>) => string,
): string {
  // A life-stage block is named for what it is, not for what it may buy: its
  // ability list is the whole childhood spread, and the two childhood blocks share
  // that list, so listing abilities could not even tell them apart.
  if (pool.origin?.kind === 'life_stage') return t(`xp-pool-${pool.origin.block}`);

  const parts: string[] = [];
  // A parameterized ability (e.g. Living Language) must show its localized param
  // hint "(Language)"/"(Sprache)", not the raw "{language}" token.
  for (const id of pool.abilities ?? [])
    parts.push(displayName(localized, id, undefined, paramHint(t)));
  for (const c of pool.categories ?? []) parts.push(t(`ability-category-${c}`));
  return parts.join(`${t('restricted-xp-list-separator')} `);
}

/**
 * Validation-issue arg keys whose value is an enum (never a rules id) and the
 * Fluent-key prefix that localizes it. The engine emits these enums as their
 * raw serialized form (`int`, `local`, `magic`), so the UI maps them through a
 * Fluent key rather than rendering the slug. `base`/`granted` are realms only in
 * the Might-realm-mismatch issue; the same `base` key is a numeric score in the
 * Characteristic issues, so numeric values are left untouched (see below).
 *
 * `category` is the V/F grouping category `category_not_permitted` and
 * `forbidden_category` name (`validation/selections.rs`). It is catalogue data
 * rather than a Rust enum, but it behaves identically here: a bare slug with no
 * i18n entry of its own, labelled by the very `category-<id>` key the picker
 * heading and the row badge already use.
 */
const ENUM_ARG_FLUENT_PREFIX: Record<string, string> = {
  characteristic: 'characteristic-',
  kind: 'reputation-type-',
  base: 'realm-',
  granted: 'realm-',
  category: 'category-',
};

/**
 * Localize one validation-issue arg value for display. The engine deliberately
 * emits raw ids/enums in `issue.args`; the id→label mapping lives in the UI, not
 * the engine. Resolution is data-driven, never slug-shaped:
 *  - a value that is a key in the ruleset i18n is a rules id → its localized name
 *    (with the param hint so a parameterized name reads "(Area) Lore", never a
 *    literal "{area} Lore");
 *  - an enum-valued arg (characteristic, reputation kind, Might realm) maps
 *    through its Fluent key — but a numeric value stays as-is, so a Characteristic
 *    base score is not mistaken for a realm;
 *  - a `key` arg names a parameter → its `param-label` Fluent string;
 *  - an `origin` arg names the pool an unspent-experience warning is about. An
 *    item-granted pool is a rules id and is already localized above; a life-stage
 *    block is no item and has no i18n entry, so it goes through the same
 *    `xp-pool-<block>` keys `restrictedPoolLabel` uses — one wording for the block
 *    wherever it appears, and never the raw `childhood_spread` slug;
 *  - anything else (free text like a trait name, or a number) passes through.
 */
export function resolveIssueArgValue(
  localized: LocalizedRuleset,
  argKey: string,
  value: string,
  t: Translate,
): string {
  if (localized.i18n[value]) return displayName(localized, value, undefined, paramHint(t));
  const prefix = ENUM_ARG_FLUENT_PREFIX[argKey];
  if (prefix && !/^-?\d+$/.test(value)) return t(`${prefix}${value}`);
  if (argKey === 'key') return t(`param-label-${value}`);
  if (argKey === 'origin') return t(`xp-pool-${value}`);
  return value;
}

/**
 * Every value in a validation-issue arg map, localized via `resolveIssueArgValue`.
 *
 * One arg is not independent of the others: an `exemplar` **qualifies** the `ability`
 * it accompanies rather than standing alone, so it never reaches a message under its
 * own name. It becomes two args instead — the `ability` label ("Latin") and a
 * `qualifier` note (" (any Dead Language)") the message places AFTER the score, so the
 * requirement reads "Latin 1" as the rulebook states it. Two consequences worth
 * keeping in mind:
 *  - a message must NOT interpolate `$exemplar` — the arg is optional in the engine's
 *    output (only where the rules data states one), and Fluent reports a missing
 *    variable. `$qualifier` is safe to interpolate because it is emitted for every
 *    `ability` arg, empty when there is no exemplar;
 *  - the message and the magus-minimums checklist go through the one
 *    `requirementAbilityLabel` / `requirementExemplarNote` pair, which is what makes
 *    them read identically.
 */
export function resolveIssueArgs(
  localized: LocalizedRuleset,
  args: Record<string, string>,
  t: Translate,
): Record<string, string> {
  const resolved: Record<string, string> = {};
  for (const [key, value] of Object.entries(args)) {
    if (key === 'exemplar') continue;
    resolved[key] = resolveIssueArgValue(localized, key, value, t);
  }
  if (args.ability) {
    // Emitted for every `ability` arg, empty where the rules name no exemplar: a
    // message interpolates it unconditionally, and Fluent throws on a missing one.
    resolved.qualifier = requirementExemplarNote(localized, args.ability, args.exemplar, t);
  }
  if (args.exemplar && args.ability) {
    resolved.ability = requirementAbilityLabel(localized, args.ability, null, args.exemplar, t);
  }
  return resolved;
}

/** How a magus's spent spell levels split between the base budget, its V/F modifier and its years as a magus. */
export interface SpellLevelAllocation {
  /**
   * The base budget (the editable figure): the effective budget minus the V/F
   * modifier and the post-Gauntlet levels.
   */
  base: number;
  /**
   * Levels charged to the unconditional side (base + post-Gauntlet), including a
   * negative modifier's penalty.
   */
  baseUsed: number;
  /** The V/F modifier itself, signed (Skilled Parens +30, Weak Parens -30, else 0). */
  bonusAmount: number;
  /** Levels drawn from a POSITIVE modifier (always 0 for a penalty). */
  bonusUsed: number;
  /** Levels the magus's years past its Gauntlet bought; 0 at the Gauntlet. */
  lifeStage: number;
  /**
   * The figure `baseUsed` is charged against — the WHOLE unconditional side,
   * `base + lifeStage`, and therefore the denominator a bar must display beside
   * it (guided-creation-review-2026-08 #18).
   *
   * It is not the same number as `base` whenever a magus has lived past its
   * Gauntlet, which is exactly the defect: `baseUsed` counts the
   * post-Gauntlet-funded levels, so pairing it with the editable `base` alone
   * displayed "150 / 120  Available: 0" — an apparent overspend on a character
   * the engine considers exactly balanced. `denominator - baseUsed === available`
   * always, which is what makes the pair readable.
   */
  denominator: number;
  /** Unconditional levels still free: `denominator - baseUsed`. Negative when overspent. */
  available: number;
}

/**
 * Splits the spell levels a magus has spent between the base budget, the
 * Virtue/Flaw modifier and the levels its years past the Gauntlet bought, so the
 * spell bar reads exactly like the XP bar: the bracketed base with its own
 * Available, and each further contribution as a separate entry.
 *
 * A **positive** modifier (Skilled Parens +30) is spent FIRST, base covers the
 * rest — the same policy the engine's XP allocator applies to restricted pools,
 * which it drains before the general pool so no earmarked XP is wasted
 * (`xp_allocation` in `effective.rs`). That allocator is a max-flow over pools
 * with eligibility constraints; a spell-levels bonus is a single *unrestricted*
 * pool, so the flow degenerates to `min(used, bonus)` and this plain arithmetic
 * yields exactly what the engine would. Unspent bonus levels therefore stay in the
 * bonus entry rather than inflating Available, mirroring unspent restricted XP.
 *
 * A **negative** modifier (Weak Parens -30) has no pool to draw from, so the
 * penalty is charged to the base first instead. `base + lifeStage - baseUsed`
 * equals `budget - used` once a positive bonus is fully drained (`used >= bonus`)
 * or the modifier is zero/negative. While a positive bonus is only partially
 * spent, Available is deliberately smaller than `budget - used` by the unspent
 * bonus amount — the unspent levels stay parked in the bonus entry rather than
 * inflating Available, exactly as an unspent restricted XP pool does.
 *
 * `lifeStage` — the slice of a magus's 30-points-a-year taken as levels of spells
 * rather than experience — is **not** a third pool. Those levels are already
 * earned, exactly as unconditional as the profile base, whereas the V/F modifier
 * is a signed adjustment that may even be a debt. So they sit on the base's side
 * of the split: they are not spent first, they raise Available, and whatever the
 * bonus does not cover is charged against `base + lifeStage` as one. Only the
 * editable `base` excludes them, because the override field edits the profile base
 * alone and the post-Gauntlet levels stay additive on top of it.
 *
 * The engine stays the single authority on the numbers themselves: `budget`
 * (base + modifier + life stage), `bonus` and `lifeStage` all come from
 * `EffectiveScores`, and validation still tests the spend against the one combined
 * budget — this split is display attribution only, never a second rule.
 */
export function spellLevelAllocation(
  used: number,
  budget: number,
  bonus: number,
  lifeStage = 0,
): SpellLevelAllocation {
  // The engine reports the total and both of its extra terms, so the base follows
  // from them rather than the UI re-deriving "override else profile base".
  const base = budget - bonus - lifeStage;
  const bonusUsed = bonus > 0 ? Math.min(used, bonus) : 0;
  // A penalty (negative bonus) is charged to the base on top of the real spend.
  const penalty = bonus < 0 ? -bonus : 0;
  const baseUsed = used - bonusUsed + penalty;
  // What `baseUsed` is charged against, and so the only honest denominator to
  // print beside it (#18). Equals `base` exactly when `lifeStage === 0`.
  const denominator = base + lifeStage;
  return {
    base,
    baseUsed,
    bonusAmount: bonus,
    bonusUsed,
    lifeStage,
    denominator,
    available: denominator - baseUsed,
  };
}

/** How the experience drawn from the general pool splits between its base and a V/F modifier. */
export interface GeneralXpAllocation {
  /** The base the player typed, or the life-stage block behind the pool: `pool - bonusAmount`. */
  base: number;
  /** Experience charged to the base, including a negative modifier's penalty. */
  baseUsed: number;
  /** The V/F modifier itself, signed (Skilled Parens +60, Weak Parens -60, else 0). */
  bonusAmount: number;
  /** Experience drawn from a POSITIVE modifier (always 0 for a penalty). */
  bonusUsed: number;
  /** Base experience still free: `base - baseUsed`. Negative when the pool is overspent. */
  available: number;
}

/**
 * The XP twin of {@link spellLevelAllocation}, and it follows the identical
 * policy for the identical reason — the two bars report the two halves of one
 * Virtue. Skilled Parens grants "an additional 60 experience points and 30 spell
 * levels during apprenticeship"
 * (Ars Magica - Definitive Edition (Core Rules).md:4966), so whatever the spell
 * bar does with the 30 the XP bar must do with the 60.
 *
 * A **positive** modifier is spent first and the base covers the rest, mirroring
 * the engine's allocator draining restricted pools before the general one; a
 * **negative** one has no pool to draw on and is charged to the base. `base -
 * baseUsed` closes against `pool - used` once a positive bonus is fully drained
 * (`used >= bonus`) or the modifier is zero/negative; while a positive bonus is
 * only partially spent, Available is deliberately smaller than `pool - used` by
 * the unspent bonus amount, which stays parked in the bonus entry rather than
 * inflating Available.
 *
 * There is deliberately **no life-stage term** here, which is the one place the
 * two differ: a magus's post-Gauntlet levels are additive to a spell budget it
 * already had, whereas its life-stage experience IS the general pool
 * (`base_general` in `effective.rs`), so it is already inside `pool`.
 *
 * `pool` is `EffectiveScores.xp_general_pool` and `bonus`
 * `EffectiveScores.xp_general_bonus` — both engine-authoritative, so this is
 * display attribution and never a second rule.
 */
export function generalXpAllocation(
  used: number,
  pool: number,
  bonus: number,
): GeneralXpAllocation {
  const base = pool - bonus;
  const bonusUsed = bonus > 0 ? Math.min(used, bonus) : 0;
  // A penalty (negative bonus) is charged to the base on top of the real spend.
  const penalty = bonus < 0 ? -bonus : 0;
  const baseUsed = used - bonusUsed + penalty;
  return { base, baseUsed, bonusAmount: bonus, bonusUsed, available: base - baseUsed };
}

/**
 * The ids of items an **error**-severity validation issue points at — the rows a
 * selected list marks as illegal (red), so a selection that became invalid after
 * the fact (a Virtue removed, an Art lowered) is visible on the row itself rather
 * than only in the issues panel. Each issue's `context` carries the offending
 * item's id (e.g. the spell whose level now exceeds its Te/Fo cap, or the
 * supernatural ability whose granting Virtue is gone).
 *
 * Error severity only: warnings are advisory (an unevaluable prerequisite, an
 * unspent restricted pool) and must not paint a row as illegal. This also makes
 * the highlight follow `ValidationMode` for free — the engine's `apply_mode`
 * clears every issue in `Silent` and downgrades all of them to warnings in
 * `Advisory`, so only `Enforced` (where an illegal state is genuinely blocking)
 * yields red rows.
 */
export function invalidSelectionIds(result: ValidationResult | null | undefined): Set<string> {
  const ids = new Set<string>();
  for (const issue of result?.issues ?? []) {
    if (issue.severity !== 'error') continue;
    if (issue.context) ids.add(issue.context);
  }
  return ids;
}

/**
 * The steps the guided wizard walks for a character type: the profile's own
 * ordered `creation_phases`, then the wizard's terminal `review` step.
 *
 * The order is the ruleset's, never imposed here — a magus declares
 * `house_specialisation` before `virtues_flaws` because the House grants a free
 * Virtue the V/F budget then has to account for. `review` is appended rather than
 * declared (the engine rejects a profile that declares it) because it is the
 * wizard's own step: it holds the findings no creation phase owns, and gates
 * Finish on the whole character.
 *
 * Empty without a profile, so a wizard opened before the ruleset loads shows
 * nothing rather than a bogus one-step flow.
 */
export function wizardPhases(profile: EntityTypeProfile | undefined): CreationPhase[] {
  if (!profile) return [];
  return [...profile.creation_phases, 'review'];
}

/** The findings attributed to one creation phase — what a wizard step shows. */
export function issuesForPhase(issues: ValidationIssue[], phase: CreationPhase): ValidationIssue[] {
  return issues.filter((issue) => issue.phase === phase);
}

/**
 * The point-item ids a wizard step's own input surface holds, so a finding filed
 * on another phase can still be traced back to the step where the offending
 * *choice* was made (manual-testing-findings #4a/#4b).
 *
 * Only `virtues_flaws` has such a surface: it is the one step that adds and
 * removes `entity.selections`, and `selection.item_ref` is the only thing the
 * engine ever puts in an issue's `context`. Every other step names nothing —
 * deliberately, and this is the half that matters: the selections stay on the
 * entity wherever the player stands, so a step that merely *coexists* with them
 * must not adopt their findings.
 *
 * Takes the selections rather than the whole `Entity` so the rule is a pure
 * function of what the step shows.
 */
export function phaseSelectedItemIds(
  selections: Selection[] | undefined,
  phase: CreationPhase | undefined,
): Set<string> {
  if (phase !== 'virtues_flaws') return new Set();
  return new Set((selections ?? []).map((selection) => selection.ref));
}

/** A finding as one wizard step shows it: the issue, plus the phase that owns it
 * when that is *not* the step being looked at. */
export interface StepIssue {
  issue: ValidationIssue;
  /**
   * Set only for a finding filed on another creation phase, admitted here because
   * its `context` names an item this step holds. The step it must be resolved on —
   * which is where its input surface is, and which is why Next stays enabled even
   * though this reads as an error.
   */
  elsewhere?: CreationPhase;
}

/**
 * What one wizard step's findings panel shows: {@link issuesForPhase}, widened by
 * the findings whose `context` names an item chosen on this very step.
 *
 * The gap it closes (manual-testing-findings #4a/#4b): Great and Poor
 * Characteristic are Virtues taken on the V/F step, but the value they constrain
 * is a Characteristic score, so the engine files
 * `characteristic_max_base_too_low` / `characteristic_min_base_too_high` on
 * `characteristics` — correctly, since that is the surface that can fix them.
 * With a strict phase filter the step where the Virtue was just taken said
 * nothing at all, while the rail's gate silently let the player walk on.
 *
 * The engine's attribution is deliberately left alone: `elsewhere` reports it
 * instead, so the panel can say which step owns the fix rather than pretend this
 * one does. That distinction is load-bearing — `canAdvance` keys strictly on the
 * owning phase, so an admitted finding reads as an error while Next stays
 * enabled, and without naming the other step that gate would look broken.
 *
 * `phase` omitted means "the whole character" (the editor's panel, and the
 * wizard's terminal Review step): everything passes through, nothing is marked.
 */
export function issuesForStep(
  issues: ValidationIssue[],
  phase: CreationPhase | undefined,
  stepItemIds: ReadonlySet<string>,
): StepIssue[] {
  if (!phase) return issues.map((issue) => ({ issue }));
  const entries: StepIssue[] = [];
  for (const issue of issues) {
    if (issue.phase === phase) {
      entries.push({ issue });
      continue;
    }
    if (issue.context && stepItemIds.has(issue.context)) {
      entries.push({ issue, elsewhere: issue.phase });
    }
  }
  return entries;
}

/**
 * Whether a phase holds an open **warning** — work still outstanding there that
 * gates nothing (manual-testing-findings #4c).
 *
 * The lower-weight twin of {@link phaseHasBlockingIssue}, and deliberately blind
 * to errors: those are the rail's blocked marker, and a step carrying both should
 * say the stronger thing only. Its whole point is the finding no `context` can
 * carry forward — `characteristic_points_unspent`, which names no item, so
 * "you still have 3 characteristic points" would otherwise be invisible from the
 * moment the player leaves that step.
 */
export function phaseHasPendingWarning(issues: ValidationIssue[], phase: CreationPhase): boolean {
  return issues.some((issue) => issue.phase === phase && issue.severity === 'warning');
}

/**
 * Whether a phase holds an error, which is what blocks advancing past it.
 *
 * Errors only: a warning is an advisory, not an illegal state, so it never gates.
 * That is deliberate but it does mean legal is not the same as complete — a magus
 * can walk past the House step with no House, because `house_unset` is a warning.
 * Such a step is *marked* untouched instead, by {@link phaseIsIncomplete}, which
 * gates nothing.
 */
export function phaseHasBlockingIssue(issues: ValidationIssue[], phase: CreationPhase): boolean {
  return issues.some((issue) => issue.phase === phase && issue.severity === 'error');
}

/**
 * The creation phases the player has recorded nothing for yet, as the engine
 * reports them — in the character type's own declared order.
 *
 * Empty before the first validation result arrives, and empty for a payload that
 * carries no report at all: "nothing to report" is the only safe reading, since a
 * missing report must never make a filled-in step look untouched.
 */
export function incompletePhases(result: ValidationResult | null | undefined): CreationPhase[] {
  return result?.completeness?.incomplete_phases ?? [];
}

/**
 * Whether a phase is one the engine reports as untouched.
 *
 * Purely informational: incompleteness has no severity, so no gate can read it.
 * That separation is the point — the wizard blocks on errors and merely *marks*
 * an empty step, so a legal-but-empty phase can still be walked past and finished
 * on (`phaseHasBlockingIssue` is the gate; this is the label).
 */
export function phaseIsIncomplete(
  result: ValidationResult | null | undefined,
  phase: CreationPhase,
): boolean {
  return incompletePhases(result).includes(phase);
}

/**
 * The index of the first phase in `phases[from..=to]` that blocks, or `null` if
 * the whole range is clear.
 *
 * The range is **inclusive of `from`**: a forward rail jump starts at a phase the
 * user may have just broken, and skipping it would make the rail a way around the
 * very gate that blocks Next. A backwards range is never blocked — Back is always
 * allowed, so the user can always reach the step that needs fixing.
 */
export function firstBlockedPhaseIndex(
  phases: CreationPhase[],
  issues: ValidationIssue[],
  from: number,
  to: number,
): number | null {
  for (let i = from; i <= to; i++) {
    const phase = phases[i];
    if (phase && phaseHasBlockingIssue(issues, phase)) return i;
  }
  return null;
}

export interface AbilityGroup {
  category: AbilityCategory;
  abilities: Ability[];
}

/**
 * Catalogue abilities grouped by category in book order (the original type
 * ordering), each group sorted alphabetically by localized name. The category
 * order comes from the engine payload (`ability_category_order`), so the UI never
 * re-hardcodes it.
 */
export function groupAbilitiesByCategory(localized: LocalizedRuleset): AbilityGroup[] {
  const groups = new Map<AbilityCategory, Ability[]>();
  for (const ability of Object.values(localized.ruleset.abilities ?? {})) {
    const list = groups.get(ability.category) ?? [];
    list.push(ability);
    groups.set(ability.category, list);
  }
  return localized.ruleset.ability_category_order
    .filter((c) => groups.has(c))
    .map((category) => ({
      category,
      abilities: groups
        .get(category)!
        .sort((a, b) =>
          localizedSortKey(localized, a.id).localeCompare(localizedSortKey(localized, b.id)),
        ),
    }));
}

/** A bought Ability score paired with its original index in `entity.ability_scores`. */
export interface IndexedAbilityScore {
  entry: AbilityScore;
  index: number;
}

export interface AbilitySelectionGroup {
  category: AbilityCategory;
  entries: IndexedAbilityScore[];
}

/**
 * Bought Ability scores grouped by category (in the engine's book order,
 * `ability_category_order`) and alpha-sorted within each group, mirroring the
 * source picker's grouping (`groupAbilitiesByCategory`). Each entry keeps its
 * original `entity.ability_scores` index so the spinner/remove wiring stays
 * correct after the reorder; entries whose ability id is unknown are dropped.
 */
export function groupAbilitySelectionsByCategory(
  localized: LocalizedRuleset,
  entries: IndexedAbilityScore[],
): AbilitySelectionGroup[] {
  const groups = new Map<AbilityCategory, IndexedAbilityScore[]>();
  for (const item of entries) {
    const ability = localized.ruleset.abilities?.[item.entry.ability];
    if (!ability) continue;
    const list = groups.get(ability.category) ?? [];
    list.push(item);
    groups.set(ability.category, list);
  }
  return localized.ruleset.ability_category_order
    .filter((c) => groups.has(c))
    .map((category) => ({
      category,
      entries: groups
        .get(category)!
        .sort((a, b) =>
          localizedSortKey(localized, a.entry.ability).localeCompare(
            localizedSortKey(localized, b.entry.ability),
          ),
        ),
    }));
}

/**
 * The `index` of a row that has no `entity.ability_scores` entry behind it, so the
 * index-addressed mutators (`adjustAbilityAt`, `removeAbilityAt`, …) must not be
 * called for it. Negative on purpose: every real index is a valid array position.
 */
export const UNBOUGHT_ROW_INDEX = -1;

/**
 * Display-only rows for Abilities the engine reports an effective-score modifier
 * for — a flat bonus (Puissant Ability) or a granted free floor (Second Sight) —
 * that the character has not bought (Issue 17).
 *
 * The Selected list is built from bought rows, so without these a character who
 * genuinely holds Puissant Magic Theory saw nothing at all about it on the
 * Abilities tab: there was no row for the badge to hang on. `ArtGrid` never had
 * the problem because all 15 Arts are always rendered.
 *
 * Only ability-level (unparameterized) modifiers get a row: the engine reports a
 * parameterized instance only when it IS bought, and an invented row could not
 * name its instance anyway (the parameter lives on the bought entry). Each row
 * reads score 0 — the bought score, which is what it is — and carries
 * {@link UNBOUGHT_ROW_INDEX} so the renderer knows not to offer edits.
 */
export function unboughtModifiedAbilities(
  scores: AbilityScore[],
  bonuses: AbilityBonus[],
  floors: AbilityFloor[],
): IndexedAbilityScore[] {
  const boughtIds = new Set(scores.map((s) => s.ability));
  const unbought = new Set<string>();
  for (const bonus of bonuses) {
    if (bonus.parameter == null && !boughtIds.has(bonus.ability)) unbought.add(bonus.ability);
  }
  for (const floor of floors) {
    if (!boughtIds.has(floor.ability)) unbought.add(floor.ability);
  }
  return [...unbought].map((ability) => ({
    entry: { ability, score: 0 },
    index: UNBOUGHT_ROW_INDEX,
  }));
}

/** Localized Art name (e.g. "Creo", "Ignem"). Arts carry no parameter. */
export function artLabel(localized: LocalizedRuleset, artId: string): string {
  return localized.i18n[artId]?.name ?? artId;
}

/** Two-letter Art abbreviation (e.g. "Cr"), or empty string when absent. */
export function artAbbreviation(localized: LocalizedRuleset, artId: string): string {
  return localized.i18n[artId]?.abbreviation ?? '';
}

/** Localized spell name (e.g. "Pilum of Fire"), falling back to the id. */
export function spellName(localized: LocalizedRuleset, spellId: string): string {
  return localized.i18n[spellId]?.name ?? spellId;
}

/**
 * Localized spell name with its parameter (the target Form of a meta-magic Vim
 * spell) interpolated. A parametrized spell's i18n name is a template
 * ("Wizard's Boost ({form})"); the `{form}` token is filled with the chosen
 * Form's localized Art name ("Ignem"), or with the localized param label
 * ("Form") when no Form is chosen yet — a source-list candidate reads
 * "Wizard's Boost (Form)". The template supplies the literal parens, so
 * `placeholderLabel` should return the plain label, unwrapped. Plain spells have
 * no token, so the name is returned as-is. Mirrors {@link abilityDisplayName}.
 */
export function spellDisplayName(
  localized: LocalizedRuleset,
  spellId: string,
  parameter: string | null | undefined,
  placeholderLabel: (key: string) => string,
): string {
  const paramKey = localized.ruleset.spells?.[spellId]?.parameters?.[0]?.key;
  const params = paramKey && parameter ? { [paramKey]: parameter } : undefined;
  // A present value is an Art id (e.g. `art.ignem`); resolve it to the Art's
  // localized name so the label reads "(Ignem)", never "(art.ignem)".
  return displayName(localized, spellId, params, placeholderLabel, (_key, value) =>
    artLabel(localized, value),
  );
}

/** A group of catalogue spells sharing one Technique/Form combination. */
export interface SpellGroup {
  technique: string;
  form: string;
  spells: Spell[];
}

/**
 * Sort key for a spell within its Te/Fo group: level first (ascending), then the
 * localized name. A **General** spell (level `null`) has no fixed level, so it
 * sorts as if its level were `+Infinity` — i.e. after every fixed-level spell,
 * as a trailing entry within the group (the picker's defined General-bucket
 * position). Ties on level break alphabetically by localized name.
 */
function compareSpellByLevelThenName(localized: LocalizedRuleset): (a: Spell, b: Spell) => number {
  return (a, b) => {
    const la = a.level ?? Number.POSITIVE_INFINITY;
    const lb = b.level ?? Number.POSITIVE_INFINITY;
    if (la !== lb) return la - lb;
    return localizedSortKey(localized, a.id).localeCompare(localizedSortKey(localized, b.id));
  };
}

/**
 * Catalogue spells grouped by Technique+Form combination, mirroring how the
 * Ability picker groups by category. Groups are ordered Form-major: by the
 * engine's Form order first, then by Technique within each Form (both from
 * `groupArtsByType`, so the UI never re-hardcodes the Art order); within a group
 * spells sort by level then name (General spells trailing — see
 * `compareSpellByLevelThenName`). Empty groups are dropped. Any Te/Fo pair not
 * found in the Art catalogue order (defensive, e.g. a spell referencing an absent
 * Art) is appended after the ordered groups.
 */
export function groupSpellsByTechniqueForm(
  localized: LocalizedRuleset,
  spells: Spell[],
): SpellGroup[] {
  const keyOf = (technique: string, form: string) => `${technique}\0${form}`;
  const buckets = new Map<string, Spell[]>();
  for (const spell of spells) {
    const key = keyOf(spell.technique, spell.form);
    const list = buckets.get(key) ?? [];
    list.push(spell);
    buckets.set(key, list);
  }

  const artGroups = groupArtsByType(localized);
  const techniques = artGroups.find((g) => g.artType === 'technique')?.arts ?? [];
  const forms = artGroups.find((g) => g.artType === 'form')?.arts ?? [];
  const compare = compareSpellByLevelThenName(localized);

  const result: SpellGroup[] = [];
  for (const form of forms) {
    for (const technique of techniques) {
      const key = keyOf(technique.id, form.id);
      const list = buckets.get(key);
      if (!list) continue;
      result.push({ technique: technique.id, form: form.id, spells: list.sort(compare) });
      buckets.delete(key);
    }
  }
  // Defensive: any leftover pair whose Arts are missing from the catalogue order.
  for (const [key, list] of buckets) {
    const [technique, form] = key.split('\0');
    result.push({ technique, form, spells: list.sort(compare) });
  }
  return result;
}

/**
 * Order a character's selected spells for display: Form-major, then Technique
 * within each Form, then catalogue level (General trailing) — the same order the
 * available list groups by. Returns a *view* pairing each selection with its
 * ORIGINAL index into the entity's `spells` array: every selected-row mutation
 * (remove / set-parameter / set-level / adjust-mastery) addresses that real
 * index, so the display order must never be written back to the entity. Sorting
 * uses the **catalogue** level, not a General spell's live edited level, so
 * inline level edits don't reorder the row mid-edit. Ties fall back to the
 * original insertion order for a stable display.
 */
export function orderSelectedSpells(
  localized: LocalizedRuleset,
  spells: SpellSelection[],
): { selection: SpellSelection; index: number }[] {
  const artGroups = groupArtsByType(localized);
  const formOrder = indexOrder(artGroups.find((g) => g.artType === 'form')?.arts ?? []);
  const techOrder = indexOrder(artGroups.find((g) => g.artType === 'technique')?.arts ?? []);
  const rank = (id: string | undefined, order: Map<string, number>) =>
    order.get(id ?? '') ?? Number.POSITIVE_INFINITY;

  return spells
    .map((selection, index) => ({ selection, index }))
    .sort((a, b) => {
      const ca = localized.ruleset.spells?.[a.selection.spell];
      const cb = localized.ruleset.spells?.[b.selection.spell];
      const fa = rank(ca?.form, formOrder);
      const fb = rank(cb?.form, formOrder);
      if (fa !== fb) return fa - fb;
      const ta = rank(ca?.technique, techOrder);
      const tb = rank(cb?.technique, techOrder);
      if (ta !== tb) return ta - tb;
      const la = ca?.level ?? Number.POSITIVE_INFINITY;
      const lb = cb?.level ?? Number.POSITIVE_INFINITY;
      if (la !== lb) return la - lb;
      return a.index - b.index;
    });
}

function indexOrder(arts: Art[]): Map<string, number> {
  return new Map(arts.map((art, i) => [art.id, i]));
}

/**
 * A group of a character's SELECTED spells sharing one Technique/Form pair. The
 * selected-list counterpart of {@link SpellGroup}: each entry keeps its original
 * `entity.spells` index so index-addressed row mutations stay correct. An empty
 * `technique`/`form` marks the trailing bucket of spells missing from the
 * catalogue — the caller renders that group without a header.
 */
export interface SelectedSpellGroup {
  technique: string;
  form: string;
  entries: { selection: SpellSelection; index: number }[];
}

/**
 * A character's selected spells grouped by Technique+Form, so the selected list
 * carries the same category headers as the available list (mirroring how
 * Abilities and V/F group their selected side).
 *
 * Built on {@link orderSelectedSpells}, which already sorts Form-major → Technique
 * → catalogue level, so consecutive runs of one Te/Fo pair are exactly the groups
 * — no re-sorting and no second source of truth for the order. A spell whose id is
 * absent from the catalogue has no Te/Fo to group under; rather than dropping the
 * row (it is a real selection the user must be able to see and remove) it lands in
 * a trailing group with empty `technique`/`form`, which the caller renders
 * header-less.
 */
export function groupSelectedSpellsByTechniqueForm(
  localized: LocalizedRuleset,
  spells: SpellSelection[],
): SelectedSpellGroup[] {
  const groups: SelectedSpellGroup[] = [];
  for (const entry of orderSelectedSpells(localized, spells)) {
    const catalogue = localized.ruleset.spells?.[entry.selection.spell];
    const technique = catalogue?.technique ?? '';
    const form = catalogue?.form ?? '';
    const last = groups[groups.length - 1];
    if (last && last.technique === technique && last.form === form) {
      last.entries.push(entry);
    } else {
      groups.push({ technique, form, entries: [entry] });
    }
  }
  return groups;
}

/**
 * A group of a character's carried equipment of one kind. A `null` kind marks the
 * trailing bucket of items missing from the catalogue — rendered header-less.
 */
export interface SelectedEquipmentGroup {
  kind: EquipmentKind | null;
  entries: { slot: EquipmentSlot; index: number }[];
}

/**
 * Carried equipment grouped by catalogue kind (weapons → shields → armor, the
 * same display order and headers as the available list), alpha-sorted by
 * localized name within each kind like the Abilities selected list. Each entry
 * keeps its original `entity.equipment` index for the index-addressed row
 * mutators. Equipment has no shared `kind` field — the three catalogue maps are
 * the only source of an item's kind — so the caller passes them in, exactly as
 * for {@link filterEquipment}. An item in none of them lands in a trailing
 * `kind: null` group so its row stays visible and removable.
 */
export function groupSelectedEquipmentByKind(
  localized: LocalizedRuleset,
  catalogue: Record<EquipmentKind, Record<string, { id: string }>>,
  slots: EquipmentSlot[],
): SelectedEquipmentGroup[] {
  const nameOf = (id: string) => localized.i18n[id]?.name ?? id;
  const kindOf = (id: string): EquipmentKind | null =>
    EQUIPMENT_KINDS.find((kind) => catalogue[kind]?.[id]) ?? null;

  const buckets = new Map<EquipmentKind | null, { slot: EquipmentSlot; index: number }[]>();
  slots.forEach((slot, index) => {
    const kind = kindOf(slot.item);
    const list = buckets.get(kind) ?? [];
    list.push({ slot, index });
    buckets.set(kind, list);
  });

  // Known kinds in display order first, then the unclassified bucket (if any).
  const order: (EquipmentKind | null)[] = [...EQUIPMENT_KINDS, null];
  return order
    .filter((kind) => buckets.has(kind))
    .map((kind) => ({
      kind,
      entries: buckets
        .get(kind)!
        .sort((a, b) => nameOf(a.slot.item).localeCompare(nameOf(b.slot.item))),
    }));
}

/**
 * A combat row's label: the weapon alone on a bare line, or the weapon joined to
 * every shield whose modifiers that line folded in ("Long Sword & Round Shield").
 * The joiner is a localized label the caller supplies (`derived-combat-shield-joiner`)
 * — it stands in for a word, so it is never hardcoded here. The spaces around it are
 * added here, because a Fluent value cannot begin or end with one.
 */
export function combatRowLabel(weaponName: string, shieldNames: string[], joiner: string): string {
  if (shieldNames.length === 0) {
    return weaponName;
  }
  return [weaponName, ...shieldNames].join(` ${joiner} `);
}

/**
 * The parameter Forms already taken by OTHER instances of a parameterized spell
 * at the same level — used to grey those Forms in a row's target-Form picker so
 * each (spell, level, Form) is takeable only once (matching the engine dedupe key
 * `(spell, level, parameter)` in `validation/magus.rs`). A parameterized spell
 * stays re-takeable for every *unused* Form (e.g. Wizard's Boost once per Form).
 * The row being edited is excluded via `exceptIndex`, and the level filter keeps
 * a Form legal at a different level available (General meta-magic spells).
 */
export function usedSpellForms(
  spells: SpellSelection[],
  spellId: string,
  level: number | null | undefined,
  exceptIndex: number,
): Set<string> {
  const used = new Set<string>();
  spells.forEach((s, i) => {
    if (i === exceptIndex || s.spell !== spellId) return;
    if ((s.level ?? null) !== (level ?? null)) return;
    if (s.parameter) used.add(s.parameter);
  });
  return used;
}

export interface ArtGroup {
  artType: ArtType;
  arts: Art[];
}

/**
 * Catalogue Arts grouped by class (Technique, Form) in book order, each group
 * sorted alphabetically by localized name. The class order comes from the engine
 * payload (`art_type_order`), so the UI never re-hardcodes it.
 */
export function groupArtsByType(localized: LocalizedRuleset): ArtGroup[] {
  const groups = new Map<ArtType, Art[]>();
  for (const art of Object.values(localized.ruleset.arts ?? {})) {
    const list = groups.get(art.art_type) ?? [];
    list.push(art);
    groups.set(art.art_type, list);
  }
  return (localized.ruleset.art_type_order ?? [])
    .filter((t) => groups.has(t))
    .map((artType) => ({
      artType,
      arts: groups
        .get(artType)!
        .sort((a, b) =>
          localizedSortKey(localized, a.id).localeCompare(localizedSortKey(localized, b.id)),
        ),
    }));
}

/**
 * The catalogue Arts of one class (Techniques or Forms), sorted by localized name.
 *
 * The single source of the Technique-only / Form-only option list: the spell filters,
 * the meta-magic Vim spells' target Form, and `ParameterPicker`'s `technique` /
 * `form` domains all read it, so there is exactly one Art picker rather than three
 * that can drift. Built on `groupArtsByType`, so the class order and the within-class
 * sort still come from the engine payload.
 */
export function artsOfType(localized: LocalizedRuleset, artType: ArtType): Art[] {
  return groupArtsByType(localized).find((group) => group.artType === artType)?.arts ?? [];
}

/** Highest whole Art score the advancement table can price (the spinner ceiling). */
export function maxArtScore(artAdvancement: { score: number }[] | undefined): number {
  return maxAbilityScore(artAdvancement);
}
