import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

import {
  abilityDisplayName,
  abilityLabel,
  firstBlockedPhaseIndex,
  incompletePhases,
  issuesForPhase,
  issuesForStep,
  phaseHasBlockingIssue,
  phaseHasPendingWarning,
  phaseIsIncomplete,
  phaseSelectedItemIds,
  wizardPhases,
  effectiveSpellMastery,
  spellMasteryXpSpent,
  artAbbreviation,
  artLabel,
  childhoodEntryPreview,
  childhoodSlotFault,
  childhoodSlots,
  combatRowLabel,
  displayName,
  eligibleForConstraint,
  exemplarLabel,
  filterAbilities,
  filterEquipment,
  filterItems,
  filterSpells,
  formatSigned,
  generalXpAllocation,
  grantItemLabel,
  grantedSelectionsForSide,
  groupAbilitiesByCategory,
  groupArtsByType,
  groupByCategory,
  groupSelectedEquipmentByKind,
  groupSelectedSpellsByTechniqueForm,
  groupSelectionsByCategory,
  groupSpellsByTechniqueForm,
  groupWarpingOwedGrants,
  incompatibleRefs,
  invalidSelectionIds,
  isDisabled,
  minLearnableLevel,
  nonTakeableReason,
  orderSelectedSpells,
  usedSpellForms,
  groupAbilitySelectionsByCategory,
  unboughtModifiedAbilities,
  UNBOUGHT_ROW_INDEX,
  mandatoryTraitRefs,
  maxAbilityScore,
  maxArtScore,
  ORDINARY_SPELL_MINIMUM_LEVEL,
  paramValueUsage,
  requirementAbilityLabel,
  requirementExemplarNote,
  resolveIssueArgValue,
  resolveIssueArgs,
  restrictedPoolLabel,
  RITUAL_MINIMUM_LEVEL_FALLBACK,
  spellDisplayName,
  spellLevelAllocation,
} from './derive';
import type {
  Ability,
  Art,
  ChildhoodPackage,
  CreationPhase,
  EntityTypeProfile,
  Grant,
  GrantConstraint,
  LocalizedRuleset,
  PointItem,
  Spell,
  ValidationIssue,
  ValidationResult,
} from './types';

// --- Fixtures ---------------------------------------------------------------

/** Engine-derived taxonomy the real backend ships on every Ruleset payload. */
const DERIVED_TAXONOMY = {
  magnitude_points: { free: 0, minor: 1, major: 3 },
  ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
} satisfies Pick<LocalizedRuleset['ruleset'], 'magnitude_points' | 'ability_category_order'>;

function item(overrides: Partial<PointItem> & Pick<PointItem, 'id'>): PointItem {
  return {
    kind: 'virtue',
    magnitude: 'minor',
    categories: ['general'],
    classification: 'narrative',
    entity_kinds: ['character'],
    ...overrides,
  };
}

/** A localized ruleset with the given items and (optionally) i18n + profiles. */
function makeRuleset(
  items: PointItem[],
  opts: {
    i18n?: LocalizedRuleset['i18n'];
    profiles?: LocalizedRuleset['ruleset']['type_profiles'];
  } = {},
): LocalizedRuleset {
  const point_items: Record<string, PointItem> = {};
  for (const it of items) point_items[it.id] = it;
  return {
    ruleset: {
      id: 'test',
      version: '1',
      point_items,
      type_profiles: opts.profiles ?? {},
      ...DERIVED_TAXONOMY,
    },
    i18n: opts.i18n ?? {},
  };
}

// --- filterItems() / filterAbilities() / filterSpells() ---------------------

describe('filterItems', () => {
  const items = [
    item({ id: 'virtue.brave', categories: ['general'], magnitude: 'minor' }),
    item({ id: 'virtue.giant', categories: ['general'], magnitude: 'major' }),
    item({ id: 'virtue.corrupt', categories: ['supernatural'], magnitude: 'minor', tainted: true }),
    item({ id: 'flaw.dark', kind: 'flaw', categories: ['story'], magnitude: 'major' }),
  ];
  const i18n = {
    'virtue.brave': { name: 'Brave', summary: 'Fearless in danger.' },
    'virtue.giant': { name: 'Giant Blood', summary: 'Descended from giants.' },
    'virtue.corrupt': { name: 'Corrupted', summary: 'Tainted by demons.' },
    'flaw.dark': { name: 'Dark Secret', summary: 'A hidden shame.' },
  };
  const rs = makeRuleset(items, { i18n });

  it('returns all items when the filter is empty', () => {
    expect(filterItems(rs, items, {}).map((i) => i.id)).toHaveLength(4);
  });

  it('matches the localized name or summary, case/diacritic-insensitively', () => {
    expect(filterItems(rs, items, { text: 'giant' }).map((i) => i.id)).toEqual(['virtue.giant']);
    // summary match
    expect(filterItems(rs, items, { text: 'demons' }).map((i) => i.id)).toEqual(['virtue.corrupt']);
  });

  // A descriptor may name two categories (virtue.sufi is "Social Status,
  // Supernatural"). Membership tests read the WHOLE list — only display and
  // grouping use the primary — so filtering on the secondary must find it.
  it('matches an item through a secondary category', () => {
    const dual = item({ id: 'virtue.sufi', categories: ['social_status', 'supernatural'] });
    const plain = item({ id: 'virtue.plain', categories: ['general'] });
    const list = [dual, plain];
    const dualRs = makeRuleset(list);
    expect(filterItems(dualRs, list, { categories: ['supernatural'] }).map((i) => i.id)).toEqual([
      'virtue.sufi',
    ]);
    expect(filterItems(dualRs, list, { categories: ['social_status'] }).map((i) => i.id)).toEqual([
      'virtue.sufi',
    ]);
  });

  it('filters by category, magnitude, and tainted', () => {
    expect(filterItems(rs, items, { categories: ['general'] }).map((i) => i.id)).toEqual([
      'virtue.brave',
      'virtue.giant',
    ]);
    expect(filterItems(rs, items, { magnitudes: ['major'] }).map((i) => i.id)).toEqual([
      'virtue.giant',
      'flaw.dark',
    ]);
    expect(filterItems(rs, items, { tainted: true }).map((i) => i.id)).toEqual(['virtue.corrupt']);
  });

  it('ANDs facets together', () => {
    expect(
      filterItems(rs, items, { categories: ['general'], magnitudes: ['minor'] }).map((i) => i.id),
    ).toEqual(['virtue.brave']);
  });

  it('shows every category when the category facet is omitted', () => {
    expect(filterItems(rs, items, { categories: [] }).map((i) => i.id)).toHaveLength(4);
  });

  it('combines the type (category) facet with magnitude and text', () => {
    // Two general virtues, then narrow by magnitude, then by text.
    expect(filterItems(rs, items, { categories: ['general'] }).map((i) => i.id)).toEqual([
      'virtue.brave',
      'virtue.giant',
    ]);
    expect(
      filterItems(rs, items, { categories: ['general'], magnitudes: ['major'] }).map((i) => i.id),
    ).toEqual(['virtue.giant']);
    expect(
      filterItems(rs, items, {
        categories: ['general'],
        magnitudes: ['major'],
        text: 'giant',
      }).map((i) => i.id),
    ).toEqual(['virtue.giant']);
    // A type filter that excludes the only text match yields nothing.
    expect(
      filterItems(rs, items, { categories: ['story'], text: 'giant' }).map((i) => i.id),
    ).toEqual([]);
  });
});

describe('filterAbilities', () => {
  const abilities: Ability[] = [
    { id: 'ability.awareness', category: 'general' },
    { id: 'ability.latin', category: 'academic' },
    { id: 'ability.magic_theory', category: 'arcane' },
  ];
  const i18n = {
    'ability.awareness': { name: 'Awareness' },
    'ability.latin': { name: 'Latin' },
    'ability.magic_theory': { name: 'Magic Theory' },
  };
  const rs = makeRuleset([], { i18n });

  it('filters by text and by category', () => {
    expect(filterAbilities(rs, abilities, { text: 'lat' }).map((a) => a.id)).toEqual([
      'ability.latin',
    ]);
    expect(filterAbilities(rs, abilities, { categories: ['arcane'] }).map((a) => a.id)).toEqual([
      'ability.magic_theory',
    ]);
  });
});

describe('filterSpells', () => {
  const spells = [
    { id: 'spell.pilum', technique: 'art.creo', form: 'art.ignem', level: 20 },
    { id: 'spell.veil', technique: 'art.perdo', form: 'art.imaginem', level: 15 },
    { id: 'spell.ward', technique: 'art.rego', form: 'art.ignem', level: null },
  ];
  const i18n = {
    'spell.pilum': { name: 'Pilum of Fire' },
    'spell.veil': { name: 'Veil of Invisibility' },
    'spell.ward': { name: 'Ward Against Heat' },
  };
  const rs = makeRuleset([], { i18n });

  it('filters by technique, form (separate and combined), text, and level', () => {
    expect(filterSpells(rs, spells, { form: 'art.ignem' }).map((s) => s.id)).toEqual([
      'spell.pilum',
      'spell.ward',
    ]);
    expect(
      filterSpells(rs, spells, { technique: 'art.creo', form: 'art.ignem' }).map((s) => s.id),
    ).toEqual(['spell.pilum']);
    expect(filterSpells(rs, spells, { text: 'veil' }).map((s) => s.id)).toEqual(['spell.veil']);
  });

  it('treats null range bounds as "no level filter" (returns all spells)', () => {
    expect(filterSpells(rs, spells, { levelMin: null, levelMax: null }).map((s) => s.id)).toEqual([
      'spell.pilum',
      'spell.veil',
      'spell.ward',
    ]);
    // No range keys at all is likewise unfiltered.
    expect(filterSpells(rs, spells, {}).map((s) => s.id)).toEqual([
      'spell.pilum',
      'spell.veil',
      'spell.ward',
    ]);
  });

  it('filters a fixed-level spell by an inclusive min/max range (null bound = open)', () => {
    // levelMin only: keeps spells at or above the bound (General spells always pass).
    expect(filterSpells(rs, spells, { levelMin: 16 }).map((s) => s.id)).toEqual([
      'spell.pilum',
      'spell.ward',
    ]);
    // levelMax only: keeps spells at or below the bound (General spells always pass).
    expect(filterSpells(rs, spells, { levelMax: 15 }).map((s) => s.id)).toEqual([
      'spell.veil',
      'spell.ward',
    ]);
    // Both bounds: an inclusive band.
    expect(filterSpells(rs, spells, { levelMin: 15, levelMax: 20 }).map((s) => s.id)).toEqual([
      'spell.pilum',
      'spell.veil',
      'spell.ward',
    ]);
    // A band excluding every fixed level leaves only the always-shown General spell.
    expect(filterSpells(rs, spells, { levelMin: 21, levelMax: 30 }).map((s) => s.id)).toEqual([
      'spell.ward',
    ]);
  });

  it('always shows a General spell (level null) regardless of the range bounds', () => {
    // ward is General (level null); it passes any bounded range because it has no
    // fixed catalogue level to test — its learned level is chosen per character.
    expect(filterSpells(rs, spells, { levelMin: 100, levelMax: 200 }).map((s) => s.id)).toEqual([
      'spell.ward',
    ]);
  });
});

describe('groupSpellsByTechniqueForm', () => {
  const arts: Art[] = [
    { id: 'art.creo', art_type: 'technique' },
    { id: 'art.rego', art_type: 'technique' },
    { id: 'art.ignem', art_type: 'form' },
    { id: 'art.vim', art_type: 'form' },
  ];
  function withSpellsRuleset(): LocalizedRuleset {
    const map: Record<string, Art> = {};
    for (const a of arts) map[a.id] = a;
    return {
      ruleset: {
        id: 't',
        version: '1',
        point_items: {},
        type_profiles: {},
        arts: map,
        art_type_order: ['technique', 'form'],
        ...DERIVED_TAXONOMY,
      },
      i18n: {
        'art.creo': { name: 'Creo' },
        'art.rego': { name: 'Rego' },
        'art.ignem': { name: 'Ignem' },
        'art.vim': { name: 'Vim' },
        'spell.pilum': { name: 'Pilum of Fire' },
        'spell.arc': { name: 'Arc of Fiery Ribbons' },
        'spell.ball': { name: 'Ball of Abysmal Flame' },
        'spell.ward': { name: 'Ward Against Heat' },
        'spell.aegis': { name: 'Aegis of the Hearth' },
        'spell.watching': { name: 'Watching Ward' },
      },
    };
  }

  it('buckets by Technique+Form in art order, sorting each by level then name', () => {
    const spells = [
      // CrIg: two fixed levels (35, 10) plus a General spell (null).
      { id: 'spell.ball', technique: 'art.creo', form: 'art.ignem', level: 35 },
      { id: 'spell.arc', technique: 'art.creo', form: 'art.ignem', level: 10 },
      { id: 'spell.ward', technique: 'art.creo', form: 'art.ignem', level: null },
      // ReVi: a General spell and a fixed one.
      { id: 'spell.aegis', technique: 'art.rego', form: 'art.vim', level: null },
      { id: 'spell.watching', technique: 'art.rego', form: 'art.vim', level: 20 },
    ];
    const groups = groupSpellsByTechniqueForm(withSpellsRuleset(), spells);
    // Groups ordered by Form first, then Technique.
    expect(groups.map((g) => [g.technique, g.form])).toEqual([
      ['art.creo', 'art.ignem'],
      ['art.rego', 'art.vim'],
    ]);
    // Within a group: level ascending, General (null) trailing, then name.
    expect(groups[0].spells.map((s) => s.id)).toEqual(['spell.arc', 'spell.ball', 'spell.ward']);
    expect(groups[1].spells.map((s) => s.id)).toEqual(['spell.watching', 'spell.aegis']);
  });

  it('orders groups by Form first, then Technique', () => {
    // Spells spanning both Techniques and both Forms, so Form-major and
    // Technique-major orderings are distinguishable.
    const spells = [
      { id: 'spell.ball', technique: 'art.creo', form: 'art.ignem', level: 35 },
      { id: 'spell.aegis', technique: 'art.rego', form: 'art.vim', level: 20 },
      { id: 'spell.arc', technique: 'art.creo', form: 'art.vim', level: 10 },
      { id: 'spell.watching', technique: 'art.rego', form: 'art.ignem', level: 15 },
    ];
    const groups = groupSpellsByTechniqueForm(withSpellsRuleset(), spells);
    // Form-major: all Ignem groups (Creo, Rego) before all Vim groups (Creo, Rego).
    expect(groups.map((g) => [g.technique, g.form])).toEqual([
      ['art.creo', 'art.ignem'],
      ['art.rego', 'art.ignem'],
      ['art.creo', 'art.vim'],
      ['art.rego', 'art.vim'],
    ]);
  });

  it('sorts same-level spells alphabetically by localized name', () => {
    const spells = [
      { id: 'spell.pilum', technique: 'art.creo', form: 'art.ignem', level: 20 },
      { id: 'spell.arc', technique: 'art.creo', form: 'art.ignem', level: 20 },
    ];
    const groups = groupSpellsByTechniqueForm(withSpellsRuleset(), spells);
    expect(groups[0].spells.map((s) => s.id)).toEqual(['spell.arc', 'spell.pilum']);
  });

  it('is empty when there are no spells', () => {
    expect(groupSpellsByTechniqueForm(withSpellsRuleset(), [])).toEqual([]);
  });

  it('appends a leftover pair whose Arts are missing from the catalogue order, with the correct technique/form (not undefined)', () => {
    // Neither art.unknown_tech nor art.unknown_form is in the `arts` fixture, so
    // the Form-major x Technique double loop never visits this pair and it must
    // fall through to the defensive leftover branch.
    const spells = [
      { id: 'spell.mystery', technique: 'art.unknown_tech', form: 'art.unknown_form', level: 5 },
    ];
    const groups = groupSpellsByTechniqueForm(withSpellsRuleset(), spells);
    expect(groups).toHaveLength(1);
    expect(groups[0].technique).toBe('art.unknown_tech');
    expect(groups[0].form).toBe('art.unknown_form');
    expect(groups[0].spells.map((s) => s.id)).toEqual(['spell.mystery']);
  });
});

