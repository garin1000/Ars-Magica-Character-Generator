// Pure helpers deriving display data from the loaded ruleset + entity. Kept out
// of components so they can be unit-tested and reused.

import type { Entity, ItemKind, LocalizedRuleset, PointItem } from './types';

/** Rules display name for an item, substituting any `{param}` placeholders. */
export function displayName(
  localized: LocalizedRuleset,
  ref: string,
  params?: Record<string, string>,
): string {
  const raw = localized.i18n[ref]?.name ?? ref;
  return raw.replace(/\{(\w+)\}/g, (_match, key: string) => params?.[key] ?? `{${key}}`);
}

export interface CategoryGroup {
  category: string;
  items: PointItem[];
}

/**
 * Point items grouped by category, both groups and items sorted by id.
 * When `kinds` is given, only items whose `kind` is in it are kept (used to
 * split the picker into separate Virtue and Flaw lists).
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
      items: items.sort((a, b) => a.id.localeCompare(b.id)),
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
