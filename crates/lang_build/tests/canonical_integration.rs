//! Canonical semantic-spine integration tests.
//!
//! These tests pin the canonical invariants end to end:
//! one resolved binding per call target, member views as the canonical
//! fact (never a flat Symbol/Policy aggregate), declaration-time return
//! ontology shared by core and source, ordinary let-binding of compile outcomes
//! (bind the RHS value to the LHS symbol — types have value semantics too),
//! and the single canonical P1 authority chain.

mod support;

use lang_build::{
    extract_single_call_site, BuildManifest, CompilationWorld, InvocationOutcome,
    OrdinaryInvocationContext, OrdinaryInvocationFailure, OrdinaryPipelineTrace, PolicyMode,
    Provenance, ResolverCode, SemanticValuePayload,
};

use support::{build_fixture_error, build_single_fixture_world, initializer_from_source};

fn family_with_const_actual(
    stage: lang_build::Stage,
) -> (
    support::AssociatedFamily,
    lang_build::SemanticSymbolIdentity,
) {
    use lang_build::{declared_policy_view, PolicyResultEntry, SemanticValueRef};

    let mut world = support::AssociatedFamily::new(&[
        "let first = (self, const let x): compile -> let r => { x; };",
        "let second = (self, mut let x): compile -> let r => { x; };",
    ]);
    let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let pattern = world
        .semantic_world()
        .symbol_in_namespace(base.core_node(), "uint8")
        .unwrap()
        .pure_p_pattern()
        .unwrap();
    let type_value = world.semantic_world().type_for_pattern(pattern).unwrap();
    let view = declared_policy_view(stage, PolicyMode::Const);
    let value = world
        .semantic_world_mut()
        .install_plain_value(
            type_value,
            view.pair.clone(),
            Provenance::new("actual material"),
        )
        .unwrap();
    let namespace = base.root_context().current_namespace;
    let binding = world
        .semantic_world_mut()
        .bind_ordinary_new(
            namespace,
            "a",
            &[PolicyResultEntry {
                value: Some(SemanticValueRef {
                    id: value,
                    type_value,
                }),
                pattern,
                view,
            }],
            None,
            Provenance::new("const actual observation"),
        )
        .unwrap();
    (world, binding)
}

#[test]
fn visible_noncallable_target_with_hidden_argument_has_no_candidate() {
    use lang_build::{declared_policy_view, PolicyResultEntry, SemanticValueRef, Stage};
    let (mut world, hidden) = family_with_const_actual(Stage::Runtime);
    assert!(world.semantic_world().symbol(hidden).is_some());
    let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let type_value = support::type_lookup_fixture("visible-noncallable-target");
    let rank = world.semantic_world().type_rank().unwrap();
    let view = declared_policy_view(Stage::Compile, PolicyMode::Const);
    let (_, _, pattern) = world
        .semantic_world_mut()
        .register_type_symbol(
            base.root_context().current_namespace,
            "noncallable_type",
            lang_build::SymbolId(900000),
            type_value,
            rank,
            None,
            view.pair.clone(),
            Provenance::new("noncallable classifier fixture"),
        )
        .unwrap();
    let value = world
        .semantic_world_mut()
        .install_plain_value(
            type_value,
            view.pair.clone(),
            Provenance::new("visible noncallable"),
        )
        .unwrap();
    assert!(world
        .semantic_world()
        .callable_entries_for_value(value)
        .is_empty());
    let noncallable = world
        .semantic_world_mut()
        .bind_ordinary_new(
            base.root_context().current_namespace,
            "noncallable",
            &[PolicyResultEntry {
                value: Some(SemanticValueRef {
                    id: value,
                    type_value,
                }),
                pattern,
                view,
            }],
            None,
            Provenance::new("noncallable binding"),
        )
        .unwrap();
    let before = format!("{:?}", world.semantic_world());
    let call =
        extract_single_call_site(&initializer_from_source("let r = a noncallable;")).unwrap();
    let result = lang_build::invoke_resolved_binding_ordinary(
        world.semantic_world_mut(),
        &[],
        noncallable,
        &call,
        &base.root_context(),
        OrdinaryInvocationContext::open_static(&[]),
        Provenance::new("empty C3"),
    );
    let Err(OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
        first_diagnostic: None,
        trace,
    }) = result
    else {
        panic!("fully observed noncallability must terminate before argument applicability: {result:?}");
    };
    assert!(!trace.c2_horizon_values.is_empty());
    assert!(trace.c3_call_entries.is_empty());
    assert!(trace.selected.is_none());
    assert_eq!(format!("{:?}", world.semantic_world()), before);
}

