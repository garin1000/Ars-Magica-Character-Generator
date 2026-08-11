import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type {
  Ability,
  CreationPhase,
  Entity,
  EntityTypeProfile,
  House,
  LocalizedRuleset,
  PointItem,
  Spell,
} from './types';

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
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
}));

// Import the singleton after the mock is registered.
import * as ipc from './ipc';
import { store, defaultPickerFilters } from './state.svelte';

// The screen the app boots on, captured at import time — before any test or
// `beforeEach` has touched the shared singleton, which is the only moment the
// initial value is still observable.
const bootView = store.view;

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

// A profile that mandates The Gift + Hermetic Magus (both free, profile-declared).
// `createCharacter` seeds exactly this data-driven set of mandatory free traits.
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
  // Almost every block below edits a character, which only exists in the editor —
  // and `revalidate()` is deliberately inert on the startup screen. Pin the editor
  // here so each test starts where it was written to run; the startup-screen
  // blocks set their own view.
  store.view = 'editor';
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
    expect(store.filters.spells).toEqual({
      search: '',
      technique: '',
      form: '',
      levelMin: null,
      levelMax: null,
    });
    expect(defaultPickerFilters().derivedArtPicker).toEqual({ technique: '', form: '' });
  });

  it('persists the Derived-Totals Technique/Form picker without dirtying the entity', () => {
    // The Totals tab unmounts on switch, so its Art picker lives on the store's
    // filters. It is not part of the saved entity snapshot, so setting it must
    // leave the dirty flag untouched (dirty derives from the entity alone).
    const dirtyBefore = store.dirty;
    store.filters.derivedArtPicker.technique = 'art.creo';
    store.filters.derivedArtPicker.form = 'art.ignem';

    // Reading it back on the store proves it survives a panel unmount.
    expect(store.filters.derivedArtPicker).toEqual({
      technique: 'art.creo',
      form: 'art.ignem',
    });
    expect(store.dirty).toBe(dirtyBefore);
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

// --- startup view + createCharacter() (M6a) ---------------------------------

describe('createCharacter', () => {
  beforeEach(() => {
    store.currentPath = null;
    store.filters = defaultPickerFilters();
  });

  it('boots on the start view', () => {
    expect(bootView).toBe('start');
  });

  it('creates a character of the chosen type and enters the editor', async () => {
    installRuleset([gift(), hermeticMagus()], [], magusProfiles);
    await store.createCharacter('magus');
    expect(store.entity.type_id).toBe('magus');
    expect(store.view).toBe('editor');
  });

  it("seeds the profile's mandatory free traits (The Gift + Hermetic Magus)", async () => {
    installRuleset([gift(), hermeticMagus()], [], magusProfiles);
    await store.createCharacter('magus');
    const refs = store.entity.selections!.map((s) => s.ref).sort();
    expect(refs).toEqual(['virtue.hermetic_magus', 'virtue.the_gift']);
  });

  it('discards the previous character entirely', async () => {
    installRuleset([item({ id: 'virtue.plain' })], [ability('ability.awareness')]);
    store.addSelection('virtue.plain');
    store.addAbility('ability.awareness');
    store.setIdentity('name', 'Marcus');

    await store.createCharacter('grog');

    expect(store.entity.type_id).toBe('grog');
    expect(store.entity.selections).toEqual([]);
    expect(store.entity.ability_scores).toEqual([]);
    expect(store.entity.name).toBeUndefined();
  });

  it('clears the current file and leaves the document not dirty', async () => {
    installRuleset([]);
    store.currentPath = '/tmp/marcus.armc';
    store.setIdentity('name', 'Marcus');

    await store.createCharacter('companion');

    expect(store.currentPath).toBeNull();
    expect(store.dirty).toBe(false);
  });

  it('resets the picker filters', async () => {
    installRuleset([]);
    store.filters.abilities.search = 'latin';
    store.filters.vf.virtue.magnitude = 'major';

    await store.createCharacter('companion');

    expect(store.filters).toEqual(defaultPickerFilters());
  });
});

// --- the guided wizard (M6b1b) -----------------------------------------------

describe('the guided wizard', () => {
  /** A magus-like profile with a real, ordered flow for the rail to walk. */
  const wizardProfiles = {
    magus: {
      ...magusProfiles.magus,
      creation_phases: [
        'concept',
        'type',
        'characteristics',
        'house_specialisation',
        'virtues_flaws',
        'abilities',
      ] as CreationPhase[],
    },
  };

  /** Install a validation result the wizard's gating reads. */
  function issues(...list: { phase: CreationPhase; severity?: 'error' | 'warning' }[]): void {
    store.result = {
      issues: list.map(({ phase, severity }) => ({
        severity: severity ?? 'error',
        code: 'x',
        phase,
        args: {},
      })),
    };
  }

  beforeEach(() => {
    installRuleset([gift(), hermeticMagus()], [], wizardProfiles);
    store.result = null;
  });

  describe('startWizard', () => {
    it('creates a real character of the chosen type and enters the wizard', async () => {
      await store.startWizard('magus');
      expect(store.view).toBe('wizard');
      expect(store.entity.type_id).toBe('magus');
    });

    it("seeds the profile's mandatory free traits, exactly as createCharacter does", async () => {
      await store.startWizard('magus');
      const refs = store.entity.selections!.map((s) => s.ref).sort();
      expect(refs).toEqual(['virtue.hermetic_magus', 'virtue.the_gift']);
    });

    it('starts a clean document at the first step', async () => {
      store.currentPath = '/tmp/old.armc';
      await store.startWizard('magus');
      expect(store.currentPath).toBeNull();
      expect(store.dirty).toBe(false);
      expect(store.wizardStep).toBe(0);
      expect(store.wizardPhase).toBe('concept');
    });

    // The wizard's entity is a real character, unlike the start screen's
    // placeholder — so validation must actually run, or nothing would ever gate.
    it('validates immediately rather than standing down like the start screen', async () => {
      vi.mocked(ipc.validateEntity).mockClear();
      await store.startWizard('magus');
      expect(ipc.validateEntity).toHaveBeenCalledTimes(1);
    });

    it("walks the profile's phases and then the wizard's own Review step", async () => {
      await store.startWizard('magus');
      expect(store.wizardPhases).toEqual([
        'concept',
        'type',
        'characteristics',
        'house_specialisation',
        'virtues_flaws',
        'abilities',
        'review',
      ]);
    });
  });

  describe('navigation', () => {
    beforeEach(async () => {
      await store.startWizard('magus');
    });

    it('advances and raises the furthest-reached step', () => {
      store.wizardNext();
      expect(store.wizardStep).toBe(1);
      expect(store.wizardFurthest).toBe(1);
    });

    it('refuses to advance past an error in the current phase', () => {
      issues({ phase: 'concept' });
      store.wizardNext();
      expect(store.wizardStep).toBe(0);
      expect(store.wizardCanAdvance).toBe(false);
    });

    it('advances past a warning — an advisory is not an illegal state', () => {
      issues({ phase: 'concept', severity: 'warning' });
      store.wizardNext();
      expect(store.wizardStep).toBe(1);
    });

    it('goes back freely, even when the phase left behind is broken', () => {
      store.wizardNext();
      issues({ phase: 'type' });
      store.wizardBack();
      expect(store.wizardStep).toBe(0);
    });

    it('never lowers the furthest-reached step by going back', () => {
      store.wizardNext();
      store.wizardNext();
      store.wizardBack();
      expect(store.wizardStep).toBe(1);
      expect(store.wizardFurthest).toBe(2);
    });

    it('is a no-op going back from the first step', () => {
      store.wizardBack();
      expect(store.wizardStep).toBe(0);
    });

    it('jumps to any already-visited step', () => {
      store.wizardNext();
      store.wizardNext();
      store.wizardBack();
      store.wizardGoTo(2);
      expect(store.wizardStep).toBe(2);
    });

    it('refuses to jump past the furthest step reached', () => {
      store.wizardGoTo(3);
      expect(store.wizardStep).toBe(0);
    });

    it('clamps a forward jump at a broken step in between', () => {
      store.wizardNext();
      store.wizardNext();
      store.wizardNext();
      store.wizardBack();
      store.wizardBack();
      store.wizardBack();
      issues({ phase: 'type' }); // step 1, between 0 and 3
      store.wizardGoTo(3);
      expect(store.wizardStep).toBe(1);
    });

    // The departure step counts too: Next is blocked when the current phase is
    // broken, so a rail jump must not be a way around that same gate.
    it('clamps a forward jump at the step being left, when that step is broken', () => {
      store.wizardNext();
      store.wizardNext();
      store.wizardBack();
      issues({ phase: 'type' }); // the step the user is standing on
      store.wizardGoTo(2);
      expect(store.wizardStep).toBe(1);
    });
  });

  describe('finishing', () => {
    beforeEach(async () => {
      await store.startWizard('magus');
    });

    /** Walk to the terminal Review step. */
    function reachReview(): void {
      while (store.wizardPhase !== 'review') store.wizardNext();
    }

    it('lands in the editor with the character untouched', () => {
      store.setIdentity('name', 'Marcus');
      const before = JSON.stringify(store.entity);
      reachReview();
      store.finishWizard();
      expect(store.view).toBe('editor');
      expect(JSON.stringify(store.entity)).toBe(before);
    });

    it('refuses to finish while any error remains, in any phase', () => {
      reachReview();
      issues({ phase: 'abilities' });
      expect(store.wizardCanFinish).toBe(false);
      store.finishWizard();
      expect(store.view).toBe('wizard');
    });

    // The Review step is the only place a finding no creation phase owns can be
    // seen, so it must not block a *step* — but it must block Finish.
    it('finishes over warnings, but not over a Review-phase error', () => {
      reachReview();
      issues({ phase: 'review', severity: 'warning' });
      expect(store.wizardCanFinish).toBe(true);
      issues({ phase: 'review' });
      expect(store.wizardCanFinish).toBe(false);
    });

    it('resets the rail, so the next wizard starts at the first step', () => {
      store.wizardNext();
      reachReview();
      store.finishWizard();
      expect(store.wizardStep).toBe(0);
      expect(store.wizardFurthest).toBe(0);
    });
  });

  describe('the unsaved-changes guard', () => {
    beforeEach(async () => {
      await store.startWizard('magus');
    });

    it('does not dirty the document by navigating', () => {
      store.wizardNext();
      store.wizardBack();
      expect(store.dirty).toBe(false);
    });

    it('dirties the document on a real edit, and mirrors that to the close guard', () => {
      store.setIdentity('name', 'Marcus');
      expect(store.dirty).toBe(true);
      expect(store.closeGuardPayload().dirty).toBe(true);
    });

    it('leaves the wizard for the start screen through newDocument, rail reset', async () => {
      store.wizardNext();
      const leaving = store.newDocument();
      if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
      await leaving;
      expect(store.view).toBe('start');
      expect(store.wizardStep).toBe(0);
      expect(store.wizardFurthest).toBe(0);
    });

    // A save records no wizard progress, so an opened character is a finished
    // document: it belongs in the editor, never mid-flow.
    it('lands an opened character in the editor, not back in the wizard', async () => {
      store.wizardNext();
      vi.mocked(ipc.loadEntity).mockResolvedValue({
        path: '/tmp/marcus.armc',
        entity: {
          schema_version: 11,
          ruleset: { id: 'test', version: '1' },
          entity_kind: 'character',
          type_id: 'magus',
          name: 'Marcus',
          selections: [],
          characteristics: {} as Entity['characteristics'],
          characteristic_descriptions: {},
          ability_scores: [],
          xp_pool: 0,
          art_scores: [],
          personality_traits: [],
          reputations: [],
        },
      });
      const opening = store.open();
      if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
      await opening;
      expect(store.view).toBe('editor');
      expect(store.wizardStep).toBe(0);
    });
  });
});

describe('open() and the startup view', () => {
  /**
   * A full, minimal character standing in for a loaded document. It carries a
   * name so the saved-baseline this load leaves behind stays distinguishable
   * from the blank entity `resetEntity()` installs for every later test — an
   * identical baseline would silently mark those tests' documents clean.
   */
  function loadedEntity(): Entity {
    return {
      schema_version: 11,
      ruleset: { id: 'test', version: '1' },
      entity_kind: 'character',
      type_id: 'companion',
      name: 'Marcus of Bonisagus',
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

  /** Drive open() to completion, confirming a discard prompt if one appears. */
  async function openAndConfirm(): Promise<void> {
    const opening = store.open();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;
  }

  beforeEach(() => {
    vi.mocked(ipc.loadEntity).mockReset();
    // Opening a document must work from either screen, so arrange the start one.
    store.view = 'start';
  });

  it('enters the editor on a successful load', async () => {
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/marcus.armc',
      entity: loadedEntity(),
    });

    await openAndConfirm();

    expect(store.view).toBe('editor');
  });

  it('leaves the view alone when the dialog is cancelled', async () => {
    vi.mocked(ipc.loadEntity).mockResolvedValue(null);

    await openAndConfirm();

    expect(store.view).toBe('start');
  });

  // The engine's canonical JSON omits `selections` when it is empty, and serde
  // re-defaults it Rust-side — JavaScript does not. So a saved character with no
  // Virtues or Flaws at all — a bare grog — arrives with the key absent, and an
  // unguarded read used to throw mid-render, aborting the editor's mount and
  // freezing the app on the previous screen. The guarantee is that such a save
  // opens and stays editable, not that the store back-fills the key.
  it('opens a saved character whose empty selections were omitted', async () => {
    installRuleset([item({ id: 'virtue.plain' })]);
    const sparse = loadedEntity() as Partial<Entity>;
    delete sparse.selections;
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/rolf.armc',
      entity: sparse as Entity,
    });

    await openAndConfirm();

    expect(store.view).toBe('editor');
    store.addSelection('virtue.plain');
    expect(store.entity.selections).toEqual([{ ref: 'virtue.plain' }]);
  });
});

