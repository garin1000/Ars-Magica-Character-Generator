import { describe, expect, it } from 'vitest';

import { rowWarningFor, rowWarnings } from './derive';
import type { ValidationIssue, ValidationResult } from './types';

// I2 (try-out finding 7, Norbert C4): a banked-XP finding marks the Ability or
// Art ROW it is about, with a warning mark distinct from the error "!" that
// `invalidSelectionIds` drives. `rowWarnings` picks the findings that mark a
// row; `rowWarningFor` answers "does this row carry one?" — for an Ability by
// id AND instance, since the engine's `context` is the Ability id alone and two
// Craft rows share it.

function banked(
  code: string,
  context: string,
  args: Record<string, string>,
  severity: ValidationIssue['severity'] = 'warning',
): ValidationIssue {
  return {
    severity,
    code,
    phase: code.startsWith('art_') ? 'arts' : 'abilities',
    context,
    args,
  };
}

function result(...issues: ValidationIssue[]): ValidationResult {
  return { issues };
}

const CARPENTRY = banked('banked_xp_at_or_above_next_level', 'ability.craft', {
  ability: 'ability.craft',
  parameter: 'Carpentry',
  banked: '12',
  needed: '10',
});

describe('rowWarnings', () => {
  it('is empty without a result', () => {
    expect(rowWarnings(null)).toEqual([]);
    expect(rowWarnings(undefined)).toEqual([]);
  });

  it('collects all four banked-XP codes', () => {
    const issues = [
      CARPENTRY,
      banked('banked_xp_at_top_score', 'ability.awareness', {
        ability: 'ability.awareness',
        parameter: '',
        banked: '3',
        score: '20',
      }),
      banked('art_banked_xp_at_or_above_next_level', 'art.creo', {
        art: 'art.creo',
        banked: '7',
        needed: '6',
      }),
      banked('art_banked_xp_at_top_score', 'art.ignem', {
        art: 'art.ignem',
        banked: '1',
        score: '30',
      }),
    ];
    expect(rowWarnings(result(...issues)).map((w) => w.issue)).toEqual(issues);
  });

  it('leaves out warnings that are not about a row of their own', () => {
    // An advisory with an Ability context is not automatically about the row's
    // stored state (a recommended minimum can name an Ability not even bought).
    const recommended = banked('magus_recommended_ability', 'ability.latin', {
      ability: 'ability.dead_language',
      min: '4',
      score: '3',
    });
    const unspent = banked('general_xp_unspent', '', { pool: '50', used: '40', unspent: '10' });
    expect(rowWarnings(result(recommended, unspent))).toEqual([]);
  });

  it('leaves out an error, which the error mark already carries', () => {
    const asError = { ...CARPENTRY, severity: 'error' as const };
    expect(rowWarnings(result(asError))).toEqual([]);
  });
});

describe('rowWarningFor', () => {
  it('marks a plain Ability row, whose finding carries an empty instance', () => {
    const awareness = banked('banked_xp_at_or_above_next_level', 'ability.awareness', {
      ability: 'ability.awareness',
      parameter: '',
      banked: '5',
      needed: '5',
    });
    const warnings = rowWarnings(result(awareness));
    expect(rowWarningFor(warnings, 'ability.awareness', undefined, {})).toEqual(awareness);
    expect(rowWarningFor(warnings, 'ability.awareness', null, {})).toEqual(awareness);
    expect(rowWarningFor(warnings, 'ability.athletics', undefined, {})).toBeUndefined();
  });

  it('marks only the instance the finding names, never every row of the Ability', () => {
    const warnings = rowWarnings(result(CARPENTRY));
    expect(rowWarningFor(warnings, 'ability.craft', { text: 'Carpentry' }, {})).toEqual(CARPENTRY);
    expect(rowWarningFor(warnings, 'ability.craft', { text: 'Smithing' }, {})).toBeUndefined();
  });

  it('matches a catalogued instance by its catalogue id', () => {
    const latin = banked('banked_xp_at_or_above_next_level', 'ability.dead_language', {
      ability: 'ability.dead_language',
      parameter: 'language.latin',
      banked: '10',
      needed: '10',
    });
    const warnings = rowWarnings(result(latin));
    expect(rowWarningFor(warnings, 'ability.dead_language', { id: 'language.latin' }, {})).toEqual(
      latin,
    );
    expect(
      rowWarningFor(warnings, 'ability.dead_language', { id: 'language.hebrew' }, {}),
    ).toBeUndefined();
  });

  it('matches a linked instance by the text its link currently resolves to', () => {
    const guild = banked('banked_xp_at_or_above_next_level', 'ability.organization_lore', {
      ability: 'ability.organization_lore',
      parameter: "Smiths' Guild of Verdi",
      banked: '10',
      needed: '10',
    });
    const warnings = rowWarnings(result(guild));
    const linked = { item: 'virtue.craft_guild_training', param: 'guild' };
    const resolved = { 'virtue.craft_guild_training\u0000guild': "Smiths' Guild of Verdi" };
    expect(rowWarningFor(warnings, 'ability.organization_lore', linked, resolved)).toEqual(guild);
  });

  it('marks an Art row by its id', () => {
    const creo = banked('art_banked_xp_at_or_above_next_level', 'art.creo', {
      art: 'art.creo',
      banked: '7',
      needed: '6',
    });
    const warnings = rowWarnings(result(creo));
    expect(rowWarningFor(warnings, 'art.creo', null, {})).toEqual(creo);
    expect(rowWarningFor(warnings, 'art.animal', null, {})).toBeUndefined();
  });
});