#[test]
fn resolved_hidden_const_actual_cannot_default_mode_or_seal_selection() {
    use lang_build::{expose_policy_slice, read_pattern, read_value, ObservationHorizon, Stage};

    for stage in [Stage::Runtime, Stage::Seal] {
        for modes in [vec![], vec![PolicyMode::Const], vec![PolicyMode::Mut]] {
            let (mut world, binding) = family_with_const_actual(stage);
            let symbol = world.semantic_world().symbol(binding).unwrap();
            let entry = &symbol.member_views[0];
            let exposed = expose_policy_slice(entry, ObservationHorizon::OpenStatic);
            assert!(read_value(&exposed).is_none());
            if stage == Stage::Runtime {
                assert!(read_pattern(&exposed).is_some());
            }
            assert_eq!(exposed.mode, PolicyMode::Const);
            let before = format!("{:?}", world.semantic_world());
            let call =
                extract_single_call_site(&initializer_from_source("let r = a probe;")).unwrap();
            let result = world.invoke_ordinary_call(
                world.package_root_node(),
                &call,
                OrdinaryInvocationContext::open_static(&modes),
                Provenance::new("hidden argument"),
            );
            let Err(OrdinaryInvocationFailure::ObservationUnavailable { obstruction, trace }) =
                result
            else {
                panic!("hidden actual must stop before preference: {result:?}");
            };
            assert_eq!(obstruction.class, "hidden-argument-value-observation");
            assert!(!trace.c3_call_entries.is_empty());
            assert!(trace.a_fully_admissible.is_empty());
            assert!(trace.bp_prime.is_empty());
            assert!(trace.selected.is_none());
            assert!(trace.dynamic_legality.is_none());
            assert_eq!(format!("{:?}", world.semantic_world()), before);
            let after = world.semantic_world().symbol(binding).unwrap();
            assert_eq!(after.member_views[0].view.mode, PolicyMode::Const);
            assert_eq!(after.member_views[0].view.pair.value.stage(), Some(stage));
        }
    }
}

#[test]
fn readable_const_actual_keeps_binding_mode_over_caller_mut_context() {
    let (mut world, _) = family_with_const_actual(lang_build::Stage::Compile);
    let call = extract_single_call_site(&initializer_from_source("let r = a probe;")).unwrap();
    let result = world.invoke_ordinary_call(
        world.package_root_node(),
        &call,
        OrdinaryInvocationContext::open_static(&[PolicyMode::Mut]),
        Provenance::new("readable const argument"),
    );
    let selected = trace_of(&result).selected.expect("readable actual selects");
    let SemanticValuePayload::CallEntry(entry) =
        &world.semantic_world().value(selected).unwrap().payload
    else {
        panic!("ordinary call entry");
    };
    let formal = &entry
        .source_closure()
        .unwrap()
        .head
        .as_ref()
        .unwrap()
        .formal_frame()
        .explicit_parameters[0];
    let lang_syntax::NormPatternElem::BindingSlot(formal) = formal else {
        panic!("formal binding slot");
    };
    assert!(
        formal.policy.is_some(),
        "const formal must win for the known const observation"
    );
}

