mod support;

use lang_build::{
    canonical_function_object_view, declared_policy_view, expose_policy_slice,
    extract_single_call_site, ExplicitP1Selection, ObservationHorizon, OrdinaryInvocationContext,
    OrdinaryInvocationFailure, PolicyMode, PolicyResultEntry, Provenance, Stage,
    ValueComponentPolicy, ValuePresence,
};
use support::{initializer_from_source, AssociatedFamily};

#[test]
fn fixed_callee_identity_precedes_shared_horizon_observations() {
    use lang_build::{
        prepare_callable_candidate_with_declared_planes, ArgProductShape,
        CandidatePrepIncompleteReason, CandidatePrepResult, CandidatePreparationContext,
        CompilationWorld, FlattenedProductInvariant, FlattenedProductMaterial,
        InvocationCallableRef, InvocationFrame, NamespaceGraphSymbol, ParameterShape, SelfPosition,
        SemanticValueId, SourceCategory, SymbolKind,
    };

    let world = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let mut delta = world.namespace_projection().empty_delta();
    let mut symbol = NamespaceGraphSymbol::new(
        delta.allocate_symbol_id(),
        "candidate",
        SymbolKind::Object,
        SourceCategory::DeclaredSymbol,
        Some(world.package_root_node()),
        Provenance::new("horizon fixture"),
    );
    // Callee exposure and body entry observe different facts after resolution.
    symbol.policy_view = Some(declared_policy_view(Stage::Compile, PolicyMode::Const));
    delta.insert_symbol(world.package_root_node(), symbol);
    let snapshot = world.namespace_projection().install_delta(delta).unwrap();
    let capability = snapshot.capability();
    let path = ["candidate".to_string()];
    let resolver = world.package_context();
    let callee = capability.resolve(&path, &resolver).unwrap();
    let body = declared_policy_view(Stage::Seal, PolicyMode::Const);
    let result = declared_policy_view(Stage::Runtime, PolicyMode::Const);

    for (horizon, callee_value_visible, body_visible) in [
        (ObservationHorizon::OpenStatic, true, false),
        (ObservationHorizon::SealStatic, true, true),
        (ObservationHorizon::Runtime, false, false),
    ] {
        let callee_entry = PolicyResultEntry {
            value: Some(callee.id),
            pattern: (),
            view: callee.policy_view.as_ref().unwrap().clone(),
        };
        let observed = expose_policy_slice(&callee_entry, horizon);
        assert_eq!(
            lang_build::read_value(&observed).is_some(),
            callee_value_visible
        );
        assert_eq!(
            lang_build::body_entry_visible_at(&body.pair, horizon),
            body_visible
        );
        assert_eq!(capability.resolve(&path, &resolver).unwrap().id, callee.id);
        let args = ArgProductShape::from_flattened(FlattenedProductMaterial {
            atoms: Vec::new(),
            provenance: Provenance::new("empty args"),
            invariant: FlattenedProductInvariant {
                no_direct_product_atom_remains: true,
            },
        });
        let prepared = prepare_callable_candidate_with_declared_planes(
            callee.id,
            &callee.name,
            callee.policy_view.clone().unwrap(),
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
            CandidatePrepResult::Incomplete { candidate, reason } => {
                assert!(!body_visible);
                assert_eq!(
                    reason,
                    CandidatePrepIncompleteReason::BodyEntryObservationHidden
                );
                candidate
            }
            other => panic!("{other:?}"),
        };
        assert_eq!(candidate.policy_planes.horizon, horizon);
        assert_eq!(
            candidate.policy_planes.callable_view,
            callee.policy_view.clone().unwrap()
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
    use lang_build::FacetView;
    let entry = PolicyResultEntry {
        value: Some(7),
        pattern: 11,
        view: declared_policy_view(Stage::Runtime, PolicyMode::Const),
    };
    let hidden = expose_policy_slice(&entry, ObservationHorizon::OpenStatic);
    assert_eq!(hidden.value, FacetView::HiddenAtHorizon(7));
    assert!(lang_build::read_value(&hidden).is_none());
    assert_eq!(lang_build::read_pattern(&hidden), Some(&11));
    assert_eq!(hidden.value_policy, entry.view.pair.value);
    assert_eq!(hidden.pattern_policy, entry.view.pair.pattern);
    assert_eq!(hidden.mode, entry.view.mode);
    let runtime = expose_policy_slice(&entry, ObservationHorizon::Runtime);
    assert_eq!(runtime.pattern, FacetView::HiddenAtHorizon(11));
    assert!(lang_build::read_pattern(&runtime).is_none());
    assert_eq!(lang_build::read_value(&runtime), Some(&7));
    assert_eq!(runtime.value_policy, hidden.value_policy);

    let mut other = entry.clone();
    other.value = Some(8);
    let other_hidden = expose_policy_slice(&other, ObservationHorizon::OpenStatic);
    assert_eq!(other_hidden.value, FacetView::HiddenAtHorizon(8));
    assert_ne!(
        hidden.value, other_hidden.value,
        "hiding preserves value identity"
    );
    other.pattern = 12;
    let other_runtime = expose_policy_slice(&other, ObservationHorizon::Runtime);
    assert_ne!(
        runtime.pattern, other_runtime.pattern,
        "hiding preserves Pattern identity"
    );
    assert_eq!(other_runtime.pattern, FacetView::HiddenAtHorizon(12));
    assert!(lang_build::read_pattern(&other_runtime).is_none());

    other.value = None;
    other.view.pair.value = ValueComponentPolicy::Absent;
    let absent = expose_policy_slice(&other, ObservationHorizon::OpenStatic);
    assert_eq!(absent.value, FacetView::Absent);
    assert_ne!(hidden.value, absent.value);
    assert!(
        lang_build::enumerate_value_facet(&[hidden, other_hidden, absent])
            .next()
            .is_none()
    );
}

#[test]
fn explicit_absence_cannot_silently_drop_an_explicit_stage() {
    let derived = declared_policy_view(Stage::Compile, PolicyMode::Const);
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
fn explicit_pin_stage_stops_selection_until_input_admissibility_is_connected() {
    let call = extract_single_call_site(&initializer_from_source("let result = () f;")).unwrap();
    for stage in ["meta", "compile", "seal", "runtime", "const + compile"] {
        let constrained =
            format!("let constrained = (self, {stage} let x):runtime -> let r => {{ x; }};");
        let ordinary = "let ordinary = (self, let x):runtime -> let r => { x; };";
        for sources in [
            [constrained.as_str(), ordinary],
            [ordinary, constrained.as_str()],
        ] {
            let mut family = AssociatedFamily::new(&sources);
            let mut context = OrdinaryInvocationContext::open_static(&[]);
            context.horizon = ObservationHorizon::Runtime;
            let outcome = family.invoke_ordinary_call(
                family.package_root_node(),
                &call,
                context,
                Provenance::new("explicit Pin stage consumer boundary"),
            );
            let Err(OrdinaryInvocationFailure::ApplicabilityUnsupported { diagnostic, trace }) =
                outcome
            else {
                panic!(
                    "stage-constrained Pin must not be ignored or treated as illegal: {outcome:?}"
                );
            };
            assert_eq!(
                diagnostic.code,
                Some(lang_build::ResolverCode::UnsupportedInputAdmissibleStage)
            );
            assert!(diagnostic.message.contains("is canonical"));
            assert!(trace.bp_prime.is_empty());
            assert!(trace.selected.is_none());
            assert!(trace.dynamic_legality.is_none());
        }
    }
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

#[test]
fn hidden_body_observation_is_an_explicit_invocation_frontier() {
    // P1 is visible; the actual declared P2 observation is hidden.
    let mut family = AssociatedFamily::new(&["compile let f = (receiver, x):seal => { (); };"]);
    let call = extract_single_call_site(&initializer_from_source("let result = () f;")).unwrap();
    let outcome = family.invoke_ordinary_call(
        family.package_root_node(),
        &call,
        OrdinaryInvocationContext::open_static(&[]),
        Provenance::new("visible callable, hidden body"),
    );
    assert!(
        matches!(outcome, Err(OrdinaryInvocationFailure::Residual { residual, .. })
        if residual.class == "hidden-body-entry-observation")
    );
}

#[test]
fn hidden_body_entries_do_not_override_visible_selection_or_failure() {
    let visible = "let f = (receiver, x):meta => (\"visible rejection\") delete;";
    let hidden = "compile let f = (receiver, x):seal => { (); };";
    let call = extract_single_call_site(&initializer_from_source("let result = () f;")).unwrap();
    for sources in [
        vec![visible, hidden],
        vec![hidden, visible],
        vec![visible, hidden, visible],
    ] {
        let ambiguous = sources.len() == 3;
        let mut family = AssociatedFamily::new(&sources);
        let outcome = family.invoke_ordinary_call(
            family.package_root_node(),
            &call,
            OrdinaryInvocationContext::open_static(&[]),
            Provenance::new("mixed body observations"),
        );
        if ambiguous {
            assert!(
                matches!(outcome, Err(OrdinaryInvocationFailure::Ambiguous { .. })),
                "{outcome:?}"
            );
        } else {
            assert!(
                matches!(
                    outcome,
                    Err(OrdinaryInvocationFailure::SelectedDelete { .. })
                ),
                "{outcome:?}"
            );
        }
    }
}

#[test]
fn hidden_body_does_not_override_a_reached_applicability_diagnostic() {
    let visible = "let f = (receiver, x:type):meta => { (); };";
    let hidden = "compile let f = (receiver, x):seal => { (); };";
    let call = extract_single_call_site(&initializer_from_source("let result = () f;")).unwrap();
    for sources in [[visible, hidden], [hidden, visible]] {
        let mut family = AssociatedFamily::new(&sources);
        let outcome = family.invoke_ordinary_call(
            family.package_root_node(),
            &call,
            OrdinaryInvocationContext::open_static(&[]),
            Provenance::new("mixed applicability and body observation"),
        );
        assert!(
            matches!(
                outcome,
                Err(OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
                    first_diagnostic: Some(_),
                    ..
                })
            ),
            "{outcome:?}"
        );
    }
}

#[test]
fn hidden_callee_in_a_type_union_does_not_override_visible_selected_failure() {
    let base = lang_build::CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let call =
        extract_single_call_site(&initializer_from_source("let result = () uint8;")).unwrap();
    for hidden_first in [false, true] {
        let mut family = AssociatedFamily::new(&[
            "let f = (receiver, x):meta => (\"visible rejection\") delete;",
        ]);
        let visible = family.target_binding().ordinary_value().unwrap();
        let world = family.semantic_world_mut();
        let classifier = world.value(visible).unwrap().type_value;
        let hidden = world
            .install_plain_value(
                classifier,
                declared_policy_view(Stage::Seal, PolicyMode::Const).pair,
                Provenance::new("hidden receiver"),
            )
            .unwrap();
        let target = world
            .symbol_in_namespace(base.core_node(), "uint8")
            .unwrap()
            .identity;
        let pattern = world.symbol(target).unwrap().pure_p_pattern().unwrap();
        let receivers = if hidden_first {
            [hidden, visible]
        } else {
            [visible, hidden]
        };
        for (index, receiver) in receivers.into_iter().enumerate() {
            world
                .admit_direct_type_member(pattern, pattern, &format!("member{index}"), receiver)
                .unwrap();
        }
        let failure = lang_build::invoke_resolved_binding_ordinary(
            world,
            &[],
            target,
            &call,
            &base.root_context(),
            OrdinaryInvocationContext::open_static(&[]),
            Provenance::new("mixed type-call observations"),
        )
        .unwrap_err();
        let OrdinaryInvocationFailure::SelectedDelete { trace, .. } = failure else {
            panic!("{failure:?}");
        };
        assert!(trace.c0_target_values.contains(&hidden));
        assert!(trace.c0_target_values.contains(&visible));
        assert_eq!(trace.c2_horizon_values, vec![visible]);
        assert_eq!(trace.a_fully_admissible.len(), 1);
    }
}
