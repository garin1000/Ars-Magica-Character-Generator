//! Read-only lookups over a loaded [`Ruleset`]: catalogue getters, iterators,
//! and counts. No logic beyond field access — split out of `ruleset.rs`; see
//! `ruleset.rs` for construction/loading and `ruleset/integrity.rs` for
//! referential-integrity validation.

use super::*;

impl Ruleset {
    /// Iterates all point items in id order.
    pub fn items(&self) -> impl Iterator<Item = &PointItem> {
        self.point_items.values()
    }

    /// Iterates all type profiles in id order.
    pub fn profiles(&self) -> impl Iterator<Item = &EntityTypeProfile> {
        self.type_profiles.values()
    }

    /// Number of point items.
    pub fn item_count(&self) -> usize {
        self.point_items.len()
    }

    /// Number of type profiles.
    pub fn profile_count(&self) -> usize {
        self.type_profiles.len()
    }

    /// Sorts each point item's parameters for canonical serialization. The
    /// ruleset's own maps are already id-ordered (`BTreeMap`); this is the single
    /// runtime entry point that normalizes the nested item data
    /// (see [`crate::types::PointItem::normalize`]).
    ///
    /// The `houses` and `mythic_companion_types` catalogues are intentionally
    /// exempt: a grant's list (and each `Grant::Choice`'s `options`), and a
    /// mythic type's required package, are order-significant authored data — like
    /// `creation_phases` — sourced from the already-canonical, pipeline-generated
    /// `rules/core/*.json`, and the assembled `Ruleset` is only ever a transient
    /// frontend payload, never written back to disk. So there is no
    /// canonical-write to normalize for.
    pub fn normalize(&mut self) {
        for item in self.point_items.values_mut() {
            item.normalize();
        }
        for profile in self.type_profiles.values_mut() {
            profile.normalize();
        }
    }

    /// Returns a [`RulesetRef`] identifying this ruleset (id + version).
    pub fn reference(&self) -> RulesetRef {
        RulesetRef::new(self.id.clone(), self.version.clone())
    }

    /// Returns `true` if `reference` names this ruleset by id and version.
    pub fn matches(&self, reference: &RulesetRef) -> bool {
        self.id == reference.id && self.version == reference.version
    }

    /// Looks up a point item by id.
    pub fn item(&self, id: &Id) -> Option<&PointItem> {
        self.point_items.get(id)
    }

    /// Looks up an entity type profile by id.
    pub fn profile(&self, id: &Id) -> Option<&EntityTypeProfile> {
        self.type_profiles.get(id)
    }

    /// Looks up an ability by id.
    pub fn ability(&self, id: &Id) -> Option<&Ability> {
        self.abilities.get(id)
    }

    /// Iterates all abilities in id order.
    pub fn abilities(&self) -> impl Iterator<Item = &Ability> {
        self.abilities.values()
    }

    /// Number of abilities in the catalogue.
    pub fn ability_count(&self) -> usize {
        self.abilities.len()
    }

    /// The Ability XP advancement table.
    pub fn advancement(&self) -> &AdvancementTable {
        &self.advancement
    }

    /// The age → maximum-Ability-score band table (Ars Magica - Definitive Edition (Core Rules).md:2366-2374). Empty when the
    /// ruleset ships no age caps.
    pub fn age_ability_caps(&self) -> &AgeAbilityCaps {
        &self.age_ability_caps
    }

    /// Looks up an Art by id.
    pub fn art(&self, id: &Id) -> Option<&Art> {
        self.arts.get(id)
    }

    /// Iterates all Arts in id order.
    pub fn arts(&self) -> impl Iterator<Item = &Art> {
        self.arts.values()
    }

    /// The ids of every Art of one class (Technique or Form), **sorted**.
    ///
    /// The sort is part of the contract, not an accident of storage: callers pair
    /// Techniques against Forms to build grids that are compared and serialized by
    /// position (`spell_level_caps`, `lab_totals`, `casting_totals`), so the order
    /// must not depend on how a ruleset's `arts.json` happened to list them. It
    /// costs nothing today — the catalogue is a `BTreeMap`, so iteration is already
    /// id-ordered — and it keeps that guarantee if the storage ever changes.
    pub fn art_ids_of(&self, art_type: ArtType) -> Vec<Id> {
        let mut ids: Vec<Id> = self
            .arts()
            .filter(|a| a.art_type == art_type)
            .map(|a| a.id.clone())
            .collect();
        ids.sort();
        ids
    }

    /// Looks up a House by id.
    pub fn house(&self, id: &Id) -> Option<&House> {
        self.houses.get(id)
    }

    /// Iterates all Houses in id order.
    pub fn houses(&self) -> impl Iterator<Item = &House> {
        self.houses.values()
    }

    /// Number of Houses in the catalogue.
    pub fn house_count(&self) -> usize {
        self.houses.len()
    }

    /// Looks up a Mythic Companion type by id.
    pub fn mythic_type(&self, id: &Id) -> Option<&MythicCompanionType> {
        self.mythic_companion_types.get(id)
    }

    /// Iterates all Mythic Companion types in id order.
    pub fn mythic_types(&self) -> impl Iterator<Item = &MythicCompanionType> {
        self.mythic_companion_types.values()
    }

