//! Selection legality: categories, traits, parameters, foci, and Gift policy.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;

/// The category an issue message names when it has room for exactly one, taken
/// from the list [`PointItem::categories_for`] put *in force* for that
/// selection rather than from the item's whole descriptor.
///
/// Both callers reach here only once every in-force category has failed their
/// test, so any of them is a truthful representative and the first is simply
/// the deterministic pick. The distinction that matters is *which list* it is
/// the first of: for a `taken_as` selection the list is the single chosen
/// category, so the message names the reading the player actually took (Sufi
/// as Social Status is reported against Social Status), while
/// [`PointItem::first_listed_category`] would have named the descriptor's
/// first — a category they explicitly declined.
///
/// Empty only for a catalogue that failed load-time integrity, which rejects a
/// categoryless item.
fn first_in_force(in_force: &[String]) -> &str {
    in_force.first().map(String::as_str).unwrap_or_default()
}

/// The categories a profile's list puts **in force** for this entity: every
/// unconditional entry, plus every conditional one whose `when` evaluates to
/// [`Tri::True`].
///
/// `Tri::False` and `Tri::Unknown` both leave an entry out of force. Unknown
/// resolving in the player's favour is deliberate and matches the rest of the
/// engine: an item whose prerequisite cannot be decided yields the non-blocking
/// `prereq_unevaluated` warning rather than an error, so a category rule that
/// cannot be decided must not silently grant or withhold. Which direction
/// "in the player's favour" points differs per caller and needs no special
/// case: dropping an undecided *permit* is the strict reading, dropping an
/// undecided *forbid* is the lenient one, and the pair is authored as
/// complements, so an undecided condition simply leaves the item to be judged
/// by the profile's other, unconditional rules.
///
/// The single resolution point for both gates below, so they cannot disagree
/// about which entries apply — the same "decide it once" discipline
/// `PointItem::categories_for` applies to an item's own categories.
fn categories_in_force<'a>(rules: &'a [CategoryRule], ctx: &PrereqCtx) -> BTreeSet<&'a str> {
    rules
        .iter()
        .filter(|rule| match rule.when() {
            None => true,
            Some(when) => ctx.evaluate(when).0 == Tri::True,
        })
        .map(CategoryRule::category)
        .collect()
}

