import { describe, expect, it } from 'vitest';

import {
  abilityDisplayName,
  abilityLabel,
  abilityXpSpent,
  effectiveSpellMastery,
  spellMasteryXpSpent,
  artAbbreviation,
  artLabel,
  artXpSpent,
  balance,
  characteristicPointsUsed,
  displayName,
  filterAbilities,
  filterEquipment,
  filterItems,
  filterSpells,
  formatSigned,
  grantedSelectionsForSide,
  groupAbilitiesByCategory,
  groupArtsByType,
  groupByCategory,
  groupSelectedEquipmentByKind,
  groupSelectedSpellsByTechniqueForm,
  groupSelectionsByCategory,
  groupSpellsByTechniqueForm,
  incompatibleRefs,
  invalidSelectionIds,
  orderSelectedSpells,
  usedSpellForms,
  groupAbilitySelectionsByCategory,
  mandatoryTraitRefs,
  maxAbilityScore,
  maxArtScore,
  paramValueUsage,
  resolveIssueArgValue,
  resolveIssueArgs,
  restrictedPoolLabel,
  spellDisplayName,
  spellLevelAllocation,
} from './derive';
import type {
  Ability,
  Art,
  CharacteristicRules,
  Entity,
  EntityTypeProfile,
  LocalizedRuleset,
  PointItem,
  Spell,
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
    category: 'general',
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

function entity(refs: string[], typeId = 'companion'): Entity {
  return {
    schema_version: 1,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: typeId,
    selections: refs.map((ref) => ({ ref })),
  };
}

// --- filterItems() / filterAbilities() / filterSpells() ---------------------

describe('filterItems', () => {
  const items = [
    item({ id: 'virtue.brave', category: 'general', magnitude: 'minor' }),
    item({ id: 'virtue.giant', category: 'general', magnitude: 'major' }),
    item({ id: 'virtue.corrupt', category: 'supernatural', magnitude: 'minor', tainted: true }),
    item({ id: 'flaw.dark', kind: 'flaw', category: 'story', magnitude: 'major' }),
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

// --- balance() --------------------------------------------------------------

describe('balance', () => {
  const profiles = {
    companion: {
      id: 'companion',
      budget: { virtue_points: 10, flaw_points: 10 },
      permitted_categories: [],
      forbidden_categories: [],
      creation_phases: [],
    },
  };

  it('sums virtue, flaw and boon points by magnitude', () => {
    const ruleset = makeRuleset(
      [
        item({ id: 'v.minor', kind: 'virtue', magnitude: 'minor' }),
        item({ id: 'v.major', kind: 'virtue', magnitude: 'major' }),
        item({ id: 'f.major', kind: 'flaw', magnitude: 'major' }),
        item({ id: 'f.free', kind: 'flaw', magnitude: 'free' }),
        item({ id: 'b.boon', kind: 'boon', magnitude: 'minor' }),
      ],
      { profiles },
    );
    // virtue side: 1 (minor v) + 3 (major v) + 1 (minor boon) = 5
    // flaw side: 3 (major f) + 0 (free f) = 3
    const result = balance(ruleset, entity(['v.minor', 'v.major', 'f.major', 'f.free', 'b.boon']));
    expect(result.virtuePoints).toBe(5);
    expect(result.flawPoints).toBe(3);
    expect(result.virtueBudget).toBe(10);
    expect(result.flawBudget).toBe(10);
  });

  it('counts a hook on the flaw side (mirrors engine is_positive)', () => {
    const ruleset = makeRuleset([item({ id: 'h.minor', kind: 'hook', magnitude: 'minor' })], {
      profiles,
    });
    const result = balance(ruleset, entity(['h.minor']));
    expect(result.virtuePoints).toBe(0);
    expect(result.flawPoints).toBe(1);
  });

  it('skips selections whose item ref is unknown', () => {
    const ruleset = makeRuleset([item({ id: 'v.minor', kind: 'virtue', magnitude: 'minor' })], {
      profiles,
    });
    const result = balance(ruleset, entity(['v.minor', 'does.not.exist']));
    expect(result.virtuePoints).toBe(1);
    expect(result.flawPoints).toBe(0);
  });

  it('reports a zero budget when the type profile is missing', () => {
    const ruleset = makeRuleset([item({ id: 'v.minor', magnitude: 'minor' })]);
    const result = balance(ruleset, entity(['v.minor'], 'unknown_type'));
    expect(result.virtueBudget).toBe(0);
    expect(result.flawBudget).toBe(0);
    // points are still summed even without a profile
    expect(result.virtuePoints).toBe(1);
  });
});

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
        item({ id: 'virtue.a_general', category: 'general' }),
        item({ id: 'virtue.b_general', category: 'general' }),
        item({ id: 'virtue.m_hermetic', category: 'hermetic' }),
        item({ id: 'virtue.z_hermetic', category: 'hermetic' }),
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
      item({ id: 'virtue.b_general', category: 'general' }),
      item({ id: 'virtue.a_general', category: 'general' }),
    ]);
    expect(groupByCategory(ruleset)[0].items.map((i) => i.id)).toEqual([
      'virtue.a_general',
      'virtue.b_general',
    ]);
  });

  it('returns an empty array for an empty ruleset', () => {
    expect(groupByCategory(makeRuleset([]))).toEqual([]);
  });

  it('keeps only items whose kind is in the given filter', () => {
    const ruleset = makeRuleset([
      item({ id: 'virtue.a', kind: 'virtue', category: 'general' }),
      item({ id: 'boon.b', kind: 'boon', category: 'general' }),
      item({ id: 'flaw.c', kind: 'flaw', category: 'general' }),
      item({ id: 'hook.d', kind: 'hook', category: 'general' }),
    ]);

    const virtues = groupByCategory(ruleset, ['virtue', 'boon']);
    expect(virtues.flatMap((g) => g.items.map((i) => i.id))).toEqual(['boon.b', 'virtue.a']);

    const flaws = groupByCategory(ruleset, ['flaw', 'hook']);
    expect(flaws.flatMap((g) => g.items.map((i) => i.id))).toEqual(['flaw.c', 'hook.d']);
  });
});