// Every selection mutator has to survive an entity whose empty `selections` the
// engine omitted, since that entity is what a loaded bare grog is. These drive
// the mutators straight off such an entity, without the open() path.
describe('editing a character whose omitted selections key left it absent', () => {
  /** The store's entity with no `selections` key at all, as a sparse save loads. */
  function sparseEntity(): Entity {
    const sparse = { ...store.entity } as Partial<Entity>;
    delete sparse.selections;
    return sparse as Entity;
  }

  it('adds a virtue/flaw selection', () => {
    installRuleset([item({ id: 'virtue.plain' })]);
    store.entity = sparseEntity();

    store.addSelection('virtue.plain');

    expect(store.entity.selections).toEqual([{ ref: 'virtue.plain' }]);
  });

  it('removes a selection without one to remove', () => {
    installRuleset([]);
    store.entity = sparseEntity();

    store.removeSelectionAt(0);

    expect(store.entity.selections).toEqual([]);
  });

  it('seeds a Mythic Companion type package', async () => {
    installRuleset([item({ id: 'flaw.blatant', kind: 'flaw', magnitude: 'major' })]);
    store.ruleset!.ruleset.mythic_companion_types = {
      'mythic_type.devil_child': {
        id: 'mythic_type.devil_child',
        required_virtues: [{ ref: 'virtue.might' }],
        required_flaws: [
          {
            default: { ref: 'flaw.blatant' },
            constraint: { kind: 'flaw', magnitude: 'major' },
          },
        ],
      },
    };
    store.entity = sparseEntity();

    await store.setMythicType('mythic_type.devil_child');

    expect(store.entity.selections).toEqual([{ ref: 'virtue.might' }, { ref: 'flaw.blatant' }]);
  });

  it('swaps a required Flaw', async () => {
    installRuleset([]);
    store.entity = sparseEntity();

    await store.setMythicRequiredFlaw('flaw.absent', 'flaw.other');

    expect(store.entity.selections).toEqual([{ ref: 'flaw.other' }]);
  });
});

