use lang_build::{
    solve_parameter_product_relation, CallableOwnerPlacement, CanonicalValueAddr,
    LocalCallableIdentity, OverloadArgShape, PackageId, PatternCoordinateId,
    PatternRelationContext, PatternRelationFailure, Provenance, SemanticOwnerGraph, TypeValueId,
};
use lang_syntax::{normalize_program, NormDecl, NormExpr, NormForm};

fn normalized_closure(source: &str) -> lang_syntax::NormClosure {
    let parsed = lang_syntax::parse(source);
    assert!(
        parsed.diagnostics.is_empty(),
        "{}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    let normalized = normalize_program(&parsed.program);
    let [NormForm::Let(NormDecl::Let { slot, .. })] = normalized.forms.as_slice() else {
        panic!("expected one let declaration");
    };
    let Some(NormExpr::Closure(closure)) = slot.initializer.as_deref() else {
        panic!("expected a closure initializer");
    };
    closure.clone()
}

fn callable_owner() -> lang_build::SemanticOwnerId {
    let mut owners = SemanticOwnerGraph::new();
    let package = owners.package_root(PackageId(77), "pattern-relation-test");
    owners.callable(
        package,
        LocalCallableIdentity(9),
        CallableOwnerPlacement::Ordinary,
    )
}

fn typed_value(
    value_type: TypeValueId,
    pattern: PatternCoordinateId,
    core: CanonicalValueAddr,
) -> OverloadArgShape {
    OverloadArgShape {
        top_pattern_name: Some("diagnostic-only-name".into()),
        type_symbol_id: None,
        value_type: Some(value_type),
        pattern_coordinate: Some(pattern),
        type_core_observation: Some(core),
        complete_type_observation: None,
        effective_view: None,
        semantic_value: None,
        value_class: lang_build::RawArgValueClass::Value,
        provenance: Provenance::new("typed relational argument"),
    }
}

#[test]
fn unnamed_binder_binds_by_identity_without_becoming_a_wildcard() {
    let closure = normalized_closure("let f = (receiver, value) => { value; };");
    let context = PatternRelationContext::for_source_callable(&closure, callable_owner(), None)
        .expect("qualified Pattern root");
    let mut params = closure
        .head
        .as_ref()
        .unwrap()
        .formal_frame()
        .explicit_parameters
        .to_vec();
    let argument = typed_value(
        support::type_lookup_fixture("relational-pattern/unnamed"),
        PatternCoordinateId(10),
        CanonicalValueAddr(100),
    );
    let named = solve_parameter_product_relation(&params, &[argument.clone()], &context).unwrap();
    let lang_syntax::NormPatternElem::BindingSlot(slot) = &mut params[0] else {
        panic!("slot")
    };
    let lang_syntax::NormPattern::Binder { name, identity, .. } = &mut slot.value_pattern else {
        panic!("ordinary binder")
    };
    assert!(identity.is_some());
    *name = None;
    let unnamed = solve_parameter_product_relation(&params, &[argument.clone()], &context).unwrap();
    assert_eq!(unnamed.solutions[0].local_bindings.len(), 1);
    assert_eq!(named.specificity, unnamed.specificity);
    assert!(unnamed.named_bindings().is_empty());
    assert_eq!(
        named.solutions[0].local_bindings.keys().collect::<Vec<_>>(),
        unnamed.solutions[0]
            .local_bindings
            .keys()
            .collect::<Vec<_>>()
    );
    let lang_syntax::NormPatternElem::BindingSlot(slot) = &mut params[0] else {
        unreachable!()
    };
    let lang_syntax::NormPattern::Binder { name, .. } = &mut slot.value_pattern else {
        unreachable!()
    };
    *name = Some("_".into());
    let wildcard = solve_parameter_product_relation(&params, &[argument], &context).unwrap();
    assert!(wildcard.solutions[0].local_bindings.is_empty());
}

#[test]
fn shared_deduce_hole_requires_one_relational_valuation() {
    let closure = normalized_closure("let f = <T: type::>(self, x: T, y: T) -> r => { r };");
    let params = closure
        .head
        .as_ref()
        .expect("head")
        .formal_frame()
        .explicit_parameters;
    let context = PatternRelationContext::for_source_callable(&closure, callable_owner(), None)
        .expect("qualified Pattern root");

    let same = solve_parameter_product_relation(
        params,
        &[
            typed_value(
                support::type_lookup_fixture("relational-pattern/type-a"),
                PatternCoordinateId(10),
                CanonicalValueAddr(100),
            ),
            typed_value(
                support::type_lookup_fixture("relational-pattern/type-b"),
                PatternCoordinateId(11),
                CanonicalValueAddr(100),
            ),
        ],
        &context,
    )
    .expect("equal Core observations satisfy one shared hole");
    assert_eq!(same.solutions.len(), 1);
    assert_eq!(same.solutions[0].holes.len(), 1);
    assert_eq!(same.solutions[0].local_bindings.len(), 2);

    let different = solve_parameter_product_relation(
        params,
        &[
            typed_value(
                support::type_lookup_fixture("relational-pattern/type-a"),
                PatternCoordinateId(10),
                CanonicalValueAddr(100),
            ),
            typed_value(
                support::type_lookup_fixture("relational-pattern/type-a"),
                PatternCoordinateId(10),
                CanonicalValueAddr(101),
            ),
        ],
        &context,
    );
    assert!(matches!(
        different,
        Err(PatternRelationFailure::Inapplicable(_))
    ));
}

#[test]
fn bare_type_lookup_and_display_name_cannot_decide_pattern_applicability() {
    let closure = normalized_closure("let f = <T: type::>(self, x: T, y: T) -> r => { r };");
    let params = closure
        .head
        .as_ref()
        .expect("head")
        .formal_frame()
        .explicit_parameters;
    let context = PatternRelationContext::for_source_callable(&closure, callable_owner(), None)
        .expect("qualified Pattern root");

    let result = solve_parameter_product_relation(
        params,
        &[
            typed_value(
                support::type_lookup_fixture("relational-pattern/shared-lookup"),
                PatternCoordinateId(20),
                CanonicalValueAddr(200),
            ),
            typed_value(
                support::type_lookup_fixture("relational-pattern/shared-lookup"),
                PatternCoordinateId(20),
                CanonicalValueAddr(201),
            ),
        ],
        &context,
    );
    assert!(
        matches!(result, Err(PatternRelationFailure::Inapplicable(_))),
        "equal TypeValueId and equal spelling cannot override unequal Core observations"
    );
}
mod support;

#[test]
fn pack_classifier_material_cannot_be_silently_dropped() {
    for annotation in ["classifier", "uint8::", "R$"] {
        let closure =
            normalized_closure(&format!("let f = (self, ...xs:{annotation}) => {{ xs; }};"));
        let context =
            PatternRelationContext::for_source_callable(&closure, callable_owner(), None).unwrap();
        let params = closure
            .head
            .as_ref()
            .unwrap()
            .formal_frame()
            .explicit_parameters;
        let argument = typed_value(
            support::type_lookup_fixture("pack/type"),
            PatternCoordinateId(10),
            CanonicalValueAddr(100),
        );
        for arguments in [
            vec![],
            vec![argument.clone()],
            vec![argument.clone(), argument.clone()],
        ] {
            assert!(
                matches!(solve_parameter_product_relation(params, &arguments, &context), Err(PatternRelationFailure::Unsupported(d)) if d.message.contains("Pack classifier"))
            );
        }
    }
}

#[test]
fn classifier_binder_does_not_resolve_an_existing_name() {
    let closure = normalized_closure("let f = (self, x:uint8) => { x; };");
    let resolve = |_name: &str| -> Option<lang_build::NamedPatternObservation> {
        panic!("a classifier-side binder must not consult an external name resolver")
    };
    let context =
        PatternRelationContext::for_source_callable(&closure, callable_owner(), Some(&resolve))
            .unwrap();
    let params = closure
        .head
        .as_ref()
        .unwrap()
        .formal_frame()
        .explicit_parameters;
    let argument = typed_value(
        support::type_lookup_fixture("joint/type"),
        PatternCoordinateId(10),
        CanonicalValueAddr(100),
    );
    let error = solve_parameter_product_relation(params, &[argument], &context).unwrap_err();
    assert!(
        matches!(error, PatternRelationFailure::Unsupported(d) if d.message.contains("classifier-side binding"))
    );
}

#[test]
fn applicability_rejects_classifier_mismatch_even_when_a_constructor_exists() {
    let world =
        lang_build::CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let integer = world.resolve_type_value("integer").unwrap();
    let target = world.resolve_type_value("uint16").unwrap();
    let semantic = world.semantic_world();
    let held = |name| {
        let whole = semantic
            .symbol_in_namespace(world.core_node(), name)
            .unwrap()
            .type_resident()
            .unwrap()
            .complete_type
            .unwrap();
        semantic.complete_type_by_whole_observation(whole).unwrap()
    };
    let actual = held("integer");
    let expected = held("uint16");
    assert!(!semantic
        .associated_member_views_for_complete_type(expected, "<ConstructOrConvert>")
        .unwrap()
        .is_empty());
    let closure = normalized_closure("let f = (self, x:uint16::) => { x; };");
    let resolve = |name: &str| {
        assert_eq!(name, "uint16");
        Some(lang_build::NamedPatternObservation {
            pattern: semantic.type_value(target).unwrap().pattern,
            core: Some(expected.core()),
        })
    };
    let context =
        PatternRelationContext::for_source_callable(&closure, callable_owner(), Some(&resolve))
            .unwrap();
    let argument = typed_value(
        integer,
        semantic.type_value(integer).unwrap().pattern,
        actual.core(),
    );
    let before = format!("{semantic:?}");
    assert!(matches!(
        solve_parameter_product_relation(
            closure
                .head
                .as_ref()
                .unwrap()
                .formal_frame()
                .explicit_parameters,
            &[argument],
            &context
        ),
        Err(PatternRelationFailure::Inapplicable(_))
    ));
    assert_eq!(format!("{:?}", world.semantic_world()), before);
    assert!(world.semantic_world().compile_instances().next().is_none());
}

#[test]
fn hole_local_policy_cannot_be_silently_dropped_by_relation_entry() {
    for source in [
        "let f = <mut let h>(self, x:h) => { x; };",
        "let f = (self, let <mut let h> x:h) => { x; };",
    ] {
        let closure = normalized_closure(source);
        let result = PatternRelationContext::for_source_callable(&closure, callable_owner(), None);
        assert!(
            matches!(result, Err(PatternRelationFailure::Unsupported(d)) if d.message.contains("Hole-local Policy"))
        );
    }
}

#[test]
fn external_universe_rank_pattern_is_not_an_anonymous_match() {
    let argument = typed_value(
        support::type_lookup_fixture("universe-rank/external"),
        PatternCoordinateId(10),
        CanonicalValueAddr(100),
    );
    let anonymous = normalized_closure("let f = (self, x:_) => { x; };");
    let context =
        PatternRelationContext::for_source_callable(&anonymous, callable_owner(), None).unwrap();
    assert!(solve_parameter_product_relation(
        anonymous
            .head
            .as_ref()
            .unwrap()
            .formal_frame()
            .explicit_parameters,
        &[argument.clone()],
        &context,
    )
    .is_ok());

    for pattern in ["_$", "(_)$", "(_ * 1)$", "_ * n"] {
        let closure = normalized_closure(&format!("let f = (self, x:{pattern}) => {{ x; }};"));
        let context =
            PatternRelationContext::for_source_callable(&closure, callable_owner(), None).unwrap();
        assert!(
            matches!(
                solve_parameter_product_relation(
                    closure
                        .head
                        .as_ref()
                        .unwrap()
                        .formal_frame()
                        .explicit_parameters,
                    &[argument.clone()],
                    &context,
                ),
                Err(PatternRelationFailure::Unsupported(_))
            ),
            "{pattern} needs its actual external/repetition Pattern consumer"
        );
    }
}

#[test]
fn matching_sequence_terms_cannot_erase_external_or_unsupported_material() {
    use lang_syntax::{NormPattern, NormPatternElem};

    let resolve = |name: &str| {
        assert_eq!(name, "matching");
        Some(lang_build::NamedPatternObservation {
            pattern: PatternCoordinateId(10),
            core: Some(CanonicalValueAddr(100)),
        })
    };
    for material in ["_$", "(_)$", "bool::", "(if)$", "(_ * 1)$", "_ * n"] {
        let closure = normalized_closure(&format!("let f = (self, x:{material}) => {{ x; }};"));
        let context =
            PatternRelationContext::for_source_callable(&closure, callable_owner(), Some(&resolve))
                .unwrap();
        let anonymous = normalized_closure("let f = (self, x:_) => { x; };");
        let NormPatternElem::BindingSlot(anonymous_slot) =
            &anonymous.head.as_ref().unwrap().params[1]
        else {
            panic!("anonymous classifier slot");
        };
        let wildcard = anonymous_slot.annotation.as_ref().unwrap().pattern.clone();
        let params = closure
            .head
            .as_ref()
            .unwrap()
            .formal_frame()
            .explicit_parameters;
        let argument = typed_value(
            support::type_lookup_fixture("sequence/external"),
            PatternCoordinateId(10),
            CanonicalValueAddr(100),
        );
        let mut supported_params = params.to_vec();
        let NormPatternElem::BindingSlot(supported_slot) = &mut supported_params[0] else {
            panic!("parameter");
        };
        let annotation = supported_slot.annotation.as_mut().unwrap();
        annotation.pattern = NormPattern::Sequence {
            elements: vec![
                wildcard.clone(),
                NormPattern::Name {
                    name: "matching".into(),
                    origin: annotation.origin.clone(),
                },
            ],
            origin: annotation.origin.clone(),
        };
        assert!(
            solve_parameter_product_relation(&supported_params, &[argument.clone()], &context,)
                .is_ok(),
            "the supported terms form a successful query on their own"
        );
        for nested in [false, true] {
            let mut params = params.to_vec();
            let NormPatternElem::BindingSlot(slot) = &mut params[0] else {
                panic!("parameter")
            };
            let annotation = slot.annotation.as_mut().unwrap();
            let mut required = annotation.pattern.clone();
            if nested {
                required = NormPattern::Sequence {
                    elements: vec![required],
                    origin: annotation.origin.clone(),
                };
            }
            annotation.pattern = NormPattern::Sequence {
                elements: vec![
                    wildcard.clone(),
                    NormPattern::Name {
                        name: "matching".into(),
                        origin: annotation.origin.clone(),
                    },
                    required,
                ],
                origin: annotation.origin.clone(),
            };
            assert!(matches!(
                solve_parameter_product_relation(&params, &[argument.clone()], &context),
                Err(PatternRelationFailure::Unsupported(_))
            ), "{material}, nested={nested}: a matching term is not permission to drop a required observation");
        }
    }
}

#[test]
fn required_classifier_observation_distinguishes_unknown_from_detached_evidence() {
    for source in [
        "let f = <T>(self, let <> T) => { self; };",
        "let f = <T>(self, x:T) => { x; };",
        "let f = (self, x:known::) => { x; };",
        "let f = (self, _ | known) => { self; };",
        "let f = (self, _ known) => { self; };",
    ] {
        let closure = normalized_closure(source);
        let resolve = |_name: &str| {
            Some(lang_build::NamedPatternObservation {
                pattern: PatternCoordinateId(10),
                core: None,
            })
        };
        let context =
            PatternRelationContext::for_source_callable(&closure, callable_owner(), Some(&resolve))
                .unwrap();
        let params = closure
            .head
            .as_ref()
            .unwrap()
            .formal_frame()
            .explicit_parameters;
        let mut argument = typed_value(
            support::type_lookup_fixture("classifier/observed"),
            PatternCoordinateId(10),
            CanonicalValueAddr(100),
        );
        // An observed detached coordinate remains a valid equality observation.
        argument.type_core_observation = None;
        assert!(solve_parameter_product_relation(params, &[argument.clone()], &context).is_ok());
        argument.pattern_coordinate = None;
        assert!(
            matches!(
                solve_parameter_product_relation(params, &[argument], &context),
                Err(PatternRelationFailure::Unsupported(d))
                    if d.message.contains("classifier observation is unavailable")
            ),
            "{source}"
        );
    }
}

#[test]
fn direct_named_query_requires_named_evidence_and_preserves_decided_mismatch() {
    let closure = normalized_closure("let f = (self, x) => { x; };");
    let mut params = closure
        .head
        .as_ref()
        .unwrap()
        .formal_frame()
        .explicit_parameters
        .to_vec();
    let lang_syntax::NormPatternElem::BindingSlot(slot) = &mut params[0] else {
        panic!("binding slot");
    };
    slot.value_pattern = lang_syntax::NormPattern::Name {
        name: "known".into(),
        origin: slot.origin.clone(),
    };
    let mut argument = typed_value(
        support::type_lookup_fixture("classifier/named"),
        PatternCoordinateId(10),
        CanonicalValueAddr(100),
    );
    let unavailable = |_name: &str| None;
    for resolve in [
        None,
        Some(&unavailable as &dyn Fn(&str) -> Option<lang_build::NamedPatternObservation>),
    ] {
        let context =
            PatternRelationContext::for_source_callable(&closure, callable_owner(), resolve)
                .unwrap();
        assert!(matches!(
            solve_parameter_product_relation(&params, &[argument.clone()], &context),
            Err(PatternRelationFailure::Unsupported(d))
                if d.message.contains("named Pattern observation is unavailable")
        ));
    }
    let resolve = |_name: &str| {
        Some(lang_build::NamedPatternObservation {
            pattern: PatternCoordinateId(10),
            core: Some(CanonicalValueAddr(100)),
        })
    };
    let context =
        PatternRelationContext::for_source_callable(&closure, callable_owner(), Some(&resolve))
            .unwrap();
    assert!(solve_parameter_product_relation(&params, &[argument.clone()], &context).is_ok());
    let mut unknown = argument.clone();
    unknown.pattern_coordinate = None;
    assert!(matches!(
        solve_parameter_product_relation(&params, &[unknown], &context),
        Err(PatternRelationFailure::Unsupported(d)) if d.message.contains("classifier observation")
    ));
    argument.type_core_observation = Some(CanonicalValueAddr(101));
    assert!(matches!(
        solve_parameter_product_relation(&params, &[argument], &context),
        Err(PatternRelationFailure::Inapplicable(_))
    ));
}

#[test]
fn binding_discard_and_unconstrained_pack_do_not_require_classifier_observation() {
    let mut argument = typed_value(
        support::type_lookup_fixture("classifier/unused"),
        PatternCoordinateId(10),
        CanonicalValueAddr(100),
    );
    argument.value_class = lang_build::RawArgValueClass::UnknownExpression;
    argument.pattern_coordinate = None;
    argument.type_core_observation = None;
    for parameter in ["x", "_", "_ _", "x:_", "...xs", "..._"] {
        let closure = normalized_closure(&format!("let f = (self, {parameter}) => {{ self; }};"));
        let context =
            PatternRelationContext::for_source_callable(&closure, callable_owner(), None).unwrap();
        assert!(
            solve_parameter_product_relation(
                closure
                    .head
                    .as_ref()
                    .unwrap()
                    .formal_frame()
                    .explicit_parameters,
                &[argument.clone()],
                &context,
            )
            .is_ok(),
            "{parameter}"
        );
    }
}

#[test]
fn type_role_query_distinguishes_unknown_role_known_non_type_and_missing_classifier() {
    use lang_build::{NonValueArgKind, RawArgValueClass};
    let closure = normalized_closure("let f = (self, x:type::) => { x; };");
    let context =
        PatternRelationContext::for_source_callable(&closure, callable_owner(), None).unwrap();
    let params = closure
        .head
        .as_ref()
        .unwrap()
        .formal_frame()
        .explicit_parameters;
    let mut argument = typed_value(
        support::type_lookup_fixture("classifier/type-role"),
        PatternCoordinateId(10),
        CanonicalValueAddr(100),
    );
    for role in [
        RawArgValueClass::Value,
        RawArgValueClass::NonValue(NonValueArgKind::ProductUnit),
    ] {
        argument.value_class = role;
        assert!(matches!(
            solve_parameter_product_relation(params, &[argument.clone()], &context),
            Err(PatternRelationFailure::Inapplicable(_))
        ));
    }
    argument.value_class = RawArgValueClass::UnknownExpression;
    assert!(
        matches!(solve_parameter_product_relation(params, &[argument.clone()], &context), Err(PatternRelationFailure::Unsupported(d)) if d.message.contains("role observation"))
    );
    argument.value_class = RawArgValueClass::NonValue(NonValueArgKind::CoreTypeProjection);
    assert!(solve_parameter_product_relation(params, &[argument.clone()], &context).is_ok());
    argument.pattern_coordinate = None;
    assert!(
        matches!(solve_parameter_product_relation(params, &[argument], &context), Err(PatternRelationFailure::Unsupported(d)) if d.message.contains("classifier observation"))
    );
}