describe('orderSelectedSpells', () => {
  const arts: Art[] = [
    { id: 'art.creo', art_type: 'technique' },
    { id: 'art.rego', art_type: 'technique' },
    { id: 'art.ignem', art_type: 'form' },
    { id: 'art.vim', art_type: 'form' },
  ];
  function rulesetWithCatalogue(): LocalizedRuleset {
    const artMap: Record<string, Art> = {};
    for (const a of arts) artMap[a.id] = a;
    const spells: Record<string, Spell> = {
      'spell.ball': { id: 'spell.ball', technique: 'art.creo', form: 'art.ignem', level: 35 },
      'spell.arc': { id: 'spell.arc', technique: 'art.creo', form: 'art.ignem', level: 10 },
      'spell.watching': {
        id: 'spell.watching',
        technique: 'art.rego',
        form: 'art.ignem',
        level: 15,
      },
      'spell.aegis': { id: 'spell.aegis', technique: 'art.rego', form: 'art.vim', level: 20 },
      // A General (null-level) Rego Vim spell — sorts after the fixed ReVi one.
      'spell.ward': { id: 'spell.ward', technique: 'art.creo', form: 'art.vim', level: null },
    };
    return {
      ruleset: {
        id: 't',
        version: '1',
        point_items: {},
        type_profiles: {},
        arts: artMap,
        spells,
        art_type_order: ['technique', 'form'],
        ...DERIVED_TAXONOMY,
      },
      i18n: {},
    };
  }

  it('orders selected spells Form-major, then Technique, then catalogue level, carrying original index', () => {
    // Deliberately scrambled insertion order.
    const selected = [
      { spell: 'spell.aegis' }, // ReVi 20
      { spell: 'spell.ball' }, // CrIg 35
      { spell: 'spell.ward' }, // CrVi General
      { spell: 'spell.watching' }, // ReIg 15
      { spell: 'spell.arc' }, // CrIg 10
    ];
    const ordered = orderSelectedSpells(rulesetWithCatalogue(), selected);
    // Form-major (Ignem before Vim); within a Form, Technique (Creo before Rego)
    // outranks level; then level ascending with General trailing.
    expect(ordered.map((o) => o.selection.spell)).toEqual([
      'spell.arc', // Ignem / Creo / 10
      'spell.ball', // Ignem / Creo / 35
      'spell.watching', // Ignem / Rego / 15
      'spell.ward', // Vim / Creo / General
      'spell.aegis', // Vim / Rego / 20
    ]);
    // The original indices are preserved for index-addressed row mutations.
    expect(ordered.map((o) => o.index)).toEqual([4, 1, 3, 2, 0]);
  });

  it('groups selected spells by Technique+Form, carrying original indices', () => {
    const selected = [
      { spell: 'spell.aegis' }, // ReVi 20
      { spell: 'spell.ball' }, // CrIg 35
      { spell: 'spell.watching' }, // ReIg 15
      { spell: 'spell.arc' }, // CrIg 10
    ];
    const groups = groupSelectedSpellsByTechniqueForm(rulesetWithCatalogue(), selected);
    // Same Form-major group order as the available list.
    expect(groups.map((g) => [g.technique, g.form])).toEqual([
      ['art.creo', 'art.ignem'],
      ['art.rego', 'art.ignem'],
      ['art.rego', 'art.vim'],
    ]);
    // Within a group: the orderSelectedSpells order; indices stay the entity's.
    expect(groups[0].entries.map((e) => e.selection.spell)).toEqual(['spell.arc', 'spell.ball']);
    expect(groups[0].entries.map((e) => e.index)).toEqual([3, 1]);
  });

  it('puts a spell missing from the catalogue in a trailing unlabeled group', () => {
    const selected = [{ spell: 'spell.unknown' }, { spell: 'spell.ball' }];
    const groups = groupSelectedSpellsByTechniqueForm(rulesetWithCatalogue(), selected);
    // Empty technique/form marks the group as header-less, so the row for an
    // unknown id stays visible rather than being silently dropped.
    const last = groups[groups.length - 1];
    expect(last.technique).toBe('');
    expect(last.form).toBe('');
    expect(last.entries.map((e) => e.selection.spell)).toEqual(['spell.unknown']);
  });

  it('is empty when no spells are selected', () => {
    expect(groupSelectedSpellsByTechniqueForm(rulesetWithCatalogue(), [])).toEqual([]);
  });
});

describe('usedSpellForms', () => {
  it('collects Forms used by OTHER instances of the same spell at the same level', () => {
    const spells = [
      { spell: 'spell.wizards_boost_form', level: 5, parameter: 'art.ignem' },
      { spell: 'spell.wizards_boost_form', level: 5, parameter: 'art.aquam' },
      { spell: 'spell.wizards_boost_form', level: 5, parameter: 'art.ignem' }, // self at index 2
    ];
    // For the row at index 2, its own Ignem is excluded; the other Ignem (index 0)
    // and Aquam (index 1) at the same level are reported.
    const used = usedSpellForms(spells, 'spell.wizards_boost_form', 5, 2);
    expect([...used].sort()).toEqual(['art.aquam', 'art.ignem']);
  });

  it('does not report Forms used at a different level', () => {
    const spells = [
      { spell: 'spell.wizards_boost_form', level: 5, parameter: 'art.ignem' },
      { spell: 'spell.wizards_boost_form', level: 10, parameter: 'art.aquam' },
    ];
    // Editing the level-5 row: the Aquam instance sits at level 10, so its Form
    // stays available here (the engine dedupe key is (spell, level, parameter)).
    const used = usedSpellForms(spells, 'spell.wizards_boost_form', 5, 0);
    expect([...used]).toEqual([]);
  });

  it('ignores other spells and unparameterized entries', () => {
    const spells = [
      { spell: 'spell.other', level: 5, parameter: 'art.ignem' },
      { spell: 'spell.wizards_boost_form', level: 5 },
    ];
    expect(usedSpellForms(spells, 'spell.wizards_boost_form', 5, 99).size).toBe(0);
  });
});

// `balance()` and its tests were deleted once the engine began surfacing
// `EffectiveScores.virtue_points`/`flaw_points` (audit G1, round 4). The rulebook
// worked example it pinned now lives on the Rust side only, in
// `validation/mod.rs::balance_computation` and
// `commands.rs::effective_scores_surface_virtue_flaw_balance` — one
// implementation, so there is nothing left to drift.

// --- mandatoryTraitRefs() ---------------------------------------------------

describe('mandatoryTraitRefs', () => {
  function profile(overrides: Partial<EntityTypeProfile>): EntityTypeProfile {
    return {
      id: 't',
      budget: { virtue_points: 10, flaw_points: 10 },
      permitted_categories: [],
      forbidden_categories: [],
      creation_phases: [],
      ...overrides,
    };
  }

  it('includes required traits and the required Gift', () => {
    const refs = mandatoryTraitRefs(
      profile({
        required_traits: ['virtue.hermetic_magus'],
        gift_policy: 'required',
        gift_id: 'virtue.the_gift',
      }),
    );
    expect(refs.has('virtue.hermetic_magus')).toBe(true);
    expect(refs.has('virtue.the_gift')).toBe(true);
    expect(refs.size).toBe(2);
  });

  it('omits the Gift when its policy is not required', () => {
    const refs = mandatoryTraitRefs(
      profile({ gift_policy: 'forbidden', gift_id: 'virtue.the_gift' }),
    );
    expect(refs.has('virtue.the_gift')).toBe(false);
    expect(refs.size).toBe(0);
  });

  it('is empty for a profile with no required traits or Gift', () => {
    expect(mandatoryTraitRefs(profile({})).size).toBe(0);
    expect(mandatoryTraitRefs(undefined).size).toBe(0);
  });
});

// --- incompatibleRefs() -----------------------------------------------------

describe('incompatibleRefs', () => {
  const ruleset = makeRuleset([
    item({ id: 'virtue.gentle_gift', incompatible_with: ['flaw.blatant_gift'] }),
    item({ id: 'flaw.blatant_gift', kind: 'flaw', incompatible_with: ['virtue.gentle_gift'] }),
    item({
      id: 'flaw.dwarf',
      kind: 'flaw',
      incompatible_with: ['flaw.small_frame', 'virtue.large'],
    }),
    item({
      id: 'flaw.small_frame',
      kind: 'flaw',
      incompatible_with: ['flaw.dwarf', 'virtue.large'],
    }),
    item({ id: 'virtue.large', incompatible_with: ['flaw.dwarf', 'flaw.small_frame'] }),
    item({ id: 'virtue.brave' }),
  ]);

  it('blocks the counterpart of a selected item in enforced mode', () => {
    const blocked = incompatibleRefs(ruleset, [{ ref: 'virtue.gentle_gift' }], 'enforced');
    expect(blocked.has('flaw.blatant_gift')).toBe(true);
    expect(blocked.has('virtue.brave')).toBe(false);
  });

  it('blocks every other member of a multi-way exclusion clique', () => {
    const blocked = incompatibleRefs(ruleset, [{ ref: 'flaw.dwarf' }], 'enforced');
    expect(blocked.has('flaw.small_frame')).toBe(true);
    expect(blocked.has('virtue.large')).toBe(true);
  });

  it('is empty in advisory and silent mode, where violations only get reported', () => {
    const selections = [{ ref: 'virtue.gentle_gift' }];
    expect(incompatibleRefs(ruleset, selections, 'advisory').size).toBe(0);
    expect(incompatibleRefs(ruleset, selections, 'silent').size).toBe(0);
  });

  it('ignores a selection whose ref is not in the ruleset', () => {
    expect(incompatibleRefs(ruleset, [{ ref: 'virtue.nonexistent' }], 'enforced').size).toBe(0);
  });

  it('is empty for selections that exclude nothing', () => {
    expect(incompatibleRefs(ruleset, [{ ref: 'virtue.brave' }], 'enforced').size).toBe(0);
  });

  it('names the selected item responsible for each block, for the reason tooltip', () => {
    const blocked = incompatibleRefs(ruleset, [{ ref: 'flaw.dwarf' }], 'enforced');
    expect(blocked.get('flaw.small_frame')).toBe('flaw.dwarf');
    expect(blocked.get('virtue.large')).toBe('flaw.dwarf');
    expect(blocked.get('virtue.brave')).toBeUndefined();
  });
});

// --- grantedSelectionsForSide() ---------------------------------------------

describe('grantedSelectionsForSide', () => {
  const ruleset = makeRuleset([
    item({ id: 'virtue.puissant', kind: 'virtue' }),
    item({ id: 'flaw.dark_secret', kind: 'flaw' }),
  ]);

  it('keeps only virtue/boon grants on the virtue side', () => {
    const rows = grantedSelectionsForSide(
      ruleset,
      [{ ref: 'virtue.puissant' }, { ref: 'flaw.dark_secret' }],
      'virtue',
    );
    expect(rows.map((r) => r.ref)).toEqual(['virtue.puissant']);
  });

  it('keeps only flaw/hook grants on the flaw side', () => {
    const rows = grantedSelectionsForSide(
      ruleset,
      [{ ref: 'virtue.puissant' }, { ref: 'flaw.dark_secret' }],
      'flaw',
    );
    expect(rows.map((r) => r.ref)).toEqual(['flaw.dark_secret']);
  });

  it('drops grants whose item ref is unknown, and tolerates no grants', () => {
    expect(grantedSelectionsForSide(ruleset, [{ ref: 'nope' }], 'virtue')).toEqual([]);
    expect(grantedSelectionsForSide(ruleset, undefined, 'virtue')).toEqual([]);
  });
});

// --- displayName() ----------------------------------------------------------

describe('displayName', () => {
  it('returns the i18n name when no params are needed', () => {
    const ruleset = makeRuleset([], { i18n: { 'virtue.keen_vision': { name: 'Keen Vision' } } });
    expect(displayName(ruleset, 'virtue.keen_vision')).toBe('Keen Vision');
  });

  it('substitutes a present param placeholder', () => {
    const ruleset = makeRuleset([], {
      i18n: { 'virtue.puissant': { name: 'Puissant {ability}' } },
    });
    expect(displayName(ruleset, 'virtue.puissant', { ability: 'Awareness' })).toBe(
      'Puissant Awareness',
    );
  });

  it('leaves the placeholder intact when the param is missing', () => {
    const ruleset = makeRuleset([], {
      i18n: { 'virtue.puissant': { name: 'Puissant {ability}' } },
    });
    expect(displayName(ruleset, 'virtue.puissant', {})).toBe('Puissant {ability}');
    expect(displayName(ruleset, 'virtue.puissant')).toBe('Puissant {ability}');
  });

  it('uses the placeholder-label resolver for a missing param', () => {
    const ruleset = makeRuleset([], {
      i18n: { 'virtue.puissant': { name: 'Puissant {ability}' } },
    });
    expect(displayName(ruleset, 'virtue.puissant', undefined, (key) => `(${key})`)).toBe(
      'Puissant (ability)',
    );
  });

  it('prefers a present param over the placeholder-label resolver', () => {
    const ruleset = makeRuleset([], {
      i18n: { 'virtue.puissant': { name: 'Puissant {ability}' } },
    });
    expect(
      displayName(ruleset, 'virtue.puissant', { ability: 'Awareness' }, (key) => `(${key})`),
    ).toBe('Puissant Awareness');
  });

  it('falls back to the ref when there is no i18n entry', () => {
    const ruleset = makeRuleset([], { i18n: {} });
    expect(displayName(ruleset, 'virtue.unknown')).toBe('virtue.unknown');
  });

  it('resolves a present param value through resolveValue (slug -> label)', () => {
    const ruleset = makeRuleset([], {
      i18n: { 'virtue.great': { name: 'Great {characteristic}' } },
    });
    const resolve = (_key: string, value: string) =>
      value === 'characteristic.per' ? 'Perception' : value;
    expect(
      displayName(
        ruleset,
        'virtue.great',
        { characteristic: 'characteristic.per' },
        undefined,
        resolve,
      ),
    ).toBe('Great Perception');
  });

  it('does not call resolveValue for an empty param (uses placeholder hint)', () => {
    const ruleset = makeRuleset([], {
      i18n: { 'virtue.great': { name: 'Great {characteristic}' } },
    });
    const resolve = () => 'should not be used';
    expect(displayName(ruleset, 'virtue.great', {}, (key) => `(${key})`, resolve)).toBe(
      'Great (characteristic)',
    );
  });
});

// --- displayName() and the optional unfilled form (slice 7, #13) -------------

describe('displayName with name_unfilled', () => {
  // Two catalogue entries — and only two — carry BOTH a `{token}` and a
  // parenthetical literal, which is the one shape whose hint substitution doubles:
  // "{language} (Dead Language)" + the "(Language)" hint reads
  // "(Language) (Dead Language)". They opt out by naming their unfilled form.
  const abilities = makeRuleset([], {
    i18n: {
      'ability.dead_language': {
        name: '{language} (Dead Language)',
        name_unfilled: 'Dead Language',
      },
      'ability.living_language': {
        name: '{language} (Living Language)',
        name_unfilled: 'Living Language',
      },
    },
  });
  const de = makeRuleset([], {
    i18n: {
      'ability.dead_language': {
        name: '{language} (Tote Sprache)',
        name_unfilled: 'Tote Sprache',
      },
    },
  });
  const hint = (key: string) => `(${key})`;

  it('an unfilled dead language renders its name_unfilled form', () => {
    expect(displayName(abilities, 'ability.dead_language', undefined, hint)).toBe('Dead Language');
    expect(displayName(abilities, 'ability.dead_language', {}, hint)).toBe('Dead Language');
    expect(displayName(abilities, 'ability.living_language', undefined, hint)).toBe(
      'Living Language',
    );
  });

  it('an unfilled dead language renders its German unfilled form', () => {
    expect(displayName(de, 'ability.dead_language', undefined, hint)).toBe('Tote Sprache');
  });

  it('a filled instance still uses the full template, not name_unfilled', () => {
    // The filled form is why the parenthetical stays in the template at all: "Latin"
    // alone would not say which Ability it is.
    expect(displayName(abilities, 'ability.dead_language', { language: 'Latin' }, hint)).toBe(
      'Latin (Dead Language)',
    );
  });

  // THIS GUARD MATTERS MORE THAN THE FIX ABOVE. The tempting "simplification" is to
  // suppress the placeholder for every unfilled template. That would break the ~36
  // templates where the token is the head of the name or sits mid-phrase — "Puissant
  // {ability}" would render as a bare "Puissant", "Affinity with {ability}" as
  // "Affinity with", and German worse still (a dangling inflected adjective with no
  // noun to agree with). These pass today; they are written down so that they keep
  // passing after option (d) lands, and so nobody re-proposes blanket suppression.
  it('an unfilled template without name_unfilled still renders the param hint', () => {
    const ruleset = makeRuleset([], {
      i18n: {
        'virtue.puissant_ability': { name: 'Puissant {ability}' },
        'virtue.affinity_ability': { name: 'Affinity with {ability}' },
        'virtue.great_characteristic': { name: 'Great {characteristic}' },
        'virtue.ways_of_the_land': { name: 'Ways Of The {land}' },
      },
    });
    const label = (key: string) => `(${key.charAt(0).toUpperCase()}${key.slice(1)})`;
    expect(displayName(ruleset, 'virtue.puissant_ability', undefined, label)).toBe(
      'Puissant (Ability)',
    );
    expect(displayName(ruleset, 'virtue.affinity_ability', undefined, label)).toBe(
      'Affinity with (Ability)',
    );
    expect(displayName(ruleset, 'virtue.great_characteristic', undefined, label)).toBe(
      'Great (Characteristic)',
    );
    expect(displayName(ruleset, 'virtue.ways_of_the_land', undefined, label)).toBe(
      'Ways Of The (Land)',
    );
  });
});