pub(crate) fn validate_permitted_categories(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    ctx: &PrereqCtx,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    // An empty permitted list means "no category restriction". The test is on
    // the DECLARED list, not on the in-force one: a profile that declares only
    // conditional permits and satisfies none of them permits nothing, which is
    // the opposite of declaring no restriction at all.
    if profile.permitted_categories.is_empty() {
        return;
    }
    let permitted = categories_in_force(&profile.permitted_categories, ctx);

    for selection in &entity.selections {
        // The profile's own gift is governed solely by `validate_gift_policy`
        // (required/allowed/forbidden). Exempt it here: it is contradictory for a
        // profile to mandate a trait via `gift_policy` yet reject its category
        // (The Gift is `special`), so gifted profiles need not whitelist it.
        if profile.gift_id.as_ref() == Some(&selection.item_ref) {
            continue;
        }

        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        // An item is permitted when *any* of its categories is: a descriptor
        // naming two categories offers two routes to the same Virtue/Flaw, and
        // the profile need only allow one of them. Suppressed Gift is "*Major,
        // Hermetic, Story*", so a companion — who may take Story but not
        // Hermetic — may take it.
        // Source: ArMDE:6803-6804.
        //
        // Taken-as aware: if the selection recorded which category it was
        // taken as (Sufi's `taken_as`, `ArMDE:5083`), only that ONE category is
        // "in force" here — a player who took Sufi as Social Status has
        // declared they are NOT taking the Supernatural reading, so the
        // Supernatural category must not rescue them from a profile that
        // forbids it. See `PointItem::categories_for`.
        //
        // Judged against the categories the PROFILE puts in force for this
        // entity, so `ArMDE:2840`'s "unless you have The Gift" opens `hermetic` to a
        // Gifted companion and to nobody else.
        let in_force = item.categories_for(&selection.params);
        if !in_force.iter().any(|c| permitted.contains(c.as_str())) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_CATEGORY_NOT_PERMITTED,
                CreationPhase::VirtuesFlaws,
                args([
                    ("item", selection.item_ref.to_string()),
                    // Every category IN FORCE failed the test, and the message
                    // has room for one, so the first of them represents the
                    // item — a deterministic pick, not a claim that it is the
                    // item's "real" category. Reading it off `categories_for`
                    // rather than the whole descriptor is what makes the
                    // message honest for a `taken_as` selection: exactly one
                    // category was judged there, so naming any other would tell
                    // the player their Virtue was rejected for a reading they
                    // explicitly did not take. Identical to
                    // `first_listed_category` whenever nothing narrowed the
                    // list.
                    ("category", first_in_force(in_force).to_string()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

pub(crate) fn validate_forbidden_categories(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    ctx: &PrereqCtx,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    if profile.forbidden_categories.is_empty() {
        return;
    }
    let forbidden = categories_in_force(&profile.forbidden_categories, ctx);

    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        // Forbidding is the true mirror of permitting: *every* one of the item's
        // categories must be forbidden before it is ruled out. A descriptor
        // naming two categories offers two routes to the same Virtue/Flaw, so one
        // forbidden route leaves the other open — otherwise the two checks
        // contradict each other, with permitting granting a route that forbidding
        // takes straight back. Suppressed Gift is "*Major, Hermetic, Story*", so a
        // companion — who may take Story but not Hermetic — may take it; and Sufi
        // is "*Minor, Social Status, Supernatural*", the mundane reading of which
        // is open to a grog who forbids only the Supernatural one.
        // For a single-category item "every" is identical to "any", so this
        // loosens nothing else in the catalogue.
        // Source: ArMDE:6803-6804
        // (Suppressed Gift's descriptor), :6809 (a companion's Flaw), :5079 and
        // :5083 (Sufi "either as a Minor Social Status Virtue or a Minor
        // Supernatural Virtue").
        //
        // Taken-as aware, via the same `categories_for` resolution the
        // permitted check above uses: a selection recording `taken_as` is
        // narrowed to that single category, so "every" degenerates to "is
        // that one forbidden" — a player who took Sufi as Social Status is
        // forbidden only if Social Status itself is forbidden, regardless of
        // whether Supernatural also is.
        //
        // Conditional-rule aware in the same way the permitted gate is, and
        // authored as its complement: `ArMDE:2840` forbids a companion `hermetic`
        // only while he lacks The Gift, so the forbid drops out for exactly the
        // character the permit appears for. An empty in-force set therefore
        // forbids nothing, which is what `all` over a non-empty category list
        // already yields.
        let in_force = item.categories_for(&selection.params);
        if !in_force.iter().all(|c| forbidden.contains(c.as_str())) {
            continue;
        }
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_FORBIDDEN_CATEGORY,
            CreationPhase::VirtuesFlaws,
            args([
                ("item", selection.item_ref.to_string()),
                // Every category in force is forbidden, and the message has room
                // for one, so the first of them represents the item — the same
                // deterministic tie-break `category_not_permitted` uses, over the
                // same taken-as-narrowed list, so a Sufi taken as Social Status
                // is reported against Social Status and never against the
                // Supernatural reading the player declined.
                ("category", first_in_force(in_force).to_string()),
            ]),
            Some(selection.item_ref.clone()),
        ));
    }
}

pub(crate) fn validate_entity_kind_applicability(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        if !item.entity_kinds.is_empty() && !item.entity_kinds.contains(&entity.entity_kind) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_WRONG_ENTITY_KIND,
                CreationPhase::VirtuesFlaws,
                args([
                    ("item", selection.item_ref.to_string()),
                    ("entity_kind", entity.entity_kind.to_string()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

/// Enforces `max_per_target`: how many copies of an item may share one
/// identical `(id, params)` target. Grant-aware — `selections` is the folded
/// bought-plus-granted list ([`crate::effective::selections_for_effects`]), so
/// a House-granted copy of a target counts the same as a bought one. This
/// closes a wrong-rules-output bug: a Flambeau magus granted a free Puissant
/// Ignem who also BUYS Puissant Ignem is taking the same Virtue for the same
/// target twice — illegal (ArMDE:4820,
/// "twice, for two different Arts") — but validated clean, and stacked +6 to
/// Ignem, before grants were folded in here.
pub(crate) fn validate_duplicate_selections(
    selections: &[Selection],
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut seen: BTreeMap<(&Id, &BTreeMap<String, Id>), usize> = BTreeMap::new();

    for selection in selections {
        let key = (&selection.item_ref, &selection.params);
        *seen.entry(key).or_insert(0) += 1;
    }

    for ((item_ref, _params), count) in &seen {
        // Selections are grouped by (item_ref, params): two selections of the
        // same parameterized item with DIFFERENT params are distinct targets and
        // do not collide here. An item may be taken up to `max_per_target` times
        // for the same target (default 1; Great Characteristic allows 2).
        let max = ruleset
            .point_items
            .get(*item_ref)
            .map_or(1, |item| usize::from(item.max_per_target));
        if *count <= max {
            continue;
        }
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_DUPLICATE_SELECTION,
            CreationPhase::VirtuesFlaws,
            args([
                ("item", item_ref.to_string()),
                ("count", count.to_string()),
                ("max", max.to_string()),
            ]),
            Some((*item_ref).clone()),
        ));
    }
}

/// Enforces `max_total`: how many copies of an item may exist TOTAL, across
/// EVERY distinct parameter target, counting granted copies. Complements
/// [`validate_duplicate_selections`] (`max_per_target`, one identical target):
/// this groups by `item_ref` alone, so e.g. a magus granted a free Puissant
/// Ignem who also buys a Puissant Perdo — two DIFFERENT targets, so
/// `max_per_target` never fires — still trips this validator if the item's
/// `max_total` says only one copy of Puissant Art may ever be held.
/// Grant-aware for the same reason as `validate_duplicate_selections`: a free
/// copy is still a copy.
pub(crate) fn validate_total_selection_cap(
    selections: &[Selection],
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut counts: BTreeMap<&Id, usize> = BTreeMap::new();

    for selection in selections {
        *counts.entry(&selection.item_ref).or_insert(0) += 1;
    }

    for (item_ref, count) in &counts {
        let Some(item) = ruleset.point_items.get(*item_ref) else {
            continue;
        };
        let max = usize::from(item.max_total);
        if *count <= max {
            continue;
        }
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_TOO_MANY_SELECTIONS,
            CreationPhase::VirtuesFlaws,
            args([
                ("item", item_ref.to_string()),
                ("count", count.to_string()),
                ("max", max.to_string()),
            ]),
            Some((*item_ref).clone()),
        ));
    }
}

/// Enforces every parameter's [`ParameterDef::max_per_value`]: how many of an
/// item's copies may name one and the same value for ONE parameter key.
///
/// Necessary (Realm) Aura for (Ability) is the case the rules state: "A
/// character may take this Flaw once for any particular Ability"
/// (ArMDE:6482). The Flaw declares two parameters and that sentence binds only
/// one of them, which is precisely what its two neighbours cannot say —
/// [`validate_duplicate_selections`] groups by the *whole* `(item_ref, params)`
/// tuple, so one Ability under two Realms collides in no key, and
/// [`validate_total_selection_cap`] groups by `item_ref` alone, so it cannot
/// say "per Ability" at all. Which key is capped, and at what, is data: no item
/// id and no parameter key appears here.
///
/// **The value is the whole target, not the bare id.** For an `ability`-domain
/// parameter aimed at a *parameterized* Ability the target is
/// `(ability, instance)` — two keys, the target's own instance key included, the
/// pair [`validate_selection_parameters`] makes mandatory and
/// [`validate_ability_bonus_targets`] already reads. Craft (Carpentry) and Craft
/// (Blacksmith) are two different Abilities, and `ArMDE:6484` explicitly
/// contemplates this Flaw "applied to Craft or Profession Abilities", so
/// counting them both as `ability.craft` would reject a character the rules
/// permit. See [`ability_instance`].
///
/// **Copies the duplicate check did not already report.** Two copies with an
/// *identical* tuple are `max_per_target`'s finding whenever that ceiling
/// rejects them, and counting them here too would draw two findings for one
/// mistake — the same division of labour [`validate_possessed_param_targets`]
/// keeps with the same neighbour. So each distinct tuple contributes at most
/// `max_per_target` copies: below that ceiling the copies are legal repeats
/// nobody else reports and they must count, at or above it the excess is
/// already the neighbour's finding and must not. An unconditional collapse to
/// one copy per tuple only *looks* equivalent, and only while the ceiling is 1:
/// at `max_per_target: 2` there is no neighbouring finding to defer to, so N
/// identical copies of a capped value would draw none at all. Catalogue shape
/// is data, so raising a ceiling is a data-only edit.
///
/// Grant-aware for the reason its neighbours are: `selections` is the folded
/// bought-plus-granted list ([`crate::effective::selections_for_effects`]), and
/// a granted copy names its Ability exactly as a bought one does.
pub(crate) fn validate_per_value_cap(
    selections: &[Selection],
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    // Grouped by item so each item is judged over all of its own copies, and by
    // tuple within it so the per-tuple count can be capped — see "copies the
    // duplicate check did not already report" above.
    let mut copies_by_item: BTreeMap<&Id, BTreeMap<&BTreeMap<String, Id>, usize>> = BTreeMap::new();
    for selection in selections {
        *copies_by_item
            .entry(&selection.item_ref)
            .or_default()
            .entry(&selection.params)
            .or_insert(0) += 1;
    }

    for (item_ref, copies_by_tuple) in copies_by_item {
        let Some(item) = ruleset.point_items.get(item_ref) else {
            continue; // unknown_ref already reported
        };
        let per_target = usize::from(item.max_per_target);
        for param in &item.parameters {
            // The default is a SENTINEL for "no ceiling the rules state", not
            // the number 255: a crafted save holding 256 copies of one value
            // must not invent a rule no rulebook contains. Skipping it also
            // bounds the counting below to the parameters that actually
            // declare a cap.
            if param.max_per_value == u8::MAX {
                continue;
            }
            let max = usize::from(param.max_per_value);
            let mut counts: BTreeMap<(&Id, Option<&str>), usize> = BTreeMap::new();
            for (params, copies) in &copies_by_tuple {
                let Some(value) = params.get(&param.key) else {
                    continue; // missing_param already reported
                };
                // Only an `ability` domain names an Ability; on any other, a
                // value that happened to spell one would pick up an instance
                // key that is not part of its target at all.
                let instance = matches!(param.domain, ParameterDomain::Ability)
                    .then(|| ability_instance(ruleset, params, value))
                    .flatten();
                *counts.entry((value, instance)).or_insert(0) += (*copies).min(per_target);
            }
            for ((value, _instance), count) in counts {
                if count <= max {
                    continue;
                }
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_TOO_MANY_FOR_PARAM_VALUE,
                    CreationPhase::VirtuesFlaws,
                    args([
                        ("item", item_ref.to_string()),
                        ("key", param.key.clone()),
                        // The Ability, not the instance: the message has room
                        // for one name and the instance is free text the player
                        // can read off the offending copies, which are on screen
                        // beside the finding. `count` is already the count for
                        // the instance, so the two agree.
                        ("value", value.to_string()),
                        ("count", count.to_string()),
                        ("max", max.to_string()),
                    ]),
                    Some(item_ref.clone()),
                ));
            }
        }
    }
}