// The startup screen holds no character, only a placeholder entity. Everything
// below pins what that means: an empty type nobody may read as a choice, no
// engine round trip for it, and nothing for the close/quit guard to warn about.
describe('the startup screen placeholder', () => {
  /** Drive newDocument() to completion, confirming a discard prompt if one appears. */
  async function newAndConfirm(): Promise<void> {
    const discarding = store.newDocument();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await discarding;
  }

  beforeEach(() => {
    store.currentPath = null;
    store.error = null;
    vi.mocked(ipc.validateEntity).mockClear();
    vi.mocked(ipc.effectiveScores).mockClear();
    vi.mocked(ipc.derivedTotals).mockClear();
  });

  afterEach(() => {
    vi.mocked(ipc.validateEntity).mockResolvedValue({ issues: [] });
    vi.mocked(ipc.saveEntity).mockReset();
    store.error = null;
  });

  it('newDocument() returns to the type choice, discarding the character', async () => {
    installRuleset([item({ id: 'virtue.plain' })]);
    store.addSelection('virtue.plain');
    store.setIdentity('name', 'Marcus');
    store.currentPath = '/tmp/marcus.armc';

    await newAndConfirm();

    expect(store.view).toBe('start');
    expect(store.entity.selections).toEqual([]);
    expect(store.entity.name).toBeUndefined();
    expect(store.currentPath).toBeNull();
  });

  it('leaves the placeholder without a type: no character means no type', async () => {
    await newAndConfirm();
    expect(store.entity.type_id).toBe('');
  });

  // The close/quit guard compares against the last save/load baseline. A
  // placeholder that read as dirty would make every quit from the startup screen
  // prompt about a character that does not exist.
  it('is not dirty, so quitting from the startup screen never prompts', async () => {
    await newAndConfirm();
    expect(store.dirty).toBe(false);
    expect(store.closeGuardPayload().dirty).toBe(false);
  });

  it('never sends the placeholder to the engine', async () => {
    store.view = 'start';

    await store.revalidate();

    expect(ipc.validateEntity).not.toHaveBeenCalled();
    expect(ipc.effectiveScores).not.toHaveBeenCalled();
    expect(ipc.derivedTotals).not.toHaveBeenCalled();
  });

  it('drops a debounced validate the discarded character left pending', async () => {
    store.setIdentity('name', 'Marcus');

    await newAndConfirm();
    vi.runAllTimers();
    await Promise.resolve();

    expect(ipc.validateEntity).not.toHaveBeenCalled();
  });

  it('discards a validate still in flight when the startup screen takes over', async () => {
    let finishStale: (result: { issues: [] }) => void = () => {};
    vi.mocked(ipc.validateEntity).mockReturnValueOnce(
      new Promise((resolve) => {
        finishStale = resolve;
      }) as ReturnType<typeof ipc.validateEntity>,
    );
    const stale = store.revalidate();
    store.result = null;

    store.view = 'start';
    await store.revalidate();

    finishStale({ issues: [] });
    await stale;
    expect(store.result).toBeNull();
  });

  it('retires a validation banner instead of stranding it on the startup screen', async () => {
    vi.mocked(ipc.validateEntity).mockRejectedValueOnce({ kind: 'invalid_entity' });
    await store.revalidate();
    expect(store.error).toEqual({ kind: 'invalid_entity' });

    store.view = 'start';
    await store.revalidate();

    expect(store.error).toBeNull();
  });

  // `error` is one shared banner channel: a failed save must stay visible, since
  // the document it names is still unwritten.
  it('keeps a file-operation banner it does not own', async () => {
    vi.mocked(ipc.saveEntity).mockRejectedValueOnce({ kind: 'io' });
    await store.saveAs();
    expect(store.error).toEqual({ kind: 'io' });

    store.view = 'start';
    await store.revalidate();

    expect(store.error).toEqual({ kind: 'io' });
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

// --- addMasteryAbilityAt() / removeMasteryAbilityAt() -----------------------

describe('mastery special abilities', () => {
  it('appends a chosen mastery ability to the spell at the given index', () => {
    store.addSpell('spell.pilum_of_fire');
    store.addMasteryAbilityAt(0, 'spell_mastery_ability.penetration');
    expect(store.entity.spells?.[0].mastery_abilities).toEqual([
      'spell_mastery_ability.penetration',
    ]);
  });

  it('allows a repeatable ability to be added more than once (keeps duplicates)', () => {
    store.addSpell('spell.pilum_of_fire');
    store.addMasteryAbilityAt(0, 'spell_mastery_ability.quiet_casting');
    store.addMasteryAbilityAt(0, 'spell_mastery_ability.quiet_casting');
    expect(store.entity.spells?.[0].mastery_abilities).toEqual([
      'spell_mastery_ability.quiet_casting',
      'spell_mastery_ability.quiet_casting',
    ]);
  });

  it('removes exactly one instance at the given position', () => {
    store.addSpell('spell.pilum_of_fire');
    store.addMasteryAbilityAt(0, 'spell_mastery_ability.quiet_casting');
    store.addMasteryAbilityAt(0, 'spell_mastery_ability.penetration');
    store.addMasteryAbilityAt(0, 'spell_mastery_ability.quiet_casting');
    store.removeMasteryAbilityAt(0, 0);
    expect(store.entity.spells?.[0].mastery_abilities).toEqual([
      'spell_mastery_ability.penetration',
      'spell_mastery_ability.quiet_casting',
    ]);
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
      animal: '',
      might: null,
      characteristics: {},
      size: 0,
      personality_traits: [],
      cord_gold: 0,
      cord_silver: 0,
      cord_bronze: 3,
      powers: [],
    });
    store.removeFamiliar();
    expect(store.entity.familiar).toBeNull();
  });

  it('seeds a familiar with the full empty statblock', () => {
    store.addFamiliar();
    expect(store.entity.familiar).toEqual({
      name: '',
      animal: '',
      might: null,
      characteristics: {},
      size: 0,
      personality_traits: [],
      cord_gold: 0,
      cord_silver: 0,
      cord_bronze: 0,
      powers: [],
    });
  });

  it('edits the familiar animal and its signed Size without clamping', () => {
    store.addFamiliar();
    store.setFamiliarAnimal('raven');
    store.setFamiliarSize(-4);
    expect(store.entity.familiar?.animal).toBe('raven');
    expect(store.entity.familiar?.size).toBe(-4);
    // Size is signed and truncated, never clamped at 0 — a raven is smaller than 0.
    store.setFamiliarSize(2.7);
    expect(store.entity.familiar?.size).toBe(2);
  });

  it("sets, scores and clears the familiar's own Magic Might", () => {
    store.addFamiliar();
    store.setFamiliarMightRealm('magic');
    expect(store.entity.familiar?.might).toEqual({ realm: 'magic', score: 0 });
    store.setFamiliarMightScore(10);
    expect(store.entity.familiar?.might).toEqual({ realm: 'magic', score: 10 });
    store.setFamiliarMightScore(-3);
    expect(store.entity.familiar?.might).toEqual({ realm: 'magic', score: 0 });
    store.setFamiliarMightRealm('faerie');
    expect(store.entity.familiar?.might).toEqual({ realm: 'faerie', score: 0 });
    store.clearFamiliarMight();
    expect(store.entity.familiar?.might).toBeNull();
  });

  it('sets a familiar Characteristic and deletes it at zero', () => {
    store.addFamiliar();
    store.setFamiliarCharacteristic('int', -3);
    store.setFamiliarCharacteristic('qik', 4);
    expect(store.entity.familiar?.characteristics).toEqual({ int: -3, qik: 4 });
    store.setFamiliarCharacteristic('qik', 0);
    expect(store.entity.familiar?.characteristics).toEqual({ int: -3 });
  });

  it("adds, edits and removes the familiar's Personality Traits", () => {
    store.addFamiliar();
    store.addFamiliarPersonalityTrait();
    store.setFamiliarPersonalityTraitName(0, 'Loyal (Marcus)');
    store.setFamiliarPersonalityTraitValue(0, 3);
    expect(store.entity.familiar?.personality_traits).toEqual([
      { name: 'Loyal (Marcus)', value: 3 },
    ]);
    store.addFamiliarPersonalityTrait();
    store.removeFamiliarPersonalityTraitAt(1);
    expect(store.entity.familiar?.personality_traits).toHaveLength(1);
  });

  it("adds, edits and removes the familiar's bond-invested powers", () => {
    store.addFamiliar();
    store.addFamiliarPower();
    store.setFamiliarPowerName(0, 'Mental communication');
    store.setFamiliarPowerLevel(0, 15);
    expect(store.entity.familiar?.powers).toEqual([{ name: 'Mental communication', level: 15 }]);
    store.setFamiliarPowerLevel(0, -5);
    expect(store.entity.familiar?.powers).toEqual([{ name: 'Mental communication', level: 0 }]);
    store.addFamiliarPower();
    store.removeFamiliarPowerAt(1);
    expect(store.entity.familiar?.powers).toHaveLength(1);
  });

  it('ignores every familiar statblock edit when there is no familiar', () => {
    expect(store.entity.familiar).toBeUndefined();
    store.setFamiliarName('Corvus');
    store.setFamiliarAnimal('raven');
    store.setFamiliarSize(-4);
    store.setFamiliarCord('gold', 3);
    store.setFamiliarMightRealm('magic');
    store.setFamiliarMightScore(10);
    store.clearFamiliarMight();
    store.setFamiliarCharacteristic('int', -3);
    store.addFamiliarPersonalityTrait();
    store.setFamiliarPersonalityTraitName(0, 'Loyal');
    store.setFamiliarPersonalityTraitValue(0, 3);
    store.removeFamiliarPersonalityTraitAt(0);
    store.addFamiliarPower();
    store.setFamiliarPowerName(0, 'Speech');
    store.setFamiliarPowerLevel(0, 20);
    store.removeFamiliarPowerAt(0);
    expect(store.entity.familiar).toBeFalsy();
  });

  it("keeps the familiar's statblock out of the character's own fields", () => {
    store.addFamiliar();
    store.setFamiliarMightRealm('faerie');
    store.setFamiliarMightScore(25);
    store.setFamiliarCharacteristic('int', 3);
    store.addFamiliarPower();
    store.setFamiliarPowerLevel(0, 400);
    store.addFamiliarPersonalityTrait();
    // The magus's own Might / powers / Characteristics / traits are untouched: the
    // familiar is a separate creature, not part of the character's point-buy.
    expect(store.entity.might).toBeFalsy();
    expect(store.entity.powers ?? []).toEqual([]);
    expect(store.entity.characteristics ?? {}).toEqual({});
    expect(store.entity.personality_traits ?? []).toEqual([]);
  });

  it('adds a talisman with its identity, and removes it', () => {
    store.addTalisman();
    expect(store.entity.talisman).toEqual({ description: '', attunements: [], effects: [] });
    store.setTalismanDescription('An ash staff shod with silver');
    expect(store.entity.talisman?.description).toBe('An ash staff shod with silver');
    store.removeTalisman();
    expect(store.entity.talisman).toBeNull();
  });

  it('adds, edits and removes talisman attunements by index', () => {
    store.addTalisman();
    store.addTalismanAttunement();
    store.setTalismanAttunementDescription(0, 'Attuned to fire');
    store.setTalismanAttunementBonus(0, 5);
    expect(store.entity.talisman?.attunements).toEqual([
      { description: 'Attuned to fire', bonus: 5 },
    ]);
    store.removeTalismanAttunementAt(0);
    expect(store.entity.talisman?.attunements).toEqual([]);
  });

  it('adds, edits (name + non-negative level) and removes instilled effects', () => {
    store.addTalisman();
    store.addTalismanEffect();
    store.setTalismanEffectName(0, 'Wielding the Invisible Sling');
    store.setTalismanEffectLevel(0, -5);
    expect(store.entity.talisman?.effects).toEqual([
      { name: 'Wielding the Invisible Sling', level: 0 },
    ]);
    store.setTalismanEffectLevel(0, 15);
    store.addTalismanEffect();
    store.removeTalismanEffectAt(1);
    expect(store.entity.talisman?.effects).toEqual([
      { name: 'Wielding the Invisible Sling', level: 15 },
    ]);
  });

  it('ignores every talisman mutator while there is no talisman', () => {
    // The panel only renders these controls with a talisman present, but a guard
    // per mutator keeps a stray call from conjuring one out of nothing.
    store.setTalismanDescription('An ash staff');
    store.addTalismanAttunement();
    store.setTalismanAttunementDescription(0, 'Warding');
    store.setTalismanAttunementBonus(0, 5);
    store.removeTalismanAttunementAt(0);
    store.addTalismanEffect();
    store.setTalismanEffectName(0, 'Lamp Without Flame');
    store.setTalismanEffectLevel(0, 10);
    store.removeTalismanEffectAt(0);
    expect(store.entity.talisman ?? null).toBeNull();
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

  it('adds a longevity ritual with nothing entered yet', () => {
    store.addLongevityRitual('self_made');
    expect(store.entity.longevity_ritual).toEqual({ source: 'self_made', bonus: null, focus: '' });
    store.removeLongevityRitual();
    expect(store.entity.longevity_ritual).toBeNull();
  });

  it('enters a bonus and a focus for a self-made ritual', () => {
    store.addLongevityRitual('self_made');
    store.setLongevityBonus(7);
    store.setLongevityFocus('A draught of quicksilver');
    expect(store.entity.longevity_ritual).toEqual({
      source: 'self_made',
      bonus: 7,
      focus: 'A draught of quicksilver',
    });
  });

  it('clearing the bonus returns the ritual to not-entered', () => {
    store.addLongevityRitual('self_made');
    store.setLongevityBonus(7);
    // Emptying the input passes null, NOT the 0 that `Number('')` yields — a
    // deliberate 0 is a claim ("the ritual grants nothing"), which is not what
    // clearing the field means. Only null restores "not entered".
    store.setLongevityBonus(null);
    expect(store.entity.longevity_ritual?.bonus).toBeNull();
    // A deliberate 0 is still storable, and is a different state from null.
    store.setLongevityBonus(0);
    expect(store.entity.longevity_ritual?.bonus).toBe(0);
    // An unparseable value (mid-typing "-") is "not entered", never a stored 0.
    store.setLongevityBonus(Number.NaN);
    expect(store.entity.longevity_ritual?.bonus).toBeNull();
  });

  it('keeps the entered bonus when switching source', () => {
    store.addLongevityRitual('self_made');
    store.setLongevityBonus(4);
    store.setLongevityFocus('An amulet of hawthorn');
    store.setLongevitySource('external');
    expect(store.entity.longevity_ritual).toEqual({
      source: 'external',
      bonus: 4,
      focus: 'An amulet of hawthorn',
    });
    store.setLongevitySource('self_made');
    expect(store.entity.longevity_ritual).toEqual({
      source: 'self_made',
      bonus: 4,
      focus: 'An amulet of hawthorn',
    });
  });

  it('ignores longevity edits when there is no ritual', () => {
    store.setLongevitySource('external');
    store.setLongevityBonus(3);
    store.setLongevityFocus('nothing');
    expect(store.entity.longevity_ritual ?? null).toBeNull();
  });
});

// Every number typed into a panel input ends up in a fixed-width Rust integer
// field. An out-of-range value makes serde reject the whole payload at the Tauri
// boundary, so `validate` / `effective_scores` / `derived_totals` all fail at once
// and every read-out freezes on stale numbers while looking current. The store
// clamps instead, so the engine always receives a representable value.
describe('integer clamps at the Tauri boundary', () => {
  it('clamps the familiar Size and Characteristics to i8', () => {
    store.addFamiliar();
    store.setFamiliarSize(200);
    expect(store.entity.familiar?.size).toBe(127);
    store.setFamiliarSize(-200);
    expect(store.entity.familiar?.size).toBe(-128);
    store.setFamiliarCharacteristic('int', 200);
    expect(store.entity.familiar?.characteristics?.int).toBe(127);
    store.setFamiliarCharacteristic('qik', -200);
    expect(store.entity.familiar?.characteristics?.qik).toBe(-128);
  });

  it('clamps the u8-backed familiar Might Score', () => {
    store.addFamiliar();
    store.setFamiliarMightRealm('magic');
    store.setFamiliarMightScore(900);
    expect(store.entity.familiar?.might?.score).toBe(255);
  });

  // A cord is bounded by the RULE, not by the serde width: "rated from 0 to +5 …
  // a score of +5 (the maximum)" (Core Rules.md:10836). Storing a 6 gave the
  // engine's three cord consumers a value they read inconsistently (one clamps at
  // 5, two do not), so one entered number produced three contradictory read-outs.
  it('clamps a familiar cord to the rules maximum of +5, not to u8', () => {
    store.addFamiliar();
    store.setFamiliarCord('gold', 5);
    expect(store.entity.familiar?.cord_gold).toBe(5);
    store.setFamiliarCord('silver', 6);
    expect(store.entity.familiar?.cord_silver).toBe(5);
    store.setFamiliarCord('bronze', 900);
    expect(store.entity.familiar?.cord_bronze).toBe(5);
    store.setFamiliarCord('gold', -1);
    expect(store.entity.familiar?.cord_gold).toBe(0);
  });

  it('clamps u16-backed device and character power levels', () => {
    store.addDevice();
    store.setDeviceLevel(0, 70000);
    expect(store.entity.devices?.[0].level).toBe(65535);
    store.setDeviceLevel(0, -5);
    expect(store.entity.devices?.[0].level).toBe(0);
    store.addPower();
    store.setPowerLevel(0, 70000);
    expect(store.entity.powers?.[0].level).toBe(65535);
    store.setPowerLevel(0, -5);
    expect(store.entity.powers?.[0].level).toBe(0);
  });

  it("clamps the u8-backed base Might Score and a Characteristic's aging points", () => {
    store.setMightRealm('magic');
    store.setMightScore(900);
    expect(store.entity.might?.score).toBe(255);
    store.setAgingPoints('str', 900);
    expect(store.entity.aging_points?.str).toBe(255);
  });

  it('clamps the i32-backed aura at both ends', () => {
    store.setAura(3e9);
    expect(store.entity.aura).toBe(2147483647);
    store.setAura(-3e9);
    expect(store.entity.aura).toBe(-2147483648);
  });

  it('clamps the u32-backed Warping Points and XP pool', () => {
    store.setWarpingPoints(5e9);
    expect(store.entity.warping_points).toBe(4294967295);
    store.setXpPool(5e9);
    expect(store.entity.xp_pool).toBe(4294967295);
  });

  // A user mid-typing a lone "-" in a signed field sends `Number('-')` — NaN.
  // Without the non-finite guard the clamp propagates NaN, serde rejects the
  // payload and every read-out freezes on stale numbers that still look current.
  it('treats a non-finite value as 0 in signed and unsigned fields alike', () => {
    store.addFamiliar();
    store.setFamiliarSize(Number('-'));
    expect(store.entity.familiar?.size).toBe(0);
    store.setFamiliarCord('gold', Number('-'));
    expect(store.entity.familiar?.cord_gold).toBe(0);
    store.addTalisman();
    store.addTalismanEffect();
    store.setTalismanEffectLevel(0, Number('-'));
    expect(store.entity.talisman?.effects?.[0].level).toBe(0);
    store.addTalismanAttunement();
    store.setTalismanAttunementBonus(0, Number('-'));
    expect(store.entity.talisman?.attunements?.[0].bonus).toBe(0);
  });

  it('clamps u16-backed power and instilled-effect levels', () => {
    store.addFamiliar();
    store.addFamiliarPower();
    store.setFamiliarPowerLevel(0, 99999);
    expect(store.entity.familiar?.powers?.[0].level).toBe(65535);
    store.addTalisman();
    store.addTalismanEffect();
    store.setTalismanEffectLevel(0, 99999);
    expect(store.entity.talisman?.effects?.[0].level).toBe(65535);
  });

  it('clamps the i8-backed longevity bonus and talisman attunement bonus', () => {
    store.addLongevityRitual('self_made');
    store.setLongevityBonus(200);
    expect(store.entity.longevity_ritual?.bonus).toBe(127);
    store.setLongevityBonus(-200);
    expect(store.entity.longevity_ritual?.bonus).toBe(-128);
    store.addTalisman();
    store.addTalismanAttunement();
    store.setTalismanAttunementBonus(0, 200);
    expect(store.entity.talisman?.attunements?.[0].bonus).toBe(127);
    store.setTalismanAttunementBonus(0, -200);
    expect(store.entity.talisman?.attunements?.[0].bonus).toBe(-128);
  });

  it('clamps a bought Characteristic to i8', () => {
    store.setCharacteristic('str', 200);
    expect(store.entity.characteristics?.str).toBe(127);
    store.setCharacteristic('sta', -200);
    expect(store.entity.characteristics?.sta).toBe(-128);
  });

  // A General spell's level is the entity's tightest numeric field (u8), and one
  // stray digit on a 25 or 50 lands past it.
  it('clamps the u8-backed General spell level, keeping the minimum of 1', () => {
    store.addSpell('spell.aegis_of_the_hearth', 5);
    store.setSpellLevelAt(0, 256);
    expect(store.entity.spells?.[0].level).toBe(255);
    store.setSpellLevelAt(0, 0);
    expect(store.entity.spells?.[0].level).toBe(1);
    store.setSpellLevelAt(0, Number('-'));
    expect(store.entity.spells?.[0].level).toBe(1);
  });

  it('clamps the u32-backed age, apparent age and spell-levels override', () => {
    store.setAge(5e9);
    expect(store.entity.age).toBe(4294967295);
    store.setApparentAge(5e9);
    expect(store.entity.apparent_age).toBe(4294967295);
    store.setSpellLevelsOverride(5e9);
    expect(store.entity.spell_levels_override).toBe(4294967295);
    // The non-positive value still clears the field instead of clamping to 0.
    store.setAge(0);
    expect(store.entity.age).toBeNull();
    store.setApparentAge(-3);
    expect(store.entity.apparent_age).toBeNull();
    store.setSpellLevelsOverride(-3);
    expect(store.entity.spell_levels_override).toBeNull();
  });

  it('clamps the i32-backed birth year and aging-log year at both ends', () => {
    store.setBirthYear(3e9);
    expect(store.entity.birth_year).toBe(2147483647);
    store.setBirthYear(-3e9);
    expect(store.entity.birth_year).toBe(-2147483648);
    // An empty field still clears it rather than clamping to 0.
    store.setBirthYear(null);
    expect(store.entity.birth_year).toBeNull();
    store.addAgingLogEntry();
    store.setAgingLogEntryYear(0, 3e9);
    expect(store.entity.aging_log?.[0].year).toBe(2147483647);
    store.setAgingLogEntryYear(0, -3e9);
    expect(store.entity.aging_log?.[0].year).toBe(-2147483648);
    store.setAgingLogEntryYear(0, Number('-'));
    expect(store.entity.aging_log?.[0].year).toBe(0);
  });
});

// A rejected validate must not latch the error banner on for the rest of the
// session: once the user corrects the offending value and a pass succeeds, the
// banner has to go. Only a *current* pass may clear it, so a stale response that
// lands after a newer edit clears nothing.
describe('revalidate error latching', () => {
  afterEach(() => {
    vi.mocked(ipc.validateEntity).mockResolvedValue({ issues: [] });
    vi.mocked(ipc.saveEntity).mockReset();
    store.error = null;
  });

  it('clears a latched error once a validate succeeds', async () => {
    vi.mocked(ipc.validateEntity).mockRejectedValueOnce({ kind: 'invalid_entity' });

    await store.revalidate();
    expect(store.error).toEqual({ kind: 'invalid_entity' });

    await store.revalidate();
    expect(store.error).toBeNull();
  });

  it('does not let a stale response clear a current error', async () => {
    // A slow first call resolves only after a second (rejecting) call has already
    // bumped the sequence — its success must not wipe the newer failure.
    let finishStale: (result: { issues: [] }) => void = () => {};
    vi.mocked(ipc.validateEntity).mockReturnValueOnce(
      new Promise((resolve) => {
        finishStale = resolve;
      }) as ReturnType<typeof ipc.validateEntity>,
    );
    const stale = store.revalidate();

    vi.mocked(ipc.validateEntity).mockRejectedValueOnce({ kind: 'invalid_entity' });
    await store.revalidate();
    expect(store.error).toEqual({ kind: 'invalid_entity' });

    finishStale({ issues: [] });
    await stale;
    expect(store.error).toEqual({ kind: 'invalid_entity' });
  });

  // The mirror case: a direct `revalidate()` does not cancel a pending debounced
  // one, so a rejection from the pass the user already corrected can land after
  // the corrected pass succeeded. It must not raise a banner for a payload that
  // no longer exists.
  it('does not let a stale rejection raise an error after a newer pass succeeded', async () => {
    let failStale: (reason: unknown) => void = () => {};
    vi.mocked(ipc.validateEntity).mockReturnValueOnce(
      new Promise((_resolve, reject) => {
        failStale = reject;
      }) as ReturnType<typeof ipc.validateEntity>,
    );
    const stale = store.revalidate();

    await store.revalidate();
    expect(store.error).toBeNull();

    failStale({ kind: 'invalid_entity' });
    await stale;
    expect(store.error).toBeNull();
  });

  // `error` is one shared banner channel — file operations publish to it too. A
  // succeeding validate may only retire an error the validation path itself
  // raised: a failed save leaves the document unsaved, so its banner must stay up
  // until the user resolves it, even though every later validate succeeds.
  it('does not let a succeeding validate clear a save failure', async () => {
    vi.mocked(ipc.saveEntity).mockRejectedValueOnce({ kind: 'io' });
    await store.saveAs();
    expect(store.error).toEqual({ kind: 'io' });

    await store.revalidate();

    expect(store.error).toEqual({ kind: 'io' });
    expect(store.dirty).toBe(true);
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

  it('sets and clears an owed Warping fill by choice_key', () => {
    store.setWarpingChoice('warping.minor_flaw.0', { ref: 'flaw.clumsy' });
    expect(store.entity.warping_choices).toEqual({
      'warping.minor_flaw.0': { ref: 'flaw.clumsy' },
    });
    // A second slot coexists without clobbering the first.
    store.setWarpingChoice('warping.supernatural_virtue.0', { ref: 'virtue.second_sight' });
    expect(store.entity.warping_choices?.['warping.minor_flaw.0']).toEqual({ ref: 'flaw.clumsy' });
    // Passing null clears just that slot.
    store.setWarpingChoice('warping.minor_flaw.0', null);
    expect(store.entity.warping_choices).toEqual({
      'warping.supernatural_virtue.0': { ref: 'virtue.second_sight' },
    });
  });

  it('keeps the chosen parameters of a parameterized owed Warping fill', () => {
    // A fill may be a parameterized item ("Master of (Form) Creatures"); the
    // chosen parameter travels with the pick, and re-picking the same slot
    // replaces the whole Selection (so a stale param cannot linger).
    store.setWarpingChoice('warping.supernatural_virtue.0', {
      ref: 'virtue.master_of_form_creatures',
      params: { form: 'art.ignem' },
    });
    expect(store.entity.warping_choices?.['warping.supernatural_virtue.0']).toEqual({
      ref: 'virtue.master_of_form_creatures',
      params: { form: 'art.ignem' },
    });
    store.setWarpingChoice('warping.supernatural_virtue.0', { ref: 'virtue.second_sight' });
    expect(store.entity.warping_choices?.['warping.supernatural_virtue.0']).toEqual({
      ref: 'virtue.second_sight',
    });
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
    // The warping-effect field is a multi-line textarea, so it must round-trip
    // embedded newlines verbatim.
    const multiLineWarping =
      'A faint aura of ozone clings to him.\nHis eyes glow faintly at night.';
    store.setWarpingEffect(multiLineWarping);
    store.setDecrepitudeEffect('Stooped and hard of hearing');
    expect(store.entity.warping_effect).toBe(multiLineWarping);
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
    expect(store.entity.selections!.map((s) => s.ref)).toEqual(['v.a', 'v.c']);
  });

  it('is a no-op for an out-of-range index', () => {
    installRuleset([item({ id: 'v.a' })]);
    store.addSelection('v.a');
    store.removeSelectionAt(5);
    expect(store.entity.selections!.map((s) => s.ref)).toEqual(['v.a']);
  });
});

// --- setParamAt() -----------------------------------------------------------

describe('setParamAt', () => {
  it('sets a param on the targeted row only', () => {
    installRuleset([item({ id: 'v.great', max_per_target: 5 })]);
    store.addSelection('v.great');
    store.addSelection('v.great');
    store.setParamAt(0, 'characteristic', 'characteristic.per');
    expect(store.entity.selections![0]).toEqual({
      ref: 'v.great',
      params: { characteristic: 'characteristic.per' },
    });
    expect(store.entity.selections![1]).toEqual({ ref: 'v.great' });
  });

  it('merges into existing params rather than replacing them', () => {
    installRuleset([item({ id: 'v.x' })]);
    store.addSelection('v.x');
    store.setParamAt(0, 'a', '1');
    store.setParamAt(0, 'b', '2');
    expect(store.entity.selections![0].params).toEqual({ a: '1', b: '2' });
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
    expect(store.entity.selections![0].params).toEqual({
      ability: 'ability.area_lore',
      area: 'Rhine',
    });
  });

  it('stores just the ability for a plain ability (no instance key)', () => {
    store.setAbilityBonusTarget(0, 'ability.awareness');
    expect(store.entity.selections![0].params).toEqual({ ability: 'ability.awareness' });
  });

  it('drops the stale instance key when switching to a plain ability', () => {
    // First point it at a parameterized instance...
    store.setAbilityBonusTarget(0, 'ability.area_lore', 'Rhine');
    expect(store.entity.selections![0].params).toHaveProperty('area', 'Rhine');
    // ...then switch the target to a plain ability: the params object is
    // rebuilt, so the stale `area` key is gone, not merged.
    store.setAbilityBonusTarget(0, 'ability.awareness');
    expect(store.entity.selections![0].params).toEqual({ ability: 'ability.awareness' });
    expect(store.entity.selections![0].params).not.toHaveProperty('area');
  });

  it('omits the instance key when a parameterized ability has no parameter value', () => {
    store.setAbilityBonusTarget(0, 'ability.area_lore', null);
    expect(store.entity.selections![0].params).toEqual({ ability: 'ability.area_lore' });
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
    store.setArtBonusTarget(0, 'art', 'art.ignem');
    expect(store.entity.selections![0].params).toEqual({ art: 'art.ignem' });
  });

  it('writes the Art under the parameter key that declared it', () => {
    // An Art-domain parameter need not be keyed "art": Master of (Form)
    // Creatures declares `form` over the Art catalogue, and storing the pick
    // under "art" would leave `form` missing (and "art" unexpected).
    installRuleset([
      item({
        id: 'virtue.master_of_form_creatures',
        category: 'supernatural',
        parameters: [{ key: 'form', type: 'ref', domain: 'art' }],
      }),
    ]);
    store.addSelection('virtue.master_of_form_creatures');
    store.setArtBonusTarget(0, 'form', 'art.ignem');
    expect(store.entity.selections![0].params).toEqual({ form: 'art.ignem' });
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
    expect(store.entity.selections!.some((s) => s.ref === 'virtue.status')).toBe(false);
    // The `choice` free-Minor defaults to its first option.
    expect(store.entity.mythic_choices?.free_minor).toEqual({ ref: 'virtue.min_a' });
  });

  it('swaps the package when the type changes and drops it when cleared', async () => {
    await store.setMythicType('mythic_type.devil');
    await store.setMythicType('mythic_type.other');
    // Devil-only rows (Puissant, the default Flaw) are dropped; the shared
    // req_major stays; the stale free_minor choice is pruned.
    expect(store.entity.selections!.some((s) => s.ref === 'virtue.puissant')).toBe(false);
    expect(store.entity.selections!.some((s) => s.ref === 'flaw.default_major')).toBe(false);
    expect(store.entity.selections).toContainEqual({ ref: 'virtue.req_major' });
    expect(store.entity.mythic_choices?.free_minor).toBeUndefined();

    await store.setMythicType(null);
    expect(store.entity.mythic_type).toBeNull();
    expect(store.entity.selections!.some((s) => s.ref === 'virtue.req_major')).toBe(false);
  });

  it('swaps a required Flaw for a substitute', async () => {
    await store.setMythicType('mythic_type.devil');
    await store.setMythicRequiredFlaw('flaw.default_major', 'flaw.other_major');
    expect(store.entity.selections!.some((s) => s.ref === 'flaw.default_major')).toBe(false);
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

// --- Markdown export (M5.6c) -------------------------------------------------

describe('exportMarkdown', () => {
  /** A profile skeleton: only its id matters here — it names the `type-<id>` key. */
  function profile(id: string): EntityTypeProfile {
    return {
      id,
      budget: { virtue_points: 10, flaw_points: 10 },
      permitted_categories: [],
      forbidden_categories: [],
      creation_phases: ['concept'],
    };
  }

  /**
   * A ruleset that exercises all three key families the engine composes from
   * catalogue data and therefore cannot enumerate: two character-type profiles, a
   * parameter key declared on each of the three parameterized catalogues
   * (Virtue/Flaw, Ability, spell), and two distinct point-item categories.
   */
  function installExportRuleset(): void {
    const localized = installRuleset(
      [
        item({
          id: 'virtue.puissant_ability',
          category: 'hermetic',
          parameters: [{ key: 'ability', type: 'ref', domain: 'ability' }],
        }),
        item({ id: 'flaw.optimistic', kind: 'flaw', category: 'personality' }),
      ],
      [ability('ability.area_lore', 'area')],
      { magus: profile('magus'), grog: profile('grog') },
    );
    const spell: Spell = {
      id: 'spell.wizards_boost',
      technique: 'art.muto',
      form: 'art.vim',
      parameters: [{ key: 'form', type: 'ref', domain: 'form' }],
    };
    localized.ruleset.spells = { [spell.id]: spell };
  }

  /** The label map the store handed the export command on its last call. */
  function sentLabels(): Record<string, string> {
    return vi.mocked(ipc.exportMarkdown).mock.calls[0][1];
  }

  /** The current-save-file hint the store handed the export command, for the prefill. */
  function sentCurrentPath(): string | null | undefined {
    return vi.mocked(ipc.exportMarkdown).mock.calls[0][3];
  }

  beforeEach(() => {
    vi.mocked(ipc.exportMarkdown).mockReset();
    vi.mocked(ipc.exportLabelKeys).mockReset();
    vi.mocked(ipc.saveEntity).mockReset();
    vi.mocked(ipc.exportMarkdown).mockResolvedValue('/tmp/marcus.md');
    vi.mocked(ipc.exportLabelKeys).mockResolvedValue(['identity-name', 'abilities-title']);
    installExportRuleset();
    store.lang = 'en';
    store.error = null;
    store.currentPath = null;
  });

  it('sends the entity and a resolved label for every key the engine asks for', async () => {
    await store.exportMarkdown();

    expect(ipc.exportMarkdown).toHaveBeenCalledTimes(1);
    expect(vi.mocked(ipc.exportMarkdown).mock.calls[0][0].type_id).toBe(store.entity.type_id);
    const labels = sentLabels();
    expect(labels['identity-name']).toBe('Name');
    expect(labels['abilities-title']).toBe('Abilities');
  });

  it('adds the three composed families the engine cannot enumerate', async () => {
    await store.exportMarkdown();

    const labels = sentLabels();
    // `type-<profile id>`: one per shipped profile, read off the ruleset.
    expect(labels['type-magus']).toBe('Magus');
    expect(labels['type-grog']).toBe('Grog');
    // `param-label-<parameter key>`: declared by a Virtue, an Ability and a spell.
    expect(labels['param-label-ability']).toBe('Ability');
    expect(labels['param-label-area']).toBe('Area');
    expect(labels['param-label-form']).toBe('Form');
    // `category-<item category>`: the Type cell of an exported Virtue/Flaw row —
    // one per distinct category the point-item catalogue uses.
    expect(labels['category-hermetic']).toBe('Hermetic');
    expect(labels['category-personality']).toBe('Personality');
  });

  it('never echoes a key back as its own label, in either language', async () => {
    // Fluent's miss behavior is to return the key, so an unresolved composed key
    // would silently ship a raw slug into the exported sheet.
    for (const lang of ['en', 'de'] as const) {
      vi.mocked(ipc.exportMarkdown).mockClear();
      store.lang = lang;
      await store.exportMarkdown();
      for (const [key, text] of Object.entries(sentLabels())) {
        expect(text, `'${key}' is unresolved in '${lang}'`).not.toBe(key);
      }
    }
    store.lang = 'en';
  });

  it('leaves the document dirty: an export is not a save', async () => {
    store.currentPath = '/tmp/marcus.armc';
    store.setIdentity('name', 'Marcus');
    expect(store.dirty).toBe(true);

    await store.exportMarkdown();

    expect(store.dirty).toBe(true);
    expect(ipc.saveEntity).not.toHaveBeenCalled();
  });

  it('never adopts the written Markdown file as the current file', async () => {
    store.currentPath = '/tmp/marcus.armc';

    await store.exportMarkdown();

    expect(store.currentPath).toBe('/tmp/marcus.armc');
  });

  it('passes the current save file so the dialog can prefill from it', async () => {
    // A save records the current file; the export dialog then defaults to that
    // file's name and directory rather than the generic kind default.
    vi.mocked(ipc.saveEntity).mockResolvedValue('/tmp/gerhard.armc');
    await store.saveAs();

    await store.exportMarkdown();

    expect(sentCurrentPath()).toBe('/tmp/gerhard.armc');
  });

  it('passes no current save file for a document that was never saved', async () => {
    expect(store.currentPath).toBeNull();

    await store.exportMarkdown();

    expect(sentCurrentPath()).toBeNull();
  });

  it('surfaces a failed export in the error banner and clears the busy flag', async () => {
    vi.mocked(ipc.exportMarkdown).mockRejectedValue({ kind: 'io' });

    await store.exportMarkdown();

    expect(store.error).toEqual({ kind: 'io' });
    expect(store.busy).toBe(false);
  });

  it('is a no-op while another file operation is in flight', async () => {
    store.currentPath = '/tmp/marcus.armc';
    let finishSave: (path: string | null) => void = () => {};
    vi.mocked(ipc.saveEntity).mockReturnValue(
      new Promise<string | null>((resolve) => {
        finishSave = resolve;
      }),
    );

    const saving = store.save();
    await store.exportMarkdown();

    expect(ipc.exportMarkdown).not.toHaveBeenCalled();

    // Let the save finish so the in-flight guard clears for later tests.
    finishSave('/tmp/marcus.armc');
    await saving;
  });
});
