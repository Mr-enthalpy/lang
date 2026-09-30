mod support;

use lang_build::{
    canonical_function_object_view, declared_policy_view, expose_policy_slice,
    extract_single_call_site, ExplicitP1Selection, ObservationHorizon, OrdinaryInvocationContext,
    OrdinaryInvocationFailure, PolicyMode, PolicyResultEntry, Provenance, Stage,
    ValueComponentPolicy, ValuePresence,
};
use support::{initializer_from_source, AssociatedFamily};

#[test]
fn one_horizon_reaches_lookup_preparation_and_invocation_frame() {
    use lang_build::{
        prepare_meta_callable_candidate_with_declared_planes, ArgProductShape,
        CallableCandidateKind, CandidatePrepDeferredReason, CandidatePrepResult,
        CandidatePreparationContext, CompilationWorld, FlattenedProductInvariant,
        FlattenedProductObject, InvocationCallableRef, InvocationFrame, ParameterShape,
        ResolveExpectation, SelfPosition, SemanticValueId, SourceCategory, SymbolKind,
        SymbolObject,
    };

    let world = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let mut delta = world.namespace_projection().empty_delta();
    let mut symbol = SymbolObject::new(
        delta.allocate_symbol_id(),
        "candidate",
        SymbolKind::Object,
        SourceCategory::DeclaredSymbol,
        Some(world.package_root_node()),
        Provenance::new("horizon fixture"),
    );
    // Lookup and body entry observe different declared facts at the same horizon.
    symbol.policy_view = Some(declared_policy_view(Stage::Compile, PolicyMode::Plain));
    delta.insert_symbol(world.package_root_node(), symbol);
    let snapshot = world.namespace_projection().install_delta(delta).unwrap();
    let capability = snapshot.capability();
    let path = ["candidate".to_string()];
    let resolver = world.package_context();
    let callee = capability.resolve(&path, &resolver).unwrap();
    let body = declared_policy_view(Stage::Seal, PolicyMode::Plain);
    let result = declared_policy_view(Stage::Runtime, PolicyMode::Const);

    for (horizon, lookup_visible, body_visible) in [
        (ObservationHorizon::OpenStatic, true, false),
        (ObservationHorizon::SealStatic, true, true),
        (ObservationHorizon::Runtime, false, false),
    ] {
        assert_eq!(
            capability
                .resolve_with_policy(&path, &resolver, ResolveExpectation::Object, horizon)
                .is_ok(),
            lookup_visible,
        );
        let args = ArgProductShape::from_flattened(FlattenedProductObject {
            atoms: Vec::new(),
            provenance: Provenance::new("empty args"),
            invariant: FlattenedProductInvariant {
                no_direct_product_atom_remains: true,
            },
        });
        let prepared = prepare_meta_callable_candidate_with_declared_planes(
            &callee,
            CallableCandidateKind::MetaFunction,
            None,
            body.clone(),
            result.clone(),
            args.clone(),
            ParameterShape::exact_arity(0, Provenance::new("zero arguments")),
            CandidatePreparationContext {
                horizon,
                provenance: Provenance::new("shared horizon"),
            },
        );
        let candidate = match prepared {
            CandidatePrepResult::Applicable(candidate) => {
                assert!(body_visible);
                candidate
            }
            CandidatePrepResult::Deferred { candidate, reason } => {
                assert!(!body_visible);
                assert_eq!(reason, CandidatePrepDeferredReason::BodyEntryPolicyMismatch);
                candidate
            }
            other => panic!("{other:?}"),
        };
        assert_eq!(candidate.policy_planes.horizon, horizon);
        assert_eq!(
            candidate.policy_planes.symbol_policy_view,
            callee.policy_view
        );
        assert_eq!(candidate.policy_planes.body_entry_policy, body);
        assert_eq!(candidate.policy_planes.return_object_policy, result);
        // Frame transport is checked independently of executing a pending body.
        let frame = InvocationFrame::new(
            InvocationCallableRef::SemanticValue(SemanticValueId(7)),
            SelfPosition::primitive_core_object(Provenance::new("fixture self")),
            args,
            candidate.policy_planes.horizon,
            Provenance::new("frame transport"),
        )
        .unwrap();
        assert_eq!(frame.horizon, horizon);
    }
}

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
