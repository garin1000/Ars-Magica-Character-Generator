// Pure helpers deriving display data from the loaded ruleset + entity. Kept out
// of components so they can be unit-tested and reused.

import type {
  Ability,
  AbilityCategory,
  Art,
  ArtType,
  Characteristic,
  CharacteristicRules,
  Entity,
  EntityTypeProfile,
  ItemKind,
  LocalizedRuleset,
  PointItem,
  RestrictedXpPool,
  Selection,
} from './types';

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
  for (const id of pool.abilities ?? []) parts.push(displayName(localized, id));
  for (const c of pool.categories ?? []) parts.push(t(`ability-category-${c}`));
  return parts.join(`${t('restricted-xp-list-separator')} `);
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

/** Localized Art name (e.g. "Creo", "Ignem"). Arts carry no parameter. */
export function artLabel(localized: LocalizedRuleset, artId: string): string {
  return localized.i18n[artId]?.name ?? artId;
}

/** Two-letter Art abbreviation (e.g. "Cr"), or empty string when absent. */
export function artAbbreviation(localized: LocalizedRuleset, artId: string): string {
  return localized.i18n[artId]?.abbreviation ?? '';
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
 * Total XP committed across Abilities and Arts together — they draw from one
 * shared bank (`entity.xp_pool`), each priced from its own advancement table.
 */
export function totalXpSpent(localized: LocalizedRuleset, entity: Entity): number {
  return (
    abilityXpSpent(localized.ruleset.advancement, entity.ability_scores) +
    artXpSpent(localized.ruleset.art_advancement, entity.art_scores)
  );
}
