// Typed wrappers around the Tauri command bridge. Every backend call goes
// through here so the rest of the app never touches `invoke` directly.

import { invoke } from '@tauri-apps/api/core';
import type {
  AgeInSagaYear,
  Characteristic,
  CrisisOutcome,
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
 * The saga year the app is set to — the year the age ↔ birth-year link is measured
 * against (guided-creation-review-2026-08 #25).
 *
 * Read from an app-settings file `arm-app` owns, NOT from the character and not from
 * the ruleset: it is a saga fact, shared by every character in one saga. Infallible
 * on the Rust side, so a first launch with no settings file simply reports the
 * engine's default (a rules value; see `arm_rules::DEFAULT_SAGA_YEAR`).
 */
export function sagaYear(): Promise<number> {
  return invoke('saga_year');
}

/** Persist the saga year. Saga state, so it survives a relaunch. */
export function setSagaYear(year: number): Promise<void> {
  return invoke('set_saga_year', { year });
}

/**
 * How old a character born in `birthYear` is in `sagaYear`, plus any advisory the
 * pair warrants — a saga year before the birth year clamps the age to 0 rather than
 * underflowing the entity's unsigned `age`.
 *
 * Asked of the engine rather than computed here: the clamp policy and the finding it
 * emits have one home, and the frontend computes no mechanics of its own.
 */
export function deriveAge(sagaYear: number, birthYear: number): Promise<AgeInSagaYear> {
  return invoke('derive_age', { sagaYear, birthYear });
}

/** Which year a character aged `age` in `sagaYear` was born in. */
export function deriveBirthYear(sagaYear: number, age: number): Promise<number> {
  return invoke('derive_birth_year', { sagaYear, age });
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
 * One Crisis's CRISIS TOTAL with every term that made it — "Simple die + age/10
 * (round up) + Decrepitude Score" (Core Rules.md:16621). All three are ADDED, and
 * the Decrepitude is the one the crisis year itself raised (`:16619`), never
 * today's.
 */
export interface CrisisTotal {
  age: number;
  die: number;
  age_modifier: number;
  decrepitude_score: number;
  total: number;
}

/**
 * Where one crisis-survival modifier comes from. A tagged union rather than a
 * string so each case can be named its own way: a Virtue resolves through the item
 * catalogue by `item` id, the familiar's bronze cord (`:10844`) through Fluent.
 */
export type CrisisModifierSource = { kind: 'trait'; item: string } | { kind: 'bronze_cord' };

/** One modifier to the crisis survival roll, ADDED with its stored sign. */
export interface CrisisModifier {
  source: CrisisModifierSource;
  amount: number;
}

/**
 * Something the rules PERMIT at a crisis, as opposed to a number the engine adds:
 * the attending doctor of `:16634`, whose Medicine belongs to a character this
 * sheet does not hold. `botch_penalty` is stored signed and added, like every other
 * aging modifier.
 */
export type CrisisAllowance = {
  kind: 'attendant';
  ability: string;
  characteristic: Characteristic;
  ease_factor: number;
  botch_penalty: number;
};

/**
 * What surviving one Crisis would take, and what the character brings to it
 * (`:16628-16638`). A read-out: the engine never throws the Stamina die and never
 * pronounces a character dead.
 *
 * `ease_factor` is absent for the Terminal row, which offers no roll at all
 * (`:16632`) — not an unbeatable one. The modifiers are itemized *and* summed
 * because the panel has to name each one; a character carrying no bronze cord shows
 * no cord line rather than a +0.
 */
export interface CrisisSurvival {
  ease_factor?: number | null;
  ritual_level: number;
  modifiers: CrisisModifier[];
  modifier_total: number;
  allowances: CrisisAllowance[];
}

/**
 * One Crisis read whole: the total, the row it landed on, what that row costs, and
 * what surviving it would take. `row` is an id — its text lives in
 * `rules/i18n/<lang>/aging.json` — and `survival` is absent for a bedridden row,
 * which is time rather than a roll (`:16626`, `:16627`).
 */
export interface CrisisPreview {
  total: CrisisTotal;
  row: string;
  outcome: CrisisOutcome;
  survival?: CrisisSurvival | null;
}

/**
 * Something a resolved year has to TELL the player, as opposed to something it
 * writes. Rendered through `aging-note-<kind>`, never as a raw tag.
 *
 * Both of today's follow the Crisis rather than the Crisis roll, so an unrolled
 * Crisis carries them: the Longevity Ritual it spends without deleting (`:16573`),
 * and the Heavy Wound a leper takes "in addition to any other result" (`:6340`)
 * that the app records nowhere, because the health track is the player's.
 */
export type AgingNote = { kind: 'longevity_ritual_spent' } | { kind: 'heavy_wound' };

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
  | {
      status: 'previewed';
      total: AgingTotal;
      outcome: AgingOutcome;
      crisis?: CrisisPreview | null;
    }
  | { status: 'rejected'; issues: ValidationIssue[] };

export type AgingApplication =
  | {
      status: 'applied';
      entity: Entity;
      total: AgingTotal;
      outcome: AgingOutcome;
      crisis?: CrisisPreview | null;
      notes?: AgingNote[];
    }
  | { status: 'rejected'; issues: ValidationIssue[] };

export type AgingReversion =
  | { status: 'reverted'; entity: Entity }
  | { status: 'rejected'; issues: ValidationIssue[] };

/**
 * Read one year's aging roll without writing anything: the total the typed `die`
 * makes at `age`, and the row it lands on. The die stays out of the entity, which
 * is what makes "the calculator does not dirty the document" mechanically true.
 *
 * `distribution` and `crisisDie` are the very arguments {@link agingApply} takes,
 * because the engine answers them by resolving the year in memory and throwing the
 * character away: the Aging Points a Crisis row awards ARE the Decrepitude increase
 * `:16619` puts first, so a Crisis read off the character as it stands would be one
 * short of the one Apply writes.
 */
export function agingPreview(
  entity: Entity,
  age: number,
  die: number,
  distribution: Partial<Record<Characteristic, number>>,
  crisisDie: number | null,
): Promise<AgingProjection> {
  return invoke('aging_preview', { entity, age, die, distribution, crisisDie });
}

/**
 * Apply one year's aging roll. `distribution` places the Aging Points the row left
 * to the player, per Characteristic; it is empty for a row that names its own.
 * `crisisDie` is the Simple Die thrown at the Crisis Table (`:16621`), or `null`
 * for a Crisis nobody has rolled yet — which the year records as owed and unrolled
 * rather than refusing.
 */
export function agingApply(
  entity: Entity,
  age: number,
  die: number,
  distribution: Partial<Record<Characteristic, number>>,
  crisisDie: number | null,
): Promise<AgingApplication> {
  return invoke('aging_apply', { entity, age, die, distribution, crisisDie });
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