// --- exemplarLabel() / requirementAbilityLabel() (slice 7, #32) --------------

describe('exemplarLabel / requirementAbilityLabel', () => {
  // "Magi must have the following minimum Abilities: Parma Magica 1, Magic Theory 1,
  // Latin 1" (Core Rules :2437) names Latin, but the engine can only enforce "any
  // Dead Language", so the rules' own exemplar is surfaced as a label beside it.
  const rs = makeRuleset([], {
    i18n: {
      'ability.dead_language': {
        name: '{language} (Dead Language)',
        name_unfilled: 'Dead Language',
      },
      'ability.parma_magica': { name: 'Parma Magica' },
      'exemplar.latin': { name: 'Latin' },
    },
  });
  const t = (key: string, args?: Record<string, string>) => {
    if (key === 'requirement-exemplar') return ` (any ${args?.ability})`;
    if (key === 'param-hint') return `(${args?.label})`;
    return key;
  };
  /** The catalogue entry the parameterized Ability needs to interpolate an instance. */
  const withCatalogue = {
    ...rs,
    ruleset: {
      ...rs.ruleset,
      abilities: {
        'ability.dead_language': { id: 'ability.dead_language', parameter: 'language' },
      },
    },
  } as unknown as LocalizedRuleset;

  it('maps the exemplar slug to its localized name, never rendering the slug', () => {
    expect(exemplarLabel(rs, 'latin')).toBe('Latin');
    // An exemplar the i18n layer does not know must be dropped, not printed raw.
    expect(exemplarLabel(rs, 'greek')).toBeNull();
    expect(exemplarLabel(rs, undefined)).toBeNull();
  });

  it('heads the requirement with the exemplar, so its score follows it directly', () => {
    // `:2437` demands "Latin 1". The label used to read "Dead Language (e.g. Latin)",
    // which put the example between the Ability and its score — "Dead Language
    // (e.g. Latin) 1" reads as though "e.g. Latin" were the thing being scored.
    expect(requirementAbilityLabel(rs, 'ability.dead_language', null, 'latin', t)).toBe('Latin');
  });

  it('states the widening the engine really enforces as a trailing note', () => {
    // The note names the GENERAL Ability, never an instance, so it stays true of
    // every dead language a troupe invents.
    expect(requirementExemplarNote(rs, 'ability.dead_language', 'latin', t)).toBe(
      ' (any Dead Language)',
    );
    expect(requirementExemplarNote(withCatalogue, 'ability.dead_language', 'latin', t)).toBe(
      ' (any Dead Language)',
    );
    // No exemplar, or one the i18n layer cannot resolve: no note at all.
    expect(requirementExemplarNote(rs, 'ability.parma_magica', undefined, t)).toBe('');
    expect(requirementExemplarNote(rs, 'ability.parma_magica', 'greek', t)).toBe('');
  });

  it('leaves a requirement with no exemplar exactly as it was', () => {
    expect(requirementAbilityLabel(rs, 'ability.parma_magica', null, undefined, t)).toBe(
      'Parma Magica',
    );
    // An unresolvable exemplar likewise falls back rather than leaking a slug.
    expect(requirementAbilityLabel(rs, 'ability.parma_magica', null, 'greek', t)).toBe(
      'Parma Magica',
    );
  });

  it('keeps the bought instance when the requirement names no exemplar', () => {
    // Interpolating the instance needs the Ability's own param key from the
    // catalogue, so this case wants an `abilities` map the bare fixture omits.
    expect(requirementAbilityLabel(withCatalogue, 'ability.dead_language', 'Latin', null, t)).toBe(
      'Latin (Dead Language)',
    );
  });

  it('names the exemplar the rules named, not whichever instance was bought', () => {
    // The demand is "Latin 1" whatever dead language the character actually holds;
    // what they hold is what the message's own score says.
    expect(
      requirementAbilityLabel(withCatalogue, 'ability.dead_language', 'Greek', 'latin', t),
    ).toBe('Latin');
  });

  it('composes the exemplar into the ability arg and its note into a qualifier arg', () => {
    // The `exemplar` arg qualifies the ability rather than standing on its own, so it
    // never reaches the message under its own name: it becomes the `ability` label and
    // a `qualifier` the message places AFTER the score. That is what makes the
    // minimums row and `issue-magus_minimum_ability` read identically.
    expect(
      resolveIssueArgs(rs, { ability: 'ability.dead_language', exemplar: 'latin', min: '1' }, t),
    ).toEqual({ ability: 'Latin', min: '1', qualifier: ' (any Dead Language)' });
  });

  it('gives an ability requirement with no exemplar an empty qualifier', () => {
    // Empty, never absent: Fluent throws on a variable the args map does not carry.
    expect(resolveIssueArgs(rs, { ability: 'ability.parma_magica', min: '1' }, t)).toEqual({
      ability: 'Parma Magica',
      min: '1',
      qualifier: '',
    });
  });
});

// --- paramValueUsage() ------------------------------------------------------

describe('paramValueUsage', () => {
  const sel = (ref: string, value?: string) => ({
    ref,
    params: value ? { characteristic: value } : undefined,
  });

  it('counts other selections of the same item per value, excluding the row itself', () => {
    const selections = [
      sel('virtue.great', 'characteristic.per'),
      sel('virtue.great', 'characteristic.per'),
      sel('virtue.great', 'characteristic.str'),
      sel('virtue.other', 'characteristic.per'), // different item, ignored
    ];
    // From row 0's perspective: one other 'per' (row 1) and one 'str' (row 2).
    const usage = paramValueUsage(selections, 'virtue.great', 'characteristic', 0);
    expect(usage.get('characteristic.per')).toBe(1);
    expect(usage.get('characteristic.str')).toBe(1);
  });

  it('ignores rows with no value for the key', () => {
    const selections = [sel('virtue.great'), sel('virtue.great', 'characteristic.per')];
    const usage = paramValueUsage(selections, 'virtue.great', 'characteristic', 0);
    expect(usage.get('characteristic.per')).toBe(1);
    expect(usage.size).toBe(1);
  });
});

// --- groupByCategory() ------------------------------------------------------

describe('groupByCategory', () => {
  it('groups items by category, sorting items by localized name (not id)', () => {
    // ids are in one order; localized names invert it within each group, so a
    // name-based sort must reorder them.
    const ruleset = makeRuleset(
      [
        item({ id: 'virtue.a_general', categories: ['general'] }),
        item({ id: 'virtue.b_general', categories: ['general'] }),
        item({ id: 'virtue.m_hermetic', categories: ['hermetic'] }),
        item({ id: 'virtue.z_hermetic', categories: ['hermetic'] }),
      ],
      {
        i18n: {
          'virtue.a_general': { name: 'Zeal' },
          'virtue.b_general': { name: 'Affinity' },
          'virtue.m_hermetic': { name: 'Verditius' },
          'virtue.z_hermetic': { name: 'Bonisagus' },
        },
      },
    );
    const groups = groupByCategory(ruleset);

    expect(groups.map((g) => g.category)).toEqual(['general', 'hermetic']);
    // Sorted by name: Affinity < Zeal, Bonisagus < Verditius.
    expect(groups[0].items.map((i) => i.id)).toEqual(['virtue.b_general', 'virtue.a_general']);
    expect(groups[1].items.map((i) => i.id)).toEqual(['virtue.z_hermetic', 'virtue.m_hermetic']);
  });

  it('falls back to id ordering when names are absent', () => {
    const ruleset = makeRuleset([
      item({ id: 'virtue.b_general', categories: ['general'] }),
      item({ id: 'virtue.a_general', categories: ['general'] }),
    ]);
    expect(groupByCategory(ruleset)[0].items.map((i) => i.id)).toEqual([
      'virtue.a_general',
      'virtue.b_general',
    ]);
  });

  it('returns an empty array for an empty ruleset', () => {
    expect(groupByCategory(makeRuleset([]))).toEqual([]);
  });

  // Grouping is single-bucket and keyed on the PRIMARY category (`categories[0]`,
  // the descriptor's first-listed one), so a dual-category item shows up under
  // exactly one heading — never once per category.
  it('groups a dual-category item under its primary category only', () => {
    const ruleset = makeRuleset([
      item({ id: 'virtue.sufi', categories: ['social_status', 'supernatural'] }),
      item({ id: 'virtue.second_sight', categories: ['supernatural'] }),
    ]);
    const groups = groupByCategory(ruleset);

    expect(groups.map((g) => g.category)).toEqual(['social_status', 'supernatural']);
    expect(groups[0].items.map((i) => i.id)).toEqual(['virtue.sufi']);
    expect(groups[1].items.map((i) => i.id)).toEqual(['virtue.second_sight']);
  });

  it('keeps only items whose kind is in the given filter', () => {
    const ruleset = makeRuleset([
      item({ id: 'virtue.a', kind: 'virtue', categories: ['general'] }),
      item({ id: 'boon.b', kind: 'boon', categories: ['general'] }),
      item({ id: 'flaw.c', kind: 'flaw', categories: ['general'] }),
      item({ id: 'hook.d', kind: 'hook', categories: ['general'] }),
    ]);

    const virtues = groupByCategory(ruleset, ['virtue', 'boon']);
    expect(virtues.flatMap((g) => g.items.map((i) => i.id))).toEqual(['boon.b', 'virtue.a']);

    const flaws = groupByCategory(ruleset, ['flaw', 'hook']);
    expect(flaws.flatMap((g) => g.items.map((i) => i.id))).toEqual(['flaw.c', 'hook.d']);
  });
});

// `characteristicPointsUsed` and its tests were deleted once the engine began
// surfacing `EffectiveScores.characteristic_points_used` (audit VA1). The
// rulebook worked example it pinned (Core Rules 2358) now lives on the Rust side
// only, in `characteristics.rs::total_cost_nets_gains_against_spends` and
// `commands.rs::effective_scores_surface_the_characteristic_points_used` — one
// implementation, so there is nothing left to drift.

// --- spellMasteryXpSpent() / effectiveSpellMastery() ------------------------

// GD4 (round-2 audit): spellMasteryXpSpent() re-implements the mastery-spend
// leg of the engine's crates/arm-rules/src/effective/xp.rs::build_spends as a
// second, independent implementation (see the function's own "KNOWN DRIFT
// RISK" doc comment above its definition in derive.ts). The advancement table
// below (1->5, 2->15, 3->30) is the EXACT fixture
// crates/arm-rules/src/effective.rs's xp_ruleset() test helper uses, so the
// "mirrors effective.rs::..." test names below are literal, checkable claims,
// not just prose.
describe('spellMasteryXpSpent', () => {
  const advancement = [
    { score: 1, total_xp: 5 },
    { score: 2, total_xp: 15 },
    { score: 3, total_xp: 30 },
  ];

  it('sums the mastery-Ability XP across every known spell', () => {
    expect(spellMasteryXpSpent(advancement, [{ mastery: 3 }, { mastery: 2 }])).toBe(45);
  });

  it('treats unmastered spells (0/null/undefined) as no XP', () => {
    expect(spellMasteryXpSpent(advancement, [{ mastery: 0 }, { mastery: null }, {}])).toBe(0);
  });

  it('surfaces over-pool spending (used exceeds a 50-XP pool)', () => {
    // Two spells at mastery 3 cost 60 XP > the single Mastered Spells pool of 50.
    const used = spellMasteryXpSpent(advancement, [{ mastery: 3 }, { mastery: 3 }]);
    expect(used).toBe(60);
    expect(used > 50).toBe(true);
  });

  it('charges only above the granted floor (Flawless Magic first point free)', () => {
    // Floor 1: mastery 1 == floor is free; mastery 3 costs table(3) - table(1) = 25.
    expect(spellMasteryXpSpent(advancement, [{ mastery: 1 }, { mastery: 3 }], 1)).toBe(25);
  });

  it('halves the charge above the floor when advancement is doubled', () => {
    // Flawless Magic doubles advancement totals: (30 - 5) charged as ceil(25/2) = 13.
    expect(spellMasteryXpSpent(advancement, [{ mastery: 1 }, { mastery: 3 }], 1, true)).toBe(13);
    // Doubling with no floor: ceil(30/2) = 15.
    expect(spellMasteryXpSpent(advancement, [{ mastery: 3 }], 0, true)).toBe(15);
  });

  it('mirrors effective.rs::flawless_magic_floors_first_mastery_free_and_halves_the_rest', () => {
    // The Rust test's own worked example, verbatim: Flawless Magic (floor 1,
    // doubled advancement) on two spells mastered at 1 and 3. Mastery 1 ==
    // the floor is free; mastery 3 costs table(3) - table(1) = 25, doubled ->
    // ceil(25/2) = 13. The Rust test asserts `alloc.total_demand == 13` for
    // this exact input; this is the TS side of the same claim.
    expect(spellMasteryXpSpent(advancement, [{ mastery: 1 }, { mastery: 3 }], 1, true)).toBe(13);
  });
});

describe('spellMasteryXpSpent — 2:1-only, guarded against silent drift (V2)', () => {
  // spellMasteryXpSpent's `doubled` flag collapses ANY GrantsSpellMastery
  // advancement ratio into `Math.ceil(payable / 2)`, which is only correct for
  // exactly a 2/1 ratio (see the function's docstring). No live call site can
  // hit a different ratio TODAY — this test reads the SHIPPED
  // `rules/core/virtues_flaws.json` (not a fixture) so that claim stays true by
  // construction rather than by memory: it fails the moment any
  // `grants_spell_mastery` effect ships a genuine (num > den) reduction ratio
  // other than 2/1, forcing whoever adds one to fix this function (or finally
  // do the engine-surfaced-total fix the docstring recommends) instead of
  // shipping a silent divergence from the engine's own arithmetic.
  it('carries no GrantsSpellMastery reduction ratio other than 2/1', () => {
    const catalogue = JSON.parse(
      readFileSync(
        fileURLToPath(new URL('../../../rules/core/virtues_flaws.json', import.meta.url)),
        'utf-8',
      ),
    ) as {
      id: string;
      effects?: { type: string; advancement_num?: number; advancement_den?: number }[];
    }[];
    const reducingGrants = catalogue
      .flatMap((item) => item.effects ?? [])
      .filter((e) => e.type === 'grants_spell_mastery')
      .filter((e) => (e.advancement_num ?? 1) > (e.advancement_den ?? 1));
    // Sanity check that this test is exercising real data, not vacuously
    // passing because the catalogue carries no such grant at all — Flawless
    // Magic must still be there.
    expect(reducingGrants.length).toBeGreaterThan(0);
    for (const grant of reducingGrants) {
      expect([grant.advancement_num, grant.advancement_den]).toEqual([2, 1]);
    }
  });
});

