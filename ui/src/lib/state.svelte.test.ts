import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Ability, Entity, House, LocalizedRuleset, PointItem } from './types';

// The AppStore methods under test are synchronous; they only *schedule* a
// debounced revalidate via setTimeout, which calls into the Tauri IPC bridge.
// Mock that bridge so the timer (if it ever fires) is inert, and use fake
// timers so it never fires mid-assertion. We assert on entity state directly.
vi.mock('./ipc', () => ({
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({
    ability_bonuses: [],
    art_bonuses: [],
    characteristic_caps: {},
    characteristic_floors: {},
  }),
  derivedTotals: vi.fn().mockResolvedValue({
    is_magus: false,
    lab_totals: [],
    casting_totals: [],
    penetration: [],
    magic_resistance: [],
    combat: [],
    soak: { addends: [], total: 0 },
    encumbrance: { load: 0, burden: 0, total: 0 },
    fatigue: [],
    wounds: [],
    size: 0,
    decrepitude_score: 0,
    warping_score: 0,
    warping_points: 0,
    surfaced_modifiers: [],
  }),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
  updateCloseGuard: vi.fn(),
}));

// Import the singleton after the mock is registered.
import * as ipc from './ipc';
import { store, defaultPickerFilters } from './state.svelte';

// --- Fixtures ---------------------------------------------------------------

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

function ability(id: string, parameter?: string): Ability {
  return { id, category: 'general', ...(parameter ? { parameter } : {}) };
}

/** Build a localized ruleset from items + abilities and install it on the store. */
function installRuleset(
  items: PointItem[],
  abilities: Ability[] = [],
  profiles: LocalizedRuleset['ruleset']['type_profiles'] = {},
): LocalizedRuleset {
  const point_items: Record<string, PointItem> = {};
  for (const it of items) point_items[it.id] = it;
  const abilityMap: Record<string, Ability> = {};
  for (const a of abilities) abilityMap[a.id] = a;
  const ruleset: LocalizedRuleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items,
      type_profiles: profiles,
      abilities: abilityMap,
      // Engine-derived taxonomy the real backend ships on every Ruleset payload.
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {},
  };
  store.ruleset = ruleset;
  return ruleset;
}