/// Extract the pipeline trace from an invocation result, success or failure.
/// Exposure regressions are trace facts and must stay observable even when
/// body execution of the selected candidate is not (yet) supported.
fn trace_of<'a>(
    result: &'a Result<InvocationOutcome, OrdinaryInvocationFailure>,
) -> &'a OrdinaryPipelineTrace {
    match result {
        Ok(lang_build::InvocationResult::SemanticResult {
            value: lang_build::ProjectedInvocationOutcome::Unit(u),
            ..
        }) => &u.trace,
        Ok(lang_build::InvocationResult::SemanticResult {
            value: lang_build::ProjectedInvocationOutcome::SingleMember(r),
            ..
        }) => &r.trace,
        Ok(lang_build::InvocationResult::Residual(_))
        | Ok(lang_build::InvocationResult::Diagnostic(_)) => {
            panic!("ordinary invocation did not produce a semantic result")
        }
        Err(OrdinaryInvocationFailure::NoTargetValues { trace })
        | Err(OrdinaryInvocationFailure::NoFullyAdmissibleCandidate { trace, .. })
        | Err(OrdinaryInvocationFailure::ApplicabilityUnsupported { trace, .. })
        | Err(OrdinaryInvocationFailure::Ambiguous { trace, .. })
        | Err(OrdinaryInvocationFailure::DynamicLegality { trace, .. })
        | Err(OrdinaryInvocationFailure::SelectedDelete { trace, .. })
        | Err(OrdinaryInvocationFailure::SelectedBody { trace, .. })
        | Err(OrdinaryInvocationFailure::SelectedImplementation { trace, .. })
        | Err(OrdinaryInvocationFailure::ResultTypeHasNoPattern { trace, .. })
        | Err(OrdinaryInvocationFailure::MigrationResultTypeChanged { trace, .. })
        | Err(OrdinaryInvocationFailure::MigrationOutputProjectionFailed { trace })
        | Err(OrdinaryInvocationFailure::ObservationUnavailable { trace, .. })
        | Err(OrdinaryInvocationFailure::ArgumentNormalization { trace, .. }) => trace,
    }
}

fn invoke(
    world: &mut CompilationWorld,
    spelling: &str,
    context: OrdinaryInvocationContext<'_>,
    provenance: &str,
) -> Result<InvocationOutcome, OrdinaryInvocationFailure> {
    let initializer = initializer_from_source(spelling);
    let call_site = extract_single_call_site(&initializer).expect("normalized call");
    world.invoke_ordinary_call(
        world.package_root_node(),
        &call_site,
        context,
        Provenance::new(provenance),
    )
}

#[test]
fn unit_argument_mode_completes_const_under_close() {
    let mut world = support::AssociatedFamily::new(&[
        "let first = (self, const let x): compile -> let r => { x; };",
        "let second = (self, mut let x): compile -> let r => { x; };",
    ]);
    let no_fabricated_modes = [];
    let call = extract_single_call_site(&initializer_from_source("let r = () probe;")).unwrap();
    let result = world.invoke_ordinary_call(
        world.package_root_node(),
        &call,
        OrdinaryInvocationContext::open_static(&no_fabricated_modes),
        Provenance::new("omitted occurrence mode under close"),
    );
    let selected = trace_of(&result).selected.unwrap_or_else(|| {
        panic!("the unit argument has decided omission before maxima: {result:?}")
    });
    let selected = world
        .semantic_world()
        .value(selected)
        .expect("selected call entry");
    let SemanticValuePayload::CallEntry(entry) = &selected.payload else {
        panic!("probe selection is an ordinary call entry");
    };
    let formal = entry
        .source_closure()
        .and_then(|closure| closure.head.as_ref())
        .and_then(|head| head.formal_frame().explicit_parameters.first())
        .expect("one explicit formal");
    let lang_syntax::NormPatternElem::BindingSlot(formal) = formal else {
        panic!("probe formal is a binding slot");
    };
    assert!(
        formal.policy.is_some(),
        "close completes omitted occurrence mode to const before maxima"
    );
}

#[test]
fn unknown_argument_policy_never_defaults_selects_or_publishes() {
    for qualification in [lang_build::OpenPolicy::Close, lang_build::OpenPolicy::Open] {
        let mut world = support::AssociatedFamily::new(&[
            "let first = (self, const let x): compile -> let r => { x; };",
            "let second = (self, mut let x): compile -> let r => { x; };",
        ]);
        let before = format!("{:?}", world.semantic_world());
        let call =
            extract_single_call_site(&initializer_from_source("let r = mystery probe;")).unwrap();
        let mut context = OrdinaryInvocationContext::open_static(&[]);
        context.omitted_argument_policy = qualification;
        let failure = world
            .invoke_ordinary_call(
                world.package_root_node(),
                &call,
                context,
                Provenance::new("unknown is not omission"),
            )
            .unwrap_err();
        let OrdinaryInvocationFailure::ApplicabilityUnsupported { diagnostic, trace } = failure
        else {
            panic!("{failure:?}");
        };
        assert!(diagnostic
            .message
            .contains("unknown material is not omitted Mode"));
        assert!(trace.bp_prime.is_empty());
        assert!(trace.selected.is_none());
        assert!(trace.dynamic_legality.is_none());
        assert_eq!(format!("{:?}", world.semantic_world()), before);
    }
}

