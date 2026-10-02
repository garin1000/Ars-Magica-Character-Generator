//! [`Doc`]'s label/name resolution primitives: the fail-loud lookups
//! ([`Doc::label`], [`Doc::name`]) that back the "never render a raw slug"
//! contract (audit finding E5), the tolerant free-text counterparts for a
//! parameter's chosen *value* ([`Doc::param_value`], [`Doc::value_template`],
//! [`Doc::parameterized_value`]), and the shared `{placeholder}` template
//! filler ([`Doc::fill_template`]) both paths use. Split out of `export.rs`;
//! see `export/sections.rs` and `export/magic.rs` for the section writers
//! built on top of these.

use super::*;

impl<'a> Doc<'a> {
    /// The language-neutral ruleset behind the localized one.
    pub(super) fn rules(&self) -> &'a Ruleset {
        &self.ruleset.ruleset
    }

    /// The localized chrome text for `key`. Records `key` as missing (see
    /// [`Doc::missing`]) when the caller's label map has no entry, so rendering
    /// can continue — the caller only learns of the failure once
    /// [`character_markdown`] has walked the whole document — while still never
    /// letting the raw key reach a *successful* result.
    pub(super) fn label(&self, key: &str) -> String {
        match self.labels.get(key) {
            Some(text) => text.clone(),
            None => {
                self.missing
                    .borrow_mut()
                    .insert(MissingLabel::Key(key.to_string()));
                key.to_string()
            }
        }
    }

    /// The localized display name for a catalogue id. Records `id` as missing
    /// (see [`Doc::missing`]) when the loaded ruleset's i18n has no entry, on the
    /// same "keep rendering, fail the whole document at the end" contract as
    /// [`Doc::label`]. Reserved for genuine catalogue lookups (a House, a Virtue,
    /// an Ability, a spell, …) where an unresolved id is foreign or stale data —
    /// **not** for a parameter's chosen *value*, which is legitimately free text
    /// as often as not (see [`Doc::param_value`]).
    pub(super) fn name(&self, id: &Id) -> String {
        match self.ruleset.display_name(id) {
            Some(name) => name.to_string(),
            None => {
                self.missing
                    .borrow_mut()
                    .insert(MissingLabel::CatalogueId(id.as_str().to_string()));
                id.as_str().to_string()
            }
        }
    }

    /// The localized short abbreviation of an Art (`Cr`, `Ig`) — the notation the
    /// rulebook and the app both use for a spell's Arts. It is rules i18n data, never
    /// composed here; an Art whose entry ships none falls back to its display name,
    /// which is long but still readable, rather than leaving the code half-written.
    pub(super) fn art_code(&self, id: &Id) -> String {
        match self.ruleset.abbreviation(id) {
            Some(abbreviation) => escape_cell(abbreviation),
            None => escape_cell(&self.name(id)),
        }
    }

    /// Appends an ATX heading whose text is the chrome label for `key`.
    pub(super) fn section(&self, out: &mut String, level: usize, key: &str) {
        heading(out, level, &self.label(key));
    }

    /// Appends a `- **<label for key>**: value` bullet.
    pub(super) fn labelled(&self, out: &mut String, key: &str, value: &str) {
        field(out, &self.label(key), value);
    }

    /// A Might Score as `<localized realm> <score>`. Shared by the being's own Might
    /// and the familiar's, which are separate scores with the same shape.
    pub(super) fn realm_score(&self, might: &MightScore) -> String {
        format!(
            "{} {}",
            self.label(&format!("realm-{}", might.realm)),
            might.score
        )
    }

    /// The localized separator for an inline list, mirroring the frontend's
    /// `restrictedPoolLabel` (`"<sep> "`).
    pub(super) fn list_separator(&self) -> String {
        format!("{} ", self.label("restricted-xp-list-separator"))
    }

    /// One parameter *value* rendered for display: a value that happens to be a
    /// catalogue id resolves to its localized name, and free text (an Area Lore's
    /// region, a Magical Focus's field) passes through as typed.
    ///
    /// Deliberately bypasses [`Doc::name`]'s strict tracking: unlike a House, a
    /// Virtue or a spell — ids the entity claims to hold, so an unresolved one is a
    /// data problem — the *value* slotted into a parameter is free text far more
    /// often than it is a catalogue id, so `None` here is the ordinary case, not a
    /// defect worth failing the export over.
    pub(super) fn param_value(&self, raw: &str) -> String {
        escape_cell(self.ruleset.display_name(&Id::new(raw)).unwrap_or(raw))
    }

    /// The display value for a parameterized Ability's stored
    /// [`AbilityParameterValue`] (CV4) — never the raw wire shape and never a
    /// bare catalogue slug (this module's own "never render a raw slug"
    /// contract, audit finding E5).
    ///
    /// `Text` goes through [`Self::param_value`] unchanged — exactly the
    /// pre-CV4 behavior, since the field was a bare string then. `Catalogued`
    /// resolves through `self.ruleset.display_name(id)` — CV7 merges each
    /// catalogue value's own-language name into `LocalizedRuleset.i18n` at
    /// load (`arm-app::ruleset_io::merge_catalogue_display_names`), the SAME
    /// map every other id's display name already lives in, so a ruleset
    /// loaded through the real app path resolves here for free. The
    /// id's-own-final-segment fallback (`language.latin` → "Latin") stays as
    /// defense-in-depth for a hand-built ruleset that never merged catalogue
    /// names in (e.g. a test fixture), never printing the raw slug either
    /// way. `Linked` (design § 6.4) resolves via [`crate::effective::resolve_link`]
    /// against effective selections — the SAME resolver the load-time fold and
    /// matching use, so "what does this link currently mean" cannot disagree
    /// across readers — and shows the deterministic ambiguity fallback rather
    /// than a blank cell (design § 4.1); never a raw id or `(item, param)` pair.
    pub(super) fn ability_param_value(&self, value: &AbilityParameterValue) -> String {
        match value {
            AbilityParameterValue::Text { text } => self.param_value(text),
            AbilityParameterValue::Catalogued { id } => {
                let owned;
                let resolved = match self.ruleset.display_name(id) {
                    Some(name) => name,
                    None => {
                        owned = humanize_catalogue_id(id.as_str());
                        owned.as_str()
                    }
                };
                escape_cell(resolved)
            }
            AbilityParameterValue::Linked { item, param } => {
                let resolved =
                    match crate::effective::resolve_link(self.entity, self.rules(), item, param) {
                        crate::effective::LinkResolution::Resolved(value) => value,
                        crate::effective::LinkResolution::Ambiguous(fallback) => fallback,
                        crate::effective::LinkResolution::Dangling => None,
                    };
                escape_cell(&resolved.unwrap_or_default())
            }
        }
    }
}

