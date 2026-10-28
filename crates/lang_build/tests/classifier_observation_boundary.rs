mod support;

use lang_build::{
    extract_single_call_site, OrdinaryInvocationContext, OrdinaryInvocationFailure, PolicyMode,
    Provenance, RawArgValueClass,
};

#[test]
fn unknown_classifier_keeps_real_competing_candidates_unselected_in_either_order() {
    let generic = "let generic = (self, _): compile -> let r:uint8:: => { self; };";
    let mut generic_only = support::AssociatedFamily::new(&[generic]);
    let call = extract_single_call_site(&support::initializer_from_source(
        "let result = missing choose;",
    ))
    .unwrap();
    let failure = generic_only
        .invoke_ordinary_call(
            generic_only.package_root_node(),
            &call,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new("unconstrained candidate control"),
        )
        .unwrap_err();
    let trace = match failure {
        OrdinaryInvocationFailure::ArgumentNormalization { trace, .. }
        | OrdinaryInvocationFailure::SelectedBody { trace, .. }
        | OrdinaryInvocationFailure::SelectedImplementation { trace, .. } => trace,
        other => panic!("the unconstrained candidate must pass first-round selection: {other:?}"),
    };
    assert_eq!(trace.a_fully_admissible.len(), 1);
    assert!(trace.selected.is_some());
    for constrained in [
        "let constrained = <T>(self, let <> T): compile -> let r:uint8:: => { self; };",
        "let constrained = (self, _ | uint8): compile -> let r:uint8:: => { self; };",
        "let constrained = (self, _ uint8): compile -> let r:uint8:: => { self; };",
        "let constrained = (self, x:type::): compile -> let r:uint8:: => { self; };",
        "let constrained = <T>(self, x:T): compile -> let r:uint8:: => { self; };",
    ] {
        for generic_first in [false, true] {
            let sources = if generic_first {
                [generic, constrained]
            } else {
                [constrained, generic]
            };
            let mut world = support::AssociatedFamily::new(&sources);
            let call = extract_single_call_site(&support::initializer_from_source(
                "let result = missing choose;",
            ))
            .unwrap();
            let raw = call.to_arg_product_shape();
            assert!(matches!(
                raw.raw_args[0].value_class,
                RawArgValueClass::UnknownExpression
            ));
            let transported =
                lang_build::overload_args_from_classified_shape(&raw, |_| None, |_| None);
            assert_eq!(
                transported[0].value_class,
                RawArgValueClass::UnknownExpression
            );
            assert!(transported[0].pattern_coordinate.is_none());
            let before = format!("{:?}", world.semantic_world());
            let failure = world
                .invoke_ordinary_call(
                    world.package_root_node(),
                    &call,
                    OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
                    Provenance::new("unknown classifier with a real competing candidate"),
                )
                .unwrap_err();
            let OrdinaryInvocationFailure::ApplicabilityUnsupported { diagnostic, trace } = failure
            else {
                panic!("{constrained}, generic_first={generic_first}: {failure:?}");
            };
            assert!(
                diagnostic.message.contains("observation")
                    && diagnostic.message.contains("unavailable"),
                "{diagnostic:?}"
            );
            assert_eq!(trace.c3_call_entries.len(), 2);
            assert!(trace.bp_prime.is_empty());
            assert!(trace.selected.is_none());
            assert!(trace.compile_instance.is_none());
            assert_eq!(format!("{:?}", world.semantic_world()), before);
        }
    }
}

#[test]
fn known_classifier_mismatch_still_removes_only_the_decided_candidate() {
    let mut world = support::AssociatedFamily::new(&[
        "let constrained = (self, _ | uint8): compile -> let r:uint8:: => { self; };",
        "let generic = (self, _): compile -> let r:uint8:: => { self; };",
    ]);
    let call = extract_single_call_site(&support::initializer_from_source(
        "let result = uint16 choose;",
    ))
    .unwrap();
    let before = format!("{:?}", world.semantic_world());
    let failure = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new("known mismatch"),
        )
        .unwrap_err();
    let OrdinaryInvocationFailure::SelectedBody { trace, .. } = failure else {
        panic!("{failure:?}");
    };
    assert_eq!(trace.c3_call_entries.len(), 2);
    assert_eq!(trace.a_fully_admissible.len(), 1);
    assert!(trace.selected.is_some());
    assert_eq!(format!("{:?}", world.semantic_world()), before);
}

#[test]
fn missing_named_observation_cannot_be_erased_after_an_alternative_matches() {
    let generic = "let generic = (self, _): compile -> let r:uint8:: => { self; };";
    let constrained = "let constrained = (self, _ | uint8 | UnobservedPattern): compile -> let r:uint8:: => { self; };";
    for sources in [[constrained, generic], [generic, constrained]] {
        let mut world = support::AssociatedFamily::new(&sources);
        let call = extract_single_call_site(&support::initializer_from_source(
            "let result=uint8 choose;",
        ))
        .unwrap();
        let before = format!("{:?}", world.semantic_world());
        let failure = world
            .invoke_ordinary_call(
                world.package_root_node(),
                &call,
                OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
                Provenance::new("missing expected Pattern observation"),
            )
            .unwrap_err();
        let OrdinaryInvocationFailure::ApplicabilityUnsupported { diagnostic, trace } = failure
        else {
            panic!("{failure:?}");
        };
        assert!(diagnostic
            .message
            .contains("named Pattern observation is unavailable"));
        assert_eq!(trace.c3_call_entries.len(), 2);
        assert!(trace.bp_prime.is_empty());
        assert!(trace.selected.is_none());
        assert!(trace.compile_instance.is_none());
        assert_eq!(format!("{:?}", world.semantic_world()), before);
    }
}
