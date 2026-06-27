// Pure helpers deriving display data from the loaded ruleset + entity. Kept out
// of components so they can be unit-tested and reused.

import type {
  Ability,
  AbilityCategory,
  Characteristic,
  CharacteristicRules,
  Entity,
  ItemKind,
  LocalizedRuleset,
  PointItem,
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

export interface Balance {
  virtuePoints: number;
  flawPoints: number;
  virtueBudget: number;
  flawBudget: number;
}

const MAGNITUDE_POINTS = { free: 0, minor: 1, major: 3 } as const;

/** Sum selected virtue/flaw points against the active type profile's budget. */
export function balance(localized: LocalizedRuleset, entity: Entity): Balance {
  const profile = localized.ruleset.type_profiles[entity.type_id];
  let virtuePoints = 0;
  let flawPoints = 0;
  for (const selection of entity.selections) {
    const item = localized.ruleset.point_items[selection.ref];
    if (!item) continue;
    const points = MAGNITUDE_POINTS[item.magnitude];
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

export interface AbilityGroup {
  category: AbilityCategory;
  abilities: Ability[];
}

const ABILITY_CATEGORY_ORDER: AbilityCategory[] = [
  'general',
  'academic',
  'arcane',
  'martial',
  'supernatural',
];

/**
 * Catalogue abilities grouped by category in book order (the original type
 * ordering), each group sorted alphabetically by localized name.
 */
export function groupAbilitiesByCategory(localized: LocalizedRuleset): AbilityGroup[] {
  const groups = new Map<AbilityCategory, Ability[]>();
  for (const ability of Object.values(localized.ruleset.abilities ?? {})) {
    const list = groups.get(ability.category) ?? [];
    list.push(ability);
    groups.set(ability.category, list);
  }
  return ABILITY_CATEGORY_ORDER.filter((c) => groups.has(c)).map((category) => ({
    category,
    abilities: groups
      .get(category)!
      .sort((a, b) =>
        localizedSortKey(localized, a.id).localeCompare(localizedSortKey(localized, b.id)),
      ),
  }));
}
