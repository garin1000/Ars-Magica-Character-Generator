// Pure helpers deriving display data from the loaded ruleset + entity. Kept out
// of components so they can be unit-tested and reused.

import type {
  Ability,
  AbilityCategory,
  AbilityScore,
  Art,
  ArtType,
  Characteristic,
  CharacteristicRules,
  Entity,
  EntityTypeProfile,
  ItemKind,
  LocalizedRuleset,
  Magnitude,
  PointItem,
  RestrictedXpPool,
  Selection,
  Spell,
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

/** A `store.t`-shaped translator, threaded in so search can index rendered labels. */
type Translate = (key: string, args?: Record<string, string>) => string;

/** The standard "(Label)" placeholder hint for an unfilled `{param}` token. */
function paramHint(t: Translate): (key: string) => string {
  return (key) => t('param-hint', { label: t(`param-label-${key}`) });
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
    if (filter.categories?.length && !filter.categories.includes(it.category)) return false;
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
  level?: number | null;
}

/**
 * Spells matching the text (localized name), Technique and Form (each optional,
 * so they filter separately or combined), and exact level. A `level` of `null`
 * (or a General spell) is matched only when the filter's `level` is `null`.
 */
export function filterSpells(
  localized: LocalizedRuleset,
  spells: Spell[],
  filter: SpellFilter,
  t?: Translate,
): Spell[] {
  const text = filter.text ? normalizeSearch(filter.text) : '';
  return spells.filter((s) => {
    // Spell names carry no `{param}` placeholder, so the rendered label equals
    // the plain name; routing through the hint-aware path just keeps the search
    // helpers uniform.
    const name = t
      ? displayName(localized, s.id, undefined, paramHint(t))
      : spellName(localized, s.id);
    if (text && !normalizeSearch(name).includes(text)) return false;
    if (filter.technique && s.technique !== filter.technique) return false;
    if (filter.form && s.form !== filter.form) return false;
    if (filter.level !== undefined && (s.level ?? null) !== filter.level) return false;
    return true;
  });
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
 */
export function displayName(
  localized: LocalizedRuleset,
  ref: string,
  params?: Record<string, string>,
  placeholderLabel?: (key: string) => string,
  resolveValue?: (key: string, value: string) => string,
): string {
  const raw = localized.i18n[ref]?.name ?? ref;
  return raw.replace(/\{(\w+)\}/g, (_match, key: string) => {
    const value = params?.[key];
    if (value !== undefined && value !== '') {
      return resolveValue ? resolveValue(key, value) : value;
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
function localizedSortKey(localized: LocalizedRuleset, id: string): string {
  return (localized.i18n[id]?.name ?? id).replace(/[{}]/g, '');
}

/**
 * Point items grouped by category; groups by category id, items alphabetically
 * by localized name within each group. When `kinds` is given, only items whose
 * `kind` is in it are kept (used to split the picker into separate Virtue and
 * Flaw lists).
 */
export function groupByCategory(localized: LocalizedRuleset, kinds?: ItemKind[]): CategoryGroup[] {
  const groups = new Map<string, PointItem[]>();
  for (const item of Object.values(localized.ruleset.point_items)) {
    if (kinds && !kinds.includes(item.kind)) continue;
    const list = groups.get(item.category) ?? [];
    list.push(item);
    groups.set(item.category, list);
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

export interface SelectionGroup {
  category: string;
  entries: IndexedSelection[];
}

/**
 * Chosen Virtue/Flaw selections grouped by category and alpha-sorted within each
 * group, mirroring the source picker's grouping (`groupByCategory`). Each entry
 * keeps its original `entity.selections` index so edit/remove wiring stays
 * correct after the reorder; entries whose item ref is unknown are dropped.
 * Categories order the same way as the source list (by category id).
 */
export function groupSelectionsByCategory(
  localized: LocalizedRuleset,
  entries: IndexedSelection[],
): SelectionGroup[] {
  const groups = new Map<string, IndexedSelection[]>();
  for (const entry of entries) {
    const item = localized.ruleset.point_items[entry.selection.ref];
    if (!item) continue;
    const list = groups.get(item.category) ?? [];
    list.push(entry);
    groups.set(item.category, list);
  }
  return [...groups.entries()]
    .map(([category, list]) => ({
      category,
      entries: list.sort((a, b) =>
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
 * House-granted selections that belong on one V/F side (virtue/boon vs
 * flaw/hook), by resolving each grant's kind against the ruleset. Grants whose
 * item is unknown are dropped. Rendered read-only in the selected list.
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

export interface Balance {
  virtuePoints: number;
  flawPoints: number;
  virtueBudget: number;
  flawBudget: number;
}

/** Sum selected virtue/flaw points against the active type profile's budget. */
export function balance(localized: LocalizedRuleset, entity: Entity): Balance {
  const profile = localized.ruleset.type_profiles[entity.type_id];
  const magnitudePoints = localized.ruleset.magnitude_points;
  let virtuePoints = 0;
  let flawPoints = 0;
  for (const selection of entity.selections) {
    const item = localized.ruleset.point_items[selection.ref];
    if (!item) continue;
    const points = magnitudePoints[item.magnitude] ?? 0;
    if (item.kind === 'virtue' || item.kind === 'boon') virtuePoints += points;
    else flawPoints += points;
  }
  return {
    virtuePoints,
    flawPoints,
    virtueBudget: profile?.budget.virtue_points ?? 0,
    flawBudget: profile?.budget.flaw_points ?? 0,
  };
}

/**
 * Total Characteristic points spent for the given scores against the cost table.
 * Positive cost rows spend points, negative ("Gain N") rows refund them; a score
 * with no table row contributes 0 (it is reported separately as out-of-range).
 */
export function characteristicPointsUsed(
  rules: CharacteristicRules | null | undefined,
  characteristics: Partial<Record<Characteristic, number>> | undefined,
): number {
  if (!rules || !characteristics) return 0;
  let total = 0;
  for (const score of Object.values(characteristics)) {
    const row = rules.costs.find((c) => c.score === score);
    if (row) total += row.cost;
  }
  return total;
}

/** Total XP committed across whole bought ability scores (Σ xp_for_score). */
export function abilityXpSpent(
  advancement: { score: number; total_xp: number }[] | undefined,
  scores: { score: number }[] | undefined,
): number {
  if (!advancement || !scores) return 0;
  let total = 0;
  for (const { score } of scores) {
    if (score <= 0) continue;
    const row = advancement.find((r) => r.score === score);
    if (row) total += row.total_xp;
  }
  return total;
}

/**
 * Total XP committed to per-spell Spell Mastery Abilities (Σ xp_for_score of each
 * spell's bought mastery). A spell's Mastery rises like an Ability, so it is
 * priced from the same advancement table; unmastered spells (0/null) cost
 * nothing. Spent from the restricted Spell-Mastery pool (Mastered Spells +50).
 */
export function spellMasteryXpSpent(
  advancement: { score: number; total_xp: number }[] | undefined,
  spells: { mastery?: number | null }[] | undefined,
): number {
  return abilityXpSpent(
    advancement,
    (spells ?? []).map((s) => ({ score: s.mastery ?? 0 })),
  );
}

/**
 * A spell's effective Spell Mastery score: the higher of its bought mastery and
 * the granted floor (Flawless Magic auto-masters every spell at 1). Mirrors the
 * engine's `effective_spell_mastery`.
 */
export function effectiveSpellMastery(bought: number | null | undefined, floor: number): number {
  return Math.max(bought ?? 0, floor);
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
 */
const ENUM_ARG_FLUENT_PREFIX: Record<string, string> = {
  characteristic: 'characteristic-',
  kind: 'reputation-type-',
  base: 'realm-',
  granted: 'realm-',
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
  return value;
}

/** Every value in a validation-issue arg map, localized via `resolveIssueArgValue`. */
export function resolveIssueArgs(
  localized: LocalizedRuleset,
  args: Record<string, string>,
  t: Translate,
): Record<string, string> {
  const resolved: Record<string, string> = {};
  for (const [key, value] of Object.entries(args)) {
    resolved[key] = resolveIssueArgValue(localized, key, value, t);
  }
  return resolved;
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

/** Total XP committed across whole bought Art scores (Σ xp_for_score). */
export function artXpSpent(
  artAdvancement: { score: number; total_xp: number }[] | undefined,
  scores: { score: number }[] | undefined,
): number {
  return abilityXpSpent(artAdvancement, scores);
}

/** Highest whole Art score the advancement table can price (the spinner ceiling). */
export function maxArtScore(artAdvancement: { score: number }[] | undefined): number {
  return maxAbilityScore(artAdvancement);
}

/**
 * First-frame placeholder for total XP committed, used only until the engine's
 * authoritative `XpAllocation` (`store.effective.xp_total_demand`) arrives.
 *
 * It deliberately does NOT re-derive the allocation in TS: pricing here would
 * fork the engine's single evaluation path and ignore Affinity (½× cost) and the
 * restricted/general split, so it could momentarily flash a wrong figure. A
 * neutral `0` placeholder is shown for the one frame before `store.effective`
 * replaces it. Parameters are kept for the call sites; they are intentionally
 * unused.
 */
export function totalXpSpent(localized: LocalizedRuleset, entity: Entity): number {
  // Params retained for the call sites but intentionally unused: pricing here is
  // deliberately avoided (see doc comment above).
  void localized;
  void entity;
  return 0;
}
