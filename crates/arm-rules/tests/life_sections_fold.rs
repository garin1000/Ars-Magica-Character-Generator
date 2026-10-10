//! G3: switching the life sections off folds every section's lab years into the
//! plan level ([`LifeStagePlan::disable_life_sections`]). The frontend store makes
//! the same edit in TypeScript, so both sides are pinned to one shared fixture,
//! `tests/fixtures/life_sections_fold.json`: this file proves the engine produces
//! each case's `after` from its `before`, and `ui/src/lib/state.svelte.test.ts`
//! proves the store does too. A drift on either side fails its own suite.

use arm_rules::LifeStagePlan;
use serde_json::Value;

fn cases() -> Vec<Value> {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/life_sections_fold.json")).unwrap();
    fixture["cases"]
        .as_array()
        .expect("the fixture lists its cases")
        .clone()
}

#[test]
fn the_shared_fold_fixture_has_cases() {
    assert!(
        cases().len() >= 3,
        "a fixture that lost its cases proves nothing"
    );
}

#[test]
fn disabling_life_sections_turns_each_fixture_case_into_its_after() {
    for case in cases() {
        let name = case["name"].as_str().unwrap_or("?");
        let mut plan: LifeStagePlan = serde_json::from_value(case["before"].clone())
            .unwrap_or_else(|error| panic!("{name}: `before` is a plan: {error}"));

        plan.disable_life_sections();

        assert_eq!(
            serde_json::to_value(&plan).unwrap(),
            case["after"],
            "{name}"
        );
    }
}