/// The instance discriminator that completes an `ability`-domain parameter's
/// target, read from the selection's own parameter map: `(Area) Lore` needs an
/// `area`, `Craft` a `craft`, and a plain Ability needs none.
///
/// The engine's one spelling of "which Ability instance does this parameter
/// name", shared by [`validate_ability_bonus_targets`] and
/// [`validate_per_value_cap`] so the two cannot drift apart — the same
/// `(ability, instance)` pair `Entity::ability_scores` rows are keyed by and
/// the frontend's `usedAbilityTargets` composes.
///
/// `None` for an unparameterized Ability, for a `target` no catalogue knows
/// (already `unknown_param_value`), and for one whose instance key is simply
/// absent — which is [`validate_selection_parameters`]'s `missing_param`, not
/// this function's finding.
///
/// `target` is assumed to name an Ability; whether the parameter's domain says
/// so is the caller's test, since an effect target is one by construction while
/// a [`ParameterDef`]'s is not.
fn ability_instance<'a>(
    ruleset: &Ruleset,
    params: &'a BTreeMap<String, Id>,
    target: &Id,
) -> Option<&'a str> {
    let instance_key = ruleset.abilities.get(target)?.parameter.as_deref()?;
    params.get(instance_key).map(Id::as_str)
}

/// Enforces every parameter's [`ParameterDef::at_most_one_of`] groups across
/// the copies of one item: at most one member of a group may be named, however
/// many copies are held.
///
/// Folk Magic is the case the rules state: `ArMDE:3919` grants the repeat and limits
/// it in the same breath — "you can align it to the same Realm as before or
/// pick a different one, although a character cannot have access to both the
/// Divine and Infernal Realms". Which values exclude each other is **data**, so
/// no realm id appears here.
///
/// Distinct from its two neighbours, and orthogonal to both:
/// [`validate_duplicate_selections`] rejects copies that share an *identical*
/// target, and [`validate_total_selection_cap`] counts copies; this one is
/// about copies whose targets *differ* in a way the rules forbid. Grant-aware
/// for the same reason they are — `selections` is the folded bought-plus-granted
/// list, and a granted Divine alignment excludes a bought Infernal one just as
/// surely as a bought pair would.
pub(crate) fn validate_exclusive_param_values(
    selections: &[Selection],
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    // Grouped by item so each item is judged over all of its own copies, and
    // reported once per parameter rather than once per offending copy.
    let mut values_by_item: BTreeMap<&Id, Vec<&BTreeMap<String, Id>>> = BTreeMap::new();
    for selection in selections {
        values_by_item
            .entry(&selection.item_ref)
            .or_default()
            .push(&selection.params);
    }

    for (item_ref, params_of_copies) in values_by_item {
        let Some(item) = ruleset.point_items.get(item_ref) else {
            continue;
        };
        for param in &item.parameters {
            for group in &param.at_most_one_of {
                let named: BTreeSet<&Id> = params_of_copies
                    .iter()
                    .filter_map(|params| params.get(&param.key))
                    .filter(|value| group.contains(*value))
                    .collect();
                if named.len() < 2 {
                    continue;
                }
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_EXCLUSIVE_PARAM_VALUES,
                    CreationPhase::VirtuesFlaws,
                    args([
                        ("item", item_ref.to_string()),
                        ("key", param.key.clone()),
                        ("count", named.len().to_string()),
                    ]),
                    Some(item_ref.clone()),
                ));
            }
        }
    }
}