describe('effectiveSpellMastery', () => {
  it('takes the granted floor when it beats the bought mastery', () => {
    expect(effectiveSpellMastery(0, 1)).toBe(1);
    expect(effectiveSpellMastery(null, 1)).toBe(1);
  });

  it('keeps the bought mastery when it beats the floor', () => {
    expect(effectiveSpellMastery(3, 1)).toBe(3);
    expect(effectiveSpellMastery(2, 0)).toBe(2);
  });
});

// --- minLearnableLevel() / nonTakeableReason() / isDisabled() ---------------
// V28 (tmp/review, full-audit round): these three moved out of SpellTab.svelte,
// where they lived as component-local functions alongside every other
// SpellTab-only helper (groupHeader, abbr, …), into derive.ts alongside every
// other eligibility computation (eligibleForConstraint, filterSpells, …).
// Characterization tests, written against the functions' NEW derive.ts home —
// they pin the exact behaviour the inline versions had, so the move (a pure
// code motion) cannot silently change it. `SpellTab.test.ts`'s "ritual minimum
// learnable level (VA2)" describe block exercises the same logic end-to-end
// through the rendered component and must stay green too.

describe('minLearnableLevel', () => {
  it('floors an ordinary spell at 1 regardless of the ritual minimum', () => {
    expect(minLearnableLevel({ id: 's', technique: 't', form: 'f' }, 25)).toBe(
      ORDINARY_SPELL_MINIMUM_LEVEL,
    );
  });

  it('floors a Ritual at the given ritual minimum', () => {
    expect(minLearnableLevel({ id: 's', technique: 't', form: 'f', ritual: true }, 25)).toBe(25);
  });

  it('falls back to the engine-mirrored default when no ritual minimum is given', () => {
    expect(minLearnableLevel({ id: 's', technique: 't', form: 'f', ritual: true })).toBe(
      RITUAL_MINIMUM_LEVEL_FALLBACK,
    );
  });
});

describe('nonTakeableReason', () => {
  const FIXED: Spell = { id: 'spell.fixed', technique: 'art.creo', form: 'art.animal', level: 10 };
  const GENERAL: Spell = { id: 'spell.general', technique: 'art.creo', form: 'art.animal' };
  const PARAMETRIZED: Spell = {
    id: 'spell.param',
    technique: 'art.creo',
    form: 'art.vim',
    level: 10,
    parameters: [{ key: 'form', type: 'ref', domain: 'form' }],
  };

  it('is null (takeable) with no cap, no budget shortfall, and not already taken', () => {
    expect(nonTakeableReason(FIXED, new Set(), new Map(), 100)).toBeNull();
  });

  it('blocks an already-selected fixed-level spell', () => {
    const reason = nonTakeableReason(FIXED, new Set([FIXED.id]), new Map(), 100);
    expect(reason).toEqual({ key: 'spell-already-taken-reason', cap: 0 });
  });

  it('does not block a parameterized spell already selected once (re-takeable per Form)', () => {
    expect(nonTakeableReason(PARAMETRIZED, new Set([PARAMETRIZED.id]), new Map(), 100)).toBeNull();
  });

  it('does not block a General spell already selected once (re-takeable)', () => {
    expect(nonTakeableReason(GENERAL, new Set([GENERAL.id]), new Map(), 100)).toBeNull();
  });

  it('blocks when the spell level exceeds the per-Technique/Form cap', () => {
    const cap = new Map([['art.creo art.animal', 9]]);
    expect(nonTakeableReason(FIXED, new Set(), cap, 100)).toEqual({
      key: 'spell-cap-reason',
      cap: 9,
    });
  });

  it('blocks when the spell level exceeds the remaining spell-levels budget', () => {
    expect(nonTakeableReason(FIXED, new Set(), new Map(), 9)).toEqual({
      key: 'spell-budget-reason',
      cap: 0,
    });
  });

  it('reports the cap alongside a budget-reason when a cap also applies', () => {
    const cap = new Map([['art.creo art.animal', 20]]);
    expect(nonTakeableReason(FIXED, new Set(), cap, 9)).toEqual({
      key: 'spell-budget-reason',
      cap: 20,
    });
  });

  it('tests a General spell at its minimum learnable level, not a nonexistent catalogue level', () => {
    // GENERAL has no fixed level; needed level is minLearnableLevel(GENERAL) = 1.
    expect(nonTakeableReason(GENERAL, new Set(), new Map(), 0)).toEqual({
      key: 'spell-budget-reason',
      cap: 0,
    });
    expect(nonTakeableReason(GENERAL, new Set(), new Map(), 1)).toBeNull();
  });

  it('tests a General Ritual at the given ritual minimum', () => {
    const ritual: Spell = {
      id: 'spell.gen_ritual',
      technique: 'art.creo',
      form: 'art.vim',
      ritual: true,
    };
    expect(nonTakeableReason(ritual, new Set(), new Map(), 19, 20)).toEqual({
      key: 'spell-budget-reason',
      cap: 0,
    });
    expect(nonTakeableReason(ritual, new Set(), new Map(), 20, 20)).toBeNull();
  });
});

describe('isDisabled', () => {
  const FIXED: Spell = { id: 'spell.fixed', technique: 'art.creo', form: 'art.animal', level: 10 };

  it('mirrors nonTakeableReason: false when takeable', () => {
    expect(isDisabled(FIXED, new Set(), new Map(), 100)).toBe(false);
  });

  it('mirrors nonTakeableReason: true when any reason applies', () => {
    expect(isDisabled(FIXED, new Set([FIXED.id]), new Map(), 100)).toBe(true);
  });
});

// --- groupAbilitiesByCategory() ---------------------------------------------

describe('groupAbilitiesByCategory', () => {
  function withAbilities(
    abilities: Ability[],
    i18n: LocalizedRuleset['i18n'] = {},
  ): LocalizedRuleset {
    const map: Record<string, Ability> = {};
    for (const a of abilities) map[a.id] = a;
    return {
      ruleset: {
        id: 't',
        version: '1',
        point_items: {},
        type_profiles: {},
        abilities: map,
        ...DERIVED_TAXONOMY,
      },
      i18n,
    };
  }

  it('groups by category in book order, sorting each group by localized name', () => {
    const groups = groupAbilitiesByCategory(
      withAbilities(
        [
          { id: 'ability.magic_theory', category: 'arcane' },
          { id: 'ability.swim', category: 'general' },
          { id: 'ability.awareness', category: 'general' },
          { id: 'ability.second_sight', category: 'supernatural' },
        ],
        {
          // German names: "Schwimmen" < "Aufmerksamkeit"? No — A < S, so Awareness
          // (Aufmerksamkeit) sorts before Swim (Schwimmen).
          'ability.swim': { name: 'Schwimmen' },
          'ability.awareness': { name: 'Aufmerksamkeit' },
        },
      ),
    );
    expect(groups.map((g) => g.category)).toEqual(['general', 'arcane', 'supernatural']);
    expect(groups[0].abilities.map((a) => a.id)).toEqual(['ability.awareness', 'ability.swim']);
  });

  it('sorts by name even when it inverts id order', () => {
    const groups = groupAbilitiesByCategory(
      withAbilities(
        [
          { id: 'ability.awareness', category: 'general' },
          { id: 'ability.bargain', category: 'general' },
        ],
        {
          'ability.awareness': { name: 'Wachsamkeit' },
          'ability.bargain': { name: 'Feilschen' },
        },
      ),
    );
    // Feilschen < Wachsamkeit, so bargain comes first despite the id order.
    expect(groups[0].abilities.map((a) => a.id)).toEqual(['ability.bargain', 'ability.awareness']);
  });

  it('is empty when the ruleset has no abilities', () => {
    expect(groupAbilitiesByCategory(withAbilities([]))).toEqual([]);
  });
});

// --- maxAbilityScore() ------------------------------------------------------

describe('maxAbilityScore', () => {
  it('returns 0 for undefined or empty advancement', () => {
    expect(maxAbilityScore(undefined)).toBe(0);
    expect(maxAbilityScore([])).toBe(0);
  });

  it('returns the highest score the table can price', () => {
    expect(maxAbilityScore([{ score: 1 }, { score: 5 }, { score: 3 }])).toBe(5);
  });
});

// --- abilityDisplayName() ---------------------------------------------------

describe('abilityDisplayName', () => {
  function withAbilityNames(
    abilities: Ability[],
    i18n: LocalizedRuleset['i18n'],
  ): LocalizedRuleset {
    const map: Record<string, Ability> = {};
    for (const a of abilities) map[a.id] = a;
    return {
      ruleset: {
        id: 't',
        version: '1',
        point_items: {},
        type_profiles: {},
        abilities: map,
        ...DERIVED_TAXONOMY,
      },
      i18n,
    };
  }

  const placeholder = (key: string) => `(${key})`;

  it('interpolates the parameter value for a parameterized ability', () => {
    const ruleset = withAbilityNames(
      [{ id: 'ability.area_lore', category: 'general', parameter: 'area' }],
      {
        'ability.area_lore': { name: '{area} Lore' },
      },
    );
    expect(abilityDisplayName(ruleset, 'ability.area_lore', 'Rhine', placeholder)).toBe(
      'Rhine Lore',
    );
  });

  it('uses the placeholder label when the param value is empty', () => {
    const ruleset = withAbilityNames(
      [{ id: 'ability.area_lore', category: 'general', parameter: 'area' }],
      {
        'ability.area_lore': { name: '{area} Lore' },
      },
    );
    expect(abilityDisplayName(ruleset, 'ability.area_lore', '', placeholder)).toBe('(area) Lore');
    expect(abilityDisplayName(ruleset, 'ability.area_lore', null, placeholder)).toBe('(area) Lore');
  });

  it('returns the plain name for a non-parameterized ability', () => {
    const ruleset = withAbilityNames([{ id: 'ability.awareness', category: 'general' }], {
      'ability.awareness': { name: 'Awareness' },
    });
    expect(abilityDisplayName(ruleset, 'ability.awareness', 'ignored', placeholder)).toBe(
      'Awareness',
    );
  });
});

// --- spellDisplayName() -----------------------------------------------------

describe('spellDisplayName', () => {
  function withSpells(): LocalizedRuleset {
    return {
      ruleset: {
        id: 't',
        version: '1',
        point_items: {},
        type_profiles: {},
        spells: {
          'spell.wizards_boost_form': {
            id: 'spell.wizards_boost_form',
            technique: 'art.muto',
            form: 'art.vim',
            level: null,
            parameters: [{ key: 'form', type: 'ref', domain: 'form' }],
          },
          'spell.pilum': {
            id: 'spell.pilum',
            technique: 'art.creo',
            form: 'art.ignem',
            level: 20,
          },
        },
        ...DERIVED_TAXONOMY,
      },
      i18n: {
        'art.ignem': { name: 'Ignem' },
        // The parametrized name is a template; the literal parens belong to the
        // template, so the placeholder hint must NOT add its own.
        'spell.wizards_boost_form': { name: "Wizard's Boost ({form})" },
        'spell.pilum': { name: 'Pilum of Fire' },
      },
    };
  }

  // The source-list hint: the plain param label (the template supplies the parens).
  const hint = (key: string) => (key === 'form' ? 'Form' : key);

  it('interpolates the chosen target Form as its localized Art name', () => {
    expect(spellDisplayName(withSpells(), 'spell.wizards_boost_form', 'art.ignem', hint)).toBe(
      "Wizard's Boost (Ignem)",
    );
  });

  it('shows the localized param hint when no Form is chosen (source candidate)', () => {
    expect(spellDisplayName(withSpells(), 'spell.wizards_boost_form', null, hint)).toBe(
      "Wizard's Boost (Form)",
    );
    expect(spellDisplayName(withSpells(), 'spell.wizards_boost_form', '', hint)).toBe(
      "Wizard's Boost (Form)",
    );
  });

  it('returns the plain name for a spell with no parameters', () => {
    expect(spellDisplayName(withSpells(), 'spell.pilum', 'art.ignem', hint)).toBe('Pilum of Fire');
  });
});

// --- abilityLabel() ---------------------------------------------------------

describe('abilityLabel', () => {
  function withAbilityNames(
    abilities: Ability[],
    i18n: LocalizedRuleset['i18n'],
  ): LocalizedRuleset {
    const map: Record<string, Ability> = {};
    for (const a of abilities) map[a.id] = a;
    return {
      ruleset: {
        id: 't',
        version: '1',
        point_items: {},
        type_profiles: {},
        abilities: map,
        ...DERIVED_TAXONOMY,
      },
      i18n,
    };
  }

  const placeholder = (key: string) => `(${key})`;

  it('appends the marker for an asterisked (requires_training) ability', () => {
    const ruleset = withAbilityNames(
      [{ id: 'ability.second_sight', category: 'supernatural', requires_training: true }],
      {
        'ability.second_sight': { name: 'Second Sight' },
      },
    );
    expect(abilityLabel(ruleset, 'ability.second_sight', undefined, placeholder, '*')).toBe(
      'Second Sight*',
    );
  });

  it('marks an asterisked ability that is not Supernatural (driven by the flag, not category)', () => {
    const ruleset = withAbilityNames(
      [{ id: 'ability.artes_liberales', category: 'academic', requires_training: true }],
      { 'ability.artes_liberales': { name: 'Artes Liberales' } },
    );
    expect(abilityLabel(ruleset, 'ability.artes_liberales', undefined, placeholder, '*')).toBe(
      'Artes Liberales*',
    );
  });

  it('does not mark an ability usable untrained', () => {
    const ruleset = withAbilityNames([{ id: 'ability.awareness', category: 'general' }], {
      'ability.awareness': { name: 'Awareness' },
    });
    expect(abilityLabel(ruleset, 'ability.awareness', undefined, placeholder, '*')).toBe(
      'Awareness',
    );
  });
});

// --- Arts: groupArtsByType / artLabel / artAbbreviation / xp ----------------

describe('art helpers', () => {
  function withArts(arts: Art[], i18n: LocalizedRuleset['i18n'] = {}): LocalizedRuleset {
    const map: Record<string, Art> = {};
    for (const a of arts) map[a.id] = a;
    return {
      ruleset: {
        id: 't',
        version: '1',
        point_items: {},
        type_profiles: {},
        arts: map,
        art_advancement: [
          { score: 1, total_xp: 1 },
          { score: 2, total_xp: 3 },
          { score: 3, total_xp: 6 },
          { score: 4, total_xp: 10 },
          { score: 5, total_xp: 15 },
        ],
        art_type_order: ['technique', 'form'],
        ...DERIVED_TAXONOMY,
      },
      i18n,
    };
  }

  it('groups Techniques before Forms, sorting each by localized name', () => {
    const groups = groupArtsByType(
      withArts(
        [
          { id: 'art.ignem', art_type: 'form' },
          { id: 'art.creo', art_type: 'technique' },
          { id: 'art.animal', art_type: 'form' },
          { id: 'art.rego', art_type: 'technique' },
        ],
        {
          'art.ignem': { name: 'Ignem' },
          'art.creo': { name: 'Creo' },
          'art.animal': { name: 'Animal' },
          'art.rego': { name: 'Rego' },
        },
      ),
    );
    expect(groups.map((g) => g.artType)).toEqual(['technique', 'form']);
    expect(groups[0].arts.map((a) => a.id)).toEqual(['art.creo', 'art.rego']);
    expect(groups[1].arts.map((a) => a.id)).toEqual(['art.animal', 'art.ignem']);
  });

  it('is empty when the ruleset has no arts', () => {
    expect(groupArtsByType(withArts([]))).toEqual([]);
  });

  it('resolves the localized name and abbreviation', () => {
    const rs = withArts([{ id: 'art.creo', art_type: 'technique' }], {
      'art.creo': { name: 'Creo', abbreviation: 'Cr' },
    });
    expect(artLabel(rs, 'art.creo')).toBe('Creo');
    expect(artAbbreviation(rs, 'art.creo')).toBe('Cr');
    // Missing abbreviation falls back to empty, name falls back to the id.
    expect(artAbbreviation(rs, 'art.unknown')).toBe('');
    expect(artLabel(rs, 'art.unknown')).toBe('art.unknown');
  });

  it('reports the highest whole score the Art advancement table can price', () => {
    const rs = withArts([]);
    expect(maxArtScore(rs.ruleset.art_advancement)).toBe(5);
  });
});

