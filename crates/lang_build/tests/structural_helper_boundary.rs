//! Complete type input and selected helper publication boundaries.

mod support;

use lang_build::{
    extract_single_call_site, CompilationWorld, ObservationHorizon, OrdinaryInvocationContext,
    OrdinaryInvocationFailure, Provenance,
};

#[test]
fn selected_struct_helper_frontier_publishes_nothing_at_each_static_horizon() {
    for horizon in [
        ObservationHorizon::OpenStatic,
        ObservationHorizon::SealStatic,
    ] {
        let mut world = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
        let site = extract_single_call_site(&support::initializer_from_source(
            "let result = uint8 struct::core;",
        ))
        .unwrap();
        let before = format!("{:?}", world.semantic_world());
        let mut context = OrdinaryInvocationContext::open_static(&[]);
        context.horizon = horizon;
        let failure = world
            .invoke_ordinary_call(
                world.package_root_node(),
                &site,
                context,
                Provenance::new("helper publication"),
            )
            .unwrap_err();
        let OrdinaryInvocationFailure::SelectedImplementation { diagnostic, trace } = failure
        else {
            panic!("complete input reaches the selected helper: {failure:?}");
        };
        assert!(diagnostic
            .message
            .contains("struct helper formation consumer is unavailable"));
        assert!(trace.selected.is_some());
        assert!(trace.compile_instance.is_some());
        assert_eq!(format!("{:?}", world.semantic_world()), before);
    }
}

#[test]
fn struct_requires_one_complete_type_argument_before_selection() {
    for input in ["()", "(uint8, uint16)"] {
        let mut world = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
        let site = extract_single_call_site(&support::initializer_from_source(&format!(
            "let result = {input} struct::core;",
        )))
        .unwrap();
        let before = format!("{:?}", world.semantic_world());
        let failure = world
            .invoke_ordinary_call(
                world.package_root_node(),
                &site,
                OrdinaryInvocationContext::open_static(&[]),
                Provenance::new("type input contract"),
            )
            .unwrap_err();
        let OrdinaryInvocationFailure::NoFullyAdmissibleCandidate { trace, .. } = failure else {
            panic!("invalid input fails applicability: {failure:?}");
        };
        assert!(trace.selected.is_none());
        assert!(trace.compile_instance.is_none());
        assert_eq!(format!("{:?}", world.semantic_world()), before);
    }
}

#[test]
fn unobserved_type_argument_stops_selection_without_publication() {
    let mut world = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let site =
        extract_single_call_site(&support::initializer_from_source("let T = 0 struct::core;"))
            .unwrap();
    let before = format!("{:?}", world.semantic_world());
    let failure = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &site,
            OrdinaryInvocationContext::open_static(&[]),
            Provenance::new("unobserved input"),
        )
        .unwrap_err();
    let OrdinaryInvocationFailure::ApplicabilityUnsupported { diagnostic, trace } = failure else {
        panic!("{failure:?}");
    };
    assert!(diagnostic
        .message
        .contains("parameter shape compatibility is not established"));
    assert!(trace.selected.is_none());
    assert!(trace.compile_instance.is_none());
    assert_eq!(format!("{:?}", world.semantic_world()), before);
}