pub(crate) fn validate_required_traits(
    type_profile: Option<&EntityTypeProfile>,
    selected_ids: &BTreeSet<&Id>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    for required_id in &profile.required_traits {
        if !selected_ids.contains(required_id) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_MISSING_REQUIRED_TRAIT,
                CreationPhase::VirtuesFlaws,
                args([("item", required_id.to_string())]),
                Some(required_id.clone()),
            ));
        }
    }
}

pub(crate) fn validate_forbidden_traits(
    type_profile: Option<&EntityTypeProfile>,
    selected_ids: &BTreeSet<&Id>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    for forbidden_id in &profile.forbidden_traits {
        if selected_ids.contains(forbidden_id) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_FORBIDDEN_TRAIT,
                CreationPhase::VirtuesFlaws,
                args([("item", forbidden_id.to_string())]),
                Some(forbidden_id.clone()),
            ));
        }
    }
}

/// Whether a parameter `value` resolves against its domain's registry: `Item`
/// → point items, `Ability` → the ability catalogue, `Art` → the art catalogue,
/// `Technique`/`Form` → the art catalogue *and* the required art class (so
/// Deficient Technique cannot target a Form; ArMDE:5909-5915),
/// `Characteristic` → [`Characteristic::from_id`], `Enumerated` → the definition's
/// own declared `values`, `Text` → any value with non-whitespace content (no
/// registry, but the domain's own documentation says "non-empty", and an empty
/// string used to resolve — so an unnamed Power validated clean).
///
/// Takes the whole [`ParameterDef`] rather than its `domain` alone because the
/// `Enumerated` domain's registry IS the definition: its legal values are data on
/// the parameter, not a catalogue the ruleset holds.
///
/// Shared by virtue/flaw parameter validation and spell parameter validation.
pub(crate) fn param_value_resolves(ruleset: &Ruleset, param: &ParameterDef, value: &Id) -> bool {
    match param.domain {
        // `require_categories` narrows this domain to a category, exactly as
        // `Technique`/`Form` narrow `Art` to one Art class: an item outside it is
        // not in this parameter's domain, so it raises the same
        // `unknown_param_value` rather than a code of its own. Membership is read
        // from the item's own `categories` (`has_category`) — a bare value names
        // an ITEM, not a `Selection` of one, so there is no `taken_as` choice to
        // narrow against, and `index_categories` is provenance, never membership.
        //
        // `forbid_tainted` narrows it the same way and for the same reason:
        // "this Flaw cannot apply to Supernatural Virtues that are affiliated to
        // the Infernal realm in the first place"
        // (ArMDE:6096), and the
        // descriptor's *Tainted* tag is precisely that affiliation. Entity-free,
        // so it belongs here rather than in `validate_possessed_param_targets`.
        ParameterDomain::Item => ruleset.point_items.get(value).is_some_and(|item| {
            item_matches_required_categories(item, param) && !(param.forbid_tainted && item.tainted)
        }),
        ParameterDomain::Ability => ruleset.abilities.contains_key(value),
        ParameterDomain::Characteristic => Characteristic::from_id(value).is_some(),
        ParameterDomain::Art => ruleset.arts.contains_key(value),
        ParameterDomain::Technique => ruleset
            .arts
            .get(value)
            .is_some_and(|a| a.art_type == crate::art::ArtType::Technique),
        ParameterDomain::Form => ruleset
            .arts
            .get(value)
            .is_some_and(|a| a.art_type == crate::art::ArtType::Form),
        ParameterDomain::Enumerated => param.values.contains(value),
        // Same resolution as `Enumerated` — the domain IS the declared list —
        // but load-time integrity additionally requires that list to be a
        // subset of the declaring item's own `categories`
        // (`ruleset::integrity::validate_parameter_defs`).
        ParameterDomain::Category => param.values.contains(value),
        // No catalogue and no declared list: the four-member `Realm` enum IS the
        // registry, the same way `Characteristic` is for `characteristic`
        // (ArMDE:3909).
        ParameterDomain::Realm => Realm::from_id(value).is_some(),
        ParameterDomain::Text => !value.as_str().trim().is_empty(),
    }
}