/** Reset the shared singleton's entity to a clean character before each test. */
function resetEntity(): void {
  store.entity = {
    schema_version: 11,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'companion',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

beforeEach(() => {
  vi.useFakeTimers();
  resetEntity();
  installRuleset([]);
});

afterEach(() => {
  // Drop any pending debounced revalidate without running it, then restore.
  vi.clearAllTimers();
  vi.useRealTimers();
});

// --- per-picker filter state (C7) -------------------------------------------

describe('picker filter state', () => {
  beforeEach(() => {
    // Isolate from other tests mutating the shared singleton's filters.
    store.filters = defaultPickerFilters();
  });

  it('defaults every picker to empty filters', () => {
    expect(store.filters.vf.virtue).toEqual({
      search: '',
      magnitude: '',
      category: '',
      taintedOnly: false,
    });
    expect(store.filters.vf.flaw).toEqual({
      search: '',
      magnitude: '',
      category: '',
      taintedOnly: false,
    });
    expect(store.filters.abilities).toEqual({ search: '', category: '' });
    expect(store.filters.spells).toEqual({ search: '', technique: '', form: '', level: null });
  });

  it('retains a picker tab’s filter state independently of the others', () => {
    // A tab switch unmounts the picker components, so their filter state lives on
    // the store: mutating one picker leaves the values readable afterwards and
    // does not bleed into the sibling pickers.
    store.filters.vf.virtue.search = 'giant';
    store.filters.vf.virtue.magnitude = 'major';
    store.filters.vf.virtue.category = 'general';
    store.filters.vf.virtue.taintedOnly = true;
    store.filters.abilities.search = 'latin';
    store.filters.abilities.category = 'academic';

    // Returning to the V/F tab restores exactly what was set.
    expect(store.filters.vf.virtue).toEqual({
      search: 'giant',
      magnitude: 'major',
      category: 'general',
      taintedOnly: true,
    });
    // The Abilities tab keeps its own, independent state.
    expect(store.filters.abilities).toEqual({ search: 'latin', category: 'academic' });
    // The untouched flaw picker still holds its defaults.
    expect(store.filters.vf.flaw).toEqual({
      search: '',
      magnitude: '',
      category: '',
      taintedOnly: false,
    });
  });
});

// --- setType() --------------------------------------------------------------

describe('setType', () => {
  it('switches the entity type id in place, keeping selections', () => {
    installRuleset([item({ id: 'virtue.plain' })]);
    store.addSelection('virtue.plain');
    store.setType('magus');
    expect(store.entity.type_id).toBe('magus');
    expect(store.entity.selections).toEqual([{ ref: 'virtue.plain' }]);
  });

  it('is a no-op when the type is unchanged', () => {
    const before = store.entity;
    store.setType('companion');
    expect(store.entity).toBe(before);
    expect(store.entity.type_id).toBe('companion');
  });

  // A profile that mandates The Gift + Hermetic Magus (both free, profile-declared).
  const magusProfiles = {
    magus: {
      id: 'magus',
      budget: { virtue_points: 10, flaw_points: 10 },
      permitted_categories: [],
      forbidden_categories: [],
      required_traits: ['virtue.hermetic_magus'],
      gift_policy: 'required' as const,
      gift_id: 'virtue.the_gift',
      is_magus: true,
      creation_phases: [],
    },
  };
  const gift = () => item({ id: 'virtue.the_gift', magnitude: 'free', category: 'special' });
  const hermeticMagus = () =>
    item({ id: 'virtue.hermetic_magus', magnitude: 'free', category: 'social_status' });

  it("auto-selects a magus's mandatory free traits (The Gift + Hermetic Magus)", () => {
    installRuleset([gift(), hermeticMagus()], [], magusProfiles);
    store.setType('magus');
    const refs = store.entity.selections.map((s) => s.ref).sort();
    expect(refs).toEqual(['virtue.hermetic_magus', 'virtue.the_gift']);
  });

  it('does not duplicate a mandatory trait the user already selected', () => {
    installRuleset([gift(), hermeticMagus()], [], magusProfiles);
    store.addSelection('virtue.the_gift');
    store.setType('magus');
    const gifts = store.entity.selections.filter((s) => s.ref === 'virtue.the_gift');
    expect(gifts).toHaveLength(1);
  });

  it("drops the previous type's mandatory traits when they aren't mandated anymore", () => {
    installRuleset([gift(), hermeticMagus(), item({ id: 'virtue.plain' })], [], magusProfiles);
    store.addSelection('virtue.plain');
    store.setType('magus');
    store.setType('companion');
    const refs = store.entity.selections.map((s) => s.ref);
    // The user's own pick survives; the auto-added mandatory traits are gone.
    expect(refs).toEqual(['virtue.plain']);
  });
});

// --- addSelection() ---------------------------------------------------------

describe('addSelection', () => {
  it('dedups a plain (non-repeatable) item: adding twice yields one row', () => {
    installRuleset([item({ id: 'virtue.plain' })]);
    store.addSelection('virtue.plain');
    store.addSelection('virtue.plain');
    expect(store.entity.selections).toEqual([{ ref: 'virtue.plain' }]);
  });

  it('allows a parameterized item to appear several times', () => {
    installRuleset([
      item({
        id: 'virtue.great',
        parameters: [{ key: 'characteristic', type: 'ref', domain: 'characteristic' }],
      }),
    ]);
    store.addSelection('virtue.great');
    store.addSelection('virtue.great');
    expect(store.entity.selections).toEqual([{ ref: 'virtue.great' }, { ref: 'virtue.great' }]);
  });

  it('allows an item with max_per_target > 1 to appear several times', () => {
    installRuleset([item({ id: 'virtue.stacks', max_per_target: 3 })]);
    store.addSelection('virtue.stacks');
    store.addSelection('virtue.stacks');
    expect(store.entity.selections).toHaveLength(2);
  });

  it('treats an unknown ref as non-repeatable (max one row)', () => {
    installRuleset([]);
    store.addSelection('virtue.unknown');
    store.addSelection('virtue.unknown');
    expect(store.entity.selections).toEqual([{ ref: 'virtue.unknown' }]);
  });
});

// --- addSpell() / removeSpellAt() -------------------------------------------

describe('addSpell', () => {
  it('adds a fixed spell without a level', () => {
    store.addSpell('spell.pilum_of_fire');
    expect(store.entity.spells).toEqual([{ spell: 'spell.pilum_of_fire' }]);
  });

  it('dedups the same (spell, level) pair', () => {
    store.addSpell('spell.pilum_of_fire');
    store.addSpell('spell.pilum_of_fire');
    expect(store.entity.spells).toEqual([{ spell: 'spell.pilum_of_fire' }]);
  });

  it('stores the chosen level for a General spell', () => {
    store.addSpell('spell.aegis_of_the_hearth', 20);
    expect(store.entity.spells).toEqual([{ spell: 'spell.aegis_of_the_hearth', level: 20 }]);
  });

  it('lets the same General spell coexist at different levels', () => {
    store.addSpell('spell.aegis_of_the_hearth', 20);
    store.addSpell('spell.aegis_of_the_hearth', 25);
    expect(store.entity.spells).toHaveLength(2);
  });
});

describe('removeSpellAt', () => {
  it('removes only the row at the given index, keeping order', () => {
    store.addSpell('spell.a');
    store.addSpell('spell.b');
    store.addSpell('spell.c');
    store.removeSpellAt(1);
    expect((store.entity.spells ?? []).map((s) => s.spell)).toEqual(['spell.a', 'spell.c']);
  });
});

// --- adjustSpellMasteryAt() -------------------------------------------------

describe('adjustSpellMasteryAt', () => {
  it('raises the bought mastery of the spell at the given index', () => {
    store.addSpell('spell.pilum_of_fire');
    store.adjustSpellMasteryAt(0, 2, 5);
    expect(store.entity.spells?.[0].mastery).toBe(2);
  });

  it('clamps at 0 (never negative) and at the given max', () => {
    store.addSpell('spell.pilum_of_fire');
    store.adjustSpellMasteryAt(0, -3, 5); // floors at 0
    expect(store.entity.spells?.[0].mastery).toBe(0);
    store.adjustSpellMasteryAt(0, 99, 5); // clamps at max
    expect(store.entity.spells?.[0].mastery).toBe(5);
  });

  it('ignores an out-of-range index', () => {
    store.addSpell('spell.pilum_of_fire');
    store.adjustSpellMasteryAt(9, 1, 5);
    expect(store.entity.spells?.[0].mastery ?? 0).toBe(0);
  });
});

// --- setSpellLevelAt() ------------------------------------------------------

describe('setSpellLevelAt', () => {
  it('sets the level on the indexed spell, leaving others', () => {
    store.addSpell('spell.aegis_of_the_hearth', 5);
    store.addSpell('spell.wind_at_back', 5);
    store.setSpellLevelAt(0, 200);
    expect(store.entity.spells?.[0].level).toBe(200);
    expect(store.entity.spells?.[1].level).toBe(5);
  });

  it('marks the document dirty', () => {
    store.addSpell('spell.aegis_of_the_hearth', 5);
    store.setSpellLevelAt(0, 30);
    expect(store.dirty).toBe(true);
  });

  it('ignores an out-of-range index', () => {
    store.addSpell('spell.aegis_of_the_hearth', 5);
    store.setSpellLevelAt(9, 200);
    expect(store.entity.spells?.[0].level).toBe(5);
  });
});

// --- Phase 7: age, personality traits, reputations -------------------------

describe('setAge', () => {
  it('stores a positive integer and clears on null', () => {
    store.setAge(30);
    expect(store.entity.age).toBe(30);
    store.setAge(null);
    expect(store.entity.age).toBe(null);
  });

  it('clamps non-positive/non-finite to null', () => {
    store.setAge(0);
    expect(store.entity.age).toBe(null);
  });
});

describe('personality traits', () => {
  it('adds, names, values (clamped to ±6), and removes by index', () => {
    store.addPersonalityTrait();
    store.setPersonalityTraitName(0, 'Brave');
    store.setPersonalityTraitValue(0, 9);
    expect(store.entity.personality_traits).toEqual([{ name: 'Brave', value: 6 }]);
    store.addPersonalityTrait();
    store.removePersonalityTraitAt(0);
    expect(store.entity.personality_traits).toHaveLength(1);
  });
});

describe('reputations', () => {
  it('adds from a grant (kind + score), edits content, removes by index', () => {
    store.addReputation('local', 4);
    store.setReputationContent(0, 'dragon slayer');
    expect(store.entity.reputations).toEqual([
      { kind: 'local', score: 4, content: 'dragon slayer' },
    ]);
    store.removeReputationAt(0);
    expect(store.entity.reputations).toEqual([]);
  });
});

describe('magic possessions', () => {
  it('sets a signed aura and clears to 0 on null', () => {
    store.setAura(-3);
    expect(store.entity.aura).toBe(-3);
    store.setAura(null);
    expect(store.entity.aura).toBe(0);
  });

  it('adds, edits (name + non-negative level) and removes devices by index', () => {
    store.addDevice();
    store.setDeviceName(0, 'Wand');
    store.setDeviceLevel(0, -5);
    expect(store.entity.devices).toEqual([{ name: 'Wand', level: 0 }]);
    store.setDeviceLevel(0, 20);
    store.addDevice();
    store.removeDeviceAt(1);
    expect(store.entity.devices).toEqual([{ name: 'Wand', level: 20 }]);
  });

  it('adds a familiar, edits name and cords, and removes it', () => {
    store.addFamiliar();
    store.setFamiliarName('Corax');
    store.setFamiliarCord('bronze', 3);
    store.setFamiliarCord('gold', -2);
    expect(store.entity.familiar).toEqual({
      name: 'Corax',
      cord_gold: 0,
      cord_silver: 0,
      cord_bronze: 3,
    });
    store.removeFamiliar();
    expect(store.entity.familiar).toBeNull();
  });

  it('adds, edits and removes talisman attunements by index', () => {
    store.addTalismanAttunement();
    store.setTalismanDescription(0, 'Attuned to fire');
    store.setTalismanBonus(0, 5);
    expect(store.entity.talisman_attunements).toEqual([
      { description: 'Attuned to fire', bonus: 5 },
    ]);
    store.removeTalismanAttunementAt(0);
    expect(store.entity.talisman_attunements).toEqual([]);
  });

  it('sets Might realm + non-negative score, and clears Might', () => {
    store.setMightRealm('infernal');
    expect(store.entity.might).toEqual({ realm: 'infernal', score: 0 });
    store.setMightScore(-4);
    expect(store.entity.might).toEqual({ realm: 'infernal', score: 0 });
    store.setMightScore(5);
    expect(store.entity.might).toEqual({ realm: 'infernal', score: 5 });
    store.clearMight();
    expect(store.entity.might ?? null).toBeNull();
  });

  it('adds, edits (name + non-negative level) and removes supernatural powers', () => {
    store.addPower();
    store.setPowerName(0, 'Curse');
    store.setPowerLevel(0, -5);
    expect(store.entity.powers).toEqual([{ name: 'Curse', level: 0 }]);
    store.setPowerLevel(0, 20);
    store.addPower();
    store.removePowerAt(1);
    expect(store.entity.powers).toEqual([{ name: 'Curse', level: 20 }]);
  });

  it('adds a self-made longevity ritual (no bonus) and switches to external', () => {
    store.addLongevityRitual('self_made');
    expect(store.entity.longevity_ritual).toEqual({ source: 'self_made', bonus: null });
    store.setLongevitySource('external');
    store.setLongevityBonus(4);
    expect(store.entity.longevity_ritual).toEqual({ source: 'external', bonus: 4 });
    store.setLongevitySource('self_made');
    expect(store.entity.longevity_ritual).toEqual({ source: 'self_made', bonus: null });
    store.removeLongevityRitual();
    expect(store.entity.longevity_ritual).toBeNull();
  });
});

describe('aged / warped state + identity', () => {
  it('sets per-Characteristic aging points and prunes zeros', () => {
    store.setAgingPoints('str', 7);
    store.setAgingPoints('qik', -3); // clamps to 0 → removed
    expect(store.entity.aging_points).toEqual({ str: 7 });
    store.setAgingPoints('str', 0); // back to 0 → removed
    expect(store.entity.aging_points).toEqual({});
  });

  it('sets non-negative stored Warping Points', () => {
    store.setWarpingPoints(15);
    expect(store.entity.warping_points).toBe(15);
    store.setWarpingPoints(-4);
    expect(store.entity.warping_points).toBe(0);
  });

  it('adds, edits and removes Twilight Scars by index', () => {
    store.addTwilightScar();
    store.setTwilightScarDescription(0, 'Silver streak in the hair');
    expect(store.entity.twilight_scars).toEqual([{ description: 'Silver streak in the hair' }]);
    store.addTwilightScar();
    store.removeTwilightScarAt(0);
    expect(store.entity.twilight_scars).toEqual([{ description: '' }]);
  });

  it('sets and clears the apparent age', () => {
    store.setApparentAge(45);
    expect(store.entity.apparent_age).toBe(45);
    store.setApparentAge(0);
    expect(store.entity.apparent_age).toBeNull();
    store.setApparentAge(50);
    store.setApparentAge(null);
    expect(store.entity.apparent_age).toBeNull();
  });

  it('sets the free-text warping and decrepitude effect fields', () => {
    store.setWarpingEffect('A faint aura of ozone clings to him');
    store.setDecrepitudeEffect('Stooped and hard of hearing');
    expect(store.entity.warping_effect).toBe('A faint aura of ozone clings to him');
    expect(store.entity.decrepitude_effect).toBe('Stooped and hard of hearing');
  });

  it('adds, edits and removes aging-log entries by index', () => {
    store.addAgingLogEntry();
    store.setAgingLogEntryYear(0, 1215);
    store.setAgingLogEntryEffect(0, 'Lost a point of Stamina');
    expect(store.entity.aging_log).toEqual([{ year: 1215, effect: 'Lost a point of Stamina' }]);
    store.addAgingLogEntry();
    store.removeAgingLogEntryAt(0);
    expect(store.entity.aging_log).toEqual([{ year: 0, effect: '' }]);
  });

  it('sets free-text identity fields and birth year', () => {
    store.setIdentity('name', 'Marcus');
    store.setIdentity('description', 'Knight of the Teutonic Order');
    store.setIdentity('concept', 'A grim knight turned magus.');
    store.setIdentity('sigil', 'the smell of ozone');
    store.setBirthYear(1194);
    expect(store.entity.name).toBe('Marcus');
    expect(store.entity.description).toBe('Knight of the Teutonic Order');
    expect(store.entity.concept).toBe('A grim knight turned magus.');
    expect(store.entity.sigil).toBe('the smell of ozone');
    expect(store.entity.birth_year).toBe(1194);
    store.setBirthYear(null);
    expect(store.entity.birth_year).toBeNull();
  });
});

// --- removeSelectionAt() ----------------------------------------------------

describe('removeSelectionAt', () => {
  it('removes only the row at the given index, keeping order', () => {
    installRuleset([
      item({ id: 'v.a', max_per_target: 5 }),
      item({ id: 'v.b', max_per_target: 5 }),
      item({ id: 'v.c', max_per_target: 5 }),
    ]);
    store.addSelection('v.a');
    store.addSelection('v.b');
    store.addSelection('v.c');
    store.removeSelectionAt(1);
    expect(store.entity.selections.map((s) => s.ref)).toEqual(['v.a', 'v.c']);
  });

  it('is a no-op for an out-of-range index', () => {
    installRuleset([item({ id: 'v.a' })]);
    store.addSelection('v.a');
    store.removeSelectionAt(5);
    expect(store.entity.selections.map((s) => s.ref)).toEqual(['v.a']);
  });
});

// --- setParamAt() -----------------------------------------------------------

describe('setParamAt', () => {
  it('sets a param on the targeted row only', () => {
    installRuleset([item({ id: 'v.great', max_per_target: 5 })]);
    store.addSelection('v.great');
    store.addSelection('v.great');
    store.setParamAt(0, 'characteristic', 'characteristic.per');
    expect(store.entity.selections[0]).toEqual({
      ref: 'v.great',
      params: { characteristic: 'characteristic.per' },
    });
    expect(store.entity.selections[1]).toEqual({ ref: 'v.great' });
  });

  it('merges into existing params rather than replacing them', () => {
    installRuleset([item({ id: 'v.x' })]);
    store.addSelection('v.x');
    store.setParamAt(0, 'a', '1');
    store.setParamAt(0, 'b', '2');
    expect(store.entity.selections[0].params).toEqual({ a: '1', b: '2' });
  });
});

// --- setAbilityBonusTarget() ------------------------------------------------

describe('setAbilityBonusTarget', () => {
  beforeEach(() => {
    installRuleset(
      [item({ id: 'virtue.puissant', max_per_target: 5 })],
      [ability('ability.awareness'), ability('ability.area_lore', 'area')],
    );
    store.addSelection('virtue.puissant');
  });

  it('stores the instance key for a parameterized ability', () => {
    store.setAbilityBonusTarget(0, 'ability.area_lore', 'Rhine');
    expect(store.entity.selections[0].params).toEqual({
      ability: 'ability.area_lore',
      area: 'Rhine',
    });
  });

  it('stores just the ability for a plain ability (no instance key)', () => {
    store.setAbilityBonusTarget(0, 'ability.awareness');
    expect(store.entity.selections[0].params).toEqual({ ability: 'ability.awareness' });
  });

  it('drops the stale instance key when switching to a plain ability', () => {
    // First point it at a parameterized instance...
    store.setAbilityBonusTarget(0, 'ability.area_lore', 'Rhine');
    expect(store.entity.selections[0].params).toHaveProperty('area', 'Rhine');
    // ...then switch the target to a plain ability: the params object is
    // rebuilt, so the stale `area` key is gone, not merged.
    store.setAbilityBonusTarget(0, 'ability.awareness');
    expect(store.entity.selections[0].params).toEqual({ ability: 'ability.awareness' });
    expect(store.entity.selections[0].params).not.toHaveProperty('area');
  });

  it('omits the instance key when a parameterized ability has no parameter value', () => {
    store.setAbilityBonusTarget(0, 'ability.area_lore', null);
    expect(store.entity.selections[0].params).toEqual({ ability: 'ability.area_lore' });
  });
});

// --- setCharacteristic() ----------------------------------------------------

describe('setCharacteristic', () => {
  it('stores a non-zero score', () => {
    store.setCharacteristic('int', 3);
    expect(store.entity.characteristics).toEqual({ int: 3 });
  });

  it('stores a negative score', () => {
    store.setCharacteristic('per', -2);
    expect(store.entity.characteristics).toEqual({ per: -2 });
  });

  it('removes the entry when the score is set to 0', () => {
    store.setCharacteristic('int', 3);
    store.setCharacteristic('per', 1);
    store.setCharacteristic('int', 0);
    expect(store.entity.characteristics).toEqual({ per: 1 });
    expect(store.entity.characteristics).not.toHaveProperty('int');
  });
});

// --- setXpPool() ------------------------------------------------------------

describe('setXpPool', () => {
  it('stores a positive integer', () => {
    store.setXpPool(45);
    expect(store.entity.xp_pool).toBe(45);
  });

  it('floors a fractional value', () => {
    store.setXpPool(45.9);
    expect(store.entity.xp_pool).toBe(45);
  });

  it('clamps non-positive and non-finite values to 0', () => {
    store.setXpPool(-5);
    expect(store.entity.xp_pool).toBe(0);
    store.setXpPool(0);
    expect(store.entity.xp_pool).toBe(0);
    store.setXpPool(Number.NaN);
    expect(store.entity.xp_pool).toBe(0);
  });
});

// --- Art actions ------------------------------------------------------------

describe('art actions', () => {
  it('upserts an Art score by id, clamping to [0, max]', () => {
    // First raise creates the entry; all 15 Arts are always present in the UI, so
    // there is no separate "add" step.
    store.adjustArt('art.ignem', 3, 20);
    expect(store.entity.art_scores).toEqual([{ art: 'art.ignem', score: 3 }]);
    store.adjustArt('art.ignem', 99, 20); // clamps at max
    expect(store.entity.art_scores?.[0].score).toBe(20);
  });

  it('drops an Art entry when its score returns to 0 (sparse save)', () => {
    store.adjustArt('art.creo', 2, 20);
    expect(store.entity.art_scores).toEqual([{ art: 'art.creo', score: 2 }]);
    store.adjustArt('art.creo', -5, 20); // clamps at 0 and removes the entry
    expect(store.entity.art_scores).toEqual([]);
  });

  it('tracks several Arts independently', () => {
    store.adjustArt('art.creo', 2, 20);
    store.adjustArt('art.ignem', 4, 20);
    expect(store.entity.art_scores).toEqual([
      { art: 'art.creo', score: 2 },
      { art: 'art.ignem', score: 4 },
    ]);
  });

  it('points a Puissant Art selection at an Art by id', () => {
    installRuleset([
      item({
        id: 'virtue.puissant_art',
        category: 'hermetic',
        parameters: [{ key: 'art', type: 'ref', domain: 'art' }],
        effects: [{ type: 'art_bonus', param: 'art', amount: 3 }],
      }),
    ]);
    store.addSelection('virtue.puissant_art');
    store.setArtBonusTarget(0, 'art.ignem');
    expect(store.entity.selections[0].params).toEqual({ art: 'art.ignem' });
  });
});

// --- House actions ----------------------------------------------------------

// Houses exercising each grant kind: a fixed Virtue (Tytalus), a Choice between
// two Puissant Arts (Flambeau), and an open Minor Virtue (Jerbiton).
const HOUSES: House[] = [
  {
    id: 'house.tytalus',
    lineage_type: 'societas',
    grants: [{ kind: 'fixed', item: 'virtue.self_confident' }],
  },
  {
    id: 'house.flambeau',
    lineage_type: 'societas',
    grants: [
      {
        kind: 'choice',
        choice_key: 'flambeau_puissant',
        options: [
          { ref: 'virtue.puissant_art', params: { art: 'art.perdo' } },
          { ref: 'virtue.puissant_art', params: { art: 'art.ignem' } },
        ],
      },
    ],
  },
  {
    id: 'house.jerbiton',
    lineage_type: 'societas',
    grants: [
      {
        kind: 'open',
        choice_key: 'jerbiton_virtue',
        constraint: { kind: 'virtue', magnitude: 'minor' },
      },
    ],
  },
];

/** Install a House catalogue on the already-installed ruleset. */
function installHouses(houses: House[]): void {
  const map: Record<string, House> = {};
  for (const h of houses) map[h.id] = h;
  store.ruleset!.ruleset.houses = map;
}

describe('setHouse', () => {
  beforeEach(() => {
    installRuleset([]);
    installHouses(HOUSES);
  });

  it('sets the chosen house id', async () => {
    await store.setHouse('house.flambeau');
    expect(store.entity.house).toBe('house.flambeau');
  });

  it('clears the house and all its choices with null', async () => {
    await store.setHouse('house.flambeau');
    store.setHouseChoice('flambeau_puissant', {
      ref: 'virtue.puissant_art',
      params: { art: 'art.ignem' },
    });
    await store.setHouse(null);
    expect(store.entity.house).toBeNull();
    expect(store.entity.house_choices).toEqual({});
  });

  it('drops choices whose choice_key the new house no longer defines', async () => {
    await store.setHouse('house.flambeau');
    store.setHouseChoice('flambeau_puissant', {
      ref: 'virtue.puissant_art',
      params: { art: 'art.ignem' },
    });
    // Tytalus grants only a fixed Virtue — no choice keys — so the stale
    // Flambeau pick must not linger.
    await store.setHouse('house.tytalus');
    expect(store.entity.house).toBe('house.tytalus');
    expect(store.entity.house_choices).toEqual({});
  });

  it('is a no-op when the house is unchanged', async () => {
    await store.setHouse('house.jerbiton');
    const before = store.entity.house_choices;
    await store.setHouse('house.jerbiton');
    // Same reference: the second call returned early rather than re-pruning.
    expect(store.entity.house_choices).toBe(before);
  });
});

describe('setHouseChoice', () => {
  it('stores a pick under the choice key', () => {
    const pick = { ref: 'virtue.puissant_art', params: { art: 'art.ignem' } };
    store.setHouseChoice('flambeau_puissant', pick);
    expect(store.entity.house_choices).toEqual({ flambeau_puissant: pick });
  });

  it('replaces an existing pick for the same key', () => {
    store.setHouseChoice('flambeau_puissant', {
      ref: 'virtue.puissant_art',
      params: { art: 'art.perdo' },
    });
    store.setHouseChoice('flambeau_puissant', {
      ref: 'virtue.puissant_art',
      params: { art: 'art.ignem' },
    });
    expect(store.entity.house_choices).toEqual({
      flambeau_puissant: { ref: 'virtue.puissant_art', params: { art: 'art.ignem' } },
    });
  });

  it('keeps picks for other keys independently', () => {
    store.setHouseChoice('a', { ref: 'virtue.x' });
    store.setHouseChoice('b', { ref: 'virtue.y' });
    expect(store.entity.house_choices).toEqual({
      a: { ref: 'virtue.x' },
      b: { ref: 'virtue.y' },
    });
  });
});

// --- addAbility() -----------------------------------------------------------

describe('addAbility', () => {
  it('adds a plain ability at score 0 and dedups a second add', () => {
    installRuleset([], [ability('ability.awareness')]);
    store.addAbility('ability.awareness');
    store.addAbility('ability.awareness');
    expect(store.entity.ability_scores).toEqual([{ ability: 'ability.awareness', score: 0 }]);
  });

  it('allows a parameterized ability to be added several times', () => {
    installRuleset([], [ability('ability.area_lore', 'area')]);
    store.addAbility('ability.area_lore');
    store.addAbility('ability.area_lore');
    expect(store.entity.ability_scores).toEqual([
      { ability: 'ability.area_lore', score: 0 },
      { ability: 'ability.area_lore', score: 0 },
    ]);
  });
});

// --- removeAbilityAt() ------------------------------------------------------

describe('removeAbilityAt', () => {
  it('removes only the row at the given index, keeping order', () => {
    store.entity.ability_scores = [
      { ability: 'ability.a', score: 1 },
      { ability: 'ability.b', score: 2 },
      { ability: 'ability.c', score: 3 },
    ];
    store.removeAbilityAt(1);
    expect(store.entity.ability_scores).toEqual([
      { ability: 'ability.a', score: 1 },
      { ability: 'ability.c', score: 3 },
    ]);
  });

  it('is a no-op for an out-of-range index', () => {
    store.entity.ability_scores = [{ ability: 'ability.a', score: 1 }];
    store.removeAbilityAt(5);
    expect(store.entity.ability_scores).toEqual([{ ability: 'ability.a', score: 1 }]);
  });
});

// --- adjustAbilityAt() ------------------------------------------------------

describe('adjustAbilityAt', () => {
  beforeEach(() => {
    store.entity.ability_scores = [
      { ability: 'ability.a', score: 2 },
      { ability: 'ability.b', score: 5 },
    ];
  });

  it('raises the score by a positive delta within range', () => {
    store.adjustAbilityAt(0, 2, 10);
    expect(store.entity.ability_scores![0].score).toBe(4);
    // other rows untouched
    expect(store.entity.ability_scores![1].score).toBe(5);
  });

  it('clamps to 0 when the delta would push below 0', () => {
    store.adjustAbilityAt(0, -5, 10);
    expect(store.entity.ability_scores![0].score).toBe(0);
  });

  it('clamps to max when the delta would push above max', () => {
    store.adjustAbilityAt(1, 99, 7);
    expect(store.entity.ability_scores![1].score).toBe(7);
  });

  it('is a no-op for an out-of-range index', () => {
    store.adjustAbilityAt(9, 3, 10);
    expect(store.entity.ability_scores!.map((a) => a.score)).toEqual([2, 5]);
  });
});

// --- setCharacteristicDescription() -----------------------------------------

describe('setCharacteristicDescription', () => {
  it('stores the (untrimmed) text when it is non-blank', () => {
    // The trim() is only an emptiness guard; the raw text is stored verbatim.
    store.setCharacteristicDescription('int', '  Sharp wit  ');
    expect(store.entity.characteristic_descriptions).toEqual({ int: '  Sharp wit  ' });
  });

  it('removes the entry when the text is blank or whitespace-only', () => {
    store.setCharacteristicDescription('int', 'Sharp wit');
    store.setCharacteristicDescription('per', 'Keen eyes');
    store.setCharacteristicDescription('int', '   ');
    expect(store.entity.characteristic_descriptions).toEqual({ per: 'Keen eyes' });
    expect(store.entity.characteristic_descriptions).not.toHaveProperty('int');
  });
});

// --- setAbilitySpecialtyAt() ------------------------------------------------

describe('setAbilitySpecialtyAt', () => {
  beforeEach(() => {
    store.entity.ability_scores = [
      { ability: 'ability.a', score: 1 },
      { ability: 'ability.b', score: 2 },
    ];
  });

  it('stores a trimmed specialty on the targeted row only', () => {
    store.setAbilitySpecialtyAt(0, '  Birds  ');
    expect(store.entity.ability_scores![0]).toEqual({
      ability: 'ability.a',
      score: 1,
      specialty: 'Birds',
    });
    expect(store.entity.ability_scores![1]).toEqual({ ability: 'ability.b', score: 2 });
  });

  it('sets specialty to undefined for blank/whitespace input', () => {
    store.setAbilitySpecialtyAt(0, 'Birds');
    store.setAbilitySpecialtyAt(0, '   ');
    expect(store.entity.ability_scores![0].specialty).toBeUndefined();
  });
});

// --- setAbilityParameterAt() ------------------------------------------------

describe('setAbilityParameterAt', () => {
  beforeEach(() => {
    store.entity.ability_scores = [
      { ability: 'ability.area_lore', score: 1 },
      { ability: 'ability.area_lore', score: 2 },
    ];
  });

  it('stores a trimmed parameter on the targeted row only', () => {
    store.setAbilityParameterAt(1, '  Rhine  ');
    expect(store.entity.ability_scores![1]).toEqual({
      ability: 'ability.area_lore',
      score: 2,
      parameter: 'Rhine',
    });
    expect(store.entity.ability_scores![0]).toEqual({ ability: 'ability.area_lore', score: 1 });
  });

  it('sets parameter to undefined for blank input', () => {
    store.setAbilityParameterAt(0, 'Rhine');
    store.setAbilityParameterAt(0, '   ');
    expect(store.entity.ability_scores![0].parameter).toBeUndefined();
  });
});

// --- Mythic Companion type selection ----------------------------------------

describe('setMythicType / required package', () => {
  // A mythic profile plus a Devil-Child-like type: a fixed status grant, a
  // `choice` free-Minor, two required Virtues (one parameterized), and one
  // required Flaw with a substitute constraint.
  function installMythic(): void {
    installRuleset(
      [
        item({ id: 'virtue.status', magnitude: 'free', category: 'social_status' }),
        item({ id: 'virtue.min_a', category: 'supernatural' }),
        item({ id: 'virtue.min_b', category: 'supernatural' }),
        item({ id: 'virtue.req_major', magnitude: 'major', category: 'supernatural' }),
        item({
          id: 'virtue.puissant',
          category: 'general',
          parameters: [{ key: 'ability', type: 'ref', domain: 'ability' }],
        }),
        item({ id: 'flaw.default_major', kind: 'flaw', magnitude: 'major', category: 'story' }),
        item({ id: 'flaw.other_major', kind: 'flaw', magnitude: 'major', category: 'story' }),
      ],
      [],
      {
        mythic_companion: {
          id: 'mythic_companion',
          budget: { virtue_points: 20, flaw_points: 10, virtue_points_per_flaw_point: 2 },
          permitted_categories: [],
          forbidden_categories: [],
          has_mythic_type: true,
          creation_phases: [],
        },
      },
    );
    store.ruleset!.ruleset.mythic_companion_types = {
      'mythic_type.devil': {
        id: 'mythic_type.devil',
        grants: [
          { kind: 'fixed', item: 'virtue.status' },
          {
            kind: 'choice',
            choice_key: 'free_minor',
            options: [{ ref: 'virtue.min_a' }, { ref: 'virtue.min_b' }],
          },
        ],
        required_virtues: [
          { ref: 'virtue.req_major' },
          { ref: 'virtue.puissant', params: { ability: 'ability.guile' } },
        ],
        required_flaws: [
          {
            default: { ref: 'flaw.default_major' },
            constraint: { kind: 'flaw', magnitude: 'major', require_categories: ['story'] },
          },
        ],
      },
      'mythic_type.other': {
        id: 'mythic_type.other',
        grants: [{ kind: 'fixed', item: 'virtue.status' }],
        required_virtues: [{ ref: 'virtue.req_major' }],
      },
    };
    store.entity.type_id = 'mythic_companion';
  }

  beforeEach(installMythic);

  it('seeds the required package (with params) and defaults the free-Minor choice', async () => {
    await store.setMythicType('mythic_type.devil');
    expect(store.entity.mythic_type).toBe('mythic_type.devil');
    // Required Virtues (incl. the parameterized Puissant) + the default Flaw are seeded.
    expect(store.entity.selections).toContainEqual({ ref: 'virtue.req_major' });
    expect(store.entity.selections).toContainEqual({
      ref: 'virtue.puissant',
      params: { ability: 'ability.guile' },
    });
    expect(store.entity.selections).toContainEqual({ ref: 'flaw.default_major' });
    // The free status/Minor Virtues are grants, never bought selections.
    expect(store.entity.selections.some((s) => s.ref === 'virtue.status')).toBe(false);
    // The `choice` free-Minor defaults to its first option.
    expect(store.entity.mythic_choices?.free_minor).toEqual({ ref: 'virtue.min_a' });
  });

  it('swaps the package when the type changes and drops it when cleared', async () => {
    await store.setMythicType('mythic_type.devil');
    await store.setMythicType('mythic_type.other');
    // Devil-only rows (Puissant, the default Flaw) are dropped; the shared
    // req_major stays; the stale free_minor choice is pruned.
    expect(store.entity.selections.some((s) => s.ref === 'virtue.puissant')).toBe(false);
    expect(store.entity.selections.some((s) => s.ref === 'flaw.default_major')).toBe(false);
    expect(store.entity.selections).toContainEqual({ ref: 'virtue.req_major' });
    expect(store.entity.mythic_choices?.free_minor).toBeUndefined();

    await store.setMythicType(null);
    expect(store.entity.mythic_type).toBeNull();
    expect(store.entity.selections.some((s) => s.ref === 'virtue.req_major')).toBe(false);
  });

  it('swaps a required Flaw for a substitute', async () => {
    await store.setMythicType('mythic_type.devil');
    await store.setMythicRequiredFlaw('flaw.default_major', 'flaw.other_major');
    expect(store.entity.selections.some((s) => s.ref === 'flaw.default_major')).toBe(false);
    expect(store.entity.selections).toContainEqual({ ref: 'flaw.other_major' });
  });
});

// --- unsaved-changes tracking (close/quit guard) ----------------------------

describe('unsaved-changes tracking', () => {
  /** A full, minimal character used as a freshly-loaded (clean) baseline. */
  function cleanEntity(): Entity {
    return {
      schema_version: 11,
      ruleset: { id: 'test', version: '1' },
      entity_kind: 'character',
      type_id: 'companion',
      selections: [],
      characteristics: {} as Entity['characteristics'],
      characteristic_descriptions: {},
      ability_scores: [],
      xp_pool: 0,
      art_scores: [],
      personality_traits: [],
      reputations: [],
    };
  }

  /** Drive a real open so the store captures a clean saved-baseline. */
  async function loadClean(path = '/tmp/marcus.armc'): Promise<void> {
    vi.mocked(ipc.loadEntity).mockResolvedValue({ path, entity: cleanEntity() });
    const opening = store.open();
    // The shared singleton may be dirty from a prior test; open() then shows the
    // discard prompt. Confirm it so this setup helper always reaches the load.
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;
  }

  it('is not dirty once a file has just been loaded', async () => {
    await loadClean();
    expect(store.dirty).toBe(false);
  });

  it('becomes dirty after any mutation', async () => {
    await loadClean();
    store.setIdentity('name', 'Marcus of Bonisagus');
    expect(store.dirty).toBe(true);
  });

  it('becomes dirty after an aging/warping annotation edit', async () => {
    await loadClean();
    store.setWarpingEffect('A stigmatic scar');
    expect(store.dirty).toBe(true);
  });

  it('clears dirty after a successful save (non-null path)', async () => {
    await loadClean();
    store.setIdentity('name', 'Marcus');
    expect(store.dirty).toBe(true);

    vi.mocked(ipc.saveEntity).mockResolvedValue('/tmp/marcus.armc');
    await store.save();
    expect(store.dirty).toBe(false);
  });

  it('stays dirty after a cancelled save (null return)', async () => {
    await loadClean();
    // Clear the current file so save() routes to Save As (which prompts and can
    // be cancelled). A cancelled prompt returns null: nothing was written.
    store.currentPath = null;
    store.setIdentity('name', 'Marcus');

    vi.mocked(ipc.saveEntity).mockResolvedValue(null);
    await store.save();
    expect(store.dirty).toBe(true);
  });

  it('stays dirty after a language reload that keeps the edited entity', async () => {
    await loadClean();
    store.setIdentity('name', 'Marcus');
    expect(store.dirty).toBe(true);

    // setLang → #reloadRuleset(false): keeps the entity, must NOT clear dirty.
    vi.mocked(ipc.loadRuleset).mockResolvedValue(installRuleset([]));
    await store.setLang('de');
    expect(store.dirty).toBe(true);
  });

  it('keeps edits made while the save dialog is open marked dirty', async () => {
    await loadClean();
    store.setIdentity('name', 'Marcus');

    // Model a slow native dialog: capture the snapshot at call time, resolve later.
    let resolveSave: (path: string | null) => void = () => {};
    vi.mocked(ipc.saveEntity).mockReturnValue(
      new Promise<string | null>((resolve) => {
        resolveSave = resolve;
      }),
    );
    const saving = store.save();
    // User keeps typing while the dialog is open.
    store.setIdentity('name', 'Marcus of Bonisagus');
    resolveSave('/tmp/marcus.armc');
    await saving;

    // The write captured the earlier state, so the newer edit is still unsaved.
    expect(store.dirty).toBe(true);
  });

  it('reports dirty + localized labels to the backend guard', async () => {
    await loadClean();
    let payload = store.closeGuardPayload();
    expect(payload.dirty).toBe(false);
    expect(payload.labels.title).toBe(store.t('close-unsaved-title'));
    expect(payload.labels.message).toBe(store.t('close-unsaved-message'));
    expect(payload.labels.discard).toBe(store.t('close-unsaved-discard'));
    expect(payload.labels.cancel).toBe(store.t('close-unsaved-cancel'));

    store.setIdentity('name', 'Marcus');
    payload = store.closeGuardPayload();
    expect(payload.dirty).toBe(true);
  });
});

// --- document file model (current file, Save vs Save As, New, in-flight) -----

describe('document file model', () => {
  function cleanEntity(): Entity {
    return {
      schema_version: 11,
      ruleset: { id: 'test', version: '1' },
      entity_kind: 'character',
      type_id: 'companion',
      selections: [],
      characteristics: {} as Entity['characteristics'],
      characteristic_descriptions: {},
      ability_scores: [],
      xp_pool: 0,
      art_scores: [],
      personality_traits: [],
      reputations: [],
    };
  }

  /** Open a file with a known path so the store tracks it as the current file. */
  async function openFile(path = '/tmp/marcus.armc'): Promise<void> {
    vi.mocked(ipc.loadEntity).mockResolvedValue({ path, entity: cleanEntity() });
    const opening = store.open();
    // The shared singleton may be dirty from a prior test; confirm the discard
    // prompt so this setup helper always reaches the load.
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;
  }

  beforeEach(() => {
    vi.mocked(ipc.saveEntity).mockReset();
    vi.mocked(ipc.loadEntity).mockReset();
    store.currentPath = null;
    store.filters = defaultPickerFilters();
  });

  it('open() sets the current file path from the loaded document', async () => {
    await openFile('/tmp/aelius.armc');
    expect(store.currentPath).toBe('/tmp/aelius.armc');
    expect(store.currentFileName).toBe('aelius.armc');
  });

  it('save() writes directly to the current file without prompting', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.saveEntity).mockResolvedValue('/tmp/marcus.armc');

    await store.save();

    // Passed the known path (no prompt) — second arg is the current file.
    expect(ipc.saveEntity).toHaveBeenCalledTimes(1);
    expect(vi.mocked(ipc.saveEntity).mock.calls[0][1]).toBe('/tmp/marcus.armc');
    expect(store.dirty).toBe(false);
  });

  it('save() with no current file falls back to Save As (prompt)', async () => {
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.saveEntity).mockResolvedValue('/tmp/new.armc');

    await store.save();

    // Prompted: path arg is null so the backend opens the dialog.
    expect(vi.mocked(ipc.saveEntity).mock.calls[0][1]).toBeNull();
    expect(store.currentPath).toBe('/tmp/new.armc');
  });

  it('saveAs() always prompts and updates the current file on success', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.saveEntity).mockResolvedValue('/tmp/renamed.armc');

    await store.saveAs();

    expect(vi.mocked(ipc.saveEntity).mock.calls[0][1]).toBeNull();
    expect(store.currentPath).toBe('/tmp/renamed.armc');
    expect(store.dirty).toBe(false);
  });

  it('saveAs() cancel leaves the current file unchanged and stays dirty', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.saveEntity).mockResolvedValue(null);

    await store.saveAs();

    expect(store.currentPath).toBe('/tmp/marcus.armc');
    expect(store.dirty).toBe(true);
  });

  it('save() failure surfaces an error and keeps the document dirty', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.saveEntity).mockRejectedValue({ kind: 'io' });

    await store.save();

    expect(store.error).toEqual({ kind: 'io' });
    expect(store.dirty).toBe(true);
    expect(store.currentPath).toBe('/tmp/marcus.armc');
  });

  it('a second save while one is in flight is a no-op (guarded)', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    // A save that stays pending models a still-open write; the second call must
    // not re-enter while the first is unresolved.
    let finishFirst: (path: string | null) => void = () => {};
    vi.mocked(ipc.saveEntity).mockReturnValue(
      new Promise<string | null>((resolve) => {
        finishFirst = resolve;
      }),
    );

    const first = store.save();
    void store.save();

    expect(ipc.saveEntity).toHaveBeenCalledTimes(1);

    // Let the first write finish so the in-flight guard clears for later tests.
    finishFirst('/tmp/marcus.armc');
    await first;
  });

  it('newDocument() resets the current file, filters, and dirty baseline', async () => {
    await openFile('/tmp/marcus.armc');
    store.filters.abilities.search = 'latin';
    // Clean document, so no discard prompt is needed.
    await store.newDocument();

    expect(store.currentPath).toBeNull();
    expect(store.dirty).toBe(false);
    expect(store.filters.abilities.search).toBe('');
    expect(store.entity.name).toBeUndefined();
  });

  it('newDocument() on a dirty document waits for the discard prompt', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');

    // Cancelling the prompt aborts: the edited document is kept.
    const cancelled = store.newDocument();
    expect(store.discardPromptOpen).toBe(true);
    store.resolveDiscardPrompt(false);
    await cancelled;
    expect(store.entity.name).toBe('Marcus');
    expect(store.currentPath).toBe('/tmp/marcus.armc');

    // Confirming discards and resets to a fresh document.
    const confirmed = store.newDocument();
    expect(store.discardPromptOpen).toBe(true);
    store.resolveDiscardPrompt(true);
    await confirmed;
    expect(store.entity.name).toBeUndefined();
    expect(store.currentPath).toBeNull();
    expect(store.dirty).toBe(false);
  });

  it('open() on a dirty document honors the discard prompt', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');

    // Ignore the setup helper's load; only the cancelled open below matters.
    vi.mocked(ipc.loadEntity).mockClear();
    vi.mocked(ipc.loadEntity).mockResolvedValue({ path: '/tmp/other.armc', entity: cleanEntity() });
    const opening = store.open();
    expect(store.discardPromptOpen).toBe(true);
    store.resolveDiscardPrompt(false);
    await opening;
    // Cancelled: the current file is unchanged and loadEntity was never called.
    expect(store.currentPath).toBe('/tmp/marcus.armc');
    expect(ipc.loadEntity).not.toHaveBeenCalled();
  });
});