// --- characteristicPointsUsed() ---------------------------------------------

describe('characteristicPointsUsed', () => {
  const rules: CharacteristicRules = {
    start_points: 7,
    costs: [
      { score: 3, cost: 6 },
      { score: 2, cost: 3 },
      { score: 1, cost: 1 },
      { score: 0, cost: 0 },
      { score: -1, cost: -1 },
      { score: -2, cost: -3 },
      { score: -3, cost: -6 },
    ],
  };

  it('nets spends against gains', () => {
    // Int +3 (6) + Per +1 (1) + Pre -3 (-6) + Com -1 (-1) + Qik +2 (3) + Str +2 (3) + Dex +1 (1) = 7
    expect(
      characteristicPointsUsed(rules, {
        int: 3,
        per: 1,
        pre: -3,
        com: -1,
        qik: 2,
        str: 2,
        dex: 1,
      }),
    ).toBe(7);
  });

  it('ignores out-of-range scores (contributes 0)', () => {
    expect(characteristicPointsUsed(rules, { str: 4 })).toBe(0);
  });

  it('returns 0 without rules or scores', () => {
    expect(characteristicPointsUsed(undefined, { int: 3 })).toBe(0);
    expect(characteristicPointsUsed(rules, undefined)).toBe(0);
  });
});

// --- abilityXpSpent() -------------------------------------------------------

describe('abilityXpSpent', () => {
  const advancement = [
    { score: 1, total_xp: 5 },
    { score: 2, total_xp: 15 },
    { score: 3, total_xp: 30 },
  ];

  it('sums total XP per whole bought score', () => {
    expect(abilityXpSpent(advancement, [{ score: 3 }, { score: 2 }])).toBe(45);
  });

  it('treats score 0 as no XP and skips unknown scores', () => {
    expect(abilityXpSpent(advancement, [{ score: 0 }, { score: 9 }])).toBe(0);
  });
});