/// Whether `item` satisfies `param`'s [`ParameterDef::require_categories`]:
/// vacuously true when the list is empty (the shape of every parameter shipped
/// today), otherwise a non-empty intersection with the item's own membership
/// categories — the same "the item is of that category" test
/// [`crate::grant::GrantConstraint::require_categories`] applies to an open
/// grant's pick.
fn item_matches_required_categories(item: &PointItem, param: &ParameterDef) -> bool {
    param.require_categories.is_empty()
        || param
            .require_categories
            .iter()
            .any(|category| item.has_category(category))
}

/// Whether a parameter value is blank — empty, or nothing but whitespace.
///
/// A blank value in ANY domain is a choice not yet made, not a wrong one: no
/// registry has a blank id either. Both parameter validators treat it as if the key
/// were absent, so it raises `missing_param` naming the key to fill rather than
/// `unknown_param_value`, which would render "has unknown text value " with nothing
/// where the offending value belongs.
pub(crate) fn param_value_is_blank(value: &Id) -> bool {
    value.as_str().trim().is_empty()
}

/// Validates that each selection of a parameterized item supplies exactly the
/// declared parameter keys (no missing, no extra) and that each provided value
/// resolves against its domain's registry: `item` → point items, `ability` →
/// the ability catalogue, `art` → the art catalogue, `characteristic` →
/// [`Characteristic::from_id`]. A value that does not resolve emits
/// `unknown_param_value`.
pub(crate) fn validate_parameters(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    for selection in &entity.selections {
        validate_selection_parameters(selection, ruleset, CreationPhase::VirtuesFlaws, issues);
    }
}