describe('restrictedPoolLabel', () => {
  /// Every V/F-granted pool names its granting item; the label ignores it and reads
  /// the eligibility instead, so which item it is does not matter here.
  const itemOrigin = { kind: 'item', item: 'virtue.educated' } as const;
  const t = (key: string) =>
    (
      ({
        'ability-category-martial': 'Martial',
        'ability-category-academic': 'Academic',
        'restricted-xp-list-separator': ',',
      }) as Record<string, string>
    )[key] ?? key;

  it('labels a category pool by localized category names', () => {
    const rs = makeRuleset([]);
    const label = restrictedPoolLabel(
      rs,
      { amount: 50, used: 0, categories: ['martial'], origin: itemOrigin },
      t,
    );
    expect(label).toBe('Martial');
  });

  it('labels a multi-category pool, separator-joined', () => {
    const rs = makeRuleset([]);
    const label = restrictedPoolLabel(
      rs,
      { amount: 50, used: 0, categories: ['academic', 'martial'], origin: itemOrigin },
      t,
    );
    expect(label).toBe('Academic, Martial');
  });

  it('labels an ability pool by localized ability names', () => {
    const rs = makeRuleset([], {
      i18n: {
        'ability.latin': { name: 'Latin' },
        'ability.artes_liberales': { name: 'Artes Liberales' },
      },
    });
    const label = restrictedPoolLabel(
      rs,
      {
        amount: 50,
        used: 30,
        abilities: ['ability.latin', 'ability.artes_liberales'],
        origin: itemOrigin,
      },
      t,
    );
    expect(label).toBe('Latin, Artes Liberales');
  });

  // B1: a parameterized ability in a restricted pool must render its localized
  // param hint, never the literal "{language}" token.
  it('renders a parameterized ability with its param hint, not the raw token', () => {
    const th = (key: string, args?: Record<string, string>) => {
      if (key === 'param-hint') return `(${args?.label})`;
      if (key === 'param-label-language') return 'Language';
      if (key === 'restricted-xp-list-separator') return ',';
      return key;
    };
    const rs = makeRuleset([], {
      i18n: {
        'ability.artes_liberales': { name: 'Artes Liberales' },
        'ability.living_language': { name: '{language}' },
      },
    });
    const label = restrictedPoolLabel(
      rs,
      {
        amount: 50,
        used: 0,
        abilities: ['ability.artes_liberales', 'ability.living_language'],
        origin: itemOrigin,
      },
      th,
    );
    expect(label).toBe('Artes Liberales, (Language)');
    expect(label).not.toContain('{language}');
  });

  // A life-stage block is named for what it IS. Listing its eligible abilities
  // would print all eleven childhood Abilities, and could not tell childhood's two
  // blocks apart — both list childhood Abilities.
  it('labels a life-stage block through its own Fluent key', () => {
    const tl = (key: string) =>
      (
        ({
          'xp-pool-childhood_native_language': 'Native language',
          'xp-pool-childhood_spread': 'Childhood',
        }) as Record<string, string>
      )[key] ?? key;
    const rs = makeRuleset([], { i18n: { 'ability.swim': { name: 'Swim' } } });
    expect(
      restrictedPoolLabel(
        rs,
        {
          amount: 75,
          used: 75,
          origin: { kind: 'life_stage', block: 'childhood_native_language' },
        },
        tl,
      ),
    ).toBe('Native language');
    expect(
      restrictedPoolLabel(
        rs,
        {
          amount: 45,
          used: 0,
          abilities: ['ability.swim'],
          origin: { kind: 'life_stage', block: 'childhood_spread' },
        },
        tl,
      ),
    ).toBe('Childhood');
  });
});

// --- B2: search matches the localized/rendered label, not the raw template ---

describe('filterAbilities localized-label search', () => {
  const abilities: Ability[] = [
    { id: 'ability.living_language', category: 'general', parameter: 'language' },
    { id: 'ability.awareness', category: 'general' },
  ];
  // param-hint renders "(<label>)"; param-label-language differs per language.
  const makeT = (languageLabel: string) => (key: string, args?: Record<string, string>) => {
    if (key === 'param-hint') return `(${args?.label})`;
    if (key === 'param-label-language') return languageLabel;
    return key;
  };

  it('matches the German rendered label ("Sprache") for the language ability', () => {
    const rs = makeRuleset([], {
      i18n: {
        'ability.living_language': { name: '{language}' },
        'ability.awareness': { name: 'Aufmerksamkeit' },
      },
    });
    expect(
      filterAbilities(rs, abilities, { text: 'spra' }, makeT('Sprache')).map((a) => a.id),
    ).toEqual(['ability.living_language']);
  });

  it('still matches the English token/label ("langu")', () => {
    const rs = makeRuleset([], {
      i18n: {
        'ability.living_language': { name: '{language}' },
        'ability.awareness': { name: 'Awareness' },
      },
    });
    expect(
      filterAbilities(rs, abilities, { text: 'langu' }, makeT('Language')).map((a) => a.id),
    ).toEqual(['ability.living_language']);
  });

  it('still matches a plain ability by its localized name', () => {
    const rs = makeRuleset([], {
      i18n: {
        'ability.living_language': { name: '{language}' },
        'ability.awareness': { name: 'Aufmerksamkeit' },
      },
    });
    expect(
      filterAbilities(rs, abilities, { text: 'aufmerk' }, makeT('Sprache')).map((a) => a.id),
    ).toEqual(['ability.awareness']);
  });
});

// --- B3: validation-issue arg label resolution ------------------------------

describe('resolveIssueArgValue / resolveIssueArgs', () => {
  const t = (key: string, args?: Record<string, string>) => {
    const table: Record<string, string> = {
      'characteristic-int': 'Intelligence',
      'reputation-type-local': 'Local',
      'realm-magic': 'Magic',
      'param-label-area': 'Area',
      'param-label-language': 'Language',
      'xp-pool-childhood_spread': 'Early childhood',
      'xp-pool-childhood_native_language': 'Native language',
      'category-supernatural': 'Supernatural',
    };
    if (key === 'param-hint') return `(${args?.label})`;
    return table[key] ?? key;
  };

  it('resolves an id-valued arg to its localized name', () => {
    const rs = makeRuleset([], { i18n: { 'ability.awareness': { name: 'Awareness' } } });
    expect(resolveIssueArgValue(rs, 'ability', 'ability.awareness', t)).toBe('Awareness');
  });

  it('resolves a parameterized id with its param hint, not the raw token', () => {
    const rs = makeRuleset([], { i18n: { 'ability.area_lore': { name: '{area} Lore' } } });
    expect(resolveIssueArgValue(rs, 'ability', 'ability.area_lore', t)).toBe('(Area) Lore');
  });

  it('resolves an enum-valued characteristic arg through its Fluent key', () => {
    const rs = makeRuleset([]);
    expect(resolveIssueArgValue(rs, 'characteristic', 'int', t)).toBe('Intelligence');
  });

  it('resolves a reputation kind and a Might realm through their Fluent keys', () => {
    const rs = makeRuleset([]);
    expect(resolveIssueArgValue(rs, 'kind', 'local', t)).toBe('Local');
    expect(resolveIssueArgValue(rs, 'base', 'magic', t)).toBe('Magic');
  });

  it('leaves a numeric enum-keyed arg (e.g. a base score) untouched', () => {
    const rs = makeRuleset([]);
    expect(resolveIssueArgValue(rs, 'base', '3', t)).toBe('3');
  });

  // `category_not_permitted` / `forbidden_category` carry the offending category
  // as a raw slug (`validation/selections.rs`), which is not a rules id and so has
  // no i18n entry — without its own prefix it printed straight into the message.
  it('resolves a category arg through its category Fluent key', () => {
    const rs = makeRuleset([]);
    expect(resolveIssueArgValue(rs, 'category', 'supernatural', t)).toBe('Supernatural');
  });

  it('resolves a param key arg through its param-label', () => {
    const rs = makeRuleset([]);
    expect(resolveIssueArgValue(rs, 'key', 'language', t)).toBe('Language');
  });

  // `restricted_xp_unspent` names the pool it is about, and the pool's origin is
  // either a granting item or a life-stage block. The engine emits both as machine
  // names, so both have to arrive as a label — a life-stage block through the same
  // `xp-pool-<block>` keys the XP bar's own pool label uses, so one wording names
  // the block wherever it appears.
  it('resolves an item-granted pool origin to the granting item name', () => {
    const rs = makeRuleset([item({ id: 'virtue.educated' })], {
      i18n: { 'virtue.educated': { name: 'Educated' } },
    });
    expect(resolveIssueArgValue(rs, 'origin', 'virtue.educated', t)).toBe('Educated');
  });

  it('resolves a life-stage pool origin through its xp-pool Fluent key', () => {
    const rs = makeRuleset([]);
    expect(resolveIssueArgValue(rs, 'origin', 'childhood_spread', t)).toBe('Early childhood');
    expect(resolveIssueArgValue(rs, 'origin', 'childhood_native_language', t)).toBe(
      'Native language',
    );
    // The block slug must never reach the screen.
    expect(resolveIssueArgValue(rs, 'origin', 'childhood_spread', t)).not.toContain(
      'childhood_spread',
    );
  });

  it('passes free text and numbers through unchanged', () => {
    const rs = makeRuleset([]);
    expect(resolveIssueArgValue(rs, 'name', 'Brave', t)).toBe('Brave');
    expect(resolveIssueArgValue(rs, 'content', 'A hidden shame', t)).toBe('A hidden shame');
    expect(resolveIssueArgValue(rs, 'score', '5', t)).toBe('5');
  });

  it('resolves a whole args map, arg by arg', () => {
    const rs = makeRuleset([], { i18n: { 'ability.area_lore': { name: '{area} Lore' } } });
    expect(
      resolveIssueArgs(rs, { ability: 'ability.area_lore', characteristic: 'int', score: '5' }, t),
    ).toEqual({
      ability: '(Area) Lore',
      characteristic: 'Intelligence',
      score: '5',
      // Every `ability` arg carries a qualifier, empty when the requirement names no
      // exemplar — Fluent throws on a variable the args map does not carry.
      qualifier: '',
    });
  });
});

// --- C4: grouping + sorting selected V/F and abilities ----------------------

describe('groupSelectionsByCategory', () => {
  const rs = makeRuleset(
    [
      item({ id: 'virtue.zeal', categories: ['general'] }),
      item({ id: 'virtue.affinity', categories: ['general'] }),
      item({ id: 'virtue.verditius', categories: ['hermetic'] }),
    ],
    {
      i18n: {
        'virtue.zeal': { name: 'Zeal' },
        'virtue.affinity': { name: 'Affinity' },
        'virtue.verditius': { name: 'Verditius' },
      },
    },
  );

  /** The bought rows' entity indices, for a group. */
  function boughtIndices(group: {
    rows: { kind: string; index?: number }[];
  }): (number | undefined)[] {
    return group.rows.filter((r) => r.kind === 'sel').map((r) => r.index);
  }

  it('groups selections by category and sorts each group by localized name', () => {
    // add-order: Zeal (general), Verditius (hermetic), Affinity (general).
    const entries = [
      { selection: { ref: 'virtue.zeal' }, index: 0 },
      { selection: { ref: 'virtue.verditius' }, index: 1 },
      { selection: { ref: 'virtue.affinity' }, index: 2 },
    ];
    const groups = groupSelectionsByCategory(rs, entries);
    expect(groups.map((g) => g.category)).toEqual(['general', 'hermetic']);
    // general sorted by name: Affinity (idx 2) before Zeal (idx 0).
    expect(boughtIndices(groups[0])).toEqual([2, 0]);
    expect(boughtIndices(groups[1])).toEqual([1]);
  });

  it('drops entries whose item ref is unknown', () => {
    const groups = groupSelectionsByCategory(rs, [{ selection: { ref: 'nope' }, index: 0 }]);
    expect(groups).toEqual([]);
  });

  // guided-creation-review-2026-08 #9: granted V/F used to be pushed into a
  // header-less group of their own BELOW the chosen ones, which put them under
  // whichever category heading happened to sort last — a Hermetic granted Virtue
  // read as Supernatural. They belong to their OWN category, like any other row.
  it('places a granted selection in its own category', () => {
    const groups = groupSelectionsByCategory(
      rs,
      [{ selection: { ref: 'virtue.zeal' }, index: 0 }],
      [{ ref: 'virtue.verditius' }],
    );
    expect(groups.map((g) => g.category)).toEqual(['general', 'hermetic']);
    expect(groups[1].rows).toEqual([
      { kind: 'granted', selection: { ref: 'virtue.verditius' }, grantIndex: 0 },
    ]);
  });

  // A granted row is ordered like any other row, so the within-group sort must
  // cover both kinds — otherwise the granted ones clump at one end of the group
  // and the list stops being alphabetical.
  it('sorts granted and bought rows together by localized name', () => {
    const groups = groupSelectionsByCategory(
      rs,
      [{ selection: { ref: 'virtue.zeal' }, index: 0 }],
      [{ ref: 'virtue.affinity' }],
    );
    expect(groups.map((g) => g.category)).toEqual(['general']);
    // Affinity (granted) sorts before Zeal (bought) — by name, not by kind.
    expect(groups[0].rows).toEqual([
      { kind: 'granted', selection: { ref: 'virtue.affinity' }, grantIndex: 0 },
      { kind: 'sel', selection: { ref: 'virtue.zeal' }, index: 0 },
    ]);
  });

  // The engine concatenates House, Mythic Companion, `grants_selection` and
  // warping grants without dedup (`effective.rs` `entity_grants`), so the same
  // ref can arrive twice. Both rows must survive, each with its own position, so
  // the caller can key them apart (a duplicate `{#each}` key throws).
  it('keeps each granted row of a doubly-granted ref, with its own position', () => {
    const groups = groupSelectionsByCategory(
      rs,
      [],
      [{ ref: 'virtue.verditius' }, { ref: 'virtue.verditius' }],
    );
    expect(groups[0].rows).toEqual([
      { kind: 'granted', selection: { ref: 'virtue.verditius' }, grantIndex: 0 },
      { kind: 'granted', selection: { ref: 'virtue.verditius' }, grantIndex: 1 },
    ]);
  });

  it('drops a granted row whose item ref is unknown', () => {
    expect(groupSelectionsByCategory(rs, [], [{ ref: 'nope' }])).toEqual([]);
  });

  // `VirtueFlawTab` addresses a bought row's removal by its `entity.selections`
  // index, so a dual-category item must yield exactly ONE row — duplicating it
  // under a second heading would give two rows the same index.
  it('lists a bought dual-category row once, under its primary category', () => {
    const dualRs = makeRuleset([
      item({ id: 'virtue.sufi', categories: ['social_status', 'supernatural'] }),
    ]);
    const groups = groupSelectionsByCategory(dualRs, [
      { selection: { ref: 'virtue.sufi' }, index: 0 },
    ]);

    expect(groups.map((g) => g.category)).toEqual(['social_status']);
    expect(groups.flatMap((g) => g.rows)).toEqual([
      { kind: 'sel', selection: { ref: 'virtue.sufi' }, index: 0 },
    ]);
  });
});

describe('groupAbilitySelectionsByCategory', () => {
  function withAbilities(abilities: Ability[], i18n: LocalizedRuleset['i18n']): LocalizedRuleset {
    const map: Record<string, Ability> = {};
    for (const a of abilities) map[a.id] = a;
    return {
      ruleset: {
        id: 't',
        version: '1',
        point_items: {},
        type_profiles: {},
        abilities: map,
        ...DERIVED_TAXONOMY,
      },
      i18n,
    };
  }

  const rs = withAbilities(
    [
      { id: 'ability.magic_theory', category: 'arcane' },
      { id: 'ability.awareness', category: 'general' },
      { id: 'ability.swim', category: 'general' },
    ],
    {
      'ability.magic_theory': { name: 'Magic Theory' },
      'ability.awareness': { name: 'Awareness' },
      'ability.swim': { name: 'Swim' },
    },
  );

  it('groups in category order and sorts each group by localized name, keeping indices', () => {
    // add-order: Swim(general,0), Magic Theory(arcane,1), Awareness(general,2).
    const entries = [
      { entry: { ability: 'ability.swim', score: 2 }, index: 0 },
      { entry: { ability: 'ability.magic_theory', score: 3 }, index: 1 },
      { entry: { ability: 'ability.awareness', score: 1 }, index: 2 },
    ];
    const groups = groupAbilitySelectionsByCategory(rs, entries);
    // ability_category_order: general before arcane.
    expect(groups.map((g) => g.category)).toEqual(['general', 'arcane']);
    // general sorted by name: Awareness (idx 2) before Swim (idx 0).
    expect(groups[0].entries.map((e) => e.index)).toEqual([2, 0]);
    expect(groups[1].entries.map((e) => e.index)).toEqual([1]);
  });

  it('drops entries whose ability id is unknown', () => {
    const groups = groupAbilitySelectionsByCategory(rs, [
      { entry: { ability: 'ability.nope', score: 1 }, index: 0 },
    ]);
    expect(groups).toEqual([]);
  });
});