/// A readable label derived from a catalogue id's own final segment
/// (`language.latin` → "Latin", `organization.house_bjornaer` → "House
/// Bjornaer") — a structural transform, not a hardcoded per-value string, and
/// strictly more readable than printing the slug outright. Mirrors the
/// frontend's `humanizeCatalogueId` (`ui/src/lib/derive.ts`) so the interim
/// fallback reads the same on both surfaces until CV6/CV8's real localized
/// resolution replaces both.
fn humanize_catalogue_id(id: &str) -> String {
    let last = id.rsplit('.').next().unwrap_or(id);
    last.split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

impl<'a> Doc<'a> {
    /// The display values for one selection's parameters, keyed as its name template
    /// expects them.
    ///
    /// A value may itself be a *parameterized* catalogue item: Puissant Ability aimed
    /// at the character's Provence Lore stores `{ability: ability.area_lore, area:
    /// "Provence"}`, where `area` is the target instance's own discriminator rather
    /// than a parameter of Puissant Ability (this is exactly what the frontend's
    /// `onSelectAbility` writes). So each value is rendered as a full instance name
    /// with the sibling discriminators folded in ("Provence Lore"), and the keys a
    /// value swallowed are dropped from the map — otherwise the row would both leak the
    /// raw template and repeat the discriminator ("Puissant {area} Lore (Provence)").
    ///
    /// `item` is the selection's own catalogue entry, needed for the parameter
    /// *domains*: a value whose domain is a fixed engine taxonomy rather than a
    /// catalogue reference has no rules-i18n entry at all and must be labelled
    /// through its own Fluent family instead of falling through to raw text
    /// (see [`Doc::taxonomy_label`]).
    pub(super) fn param_display_values(
        &self,
        item: &PointItem,
        params: &BTreeMap<String, SelectionParamValue>,
    ) -> BTreeMap<String, String> {
        let mut swallowed: BTreeSet<String> = BTreeSet::new();
        let mut rendered: BTreeMap<String, String> = BTreeMap::new();
        for (key, value) in params {
            // C0b: only `Single` values are ever produced; a `Multi` value has
            // no display rendering defined yet (C5b's job) and is skipped here
            // rather than guessed at.
            let Some(value) = value.as_single() else {
                continue;
            };
            if let Some(label) = self.taxonomy_label(item, key, value) {
                rendered.insert(key.clone(), label);
                continue;
            }
            // Only the value's own placeholders that a sibling parameter can fill; an
            // unfillable one keeps its slot label, as everywhere else.
            let fills: BTreeMap<String, String> = self
                .placeholder_keys(value)
                .into_iter()
                .filter_map(|placeholder| {
                    let sibling = params.get(&placeholder)?.as_single()?;
                    Some((placeholder, self.param_value(sibling.as_str())))
                })
                .collect();
            swallowed.extend(fills.keys().cloned());
            rendered.insert(key.clone(), self.parameterized_value(value, &fills));
        }
        rendered.retain(|key, _| !swallowed.contains(key));
        rendered
    }

    /// The localized label for a parameter value that names a member of a fixed
    /// **engine taxonomy** rather than a catalogue entry — `None` for every domain
    /// whose values are catalogue ids or free text, which the ordinary
    /// [`Doc::param_value`] path already handles.
    ///
    /// A taxonomy member (a category id such as `social_status`) has no rules-i18n
    /// entry of its own, so the tolerant value path would find nothing and print
    /// the bare slug — the exact "raw ID as a user-facing label" breach the sheet's
    /// chrome contract exists to prevent. Its label lives in the same
    /// `category-<id>` Fluent family the Type cell and the in-app badge already
    /// read, so nothing new has to be declared: the caller's map is composed from
    /// the catalogue's own categories, and load-time integrity requires a
    /// `Category` parameter's values to be a subset of its item's categories.
    ///
    /// The `match` is exhaustive on purpose: a future domain whose values are a
    /// taxonomy (a `Realm`, say, labelled through the existing `realm-<id>` family
    /// [`Doc::realm_score`] uses) belongs in a new arm here, and leaving it in the
    /// `None` group is then a deliberate, visible choice rather than an oversight.
    fn taxonomy_label(&self, item: &PointItem, key: &str, value: &Id) -> Option<String> {
        let domain = item.parameters.iter().find(|p| p.key == key)?.domain;
        match domain {
            ParameterDomain::Category => Some(self.label(&format!("category-{value}"))),
            // Folk Magic's realm axis (ArMDE:3909, :3919). The stored value is `realm.<slug>`
            // and the Fluent family is keyed on the bare slug — the same
            // `realm-<id>` keys `Doc::realm_score` reads for a Might score, so
            // nothing new has to be declared. A value that is not a Realm at
            // all cannot be labelled and falls through to the ordinary value
            // path; it is already an `unknown_param_value` on the sheet's own
            // validation, so the raw text is the honest thing to show.
            ParameterDomain::Realm => {
                Realm::from_id(value).map(|realm| self.label(&format!("realm-{realm}")))
            }
            // X6a/e5: the closed 5-member `AbilityCategory` enum, labelled
            // through the already-shipped `ability-category-<slug>` family
            // (the ability filter UI) — the same idiom as `Realm` above.
            ParameterDomain::AbilityCategory => crate::ability::AbilityCategory::from_id(value)
                .map(|category| self.label(&format!("ability-category-{category}"))),
            ParameterDomain::Ability
            | ParameterDomain::Art
            | ParameterDomain::Technique
            | ParameterDomain::Form
            | ParameterDomain::Characteristic
            | ParameterDomain::Item
            | ParameterDomain::Enumerated
            | ParameterDomain::Text
            // A number is printed as itself, not looked up in a taxonomy.
            | ParameterDomain::Number
            // A spell id is a catalogue-shaped id (resolved against the
            // character's own learned spells, not a fixed taxonomy), so it
            // takes the ordinary value path exactly like Ability/Item.
            | ParameterDomain::Spell => None,
        }
    }

    /// The template text for a parameter *value* stored as an [`Id`] — which, like
    /// [`Doc::param_value`], is a nested catalogue reference as often as it is free
    /// text (`fire`, a Magical Focus's field), so resolution here never records a
    /// miss. Distinct from [`Doc::name`], which is for an id the entity claims to
    /// *hold* (a House, a bought Ability, a known spell) rather than a value it
    /// merely *chose*.
    pub(super) fn value_template(&self, id: &Id) -> String {
        self.ruleset
            .display_name(id)
            .unwrap_or_else(|| id.as_str())
            .to_string()
    }

    /// The `{placeholder}` keys a parameter value's own localized template
    /// mentions, in the order-free set [`param_display_values`] needs. An
    /// unterminated brace is literal text and names no key, matching
    /// [`Doc::fill_template`].
    fn placeholder_keys(&self, id: &Id) -> BTreeSet<String> {
        let name = self.value_template(id);
        let mut keys = BTreeSet::new();
        let mut rest = name.as_str();
        while let Some(open) = rest.find('{') {
            let after = &rest[open + 1..];
            let Some(close) = after.find('}') else {
                return keys;
            };
            keys.insert(after[..close].to_string());
            rest = &after[close + 1..];
        }
        keys
    }

    /// A localized **item** name with its chosen parameters folded in — `id` is an
    /// id the entity claims to hold (a Virtue/Flaw, a bought or granted Ability, a
    /// known spell), so an unresolved one goes through [`Doc::name`]'s strict
    /// tracking. See [`Doc::parameterized_value`] for the sibling case, a
    /// parameter *value* that may not be a catalogue id at all.
    pub(super) fn parameterized_name(&self, id: &Id, values: &BTreeMap<String, String>) -> String {
        if let Some(unfilled) = self.unfilled_name(id, values) {
            return unfilled;
        }
        self.fill_template(&self.name(id), values, self.rules().item(id))
    }

    /// The entry's own [`I18nEntry::name_unfilled`], when it declares one and **no**
    /// placeholder in its template is filled.
    ///
    /// The sheet honours the same opt-out the in-app label path does
    /// (`displayName` in `ui/src/lib/derive.ts`), so an exported Ability and the one on
    /// screen are worded identically. Without this the sheet alone would print the
    /// doubled *"(Language) (Dead Language)"* the field exists to remove. Returns
    /// `None` for every entry that declares no unfilled form — which is almost all of
    /// them, and they keep the slot hint, since "Puissant (Ability)" is right and a
    /// bare "Puissant" would not be.
    fn unfilled_name(&self, id: &Id, values: &BTreeMap<String, String>) -> Option<String> {
        let unfilled = self.ruleset.entry(id)?.name_unfilled.as_deref()?;
        let keys = self.placeholder_keys(id);
        if keys.is_empty() || keys.iter().any(|key| values.contains_key(key)) {
            return None;
        }
        Some(escape_cell(unfilled).to_string())
    }

    /// A parameter value's own localized template, with its chosen (sibling)
    /// parameters folded in — the counterpart of [`Doc::parameterized_name`] for a
    /// value that may be a nested catalogue reference or plain free text, so
    /// resolution goes through the tolerant [`Doc::value_template`] rather than
    /// [`Doc::name`].
    pub(super) fn parameterized_value(&self, id: &Id, values: &BTreeMap<String, String>) -> String {
        if let Some(unfilled) = self.unfilled_name(id, values) {
            return unfilled;
        }
        self.fill_template(&self.value_template(id), values, None)
    }

    /// Fills `template`'s `{key}` placeholders from `values`
    /// ("Puissant {ability}" → "Puissant Awareness"); a placeholder with no value
    /// shows the localized slot label instead ("Puissant (Ability)"), and a value
    /// the template never mentions is appended in parentheses ("Minor Magical
    /// Focus (fire)") so a chosen parameter can never be silently dropped.
    /// `item` is the catalogue entry whose name this is, when it has one; see
    /// [`Doc::template_extras`] for what it changes.
    fn fill_template(
        &self,
        template: &str,
        values: &BTreeMap<String, String>,
        item: Option<&PointItem>,
    ) -> String {
        let template = escape_cell(template);
        let mut filled = String::new();
        let mut consumed: BTreeSet<&str> = BTreeSet::new();
        let mut rest = template.as_str();
        while let Some(open) = rest.find('{') {
            filled.push_str(&rest[..open]);
            let after = &rest[open + 1..];
            let Some(close) = after.find('}') else {
                // An unterminated brace is literal text, not a placeholder.
                filled.push('{');
                rest = after;
                continue;
            };
            let key = &after[..close];
            match values.get(key) {
                Some(value) => {
                    filled.push_str(value);
                    consumed.insert(key);
                }
                None => {
                    filled.push('(');
                    filled.push_str(&self.label(&format!("param-label-{key}")));
                    filled.push(')');
                }
            }
            rest = &after[close + 1..];
        }
        filled.push_str(rest);
        let extras = Self::template_extras(values, &consumed, item);
        if extras.is_empty() {
            return filled;
        }
        format!("{filled} ({})", extras.join(&self.list_separator()))
    }

    /// The values `template` did not consume, in the order they are appended.
    ///
    /// D81.8: an item declaring [`PointItem::unordered_param_groups`] has its
    /// grouped values printed as one entry per group, the members joined by a
    /// space with Technique before Form ("Creo Ignem"). This is the same pairing
    /// the in-app picker shows (`ParameterPicker.svelte::paramGroups`). Without
    /// it, Incompatible Arts printed its four Arts in key order and the two
    /// barred combinations could not be read off the sheet (ArMDE:6290-6292).
    /// Ungrouped values trail afterwards in key order, which is the whole output
    /// for an item with no groups, exactly as before.
    fn template_extras(
        values: &BTreeMap<String, String>,
        consumed: &BTreeSet<&str>,
        item: Option<&PointItem>,
    ) -> Vec<String> {
        let open = |key: &str| values.contains_key(key) && !consumed.contains(key);
        let mut extras = Vec::new();
        let mut grouped: BTreeSet<&str> = BTreeSet::new();
        if let Some(item) = item {
            for group in &item.unordered_param_groups {
                let mut members: Vec<&str> = group
                    .iter()
                    .map(String::as_str)
                    .filter(|key| open(key))
                    .collect();
                if members.is_empty() {
                    continue;
                }
                members.sort_by_key(|key| group_member_rank(item, key));
                grouped.extend(members.iter().copied());
                let words: Vec<&str> = members.iter().map(|key| values[*key].as_str()).collect();
                extras.push(words.join(" "));
            }
        }
        extras.extend(
            values
                .iter()
                .filter(|(key, _)| open(key) && !grouped.contains(key.as_str()))
                .map(|(_, value)| value.clone()),
        );
        extras
    }
}

/// Where a grouped parameter sits within its printed group (D81.8): Technique,
/// then Form, then any other domain, each tier in the item's declared order —
/// the order Hermetic Arts are spoken in ("Creo Ignem").
fn group_member_rank(item: &PointItem, key: &str) -> (u8, usize) {
    let Some(index) = item.parameters.iter().position(|p| p.key == key) else {
        return (u8::MAX, usize::MAX);
    };
    let tier = match item.parameters[index].domain {
        ParameterDomain::Technique => 0,
        ParameterDomain::Form => 1,
        _ => 2,
    };
    (tier, index)
}