/// The parameter checks for ONE selection: declared-vs-provided keys
/// (`missing_param` / `unexpected_param`) and each value's domain resolution
/// (`unknown_param_value`). A key whose value is **blank** counts as unfilled, so
/// it raises `missing_param` — see [`param_value_is_blank`].
///
/// Shared by bought selections ([`validate_parameters`]) and the *derived* picks
/// that never live on `entity.selections` — House / Mythic-type Open grants and
/// the Warping-owed fills — so "{form} Monstrosity" chosen for an open grant is
/// held to the same standard as one bought on the V/F tab.
///
/// `phase` is the caller's, not this function's: the same three codes are fixed on
/// the V/F step for a bought selection, on the House or Mythic-type step for an
/// open grant, and only in the finished character for a Warping fill.
pub(crate) fn validate_selection_parameters(
    selection: &Selection,
    ruleset: &Ruleset,
    phase: CreationPhase,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
        return;
    };

    let declared: BTreeSet<&str> = item.parameters.iter().map(|p| p.key.as_str()).collect();
    let provided: BTreeSet<&str> = selection.params.keys().map(String::as_str).collect();
    // A key present with a BLANK value has supplied nothing, so it counts as
    // unfilled below: an empty text box is a choice not yet made. `provided` still
    // holds it, so a blank value under an *undeclared* key is still reported as the
    // stray key it is rather than vanishing.
    let filled: BTreeSet<&str> = selection
        .params
        .iter()
        .filter(|(_, value)| !param_value_is_blank(value))
        .map(|(key, _)| key.as_str())
        .collect();

    // A parameter targeting a PARAMETERIZED ability also expects the instance
    // discriminator, supplied under the target ability's own param key
    // ((Area) Lore → "area"). So Puissant on (Area) Lore needs both keys; on a
    // plain ability the instance key would be an unexpected extra.
    let mut expected = declared.clone();
    for param in &item.parameters {
        if matches!(param.domain, ParameterDomain::Ability)
            && let Some(target) = selection.params.get(&param.key)
            && let Some(ability) = ruleset.abilities.get(target)
            && let Some(instance_key) = ability.parameter.as_deref()
        {
            expected.insert(instance_key);
        }
    }

    for missing in expected.difference(&filled) {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_MISSING_PARAM,
            phase,
            args([
                ("item", selection.item_ref.to_string()),
                ("key", missing.to_string()),
            ]),
            Some(selection.item_ref.clone()),
        ));
    }

    for extra in provided.difference(&expected) {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_UNEXPECTED_PARAM,
            phase,
            args([
                ("item", selection.item_ref.to_string()),
                ("key", extra.to_string()),
            ]),
            Some(selection.item_ref.clone()),
        ));
    }

    // Resolve values for domains that have a registry (Item -> point items,
    // Ability -> ability catalogue, Art -> art catalogue).
    for param in &item.parameters {
        let Some(value) = selection.params.get(&param.key) else {
            continue; // missing already reported above
        };
        if param_value_is_blank(value) {
            continue; // reported as `missing_param` above, not as an unprintable value
        }
        let resolves = param_value_resolves(ruleset, param, value);
        if !resolves {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNKNOWN_PARAM_VALUE,
                phase,
                args([
                    ("item", selection.item_ref.to_string()),
                    ("key", param.key.clone()),
                    ("value", value.to_string()),
                    ("domain", param.domain.to_string()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

/// Validates that every ability-bonus effect (e.g. Puissant Ability +2) targets
/// an ability instance the character actually holds. The target is
/// `(ability, parameter)`: for a parameterized ability ((Area) Lore) the instance
/// value is read from the selection's matching key, so Puissant "Brandenburg Lore"
/// must have a bought Brandenburg Lore row. A dangling target (e.g. the ability was
/// removed) means the +2 attaches to nothing, so flag it. Effect-driven — no virtue
/// id is hardcoded.
///
/// Filed on [`CreationPhase::Abilities`], the step that owns the fix, even though
/// the offending value is the Virtue's parameter. Puissant Ability is "choose one
/// Ability" (ArMDE:4814-4816) with no
/// requirement that a score already exists, and abilities are bought on a later
/// step — so filing this on `virtues_flaws` deadlocked the guided wizard, blocking
/// a step that could not offer the fix.
pub(crate) fn validate_ability_bonus_targets(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            // The exhaustive Effect match lives once, in effect_target (V71):
            // adding a variant is a compile error there, not here. Affinity
            // reduces the cost of buying one ability, so it too must target a
            // held instance — the cost break attaches to nothing otherwise,
            // exactly like a dangling Puissant — which is why both
            // `AbilityBonus` and `AffinityAbilityCost` collapse to the same
            // `EffectTarget::AbilityParam` arm.
            let param = match effect_target(effect) {
                EffectTarget::AbilityParam(param) => param,
                EffectTarget::CharacteristicLimit { .. } | EffectTarget::Other => continue,
            };
            let Some(target) = selection.params.get(param) else {
                continue; // missing ability key already reported by validate_parameters
            };
            // The instance discriminator, if the target ability is
            // parameterized. An effect's target is an Ability by construction
            // (`EffectTarget::AbilityParam`), so no domain test is needed here;
            // sharing the helper is what keeps this validator and
            // `validate_per_value_cap` composing one `(ability, instance)`
            // target rather than two spellings of it.
            let instance = ability_instance(ruleset, &selection.params, target);
            let has_instance = entity
                .ability_scores
                .iter()
                .any(|a| &a.ability == target && a.parameter.as_deref() == instance);
            if !has_instance {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_ABILITY_BONUS_DANGLING_TARGET,
                    // The Abilities step, not the V/F step that names the target —
                    // see this function's doc comment.
                    CreationPhase::Abilities,
                    args([
                        ("item", selection.item_ref.to_string()),
                        ("ability", target.to_string()),
                        ("parameter", instance.unwrap_or("").to_string()),
                    ]),
                    Some(selection.item_ref.clone()),
                ));
            }
        }
    }
}

/// Validates every [`ParameterDef::require_possessed`] target against what the
/// entity actually holds, and against the claims its other selections have
/// already staked.
///
/// False Power is the case the rules state: the Flaw is taken "once for each
/// appropriate Supernatural Virtue that the character possesses"
/// (ArMDE:6096). Two failures follow
/// from that one sentence, so both live here:
///
/// - **possesses** → [`ValidationIssue::CODE_PARAM_TARGET_NOT_POSSESSED`] when
///   the named Virtue is held by nobody. Possession is read straight off
///   [`PrereqCtx::present_ids`] — bought selections ++ granted rows, the very
///   set `Prereq::Has` consults, so a House-granted Supernatural Virtue counts
///   as possessed and the engine keeps exactly one notion of "held".
/// - **once for each** → [`ValidationIssue::CODE_PARAM_TARGET_ALREADY_CLAIMED`]
///   when a second selection names a Virtue an earlier one already claims. This
///   is the gap [`validate_duplicate_selections`] cannot close: its duplicate
///   key is `(item_ref, params)`, and `flaw.false_power` / `flaw.false_power_minor`
///   are different ids, so one Major and one Minor naming the same Virtue
///   collide in no key. A repeat of the SAME id is deliberately left to
///   `max_per_target`, or one mistake would draw two findings.
///
/// Filed on [`CreationPhase::VirtuesFlaws`], the step that raised it —
/// deliberately unlike [`validate_ability_bonus_targets`], which had to move to
/// [`CreationPhase::Abilities`]. The distinction is where the *fix* lives, not
/// where the value lives: Puissant Ability's target is bought on a later step,
/// so filing on V/F blocked a step that could not offer the remedy, whereas both
/// remedies here — name a different Virtue, or buy the one named — are on the
/// V/F step itself. Do not "align" this with its neighbour; that reintroduces
/// the wizard deadlock.
///
/// `selections` is the caller's folded bought-plus-granted list, for the same
/// reason [`validate_duplicate_selections`] takes it: a warping-owed Major Flaw
/// slot (`ArMDE:16561`) or an open House grant is filled with a player-chosen
/// [`Selection`], parameters and all, and such a copy claims its target exactly
/// as a bought one does.
pub(crate) fn validate_possessed_param_targets(
    selections: &[Selection],
    ruleset: &Ruleset,
    ctx: &PrereqCtx,
    issues: &mut Vec<ValidationIssue>,
) {
    // Which selection already claims each `(parameter key, target)` pair, in
    // selection order — so the FIRST claimant keeps the claim and only later
    // ones are reported.
    let mut claimed: BTreeMap<(&str, &Id), &Id> = BTreeMap::new();

    for selection in selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue; // unknown_ref already reported
        };
        for param in &item.parameters {
            if !param.require_possessed {
                continue;
            }
            let Some(value) = selection.params.get(&param.key) else {
                continue; // missing_param already reported
            };
            // A value that is not in the parameter's domain at all — unknown,
            // wrong category, Tainted — is already `unknown_param_value`, and
            // asking whether the character "holds" it on top of that would only
            // repeat one mistake.
            if param_value_is_blank(value) || !param_value_resolves(ruleset, param, value) {
                continue;
            }
            if !ctx.present_ids.contains(value) {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_PARAM_TARGET_NOT_POSSESSED,
                    CreationPhase::VirtuesFlaws,
                    args([
                        ("item", selection.item_ref.to_string()),
                        ("key", param.key.clone()),
                        ("value", value.to_string()),
                    ]),
                    Some(selection.item_ref.clone()),
                ));
                continue;
            }
            match claimed.entry((param.key.as_str(), value)) {
                std::collections::btree_map::Entry::Vacant(slot) => {
                    slot.insert(&selection.item_ref);
                }
                std::collections::btree_map::Entry::Occupied(slot) => {
                    let other = *slot.get();
                    if other == &selection.item_ref {
                        continue; // a same-id repeat is `max_per_target`'s finding
                    }
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_PARAM_TARGET_ALREADY_CLAIMED,
                        CreationPhase::VirtuesFlaws,
                        args([
                            ("item", selection.item_ref.to_string()),
                            ("key", param.key.clone()),
                            ("value", value.to_string()),
                            ("other", other.to_string()),
                        ]),
                        Some(selection.item_ref.clone()),
                    ));
                }
            }
        }
    }
}

