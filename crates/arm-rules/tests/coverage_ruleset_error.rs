//! Coverage slice (D76 follow-up): `RulesetError`'s custom `Deserialize`
//! mirrors its custom `Serialize` map shape (`{kind, source, message}` for
//! `Parse`, `{kind, errors}` for `Integrity`) and matches on the `kind`
//! string by hand — every existing round-trip test only ever serializes a
//! real `RulesetError` and deserializes it back, so `kind`'s only ever
//! "parse" or "integrity". The `other => Err(...)` arm, reached when `kind`
//! names neither, had no test at all.

use arm_rules::ruleset::RulesetError;

#[test]
fn deserializing_an_unknown_kind_fails_naming_the_offending_value() {
    let err = serde_json::from_str::<RulesetError>(r#"{"kind": "bogus"}"#)
        .expect_err("an unrecognized `kind` must not deserialize to any variant");
    assert!(
        err.to_string().contains("bogus"),
        "the deserialize error should name the offending kind, got: {err}"
    );
}