// --- spellMasteryXpSpent() / effectiveSpellMastery() ------------------------

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

  it('prices Art XP from the (triangular) Art table and reports the ceiling', () => {
    const rs = withArts([]);
    const adv = rs.ruleset.art_advancement;
    // Creo 5 (15) + Ignem 3 (6) = 21; score 0 is free.
    expect(artXpSpent(adv, [{ score: 5 }, { score: 3 }, { score: 0 }])).toBe(21);
    expect(maxArtScore(adv)).toBe(5);
  });
});

describe('restrictedPoolLabel', () => {
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
    const label = restrictedPoolLabel(rs, { amount: 50, used: 0, categories: ['martial'] }, t);
    expect(label).toBe('Martial');
  });

  it('labels a multi-category pool, separator-joined', () => {
    const rs = makeRuleset([]);
    const label = restrictedPoolLabel(
      rs,
      { amount: 50, used: 0, categories: ['academic', 'martial'] },
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
      { amount: 50, used: 30, abilities: ['ability.latin', 'ability.artes_liberales'] },
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
      { amount: 50, used: 0, abilities: ['ability.artes_liberales', 'ability.living_language'] },
      th,
    );
    expect(label).toBe('Artes Liberales, (Language)');
    expect(label).not.toContain('{language}');
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

  it('resolves a param key arg through its param-label', () => {
    const rs = makeRuleset([]);
    expect(resolveIssueArgValue(rs, 'key', 'language', t)).toBe('Language');
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
    ).toEqual({ ability: '(Area) Lore', characteristic: 'Intelligence', score: '5' });
  });
});

// --- C4: grouping + sorting selected V/F and abilities ----------------------

describe('groupSelectionsByCategory', () => {
  const rs = makeRuleset(
    [
      item({ id: 'virtue.zeal', category: 'general' }),
      item({ id: 'virtue.affinity', category: 'general' }),
      item({ id: 'virtue.verditius', category: 'hermetic' }),
    ],
    {
      i18n: {
        'virtue.zeal': { name: 'Zeal' },
        'virtue.affinity': { name: 'Affinity' },
        'virtue.verditius': { name: 'Verditius' },
      },
    },
  );

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
    expect(groups[0].entries.map((e) => e.index)).toEqual([2, 0]);
    expect(groups[1].entries.map((e) => e.index)).toEqual([1]);
  });

  it('drops entries whose item ref is unknown', () => {
    const groups = groupSelectionsByCategory(rs, [{ selection: { ref: 'nope' }, index: 0 }]);
    expect(groups).toEqual([]);
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
});

describe('invalidSelectionIds', () => {
  it('collects the context id of every error-severity issue', () => {
    const result = {
      issues: [
        { severity: 'error' as const, code: 'prereq_not_met', args: {}, context: 'spell.pilum' },
        {
          severity: 'error' as const,
          code: 'supernatural_ability_requires_virtue',
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
          args: {},
          context: 'spell.pilum',
        },
      ],
    };
    expect(invalidSelectionIds(result).size).toBe(0);
  });

  it('ignores issues carrying no context (nothing to highlight)', () => {
    const result = {
      issues: [{ severity: 'error' as const, code: 'over_spell_levels', args: {}, context: null }],
    };
    expect(invalidSelectionIds(result).size).toBe(0);
  });

  it('is empty for a null result (not yet validated, or Silent mode)', () => {
    expect(invalidSelectionIds(null).size).toBe(0);
  });
});

describe('formatSigned', () => {
  it('prefixes positive numbers with a plus', () => {
    expect(formatSigned(3)).toBe('+3');
  });

  it('renders zero plain, without a sign', () => {
    expect(formatSigned(0)).toBe('0');
  });

  it('uses an ASCII hyphen-minus (U+002D) for negatives, not the math minus U+2212', () => {
    expect(formatSigned(-2)).toBe('-2');
    expect(formatSigned(-2)).not.toContain('−');
  });
});