    /// Number of Mythic Companion types in the catalogue.
    pub fn mythic_type_count(&self) -> usize {
        self.mythic_companion_types.len()
    }

    /// Looks up a spell by id.
    pub fn spell(&self, id: &Id) -> Option<&Spell> {
        self.spells.get(id)
    }

    /// Iterates all spells in id order.
    pub fn spells(&self) -> impl Iterator<Item = &Spell> {
        self.spells.values()
    }

    /// Number of spells in the catalogue.
    pub fn spell_count(&self) -> usize {
        self.spells.len()
    }

    /// Looks up a Spell Mastery special ability by id.
    pub fn spell_mastery_ability(&self, id: &Id) -> Option<&SpellMasteryAbility> {
        self.spell_mastery_abilities.get(id)
    }

    /// Iterates all Spell Mastery special abilities in id order.
    pub fn spell_mastery_abilities(&self) -> impl Iterator<Item = &SpellMasteryAbility> {
        self.spell_mastery_abilities.values()
    }

    /// Number of Spell Mastery special abilities in the catalogue.
    pub fn spell_mastery_ability_count(&self) -> usize {
        self.spell_mastery_abilities.len()
    }

    /// Looks up a weapon by id.
    pub fn weapon(&self, id: &Id) -> Option<&Weapon> {
        self.weapons.get(id)
    }

    /// Iterates all weapons in id order.
    pub fn weapons(&self) -> impl Iterator<Item = &Weapon> {
        self.weapons.values()
    }

    /// Number of weapons in the catalogue.
    pub fn weapon_count(&self) -> usize {
        self.weapons.len()
    }

    /// Looks up a shield by id.
    pub fn shield(&self, id: &Id) -> Option<&Shield> {
        self.shields.get(id)
    }

    /// Iterates all shields in id order.
    pub fn shields(&self) -> impl Iterator<Item = &Shield> {
        self.shields.values()
    }

    /// Number of shields in the catalogue.
    pub fn shield_count(&self) -> usize {
        self.shields.len()
    }

    /// Looks up an armor entry by id.
    ///
    /// Named `armor_item` rather than the sibling singular `armor` because the
    /// plural iterator already claims `armor()` — "armor" is an uncountable noun,
    /// so it has no distinct plural to hand the iterator. This is the one
    /// deliberate exception to the singular-noun lookup convention used by every
    /// other catalogue accessor (`item`, `ability`, `weapon`, `shield`, …).
    pub fn armor_item(&self, id: &Id) -> Option<&Armor> {
        self.armor.get(id)
    }

    /// Iterates all armor entries in id order.
    pub fn armor(&self) -> impl Iterator<Item = &Armor> {
        self.armor.values()
    }

    /// Number of armor entries in the catalogue.
    pub fn armor_count(&self) -> usize {
        self.armor.len()
    }

    /// Number of Arts in the catalogue.
    pub fn art_count(&self) -> usize {
        self.arts.len()
    }

    /// The Art XP advancement table.
    pub fn art_advancement(&self) -> &AdvancementTable {
        &self.art_advancement
    }

    /// The Characteristic point-buy rules, if the ruleset ships them.
    pub fn characteristic_rules(&self) -> Option<&CharacteristicRules> {
        self.characteristic_rules.as_ref()
    }

    /// The life-stage experience rules, if the ruleset ships them.
    pub fn life_stages(&self) -> Option<&LifeStageRules> {
        self.life_stages.as_ref()
    }

    /// The aging tables, if the ruleset ships them. `None` stands the aging
    /// subsystem down.
    pub fn aging(&self) -> Option<&AgingRules> {
        self.aging.as_ref()
    }

    /// Looks up a Sample Childhood package by id.
    pub fn childhood(&self, id: &Id) -> Option<&ChildhoodPackage> {
        self.childhoods.get(id)
    }

    /// Iterates all Sample Childhood packages in id order.
    pub fn childhoods(&self) -> impl Iterator<Item = &ChildhoodPackage> {
        self.childhoods.values()
    }

    /// Ability categories that may only be bought with a permitting Virtue
    /// (Ars Magica - Definitive Edition (Core Rules).md:2315). Empty for a ruleset that gates none.
    pub fn categories_requiring_virtue(&self) -> &BTreeSet<AbilityCategory> {
        &self.categories_requiring_virtue
    }

    /// The scholarly-language expectation for Academic Abilities (Ars Magica - Definitive Edition (Core Rules).md:7151), if the
    /// ruleset states one.
    pub fn scholarly_language_requirement(&self) -> Option<&ScholarlyLanguageRequirement> {
        self.scholarly_language.as_ref()
    }

    /// Iterates over point items of the given [`ItemKind`].
    pub fn items_by_kind(&self, kind: ItemKind) -> impl Iterator<Item = &PointItem> {
        self.point_items.values().filter(move |i| i.kind == kind)
    }

    /// Iterates over point items in the given category.
    pub fn items_by_category<'a>(
        &'a self,
        category: &'a str,
    ) -> impl Iterator<Item = &'a PointItem> {
        self.point_items
            .values()
            .filter(move |i| i.category == category)
    }
}