// Issue 17: the engine reports a Puissant bonus (and a granted floor) for an
// Ability the character has not bought, but the Selected list is built from bought
// rows, so the Abilities surface said nothing about it. These are the rows that
// carry it — display-only, hence UNBOUGHT_ROW_INDEX rather than an entity index.
describe('unboughtModifiedAbilities', () => {
  const bought = [{ ability: 'ability.awareness', score: 2 }];

  it('yields a display-only row for a bonus with no bought score', () => {
    expect(
      unboughtModifiedAbilities(bought, [{ ability: 'ability.magic_theory', bonus: 2 }], []),
    ).toEqual([
      { entry: { ability: 'ability.magic_theory', score: 0 }, index: UNBOUGHT_ROW_INDEX },
    ]);
  });

  it('yields one for a granted free floor with no bought score', () => {
    // Second Sight's grant is the same shape of "the character has it and the tab
    // is silent", so the floor path gets the same row.
    expect(unboughtModifiedAbilities(bought, [], [{ ability: 'ability.swim', floor: 1 }])).toEqual([
      { entry: { ability: 'ability.swim', score: 0 }, index: UNBOUGHT_ROW_INDEX },
    ]);
  });

  it('omits an ability that already has a bought row to hang the badge on', () => {
    expect(
      unboughtModifiedAbilities(
        bought,
        [{ ability: 'ability.awareness', bonus: 2 }],
        [{ ability: 'ability.awareness', floor: 1 }],
      ),
    ).toEqual([]);
  });

  it('emits one row when a bonus and a floor name the same unbought ability', () => {
    expect(
      unboughtModifiedAbilities(
        [],
        [{ ability: 'ability.swim', bonus: 2 }],
        [{ ability: 'ability.swim', floor: 1 }],
      ),
    ).toEqual([{ entry: { ability: 'ability.swim', score: 0 }, index: UNBOUGHT_ROW_INDEX }]);
  });

  it('omits a parameterized instance, which the row could not name', () => {
    // The engine only reports a parameterized instance that IS bought (its
    // catalogue entries all carry no parameter), so such a row never needs
    // inventing — and could not be rendered, having no parameter field to edit.
    expect(
      unboughtModifiedAbilities(
        [{ ability: 'ability.area_lore', score: 2, parameter: 'Brandenburg' }],
        [{ ability: 'ability.area_lore', parameter: 'Bavaria', bonus: 2 }],
        [],
      ),
    ).toEqual([]);
  });
});

describe('filterEquipment', () => {
  const catalogue = {
    weapons: {
      'weapon.axe': { id: 'weapon.axe' },
      'weapon.long_sword': { id: 'weapon.long_sword' },
    },
    shields: { 'shield.round': { id: 'shield.round' } },
    armor: { 'armor.leather': { id: 'armor.leather' } },
  };
  const i18n = {
    'weapon.axe': { name: 'Axe' },
    'weapon.long_sword': { name: 'Long Sword' },
    'shield.round': { name: 'Round Shield' },
    'armor.leather': { name: 'Leather Scale' },
  };
  const rs = makeRuleset([], { i18n });

  it('returns all three kinds (weapons, shields, armor) sorted by localized name', () => {
    const groups = filterEquipment(rs, catalogue, {});
    expect(groups.map((g) => g.kind)).toEqual(['weapons', 'shields', 'armor']);
    expect(groups[0].ids).toEqual(['weapon.axe', 'weapon.long_sword']);
  });

  it('filters by localized name text across kinds, dropping empty groups', () => {
    const groups = filterEquipment(rs, catalogue, { text: 'leather' });
    expect(groups.map((g) => g.kind)).toEqual(['armor']);
    expect(groups[0].ids).toEqual(['armor.leather']);
  });

  it('restricts to a single kind when the kind facet is set', () => {
    const groups = filterEquipment(rs, catalogue, { kind: 'shields' });
    expect(groups.map((g) => g.kind)).toEqual(['shields']);
    expect(groups[0].ids).toEqual(['shield.round']);
  });
});

describe('combatRowLabel', () => {
  it('names the weapon alone when the line carries no shield', () => {
    expect(combatRowLabel('Long Sword', [], '&')).toBe('Long Sword');
  });

  it('joins the weapon to the shield whose modifiers the line folded in', () => {
    expect(combatRowLabel('Long Sword', ['Round Shield'], '&')).toBe('Long Sword & Round Shield');
  });

  it('lists every shield when several are equipped and summed', () => {
    expect(combatRowLabel('Long Sword', ['Round Shield', 'Buckler'], '&')).toBe(
      'Long Sword & Round Shield & Buckler',
    );
  });

  it('uses the localized joiner verbatim', () => {
    expect(combatRowLabel('Langschwert', ['Tartsche'], 'und')).toBe('Langschwert und Tartsche');
  });
});

describe('groupSelectedEquipmentByKind', () => {
  const catalogue = {
    weapons: {
      'weapon.axe': { id: 'weapon.axe' },
      'weapon.long_sword': { id: 'weapon.long_sword' },
    },
    shields: { 'shield.round': { id: 'shield.round' } },
    armor: { 'armor.leather': { id: 'armor.leather' } },
  };
  const i18n = {
    'weapon.axe': { name: 'Axe' },
    'weapon.long_sword': { name: 'Long Sword' },
    'shield.round': { name: 'Round Shield' },
    'armor.leather': { name: 'Leather Scale' },
  };
  const rs = makeRuleset([], { i18n });

  it('groups carried equipment by kind in display order, name-sorted, keeping indices', () => {
    const slots = [
      { item: 'armor.leather' },
      { item: 'weapon.long_sword' },
      { item: 'shield.round' },
      { item: 'weapon.axe' },
    ];
    const groups = groupSelectedEquipmentByKind(rs, catalogue, slots);
    // Same weapons → shields → armor order as the available list; empty dropped.
    expect(groups.map((g) => g.kind)).toEqual(['weapons', 'shields', 'armor']);
    // Alpha by localized name within a kind (mirroring the Abilities selected list).
    expect(groups[0].entries.map((e) => e.slot.item)).toEqual(['weapon.axe', 'weapon.long_sword']);
    // Original entity indices ride along for the index-addressed row mutators.
    expect(groups[0].entries.map((e) => e.index)).toEqual([3, 1]);
  });

  it('puts an item missing from the catalogue in a trailing unlabeled group', () => {
    const slots = [{ item: 'weapon.mystery' }, { item: 'weapon.axe' }];
    const groups = groupSelectedEquipmentByKind(rs, catalogue, slots);
    // A null kind marks the group as header-less, so an unknown item's row stays
    // visible instead of vanishing from the selected list.
    const last = groups[groups.length - 1];
    expect(last.kind).toBeNull();
    expect(last.entries.map((e) => e.slot.item)).toEqual(['weapon.mystery']);
  });

  it('is empty when nothing is carried', () => {
    expect(groupSelectedEquipmentByKind(rs, catalogue, [])).toEqual([]);
  });
});

describe('spellLevelAllocation', () => {
  it('spends a positive V/F bonus before the base, like the engine drains restricted XP first', () => {
    // Base 120 + Skilled Parens 30 = a 150 budget, of which 30 levels are used.
    const a = spellLevelAllocation(30, 150, 30);
    expect(a.base).toBe(120);
    // The bonus covers all 30, so nothing is charged to the base...
    expect(a.bonusUsed).toBe(30);
    expect(a.baseUsed).toBe(0);
    // ...and Available is the base remainder, so the line closes: 120 - 0.
    expect(a.available).toBe(120);
  });

  it('leaves unspent bonus levels in the bonus entry, not in Available', () => {
    // Only 10 of the 30 bonus levels are used; the other 20 stay visible as the
    // bonus entry's remainder rather than inflating Available (XP-bar semantics).
    const a = spellLevelAllocation(10, 150, 30);
    expect(a.bonusUsed).toBe(10);
    expect(a.bonusAmount).toBe(30);
    expect(a.baseUsed).toBe(0);
    expect(a.available).toBe(120);
  });

  it('charges the overflow above the bonus to the base', () => {
    // 45 used: the bonus absorbs its 30, the remaining 15 come from the base.
    const a = spellLevelAllocation(45, 150, 30);
    expect(a.bonusUsed).toBe(30);
    expect(a.baseUsed).toBe(15);
    expect(a.available).toBe(105);
  });

  it('reports a negative Available when the whole budget is overspent', () => {
    // 160 used against base 120 + bonus 30: 30 from the bonus, 130 charged to the
    // base, so Available is -10 — the same figure as budget - used.
    const a = spellLevelAllocation(160, 150, 30);
    expect(a.bonusUsed).toBe(30);
    expect(a.baseUsed).toBe(130);
    expect(a.available).toBe(-10);
  });

  it('charges a negative V/F modifier to the base first (Weak Parens)', () => {
    // Base 120 - Weak Parens 30 = a 90 budget, with 45 levels of spells chosen.
    // The penalty takes the first 30 off the base, so the base line still closes.
    const a = spellLevelAllocation(45, 90, -30);
    expect(a.base).toBe(120);
    expect(a.bonusAmount).toBe(-30);
    expect(a.bonusUsed).toBe(0);
    expect(a.baseUsed).toBe(75); // 45 spell levels + the 30-level penalty
    expect(a.available).toBe(45); // 120 - 75, and also budget (90) - used (45)
  });

  it('charges everything to the base when no V/F touches the budget', () => {
    const a = spellLevelAllocation(20, 120, 0);
    expect(a.base).toBe(120);
    expect(a.bonusUsed).toBe(0);
    expect(a.baseUsed).toBe(20);
    expect(a.available).toBe(100);
  });

  it('assumes no post-Gauntlet levels when the term is omitted', () => {
    // The default keeps every pre-6b5a caller reading exactly as before.
    const a = spellLevelAllocation(20, 120, 0);
    expect(a.lifeStage).toBe(0);
    expect(a.base).toBe(120);
    expect(a.available).toBe(100);
  });

  it('keeps the post-Gauntlet levels out of the editable base', () => {
    // 35 years past the Gauntlet with 300 of the points taken as levels of spells:
    // the engine's budget is 120 + 0 + 300, so the base is still the profile's 120.
    const a = spellLevelAllocation(0, 420, 0, 300);
    expect(a.base).toBe(120);
    expect(a.lifeStage).toBe(300);
    expect(a.baseUsed).toBe(0);
    // Already-earned levels are spendable, so they count towards Available.
    expect(a.available).toBe(420);
  });

  it('spends the post-Gauntlet levels on the same side as the base', () => {
    // Nothing is earmarked: 150 levels of spells simply draw on the 420 earned.
    const a = spellLevelAllocation(150, 420, 0, 300);
    expect(a.baseUsed).toBe(150);
    expect(a.available).toBe(270); // = budget - used
  });

  it('breaks a Skilled Parens magus past its Gauntlet into three summing parts', () => {
    // 120 base + 30 Skilled Parens + 300 post-Gauntlet = the engine's 450 budget.
    const a = spellLevelAllocation(45, 450, 30, 300);
    expect(a.base).toBe(120);
    expect(a.bonusAmount).toBe(30);
    expect(a.lifeStage).toBe(300);
    expect(a.base + a.bonusAmount + a.lifeStage).toBe(450);
    // The bonus is still spent first; the rest lands on base + post-Gauntlet.
    expect(a.bonusUsed).toBe(30);
    expect(a.baseUsed).toBe(15);
    expect(a.available).toBe(405); // = budget - used
  });

  it('still charges a Weak Parens penalty to the base when post-Gauntlet levels exist', () => {
    // 120 - 30 + 300 = a 390 budget with 45 levels chosen: the penalty takes its 30
    // off the unconditional side, exactly as it does without a life stage.
    const a = spellLevelAllocation(45, 390, -30, 300);
    expect(a.base).toBe(120);
    expect(a.lifeStage).toBe(300);
    expect(a.bonusUsed).toBe(0);
    expect(a.baseUsed).toBe(75); // 45 spell levels + the 30-level penalty
    expect(a.available).toBe(345); // 420 - 75, and also budget (390) - used (45)
  });

  it('reports a negative Available when a post-Gauntlet budget is overspent', () => {
    const a = spellLevelAllocation(430, 420, 0, 300);
    expect(a.baseUsed).toBe(430);
    expect(a.available).toBe(-10);
  });

  // guided-creation-review-2026-08 #18, D2 answer (a). `baseUsed` charges the
  // post-Gauntlet levels to the unconditional side, so the figure the bar shows
  // beside it must be that whole side — `base + lifeStage` — not the editable base
  // alone. With the base alone the bar read "150 / [120]  Available: 0": a primary
  // budget read-out claiming an overspend while reporting nothing left, on a
  // character the engine considers exactly balanced.
  it('closes the displayed pair when post-Gauntlet levels fund the spend', () => {
    // Base 120 + 30 levels bought past the Gauntlet = the engine's 150 budget,
    // every one of them spent.
    const a = spellLevelAllocation(150, 150, 0, 30);
    expect(a.base).toBe(120);
    expect(a.lifeStage).toBe(30);
    // The pair the bar displays: 150 / 150, and nothing left.
    expect(a.baseUsed).toBe(150);
    expect(a.denominator).toBe(150);
    expect(a.available).toBe(0);
    // The invariant the display rests on, stated once: the pair always closes.
    expect(a.denominator - a.baseUsed).toBe(a.available);
  });

  it('makes the denominator the base itself when no post-Gauntlet levels are funded', () => {
    // The other direction of option (a): with nothing earned past the Gauntlet the
    // denominator IS the editable base, so the overwhelming case reads as before.
    const a = spellLevelAllocation(120, 120, 0, 0);
    expect(a.base).toBe(120);
    expect(a.denominator).toBe(120);
    expect(a.baseUsed).toBe(120);
    expect(a.available).toBe(0);
  });
});

describe('generalXpAllocation', () => {
  it('leaves an unmodified pool exactly as it is', () => {
    // The overwhelming case: no Virtue touches the pool, so nothing is split and
    // the bar reads as it always has.
    const a = generalXpAllocation(90, 240, 0);
    expect(a.base).toBe(240);
    expect(a.baseUsed).toBe(90);
    expect(a.bonusAmount).toBe(0);
    expect(a.available).toBe(150);
  });

  it('spends Skilled Parens before the base', () => {
    // "You gain an additional 60 experience points … during apprenticeship"
    // (Core Rules.md:4966): a typed 240 becomes a pool of 300, of which 50 are
    // spent — all of them off the bonus, exactly as the engine drains a restricted
    // pool before the general one.
    const a = generalXpAllocation(50, 300, 60);
    expect(a.base).toBe(240);
    expect(a.bonusUsed).toBe(50);
    expect(a.baseUsed).toBe(0);
    expect(a.available).toBe(240);
  });

  it('charges the overflow above the bonus to the base', () => {
    const a = generalXpAllocation(300, 300, 60);
    expect(a.bonusUsed).toBe(60);
    expect(a.baseUsed).toBe(240);
    // THE BUG THIS CLOSES: a flat magus that spends all 300 used to read
    // Available: -60, because the bar charged the spend against the typed 240.
    expect(a.available).toBe(0);
  });

  it('reports a negative Available only once the whole pool is overspent', () => {
    const a = generalXpAllocation(310, 300, 60);
    expect(a.bonusUsed).toBe(60);
    expect(a.baseUsed).toBe(250);
    expect(a.available).toBe(-10);
  });

  it('charges a Weak Parens penalty to the base first', () => {
    // A typed 240 with Weak Parens is a pool of 180. The penalty has no pool of
    // its own to draw on, so it is charged to the base and the base line closes:
    // 240 - (60 + 60) = 120, which is also pool (180) - used (60).
    const a = generalXpAllocation(60, 180, -60);
    expect(a.base).toBe(240);
    expect(a.bonusUsed).toBe(0);
    expect(a.baseUsed).toBe(120);
    expect(a.available).toBe(120);
  });
});

