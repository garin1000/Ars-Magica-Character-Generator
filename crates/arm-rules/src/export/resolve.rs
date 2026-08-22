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
    pub(super) fn param_display_values(
        &self,
        params: &BTreeMap<String, Id>,
    ) -> BTreeMap<String, String> {
        let mut swallowed: BTreeSet<String> = BTreeSet::new();
        let mut rendered: BTreeMap<String, String> = BTreeMap::new();
        for (key, value) in params {
            // Only the value's own placeholders that a sibling parameter can fill; an
            // unfillable one keeps its slot label, as everywhere else.
            let fills: BTreeMap<String, String> = self
                .placeholder_keys(value)
                .into_iter()
                .filter_map(|placeholder| {
                    let sibling = params.get(&placeholder)?;
                    Some((placeholder, self.param_value(sibling.as_str())))
                })
                .collect();
            swallowed.extend(fills.keys().cloned());
            rendered.insert(key.clone(), self.parameterized_value(value, &fills));
        }
        rendered.retain(|key, _| !swallowed.contains(key));
        rendered
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
        self.fill_template(&self.name(id), values)
    }

    /// A parameter value's own localized template, with its chosen (sibling)
    /// parameters folded in — the counterpart of [`Doc::parameterized_name`] for a
    /// value that may be a nested catalogue reference or plain free text, so
    /// resolution goes through the tolerant [`Doc::value_template`] rather than
    /// [`Doc::name`].
    pub(super) fn parameterized_value(&self, id: &Id, values: &BTreeMap<String, String>) -> String {
        self.fill_template(&self.value_template(id), values)
    }

    /// Fills `template`'s `{key}` placeholders from `values`
    /// ("Puissant {ability}" → "Puissant Awareness"); a placeholder with no value
    /// shows the localized slot label instead ("Puissant (Ability)"), and a value
    /// the template never mentions is appended in parentheses ("Minor Magical
    /// Focus (fire)") so a chosen parameter can never be silently dropped.
    fn fill_template(&self, template: &str, values: &BTreeMap<String, String>) -> String {
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
        let extras: Vec<&str> = values
            .iter()
            .filter(|(key, _)| !consumed.contains(key.as_str()))
            .map(|(_, value)| value.as_str())
            .collect();
        if extras.is_empty() {
            return filled;
        }
        format!("{filled} ({})", extras.join(&self.list_separator()))
    }
}
