mod support;

use lang_build::{
    canonical_function_object_view, declared_policy_view, expose_policy_slice,
    extract_single_call_site, ExplicitP1Selection, ObservationHorizon, OrdinaryInvocationContext,
    OrdinaryInvocationFailure, PolicyMode, PolicyResultEntry, Provenance, Stage,
    ValueComponentPolicy, ValuePresence,
};
use support::{initializer_from_source, AssociatedFamily};

#[test]
fn hidden_facets_retain_the_same_resolved_observation() {
    let entry = PolicyResultEntry {
        value: Some(7),
        pattern: 11,
        view: declared_policy_view(Stage::Runtime, PolicyMode::Const),
    };
    let hidden = expose_policy_slice(&entry, ObservationHorizon::OpenStatic);
    assert!(lang_build::read_value(&hidden).is_none());
    assert_eq!(lang_build::read_pattern(&hidden), Some(&11));
    assert_eq!(hidden.value_policy, entry.view.pair.value);
    assert_eq!(hidden.pattern_policy, entry.view.pair.pattern);
    assert_eq!(hidden.mode, entry.view.mode);
    let runtime = expose_policy_slice(&entry, ObservationHorizon::Runtime);
    assert_eq!(lang_build::read_value(&runtime), Some(&7));
    assert_eq!(runtime.value_policy, hidden.value_policy);
}

#[test]
fn explicit_absence_cannot_silently_drop_an_explicit_stage() {
    let derived = declared_policy_view(Stage::Compile, PolicyMode::Plain);
    let selection = ExplicitP1Selection {
        presence: Some(ValuePresence::Absent),
        value_stage: Some(Stage::Compile),
        ..Default::default()
    };
    assert!(canonical_function_object_view(
        Some(&selection),
        &derived,
        &derived,
        None,
        &Provenance::new("contradictory query"),
    )
    .is_err());
    assert_eq!(ValueComponentPolicy::Absent.stage(), None);
}

#[test]
fn ordinary_pipeline_does_not_rank_static_stage_atoms() {
    // Explicit substrate material; this does not evaluate source closures.
    let cases = [
        (
            "let f = (self, x):meta => { (); };",
            "let f = (self, x):compile => { (); };",
            ObservationHorizon::OpenStatic,
        ),
        (
            "let f = (self, x):compile => { (); };",
            "let f = (self, x):seal => { (); };",
            ObservationHorizon::SealStatic,
        ),
    ];
    let expr = initializer_from_source("let result = () f;");
    let call = extract_single_call_site(&expr).unwrap();
    for (left, right, horizon) in cases {
        let mut family = AssociatedFamily::new(&[left, right]);
        let mut context = OrdinaryInvocationContext::open_static(&[]);
        context.horizon = horizon;
        if horizon == ObservationHorizon::SealStatic {
            context.execution_env = lang_build::ExecutionEnv::SealStatic;
            context.policy_env = lang_build::PolicyEnv::SealStatic;
        }
        let outcome = family.invoke_ordinary_call(
            family.package_root_node(),
            &call,
            context,
            Provenance::new("single-stage maxima"),
        );
        assert!(
            matches!(outcome, Err(OrdinaryInvocationFailure::Ambiguous { .. })),
            "{outcome:?}"
        );
    }
}