/// Validates every [`ParameterDef::require_power`] target against the powers the
/// being actually holds.
///
/// Restricted Power, Slow Power and Variable Power each modify "one of the
/// character's supernatural powers" (ArMDE:6689, :6761, :5205). The target is free text because a power is an
/// anonymous *instance* of a Power Virtue rather than a catalogue entry — a
/// Greater Power's levels may be spent on "several powers" (`ArMDE:4021`) — so only
/// [`Entity::powers`] can say whether the named one exists. A name no power
/// carries is a Flaw attached to nothing, exactly as a dangling Puissant is.
///
/// Filed on [`CreationPhase::Review`], the phase that owns "Might and powers",
/// following [`validate_ability_bonus_targets`]'s rule: the finding goes where
/// the *fix* is, not where the offending value is. The remedy — add the power, or
/// correct its spelling — is on the Review step, and filing it on the V/F step
/// that raised it would block a step which cannot offer that remedy. The names on
/// both sides are already trimmed (`Entity::normalize` trims parameter values;
/// this compares against a trimmed power name), and case is deliberately not
/// folded, matching the engine's one existing decision about free-text identity.
///
/// `selections` is the caller's folded bought-plus-granted list, for the same
/// reason [`validate_possessed_param_targets`] takes it: a warping-owed Major
/// Flaw slot or an open House grant is filled with a player-chosen [`Selection`],
/// parameters and all, and such a copy names a power exactly as a bought one does.
pub(crate) fn validate_power_targets(
    entity: &Entity,
    selections: &[Selection],
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    for selection in selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue; // unknown_ref already reported
        };
        for param in &item.parameters {
            if !param.require_power {
                continue;
            }
            let Some(value) = selection.params.get(&param.key) else {
                continue; // missing_param already reported
            };
            // A blank descriptor is `missing_param`'s finding; asking which power
            // it names on top of that would only repeat one mistake.
            if param_value_is_blank(value) {
                continue;
            }
            if entity
                .powers
                .iter()
                .any(|power| power.name.trim() == value.as_str())
            {
                continue;
            }
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_POWER_DANGLING_TARGET,
                CreationPhase::Review,
                args([
                    ("item", selection.item_ref.to_string()),
                    ("key", param.key.clone()),
                    ("power", value.to_string()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

/// Enforces the "one Magical Focus per magus" limit (ArMDE:4542) by
/// counting [`Effect::MagicalFocus`] across everything that feeds the effective
/// layer (bought selections plus House / Mythic-type grants, e.g. Mythic Blood's
/// bundled Minor Focus). More than one Focus is illegal. This counts the *effect*
/// rather than using pairwise `incompatible_with`, so it also catches two Minor
/// Foci with different descriptors (distinct selections that no incompatibility
/// pair would flag). Effect-driven — no virtue id is hardcoded.
///
/// `selections` is the caller's already-folded bought-plus-granted list
/// ([`crate::effective::selections_for_effects`]), computed once in
/// [`super::validate`] and shared with the other grant-aware sub-validators
/// rather than re-resolved here.
pub(crate) fn validate_magical_focus(
    selections: &[Selection],
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut foci = 0usize;
    for selection in selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        foci += item
            .effects
            .iter()
            .filter(|e| matches!(e, Effect::MagicalFocus { .. }))
            .count();
    }
    if foci > 1 {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_MULTIPLE_MAGICAL_FOCI,
            CreationPhase::VirtuesFlaws,
            args([("count", foci.to_string())]),
            None,
        ));
    }
}

/// Enforces the type's Gift policy (required / allowed / forbidden). The policy
/// per type is data; this is the mechanism the book's Gift rules map onto.
///
/// Source: ArMDE:2868-2877 (The Gift:
/// "all magi must have this Virtue"; "Grogs can never have The Gift"); magi must
/// take The Gift at :2858; only magi may take the Hermetic Magus Social Status
/// at :2293 and :4067-4069.
pub(crate) fn validate_gift_policy(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    let Some(policy) = profile.gift_policy else {
        return;
    };

    // Shared with the Supernatural free-slot computation so both use one
    // definition of "has The Gift".
    let has_gift = crate::effective::has_the_gift(entity, ruleset, profile);

    match policy {
        GiftPolicy::Required => {
            if !has_gift {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_GIFT_REQUIRED,
                    CreationPhase::VirtuesFlaws,
                    BTreeMap::new(),
                    None,
                ));
            }
        }
        GiftPolicy::Forbidden => {
            if has_gift {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_GIFT_FORBIDDEN,
                    CreationPhase::VirtuesFlaws,
                    BTreeMap::new(),
                    None,
                ));
            }
        }
        GiftPolicy::Allowed => {}
    }
}