#[test]
fn pure_type_binding_mode_survives_classification_and_preference() {
    let (mut world, _) = family_with_const_actual(lang_build::Stage::Compile);
    let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let pattern = world
        .semantic_world()
        .symbol_in_namespace(base.core_node(), "uint8")
        .unwrap()
        .pure_p_pattern()
        .unwrap();
    let complete = world
        .semantic_world()
        .symbol_in_namespace(base.core_node(), "uint8")
        .unwrap()
        .pure_p()
        .unwrap()
        .complete_type
        .and_then(|whole| {
            world
                .semantic_world()
                .complete_type_by_whole_observation(whole)
        })
        .unwrap()
        .clone();
    world
        .semantic_world_mut()
        .bind_ordinary_new(
            base.root_context().current_namespace,
            "T",
            &[lang_build::PolicyResultEntry {
                value: None,
                pattern,
                view: lang_build::declared_policy_view(lang_build::Stage::Compile, PolicyMode::Mut),
            }],
            Some(&complete),
            Provenance::new("mut pure type observation"),
        )
        .unwrap();
    let call = extract_single_call_site(&initializer_from_source("let r = T probe;")).unwrap();
    let result = world.invoke_ordinary_call(
        world.package_root_node(),
        &call,
        OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
        Provenance::new("known mode prevails"),
    );
    let selected = trace_of(&result)
        .selected
        .expect("observed type binding can select");
    let SemanticValuePayload::CallEntry(entry) =
        &world.semantic_world().value(selected).unwrap().payload
    else {
        panic!("call entry");
    };
    let lang_syntax::NormPatternElem::BindingSlot(formal) = &entry
        .source_closure()
        .unwrap()
        .head
        .as_ref()
        .unwrap()
        .formal_frame()
        .explicit_parameters[0]
    else {
        panic!("formal");
    };
    assert!(
        matches!(&formal.policy.as_ref().unwrap().constraint.atoms[0], lang_syntax::NormPolicyAtom::Name { text, .. } if text == "mut")
    );
}

#[test]
fn receiver_binding_mode_survives_shared_value_identity() {
    let mut world = support::AssociatedFamily::new(&[
        "const let first = (self, x): compile -> let r => (\"const self\") delete;",
        "mut let second = (self, x): compile -> let r => (\"mut self\") delete;",
    ]);
    let receiver = world.target_binding().ordinary_value().unwrap();
    let record = world.semantic_world().value(receiver).unwrap().clone();
    assert_eq!(record.mode, PolicyMode::Const);
    let expected = world.semantic_world().callable_entries_for_value(receiver)[1];
    let namespace = world.package_root_node();
    let binding = world
        .semantic_world_mut()
        .bind_ordinary_new(
            namespace,
            "observed",
            &[lang_build::PolicyResultEntry {
                value: Some(lang_build::SemanticValueRef {
                    id: receiver,
                    type_value: record.type_value,
                }),
                pattern: record.pattern,
                view: lang_build::declared_policy_view(lang_build::Stage::Compile, PolicyMode::Mut),
            }],
            None,
            Provenance::new("mut binding of same receiver"),
        )
        .unwrap();
    let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let call = extract_single_call_site(&initializer_from_source("let r = () observed;")).unwrap();
    let before = format!("{:?}", world.semantic_world());
    let failure = lang_build::invoke_resolved_binding_ordinary(
        world.semantic_world_mut(),
        &[],
        binding,
        &call,
        &base.root_context(),
        OrdinaryInvocationContext::open_static(&[]),
        Provenance::new("receiver edge mode"),
    )
    .unwrap_err();
    let OrdinaryInvocationFailure::SelectedDelete {
        diagnostic, trace, ..
    } = failure
    else {
        panic!("{failure:?}");
    };
    assert!(diagnostic.message.contains("mut self"));
    assert_eq!(trace.selected, Some(expected));
    assert_eq!(format!("{:?}", world.semantic_world()), before);
    assert_eq!(
        world.semantic_world().value(receiver).unwrap().mode,
        PolicyMode::Const
    );
}

// ---------------------------------------------------------------------------
// Fixture build smoke: the committed semantic workspaces must build.
// ---------------------------------------------------------------------------

#[test]
fn fixture_type_binding_builds() {
    let _ = build_single_fixture_world("type_binding", "app");
}

