// Hand-mirrored from the `arm-rules` serde types. Kept minimal: only the shapes
// that cross the Tauri boundary for Milestone 2. Codegen (ts-rs/specta) is out
// of scope here.

export type Magnitude = 'free' | 'minor' | 'major';
export type ItemKind = 'virtue' | 'flaw' | 'boon' | 'hook';
export type EntityKind = 'character' | 'covenant';
export type ValidationMode = 'enforced' | 'advisory' | 'silent';
export type IssueSeverity = 'error' | 'warning';

export interface ParameterDef {
  key: string;
  type: string;
  domain: string;
}

export interface PointItem {
  id: string;
  kind: ItemKind;
  magnitude: Magnitude;
  category: string;
  entity_kinds: EntityKind[];
  parameters?: ParameterDef[];
}

export interface PointBudget {
  virtue_points: number;
  flaw_points: number;
  max_major_virtues?: number | null;
  max_major_flaws?: number | null;
}

export interface EntityTypeProfile {
  id: string;
  budget: PointBudget;
  permitted_categories: string[];
  forbidden_categories: string[];
  creation_phases: string[];
}

export interface I18nEntry {
  name: string;
  summary?: string | null;
  description?: string | null;
}

// `Ruleset` serializes its maps as JSON objects keyed by id.
export interface Ruleset {
  id: string;
  version: string;
  point_items: Record<string, PointItem>;
  type_profiles: Record<string, EntityTypeProfile>;
}

export interface LocalizedRuleset {
  ruleset: Ruleset;
  i18n: Record<string, I18nEntry>;
}

export interface RulesetRef {
  id: string;
  version: string;
}

export interface Selection {
  ref: string;
  params?: Record<string, string>;
}

export interface Entity {
  schema_version: number;
  ruleset: RulesetRef;
  entity_kind: EntityKind;
  type_id: string;
  selections: Selection[];
}

export interface ValidationIssue {
  severity: IssueSeverity;
  code: string;
  message: string;
  context?: string | null;
}

export interface ValidationResult {
  issues: ValidationIssue[];
}

// Tauri command errors are rejected as `{ kind, message? }`.
export interface AppError {
  kind: 'io' | 'ruleset' | 'not_loaded' | 'serialize';
  message?: string;
}
