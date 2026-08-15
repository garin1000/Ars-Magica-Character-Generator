// Typed wrappers around the Tauri command bridge. Every backend call goes
// through here so the rest of the app never touches `invoke` directly.

import { invoke } from '@tauri-apps/api/core';
import type {
  Characteristic,
  DerivedTotals,
  EffectiveScores,
  Entity,
  LocalizedRuleset,
  ValidationIssue,
  ValidationMode,
  ValidationResult,
} from './types';

export function loadRuleset(lang: string): Promise<LocalizedRuleset> {
  return invoke('load_ruleset', { lang });
}

export function validateEntity(entity: Entity, mode: ValidationMode): Promise<ValidationResult> {
  return invoke('validate_entity', { entity, mode });
}

export function effectiveScores(entity: Entity): Promise<EffectiveScores> {
  return invoke('effective_scores', { entity });
}

export function derivedTotals(entity: Entity): Promise<DerivedTotals> {
  return invoke('derived_totals', { entity });
}

/**
 * Write the entity to disk. `path === null` prompts (Save As / first Save);
 * a concrete path writes directly with no dialog. Resolves to the written path,
 * or `null` when a prompt was cancelled.
 */
export function saveEntity(entity: Entity, path: string | null): Promise<string | null> {
  return invoke('save_entity', { entity, path });
}

/**
 * Write the entity as a Markdown character sheet. `labels` is the document chrome
 * the engine prints — `{ Fluent message name -> resolved text }`, so no
 * user-facing string lives in Rust. `path === null` prompts for a destination.
 * `currentPath` is the document's own save file, used only to prefill that prompt's
 * name and directory — never as a write target.
 * Resolves to the written path, or `null` when a prompt was cancelled.
 */
export function exportMarkdown(
  entity: Entity,
  labels: Record<string, string>,
  path: string | null = null,
  currentPath: string | null = null,
): Promise<string | null> {
  return invoke('export_markdown', { entity, labels, path, currentPath });
}

/**
 * The document-chrome label keys the Markdown export needs. The engine owns the
 * list, so the frontend resolves exactly those rather than keeping a copy.
 */
export function exportLabelKeys(): Promise<string[]> {
  return invoke('export_label_keys');
}

/**
 * The outcome of taking a Sample Childhood package: either the rewritten entity,
 * or the reasons the package could not be taken.
 *
 * A rejection is an ordinary outcome, not an error: an unanswered slot is a finding
 * about the form the player just submitted, so the engine reports it as localizable
 * `ValidationIssue`s (rendered through the same `issue-<code>` path as any other
 * finding) rather than failing the command.
 */
export type ChildhoodApplication =
  | { status: 'applied'; entity: Entity }
  | { status: 'rejected'; issues: ValidationIssue[] };

/**
 * Take the Sample Childhood package `packageId`, answering its parameter slots with
 * `slotValues` (slot key -> the player's value). The engine owns every decision:
 * what the package writes, that the write is a monotone raise, and what makes it
 * impossible.
 */
export function applyChildhoodPackage(
  entity: Entity,
  packageId: string,
  slotValues: Record<string, string>,
): Promise<ChildhoodApplication> {
  return invoke('apply_childhood_package', { entity, packageId, slotValues });
}

/**
 * Where one aging row's points go. `kind` is the question to ask: the table names
 * the Characteristic itself, the player picks it, or the points must reach the next
 * level of Decrepitude. `characteristic` is present only for `named`.
 */
export type AgingPointTarget =
  | { kind: 'named'; characteristic: Characteristic }
  | { kind: 'player_choice' }
  | { kind: 'next_decrepitude_level' };

/**
 * One award an aging row makes. `points` is null only when the next Decrepitude
 * level lies beyond the advancement table — reported as unpriceable, never
 * silently costed at zero.
 */
export interface AgingPointAward {
  target: AgingPointTarget;
  points: number | null;
}

/** The Living Conditions modifier, split into the two places it comes from. */
export interface LivingConditionsModifier {
  rows: string[];
  from_table: number;
  from_traits: number;
  total: number;
}

/**
 * One year's AGING TOTAL with every term that made it, so the calculator shows the
 * arithmetic instead of a bare number. The two modifiers carry the book's own sign
 * and are SUBTRACTED; the trait modifier is ADDED with its stored sign.
 */
export interface AgingTotal {
  age: number;
  die: number;
  age_modifier: number;
  living_conditions: LivingConditionsModifier;
  longevity_bonus: number;
  trait_modifier: number;
  uncapped_total: number;
  total: number;
  // Whether a Longevity Ritual's under-35 ceiling cut THIS roll down.
  capped_by_longevity: boolean;
}

/** What the aging table does at a total. A crisis is flagged here, never resolved. */
export interface AgingOutcome {
  total: number;
  apparent_age_increases: boolean;
  awards: AgingPointAward[];
  crisis: boolean;
}

/**
 * The outcome of previewing, applying or reverting one aging roll.
 *
 * A refusal is an ordinary outcome, not an error — a die typed against a year
 * already rolled is a finding about the form the player just submitted — so the
 * engine reports it as localizable `ValidationIssue`s, rendered through the same
 * `issue-<code>` path as any other finding. Shaped exactly like
 * `ChildhoodApplication`.
 */
export type AgingProjection =
  | { status: 'previewed'; total: AgingTotal; outcome: AgingOutcome }
  | { status: 'rejected'; issues: ValidationIssue[] };

export type AgingApplication =
  | { status: 'applied'; entity: Entity; total: AgingTotal; outcome: AgingOutcome }
  | { status: 'rejected'; issues: ValidationIssue[] };

export type AgingReversion =
  | { status: 'reverted'; entity: Entity }
  | { status: 'rejected'; issues: ValidationIssue[] };

/**
 * Read one year's aging roll without writing anything: the total the typed `die`
 * makes at `age`, and the row it lands on. The die stays out of the entity, which
 * is what makes "the calculator does not dirty the document" mechanically true.
 */
export function agingPreview(entity: Entity, age: number, die: number): Promise<AgingProjection> {
  return invoke('aging_preview', { entity, age, die });
}

/**
 * Apply one year's aging roll. `distribution` places the Aging Points the row left
 * to the player, per Characteristic; it is empty for a row that names its own.
 */
export function agingApply(
  entity: Entity,
  age: number,
  die: number,
  distribution: Partial<Record<Characteristic, number>>,
): Promise<AgingApplication> {
  return invoke('aging_apply', { entity, age, die, distribution });
}

/** Take one applied aging year back off, exactly. */
export function agingRevert(entity: Entity, age: number): Promise<AgingReversion> {
  return invoke('aging_revert', { entity, age });
}

/** An opened document: the entity plus the file it was read from. */
export interface LoadedEntity {
  path: string;
  entity: Entity;
}

export function loadEntity(): Promise<LoadedEntity | null> {
  return invoke('load_entity');
}

/** Localized strings for the "discard unsaved changes?" dialog, shown from Rust. */
export interface CloseGuardLabels {
  title: string;
  message: string;
  discard: string;
  cancel: string;
}

/**
 * Mirror the frontend's dirty flag (and the localized dialog strings) to the
 * backend close/quit guard, which owns the actual confirmation prompt so it can
 * intercept every quit path — window close and macOS Cmd+Q alike.
 */
export function updateCloseGuard(dirty: boolean, labels: CloseGuardLabels): Promise<void> {
  return invoke('update_close_guard', { dirty, labels });
}