describe('invalidSelectionIds', () => {
  it('collects the context id of every error-severity issue', () => {
    const result = {
      issues: [
        {
          severity: 'error' as const,
          code: 'prereq_not_met',
          phase: 'virtues_flaws' as const,
          args: {},
          context: 'spell.pilum',
        },
        {
          severity: 'error' as const,
          code: 'supernatural_ability_requires_virtue',
          phase: 'abilities' as const,
          args: {},
          context: 'ability.dowsing',
        },
      ],
    };
    expect([...invalidSelectionIds(result)].sort()).toEqual(['ability.dowsing', 'spell.pilum']);
  });

  it('ignores warnings, so only genuinely illegal rows are flagged', () => {
    const result = {
      issues: [
        {
          severity: 'warning' as const,
          code: 'prereq_unevaluated',
          phase: 'virtues_flaws' as const,
          args: {},
          context: 'spell.pilum',
        },
      ],
    };
    expect(invalidSelectionIds(result).size).toBe(0);
  });

  it('ignores issues carrying no context (nothing to highlight)', () => {
    const result = {
      issues: [
        {
          severity: 'error' as const,
          code: 'over_spell_levels',
          phase: 'spells' as const,
          args: {},
          context: null,
        },
      ],
    };
    expect(invalidSelectionIds(result).size).toBe(0);
  });

  it('is empty for a null result (not yet validated, or Silent mode)', () => {
    expect(invalidSelectionIds(null).size).toBe(0);
  });
});

// --- grantItemLabel() / eligibleForConstraint() / groupWarpingOwedGrants() ---

/** A `store.t`-shaped stub: renders `param-hint` as "(Label)" like the real .ftl. */
const translate = (key: string, args?: Record<string, string>) =>
  key === 'param-hint' ? `(${args?.label ?? ''})` : key.replace('param-label-', '');

describe('grantItemLabel', () => {
  it('renders a localized hint for an unfilled param, never the raw braces', () => {
    const ruleset = makeRuleset([], {
      i18n: { 'virtue.monstrosity': { name: '{form} Monstrosity' } },
    });
    const label = grantItemLabel(ruleset, 'virtue.monstrosity', translate);
    expect(label).toBe('(form) Monstrosity');
    expect(label).not.toContain('{');
  });

  it('resolves a filled param ref to its own localized name', () => {
    const ruleset = makeRuleset([], {
      i18n: {
        'virtue.monstrosity': { name: '{form} Monstrosity' },
        'art.ignem': { name: 'Ignem' },
      },
    });
    expect(grantItemLabel(ruleset, 'virtue.monstrosity', translate, { form: 'art.ignem' })).toBe(
      'Ignem Monstrosity',
    );
  });

  it('falls back to the ref when there is no i18n entry', () => {
    expect(grantItemLabel(makeRuleset([]), 'virtue.unknown', translate)).toBe('virtue.unknown');
  });
});

describe('eligibleForConstraint', () => {
  const items = [
    item({
      id: 'virtue.minor_general',
      kind: 'virtue',
      magnitude: 'minor',
      categories: ['general'],
    }),
    item({
      id: 'virtue.major_general',
      kind: 'virtue',
      magnitude: 'major',
      categories: ['general'],
    }),
    item({
      id: 'virtue.minor_super',
      kind: 'virtue',
      magnitude: 'minor',
      categories: ['supernatural'],
    }),
    item({ id: 'flaw.minor_general', kind: 'flaw', magnitude: 'minor', categories: ['general'] }),
    item({
      id: 'virtue.minor_super_warping',
      kind: 'virtue',
      magnitude: 'minor',
      categories: ['supernatural'],
      effects: [{ type: 'warping_grant', score: 1, points: 5 }],
    }),
  ];
  const ruleset = makeRuleset(items);

  it('keeps only the constraint kind and magnitude', () => {
    const ids = eligibleForConstraint(ruleset, { kind: 'flaw', magnitude: 'minor' }).map(
      (it) => it.id,
    );
    expect(ids).toEqual(['flaw.minor_general']);
  });

  it('applies the required- and forbidden-category lists', () => {
    const required = eligibleForConstraint(ruleset, {
      kind: 'virtue',
      magnitude: 'minor',
      require_categories: ['supernatural'],
    }).map((it) => it.id);
    expect(required).toEqual(['virtue.minor_super', 'virtue.minor_super_warping']);

    const forbidden = eligibleForConstraint(ruleset, {
      kind: 'virtue',
      magnitude: 'minor',
      forbid_categories: ['supernatural'],
    }).map((it) => it.id);
    expect(forbidden).toEqual(['virtue.minor_general']);
  });

  it('excludes Warping-granting items only when asked (the recursion guard)', () => {
    const constraint = {
      kind: 'virtue' as const,
      magnitude: 'minor' as const,
      require_categories: ['supernatural'],
    };
    expect(
      eligibleForConstraint(ruleset, constraint, { excludeWarpingSources: true }).map(
        (it) => it.id,
      ),
    ).toEqual(['virtue.minor_super']);
  });

  // Mirrors the engine's `open_pick_satisfies`: both lists are matched against
  // EVERY category the item carries (`require_categories` wants a non-empty
  // intersection, `forbid_categories` an empty one), so a secondary category
  // both admits a pick and rules one out.
  it('matches the require- and forbid-lists against a secondary category too', () => {
    const dualRuleset = makeRuleset([
      item({
        id: 'virtue.sufi',
        kind: 'virtue',
        magnitude: 'minor',
        categories: ['social_status', 'supernatural'],
      }),
      item({
        id: 'virtue.plain_status',
        kind: 'virtue',
        magnitude: 'minor',
        categories: ['social_status'],
      }),
    ]);

    expect(
      eligibleForConstraint(dualRuleset, {
        kind: 'virtue',
        require_categories: ['supernatural'],
      }).map((it) => it.id),
    ).toEqual(['virtue.sufi']);

    expect(
      eligibleForConstraint(dualRuleset, {
        kind: 'virtue',
        forbid_categories: ['supernatural'],
      }).map((it) => it.id),
    ).toEqual(['virtue.plain_status']);
  });

  it('sorts a brace-led name by its unwrapped word, not the brace glyph', () => {
    const localized = makeRuleset(
      [
        item({ id: 'virtue.zebra_mastery', kind: 'virtue', magnitude: 'minor' }),
        item({ id: 'virtue.alpha', kind: 'virtue', magnitude: 'minor' }),
        item({ id: 'virtue.beta', kind: 'virtue', magnitude: 'minor' }),
      ],
      {
        i18n: {
          'virtue.zebra_mastery': { name: '{zebra} Mastery' },
          'virtue.alpha': { name: 'Alpha' },
          'virtue.beta': { name: 'Beta' },
        },
      },
    );
    const ids = eligibleForConstraint(localized, { kind: 'virtue' }).map((it) => it.id);
    // "{zebra} Mastery" sorts under Z, after Alpha and Beta — sorting by the raw
    // "{" would have clustered it first instead.
    expect(ids).toEqual(['virtue.alpha', 'virtue.beta', 'virtue.zebra_mastery']);
  });
});

describe('groupWarpingOwedGrants', () => {
  const open = (choice_key: string, constraint: GrantConstraint): Grant => ({
    kind: 'open',
    choice_key,
    constraint,
  });

  it('buckets the three owed kinds with their label and count keys', () => {
    const groups = groupWarpingOwedGrants([
      open('warping.minor_flaw.0', { kind: 'flaw', magnitude: 'minor' }),
      open('warping.minor_flaw.1', { kind: 'flaw', magnitude: 'minor' }),
      open('warping.supernatural_virtue.0', {
        kind: 'virtue',
        magnitude: 'minor',
        require_categories: ['supernatural'],
      }),
      open('warping.major_flaw.0', { kind: 'flaw', magnitude: 'major' }),
    ]);
    expect(
      groups.map((g) => ({ label: g.labelKey, count: g.countKey, slots: g.grants.length })),
    ).toEqual([
      { label: 'warping-slot-minor-flaw', count: 'warping-owed-minor-flaws', slots: 2 },
      {
        label: 'warping-slot-supernatural-virtue',
        count: 'warping-owed-supernatural-virtues',
        slots: 1,
      },
      { label: 'warping-slot-major-flaw', count: 'warping-owed-major-flaws', slots: 1 },
    ]);
    expect(groups[0].grants.map((g) => g.choice_key)).toEqual([
      'warping.minor_flaw.0',
      'warping.minor_flaw.1',
    ]);
  });

  it('keeps the constraint on each slot, so the picker can filter it', () => {
    const constraint: GrantConstraint = {
      kind: 'virtue',
      magnitude: 'minor',
      require_categories: ['supernatural'],
    };
    const groups = groupWarpingOwedGrants([open('warping.supernatural_virtue.0', constraint)]);
    expect(groups[0].grants[0].constraint).toEqual(constraint);
  });

  it('skips non-open grants and returns no empty groups', () => {
    const groups = groupWarpingOwedGrants([
      { kind: 'fixed', item: 'virtue.minor_general' },
      { kind: 'choice', choice_key: 'x', options: [{ ref: 'virtue.minor_general' }] },
    ]);
    expect(groups).toEqual([]);
  });
});

describe('formatSigned', () => {
  it('prefixes positive numbers with a plus', () => {
    expect(formatSigned(3)).toBe('+3');
  });

  it('renders zero plain, without a sign', () => {
    expect(formatSigned(0)).toBe('0');
  });

  // E4 (round-1 audit): DerivedTotalsPanel.svelte negates a stored bonus
  // (`formatSigned(-d.longevity.bonus)`) and documents that a zero bonus must
  // read "0", never "-0" — a real production input, not an academic edge case.
  it('renders negative zero plain, without a sign (a bonus of 0 negated must never read "-0")', () => {
    expect(formatSigned(-0)).toBe('0');
  });

  it('uses an ASCII hyphen-minus (U+002D) for negatives, not the math minus U+2212', () => {
    expect(formatSigned(-2)).toBe('-2');
    expect(formatSigned(-2)).not.toContain('−');
  });
});

// --- the guided wizard's phase helpers --------------------------------------

describe('wizardPhases', () => {
  function profile(phases: CreationPhase[]): EntityTypeProfile {
    return {
      id: 't',
      budget: { virtue_points: 10, flaw_points: 10 },
      creation_phases: phases,
    };
  }

  it("walks the profile's declared order, then its own Review step", () => {
    // The magus order is the interesting one: the House step comes BEFORE
    // Virtues & Flaws, because the House grants a free Virtue that the V/F
    // budget then has to account for.
    expect(
      wizardPhases(profile(['concept', 'house_specialisation', 'virtues_flaws', 'experience'])),
    ).toEqual(['concept', 'house_specialisation', 'virtues_flaws', 'experience', 'review']);
  });

  it('appends Review exactly once, and never a second one', () => {
    const phases = wizardPhases(profile(['concept']));
    expect(phases.filter((p) => p === 'review')).toHaveLength(1);
    expect(phases.at(-1)).toBe('review');
  });

  it('is a lone Review step for a type that declares no guided flow', () => {
    expect(wizardPhases(profile([]))).toEqual(['review']);
  });

  it('is empty with no profile at all, so the wizard has nothing to show', () => {
    expect(wizardPhases(undefined)).toEqual([]);
  });
});

describe('issuesForPhase', () => {
  const issue = (code: string, phase: CreationPhase): ValidationIssue => ({
    severity: 'error',
    code,
    phase,
    args: {},
  });

  it("keeps only the step's own findings", () => {
    const issues = [
      issue('unbalanced_virtues', 'virtues_flaws'),
      issue('characteristic_overspent', 'characteristics'),
    ];
    expect(issuesForPhase(issues, 'virtues_flaws').map((i) => i.code)).toEqual([
      'unbalanced_virtues',
    ]);
  });

  it('is empty for a phase nothing was filed under', () => {
    expect(issuesForPhase([issue('unknown_type', 'review')], 'arts')).toEqual([]);
  });

  // Slice 2 (#11): the life-stage and childhood findings follow their input surface
  // onto the new `experience` step. Filing them under `abilities` would let the
  // wizard gate the Abilities step on a value that step no longer has an input for.
  it('routes a life-stage finding to the experience step, not to abilities', () => {
    const findings = [
      issue('life_stage_age_unset', 'experience'),
      issue('not_enough_xp', 'abilities'),
    ];
    expect(issuesForPhase(findings, 'experience').map((i) => i.code)).toEqual([
      'life_stage_age_unset',
    ]);
    expect(issuesForPhase(findings, 'abilities').map((i) => i.code)).toEqual(['not_enough_xp']);
  });

  // Slice 11 (#30): the two unspent-budget warnings are owned by the steps that
  // hold their budgets — the XP pool on `abilities`, the spell levels on `spells`.
  // The "and nowhere else" half is the one that bites: a budget warning surfacing on
  // the wrong step points the player at a surface with no control for it.
  it('routes each unspent-budget warning to the step that owns that budget', () => {
    const findings = [
      issue('general_xp_unspent', 'abilities'),
      issue('spell_levels_unspent', 'spells'),
    ];
    expect(issuesForPhase(findings, 'abilities').map((i) => i.code)).toEqual([
      'general_xp_unspent',
    ]);
    expect(issuesForPhase(findings, 'spells').map((i) => i.code)).toEqual(['spell_levels_unspent']);
    // Neither leaks onto the other's step, nor onto the step that funds them.
    expect(issuesForPhase(findings, 'experience')).toEqual([]);
  });
});

describe('phaseSelectedItemIds', () => {
  const selections = [{ ref: 'virtue.great_characteristic' }, { ref: 'flaw.poor_characteristic' }];

  it('names the items the Virtues & Flaws step itself holds', () => {
    expect(phaseSelectedItemIds(selections, 'virtues_flaws')).toEqual(
      new Set(['virtue.great_characteristic', 'flaw.poor_characteristic']),
    );
  });

  it('names nothing on a step that holds no point-item picker', () => {
    // The V/F selections stay on the entity while the player stands on
    // Characteristics, but that step is not where they were chosen — so a finding
    // about one must not be pulled onto it.
    expect(phaseSelectedItemIds(selections, 'characteristics')).toEqual(new Set());
    expect(phaseSelectedItemIds(selections, 'abilities')).toEqual(new Set());
  });

  it('is empty for a character with no selections at all', () => {
    expect(phaseSelectedItemIds(undefined, 'virtues_flaws')).toEqual(new Set());
  });
});