#[test]
fn type_binding_is_fresh_symbol_no_alias_no_reroot() {
    let world = build_single_fixture_world("type_binding", "app");
    let t = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "T")
        .expect("destination symbol T installed");
    let uint8 = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8");

    // Fresh destination Symbol: T is its own name binding, not an alias
    // facet of uint8.
    assert_ne!(
        t.identity, uint8.identity,
        "let binding creates a fresh destination Symbol, never an alias"
    );

    // Ordinary binding semantics: the RHS value (the uint8 type value) is
    // bound to the symbol T — T reads the same PatternValue.
    assert_eq!(
        t.pure_p_pattern(),
        uint8.pure_p_pattern(),
        "the bound type value is the RHS value itself"
    );

    // No reroot: carrier rebinding does not rewrite the Pattern's owning
    // binding; uint8's PatternValue stays owned by uint8.
    let pattern = uint8.pure_p_pattern().expect("core uint8 pure-P");
    assert_eq!(
        world.semantic_world().pattern_declaration(pattern),
        Some(uint8.identity),
        "carrier rebinding must not reroot the RHS PatternValue"
    );
}

// ---------------------------------------------------------------------------
#[test]
fn ordinary_receiver_owns_terminal_call_entry() {
    let world = support::AssociatedFamily::new(&[
        "let member = (self, t:type):compile -> let r:type => { t; };",
    ]);
    let make_type = world.target_binding();
    assert_eq!(make_type.ordinary_value().iter().count(), 1);
    let function_value = make_type.ordinary_value().unwrap();
    let function_obj = world
        .semantic_world()
        .value(function_value)
        .expect("function object value");
    assert!(matches!(
        function_obj.payload,
        SemanticValuePayload::PlainValue
    ));

    let entries = world
        .semantic_world()
        .associated_values_for_value(function_value, "()")
        .unwrap_or(&[]);
    assert_eq!(entries.len(), 1, "one () call entry on the function object");
    let call_obj = world
        .semantic_world()
        .value(entries[0])
        .expect("call entry value");
    assert!(matches!(
        call_obj.payload,
        SemanticValuePayload::CallEntry(_)
    ));

    // Terminal FunctionItem: the call entry has its own type/pattern and an
    // empty Val2.
    assert_ne!(function_obj.pattern, call_obj.pattern);
    assert_ne!(function_obj.type_value, call_obj.type_value);
    assert!(
        world
            .semantic_world()
            .associated_values_for_pattern(call_obj.pattern, "()")
            .is_none(),
        "call entry is terminal: no further ()"
    );
}

// ---------------------------------------------------------------------------
// Complete type helper selection uses the ordinary candidate and result planes.
// ---------------------------------------------------------------------------

#[test]
fn struct_helper_frontier_follows_ordinary_selection() {
    let mut world =
        CompilationWorld::from_manifest(&BuildManifest::new("app", vec!["app".to_string()]))
            .unwrap();
    let before = format!("{:?}", world.semantic_world());
    let failure = invoke(
        &mut world,
        "let T = uint8 struct::core;",
        OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
        "selected struct helper",
    )
    .unwrap_err();
    let OrdinaryInvocationFailure::SelectedImplementation { diagnostic, trace } = failure else {
        panic!("complete input reaches helper selection: {failure:?}");
    };
    assert_eq!(trace.c0_target_values.len(), 1);
    assert_eq!(trace.c1_visible_values, trace.c0_target_values);
    assert_eq!(trace.c3_call_entries.len(), 1);
    assert!(trace.selected.is_some());
    assert!(trace.compile_instance.is_some());
    assert!(diagnostic
        .message
        .contains("struct helper formation consumer is unavailable"));
    assert_eq!(format!("{:?}", world.semantic_world()), before);
}

// ---------------------------------------------------------------------------
#[test]
fn unwired_lexical_alias_creates_no_semantic_entity() {
    let error = build_fixture_error("lexical_alias_unwired", "app");
    assert_eq!(error.diagnostics.len(), 1);
    assert_eq!(
        error.diagnostics[0].code,
        Some(ResolverCode::UnsupportedLexicalAlias)
    );
    assert!(error.diagnostics[0]
        .message
        .contains("lexical Path alias formation/composition consumer is unavailable"));
    assert!(error.diagnostics[0]
        .message
        .contains("without installing or forwarding a semantic entity"));
}