describe('issuesForStep', () => {
  const issue = (
    severity: 'error' | 'warning',
    code: string,
    phase: CreationPhase,
    context?: string,
  ): ValidationIssue => ({ severity, code, phase, args: {}, context });

  const greatCharacteristic = issue(
    'error',
    'characteristic_max_base_too_low',
    'characteristics',
    'virtue.great_characteristic',
  );

  it("keeps the step's own findings unmarked", () => {
    const [entry] = issuesForStep(
      [issue('error', 'unbalanced_virtues', 'virtues_flaws')],
      'virtues_flaws',
      new Set(),
    );
    expect(entry.issue.code).toBe('unbalanced_virtues');
    expect(entry.elsewhere).toBeUndefined();
  });

  // manual-testing-findings #4a/#4b: Great and Poor Characteristic are taken on
  // the V/F step but the value they constrain is a Characteristic score, so the
  // engine files them on `characteristics` — and the V/F step, where the user is
  // standing, said nothing at all.
  it("admits another phase's finding when its context names an item this step holds", () => {
    const entries = issuesForStep(
      [greatCharacteristic],
      'virtues_flaws',
      new Set(['virtue.great_characteristic']),
    );
    expect(entries.map((e) => e.issue.code)).toEqual(['characteristic_max_base_too_low']);
    expect(entries[0].elsewhere).toBe('characteristics');
  });

  it('leaves a foreign finding out when this step holds no such item', () => {
    expect(issuesForStep([greatCharacteristic], 'virtues_flaws', new Set())).toEqual([]);
  });

  it('leaves a foreign finding with no context out — it names no item to trace', () => {
    expect(
      issuesForStep(
        [issue('warning', 'characteristic_points_unspent', 'characteristics')],
        'virtues_flaws',
        new Set(['virtue.great_characteristic']),
      ),
    ).toEqual([]);
  });

  it('marks nothing when the owning phase IS the current step', () => {
    const [entry] = issuesForStep(
      [greatCharacteristic],
      'characteristics',
      new Set(['virtue.great_characteristic']),
    );
    expect(entry.elsewhere).toBeUndefined();
  });

  it('passes every finding through unfiltered when no step is named', () => {
    const all = [greatCharacteristic, issue('warning', 'house_unset', 'house_specialisation')];
    const entries = issuesForStep(all, undefined, new Set());
    expect(entries.map((e) => e.issue.code)).toEqual([
      'characteristic_max_base_too_low',
      'house_unset',
    ]);
    expect(entries.every((e) => e.elsewhere === undefined)).toBe(true);
  });
});

describe('phaseHasPendingWarning', () => {
  const issue = (
    severity: 'error' | 'warning',
    code: string,
    phase: CreationPhase,
  ): ValidationIssue => ({ severity, code, phase, args: {} });

  it('reports a warning left open on that phase', () => {
    expect(
      phaseHasPendingWarning(
        [issue('warning', 'characteristic_points_unspent', 'characteristics')],
        'characteristics',
      ),
    ).toBe(true);
  });

  it('ignores an error — that is the blocked marker, not this one', () => {
    expect(
      phaseHasPendingWarning(
        [issue('error', 'characteristic_overspent', 'characteristics')],
        'characteristics',
      ),
    ).toBe(false);
  });

  it("ignores another phase's warning", () => {
    expect(
      phaseHasPendingWarning([issue('warning', 'house_unset', 'house_specialisation')], 'arts'),
    ).toBe(false);
  });
});

describe('phaseHasBlockingIssue', () => {
  const issue = (
    severity: 'error' | 'warning',
    code: string,
    phase: CreationPhase,
  ): ValidationIssue => ({ severity, code, phase, args: {} });

  it('blocks on an error in that phase', () => {
    expect(
      phaseHasBlockingIssue(
        [issue('error', 'unbalanced_virtues', 'virtues_flaws')],
        'virtues_flaws',
      ),
    ).toBe(true);
  });

  it('does not block on a warning — an advisory is not an illegal state', () => {
    // house_unset is a warning, which is exactly why the wizard lets a magus
    // walk past the House step with no House chosen (legal, if incomplete).
    expect(
      phaseHasBlockingIssue(
        [issue('warning', 'house_unset', 'house_specialisation')],
        'house_specialisation',
      ),
    ).toBe(false);
  });

  it("does not block on another phase's error", () => {
    expect(
      phaseHasBlockingIssue([issue('error', 'over_spell_levels', 'spells')], 'abilities'),
    ).toBe(false);
  });
});

describe('phaseIsIncomplete', () => {
  const result = (phases: CreationPhase[]): ValidationResult => ({
    issues: [],
    completeness: { incomplete_phases: phases },
  });

  it('marks a phase the engine reports as untouched', () => {
    expect(phaseIsIncomplete(result(['abilities', 'aging']), 'abilities')).toBe(true);
  });

  it('leaves a phase with choices recorded unmarked', () => {
    expect(phaseIsIncomplete(result(['aging']), 'abilities')).toBe(false);
  });

  it('marks nothing before the first validation result arrives', () => {
    expect(phaseIsIncomplete(null, 'abilities')).toBe(false);
  });

  // The wizard's own closing step is never in the report, and neither is a phase
  // the character's type does not declare — so neither can be marked.
  it('marks nothing for a phase the engine never reported on', () => {
    expect(phaseIsIncomplete(result(['abilities']), 'review')).toBe(false);
  });

  // Incompleteness must never be mistaken for a finding: it carries no severity,
  // so nothing that gates on `error` can ever see it.
  it('is independent of the findings', () => {
    const blocked: ValidationResult = {
      issues: [{ severity: 'error', code: 'x', phase: 'abilities', args: {} }],
      completeness: { incomplete_phases: [] },
    };
    expect(phaseIsIncomplete(blocked, 'abilities')).toBe(false);
    expect(phaseHasBlockingIssue(blocked.issues, 'abilities')).toBe(true);
  });
});

describe('incompletePhases', () => {
  it('lists the untouched phases in the order the engine sent them', () => {
    const result: ValidationResult = {
      issues: [],
      completeness: { incomplete_phases: ['house_specialisation', 'virtues_flaws'] },
    };
    expect(incompletePhases(result)).toEqual(['house_specialisation', 'virtues_flaws']);
  });

  it('is empty with no result at all', () => {
    expect(incompletePhases(null)).toEqual([]);
  });

  // An engine payload always carries the report; a fixture (or an older payload)
  // need not, and must read as "nothing to report" rather than crashing the rail.
  it('is empty when the payload carries no report', () => {
    expect(incompletePhases({ issues: [] })).toEqual([]);
  });
});

describe('firstBlockedPhaseIndex', () => {
  const phases: CreationPhase[] = [
    'concept',
    'characteristics',
    'virtues_flaws',
    'abilities',
    'review',
  ];
  const err = (phase: CreationPhase): ValidationIssue => ({
    severity: 'error',
    code: 'x',
    phase,
    args: {},
  });

  it('is null when nothing in the range blocks', () => {
    expect(firstBlockedPhaseIndex(phases, [err('review')], 0, 3)).toBeNull();
  });

  it('finds a phase blocked in the middle of the range', () => {
    expect(firstBlockedPhaseIndex(phases, [err('virtues_flaws')], 0, 3)).toBe(2);
  });

  it('includes the departure phase, so a rail jump cannot smuggle past the Next gate', () => {
    // Advance to 3, walk back to 1 and break it: jumping forward to 3 must clamp
    // to 1, not sail over it. If the scan skipped `from`, the rail would be a way
    // around the very gate that blocks Next.
    expect(firstBlockedPhaseIndex(phases, [err('characteristics')], 1, 3)).toBe(1);
  });

  it('includes the destination phase', () => {
    expect(firstBlockedPhaseIndex(phases, [err('abilities')], 1, 3)).toBe(3);
  });

  it('is null for a backwards range — Back is never gated', () => {
    expect(firstBlockedPhaseIndex(phases, [err('characteristics')], 3, 1)).toBeNull();
  });
});

// --- 6b3b: Sample Childhood package helpers ---------------------------------

describe('childhoodSlots / childhoodEntryPreview / childhoodSlotFault', () => {
  /**
   * Fluent stand-in with the real strings' shapes: `childhood-slot-label` is the
   * bare name, `-nth` appends a 1-based ordinal, `childhood-entry` is "name score".
   * Keys resolve to themselves otherwise, so a helper reaching for a key that does
   * not exist shows up as a slug in the assertion.
   */
  const t = (key: string, args?: Record<string, string>) => {
    if (key === 'param-hint') return `(${args?.label})`;
    if (key === 'param-label-area') return 'Area';
    if (key === 'param-label-language') return 'Language';
    if (key === 'childhood-slot-label') return `${args?.name}`;
    if (key === 'childhood-slot-label-nth') return `${args?.name} (${args?.index})`;
    if (key === 'childhood-entry') return `${args?.name} ${args?.score}`;
    return key;
  };

  /** A ruleset that knows the childhood Abilities, their parameters and the plan rules. */
  function childhoodRuleset(): LocalizedRuleset {
    return {
      ruleset: {
        id: 't',
        version: '1',
        point_items: {},
        type_profiles: {},
        abilities: {
          'ability.area_lore': { id: 'ability.area_lore', category: 'general', parameter: 'area' },
          'ability.athletics': { id: 'ability.athletics', category: 'general' },
          'ability.awareness': { id: 'ability.awareness', category: 'general' },
          'ability.folk_ken': { id: 'ability.folk_ken', category: 'general' },
          'ability.living_language': {
            id: 'ability.living_language',
            category: 'general',
            parameter: 'language',
          },
          'ability.stealth': { id: 'ability.stealth', category: 'general' },
          'ability.survival': { id: 'ability.survival', category: 'general' },
        },
        life_stages: {
          childhood: {
            years: 5,
            native_language_ability: 'ability.living_language',
            native_language_xp: 75,
            spread_xp: 45,
            spread_abilities: ['ability.area_lore', 'ability.living_language'],
          },
          later_life: { xp_per_year: 15 },
        },
        ...DERIVED_TAXONOMY,
      },
      i18n: {
        'ability.area_lore': { name: '{area} Lore' },
        'ability.athletics': { name: 'Athletics' },
        'ability.awareness': { name: 'Awareness' },
        'ability.folk_ken': { name: 'Folk Ken' },
        'ability.living_language': { name: '{language}' },
        'ability.stealth': { name: 'Stealth' },
        'ability.survival': { name: 'Survival' },
      },
    } as unknown as LocalizedRuleset;
  }

  /** Traveling Childhood as `rules/core/childhoods.json` ships it: three slots, two on one Ability. */
  const traveling: ChildhoodPackage = {
    id: 'childhood.traveling',
    entries: [
      { ability: 'ability.area_lore', score: 1, slot: 'area_a' },
      { ability: 'ability.area_lore', score: 1, slot: 'area_b' },
      { ability: 'ability.folk_ken', score: 2 },
      { ability: 'ability.living_language', score: 5, native: true },
      { ability: 'ability.living_language', score: 1, slot: 'language' },
      { ability: 'ability.survival', score: 2 },
    ],
  };

  /** Exploring Childhood: exactly one slot, so its label must carry no ordinal. */
  const exploring: ChildhoodPackage = {
    id: 'childhood.exploring',
    entries: [
      { ability: 'ability.area_lore', score: 2, slot: 'area' },
      { ability: 'ability.athletics', score: 1 },
      { ability: 'ability.awareness', score: 1 },
      { ability: 'ability.living_language', score: 5, native: true },
      { ability: 'ability.stealth', score: 1 },
      { ability: 'ability.survival', score: 2 },
    ],
  };

  /** Athletic Childhood: nothing to ask the player at all. */
  const athletic: ChildhoodPackage = {
    id: 'childhood.athletic',
    entries: [
      { ability: 'ability.athletics', score: 2 },
      { ability: 'ability.living_language', score: 5, native: true },
    ],
  };

  const rs = childhoodRuleset();

  describe('childhoodSlots', () => {
    it('asks for every slotted entry, in the package order, with its ability', () => {
      const slots = childhoodSlots(rs, traveling, t);
      expect(slots.map((s) => s.slot)).toEqual(['area_a', 'area_b', 'language']);
      expect(slots.map((s) => s.ability)).toEqual([
        'ability.area_lore',
        'ability.area_lore',
        'ability.living_language',
      ]);
    });

    it('numbers the labels only where one Ability holds two slots', () => {
      // Two Area Lores are indistinguishable without an ordinal; the single
      // language slot needs none, so it must not get one.
      expect(childhoodSlots(rs, traveling, t).map((s) => s.label)).toEqual([
        '(Area) Lore (1)',
        '(Area) Lore (2)',
        '(Language)',
      ]);
    });

    it('leaves a lone slot unnumbered', () => {
      expect(childhoodSlots(rs, exploring, t).map((s) => s.label)).toEqual(['(Area) Lore']);
    });

    it('asks nothing for a package with no slotted entry', () => {
      expect(childhoodSlots(rs, athletic, t)).toEqual([]);
    });

    it('never renders a raw parameter token, an ability id or the slot key', () => {
      for (const slot of childhoodSlots(rs, traveling, t)) {
        expect(slot.label).not.toContain('{');
        expect(slot.label).not.toContain('ability.');
        // The slot key is per-package data, so it can only ever be a machine key.
        expect(slot.label).not.toContain(slot.slot);
      }
    });
  });

  describe('childhoodEntryPreview', () => {
    it('reads out every entry with its score, in package order', () => {
      const rows = childhoodEntryPreview(
        rs,
        traveling,
        { native_language: 'German' },
        { area_a: 'Rhine' },
        t,
      );
      expect(rows).toEqual([
        'Rhine Lore 1',
        '(Area) Lore 1',
        'Folk Ken 2',
        'German 5',
        '(Language) 1',
        'Survival 2',
      ]);
    });

    it('names the plan native language on the native entry, never its token or id', () => {
      const rows = childhoodEntryPreview(rs, traveling, { native_language: 'German' }, {}, t);
      expect(rows).toContain('German 5');
      expect(rows.join(' | ')).not.toContain('{language}');
      expect(rows.join(' | ')).not.toContain('ability.living_language');
    });

    it('falls back to the localized parameter hint while no native language is chosen', () => {
      const rows = childhoodEntryPreview(rs, traveling, null, {}, t);
      expect(rows).toContain('(Language) 5');
    });

    it('never leaves a brace placeholder or an ability slug in a row', () => {
      const rows = childhoodEntryPreview(rs, exploring, { native_language: 'German' }, {}, t);
      for (const row of rows) {
        expect(row).not.toContain('{');
        expect(row).not.toContain('ability.');
      }
    });
  });

  describe('childhoodSlotFault', () => {
    const plan = { native_language: 'German' };

    it('reports an unanswered slot as empty', () => {
      expect(childhoodSlotFault(rs, traveling, 'area_a', {}, plan)).toBe('empty');
      expect(childhoodSlotFault(rs, traveling, 'area_a', { area_a: '   ' }, plan)).toBe('empty');
    });

    it('accepts distinct answers', () => {
      const slots = { area_a: 'Rhine', area_b: 'Provence', language: 'Italian' };
      expect(childhoodSlotFault(rs, traveling, 'area_a', slots, plan)).toBeNull();
      expect(childhoodSlotFault(rs, traveling, 'area_b', slots, plan)).toBeNull();
      expect(childhoodSlotFault(rs, traveling, 'language', slots, plan)).toBeNull();
    });

    it('faults both slots of one Ability answered alike — they would merge into one row', () => {
      const slots = { area_a: 'Rhine', area_b: 'Rhine', language: 'Italian' };
      expect(childhoodSlotFault(rs, traveling, 'area_a', slots, plan)).toBe('duplicate');
      expect(childhoodSlotFault(rs, traveling, 'area_b', slots, plan)).toBe('duplicate');
    });

    it('does not fault two slots of different Abilities sharing a value', () => {
      // "Rhine Lore" and the language "Rhine" are two different rows.
      const slots = { area_a: 'Rhine', area_b: 'Provence', language: 'Rhine' };
      expect(childhoodSlotFault(rs, traveling, 'area_a', slots, plan)).toBeNull();
      expect(childhoodSlotFault(rs, traveling, 'language', slots, plan)).toBeNull();
    });

    it('faults a childhood language repeating the native language', () => {
      // Childhood's spread buys a Living Language *other than* the native one.
      // Source: Ars Magica - Definitive Edition (Core Rules).md:2378
      const slots = { area_a: 'Rhine', area_b: 'Provence', language: 'German' };
      expect(childhoodSlotFault(rs, traveling, 'language', slots, plan)).toBe('native');
    });

    it('lets an Area Lore be named after the native language', () => {
      // Only the childhood's own language Ability is restricted; an Area Lore
      // called "German" is perfectly ordinary.
      const slots = { area_a: 'German', area_b: 'Provence', language: 'Italian' };
      expect(childhoodSlotFault(rs, traveling, 'area_a', slots, plan)).toBeNull();
    });

    it('cannot fault a language against a native language that is not chosen yet', () => {
      const slots = { area_a: 'Rhine', area_b: 'Provence', language: 'German' };
      expect(childhoodSlotFault(rs, traveling, 'language', slots, null)).toBeNull();
    });

    it('has nothing to say about a slot the package does not declare', () => {
      expect(childhoodSlotFault(rs, exploring, 'area_b', { area: 'Rhine' }, plan)).toBeNull();
    });
  });
});
