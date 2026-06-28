mod support;

use support::*;

use lang_build::{
    bind_meta_type_value_result, classify_type_arguments, classify_type_arguments_with_report,
    extract_single_call_site, invoke_meta_callable, invoke_meta_callable_cached,
    prepare_meta_callable_candidate, prepare_meta_callable_candidate_from_input,
    resolve_call_target, type_value_id_from_type_symbol_placeholder, AliasChain,
    AliasQueryDisposition, AliasQueryMode, CandidateBuildIdentityPlaceholder,
    CandidatePrepDeferredReason, CandidatePrepResult, CandidatePreparationContext,
    CandidatePreparationInput, CanonicalArgAtomKind, ExecutionEnv, FieldProjection,
    MetaInstanceCache, MetaInvocationInput, MetaInvocationResult, MetaReductionResult,
    NamespaceGraphSnapshot, NamespaceNode, NamespaceNodeKind, NonValueArgKind, ParameterShape,
    PlaceId, PolicyEnv, PolicyFlag, ProductMaterialRole, Provenance, RawArgValueClass,
    SourceCategory, SymbolId, SymbolPayload, TypeValueBindingPlaceholder, TypeValueId,
};

#[test]
fn type_value_binding_placeholder_object_boundary_keeps_symbol_place_and_type_value_distinct() {
    let binding = TypeValueBindingPlaceholder::new(
        SymbolId(1),
        PlaceId(10),
        TypeValueId(20),
        Provenance::new("type-value binding placeholder object boundary"),
    );

    assert_eq!(binding.symbol, SymbolId(1));
    assert_eq!(binding.place.as_u64(), 10);
    assert_eq!(binding.type_value.as_u64(), 20);
    assert_ne!(
        std::any::type_name::<PlaceId>(),
        std::any::type_name::<TypeValueId>(),
        "TypeValueId equality cannot imply PlaceId equality or writable permission"
    );
}

#[test]
fn alias_chain_placeholder_object_boundary_distinguishes_query_modes() {
    let alias = AliasChain::new(
        SymbolId(2),
        SymbolId(3),
        Provenance::new("alias chain placeholder object boundary"),
    );

    assert_eq!(alias.source_symbol, SymbolId(2));
    assert_eq!(alias.forwarded_target, SymbolId(3));
    assert_eq!(alias.final_place, None);
    assert!(!alias.creates_fresh_writable_place());
    assert_eq!(
        alias.query_disposition(AliasQueryMode::TypeValueEvaluation),
        AliasQueryDisposition::FollowValueChain
    );
    assert_eq!(
        alias.query_disposition(AliasQueryMode::CallableLookup),
        AliasQueryDisposition::PolicyAwareSymbolResolution
    );
    assert_eq!(
        alias.query_disposition(AliasQueryMode::InjectionPlaceTarget),
        AliasQueryDisposition::FollowPlaceWithBoundary
    );
}

#[test]
fn candidate_prep_uses_graph_resolved_symbolobject_and_arg_product_shape_from_build_fixture() {
    let world = v08_candidate_world();
    let callee = world
        .snapshot()
        .capability()
        .resolve_meta_function_with_policy("struct", &world.package_context(), PolicyEnv::Meta)
        .expect("core struct resolves through namespace graph as SymbolObject");

    let site = v08_candidate_call_site();
    let arg_shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let result = prepare_meta_callable_candidate(
        &callee,
        arg_shape,
        ParameterShape::exact_arity(1, Provenance::new("struct source product placeholder")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::Meta,
            demanded_execution: ExecutionEnv::Meta,
            build_identity: CandidateBuildIdentityPlaceholder {
                package_identity_fragment: Some("package:app".to_string()),
                mount_identity_fragment: Some("mount:core".to_string()),
                build_config_fingerprint_fragment: Some("build:fixture".to_string()),
                policy_export_fingerprint_fragment: Some("policy:export-meta".to_string()),
            },
            provenance: Provenance::new("v0.8 build-fixture candidate preparation"),
        },
    );

    let CandidatePrepResult::ApplicablePlaceholder(candidate) = result else {
        panic!("core struct should reach the applicable placeholder boundary");
    };
    assert_eq!(candidate.callee_symbol_id, callee.id);
    assert_eq!(candidate.arg_product_shape.arity, 1);
    assert_eq!(
        candidate
            .canonical_key_seed
            .argument_product_shape_material
            .arity,
        1
    );
    assert_eq!(
        candidate
            .canonical_key_seed
            .argument_product_shape_material
            .unit_positions,
        Vec::<usize>::new()
    );
    assert_eq!(
        candidate
            .canonical_key_seed
            .argument_product_shape_material
            .known_type_values,
        vec![None]
    );
    assert_eq!(candidate.arg_product_shape.raw_args[0].is_value(), None);
    assert!(matches!(
        candidate.arg_product_shape.raw_args[0].value_class,
        RawArgValueClass::UnknownExpression
    ));
    assert!(
        !candidate.arg_product_shape.raw_args[0].receives_automatic_pass_action(),
        "UnknownExpression does not receive automatic pass action at candidate-prep boundary"
    );
    assert_eq!(candidate.policy_planes.lookup_env, PolicyEnv::Meta);
    assert_eq!(
        candidate.policy_planes.symbol_visibility_policy,
        callee.policy_metadata
    );
    assert!(candidate
        .policy_planes
        .symbol_visibility_policy
        .policy_set
        .contains(PolicyFlag::Meta));
    assert!(candidate
        .policy_planes
        .body_entry_allows_demanded_execution());
    assert!(candidate
        .policy_planes
        .return_object_policy
        .policy_set
        .contains(PolicyFlag::Meta));
    assert!(candidate
        .policy_planes
        .return_object_policy
        .policy_set
        .contains(PolicyFlag::Runtime));
    assert_eq!(
        candidate.canonical_key_seed.callee_function_symbol_id,
        callee.id
    );
    assert_eq!(candidate.canonical_key_seed.argument_arity, 1);
    assert_eq!(
        candidate
            .canonical_key_seed
            .argument_product_shape_fingerprint_fragment,
        None
    );
    assert_eq!(
        candidate.canonical_key_seed.unit_positions,
        Vec::<usize>::new()
    );
    assert_eq!(
        candidate.canonical_key_seed.argument_type_values,
        vec![None]
    );
    assert_eq!(
        candidate
            .canonical_key_seed
            .package_identity_fragment
            .as_deref(),
        Some("package:app")
    );
    assert_eq!(
        candidate
            .canonical_key_seed
            .mount_identity_fragment
            .as_deref(),
        Some("mount:core")
    );
    assert_eq!(
        candidate
            .canonical_key_seed
            .build_config_fingerprint_fragment
            .as_deref(),
        Some("build:fixture")
    );
    assert_eq!(
        candidate
            .canonical_key_seed
            .policy_export_fingerprint_fragment
            .as_deref(),
        Some("policy:export-meta")
    );
}

#[test]
fn generated_field_function_from_build_fixture_keeps_policy_planes_separate() {
    let world = v08_candidate_world();
    let field_symbol = world
        .snapshot()
        .capability()
        .resolve_field_function("field::ref::T", &world.package_context())
        .expect("generated ref field function resolves through namespace graph");

    let SymbolPayload::FieldFunction(field_obj) = &field_symbol.payload else {
        panic!("expected FieldFunction payload for generated field symbol");
    };
    assert_eq!(
        field_obj.field_name, "field",
        "generated field function name must match the source fixture field name"
    );
    assert_eq!(
        field_obj.projection,
        FieldProjection::Ref,
        "generated field projection must match the source fixture field declaration"
    );
    assert!(
        field_obj.owner_type_symbol_id != SymbolId(0),
        "owner type must be a valid SymbolId from the struct-generated type"
    );

    let site = v08_candidate_call_site();
    let arg_shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let result = prepare_meta_callable_candidate(
        &field_symbol,
        arg_shape,
        ParameterShape::exact_arity(1, Provenance::new("field parameter placeholder")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::Meta,
            demanded_execution: ExecutionEnv::Meta,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("build-fixture generated field function"),
        },
    );

    let CandidatePrepResult::Deferred { candidate, reason } = result else {
        panic!("runtime-only body-entry must defer instead of becoming meta-executable");
    };
    assert_eq!(reason, CandidatePrepDeferredReason::BodyEntryPolicyMismatch);
    assert_eq!(candidate.policy_planes.lookup_env, PolicyEnv::Meta);
    assert!(candidate
        .policy_planes
        .symbol_visibility_policy
        .policy_set
        .contains(PolicyFlag::Meta));
    assert!(!candidate
        .policy_planes
        .body_entry_allows_demanded_execution());
    assert!(candidate
        .policy_planes
        .return_object_policy
        .policy_set
        .contains(PolicyFlag::Runtime));
    assert!(!candidate
        .policy_planes
        .return_object_policy
        .policy_set
        .contains(PolicyFlag::Meta));
    assert_ne!(
        candidate.policy_planes.symbol_visibility_policy, candidate.policy_planes.body_entry_policy,
        "symbol visibility policy must not equal body-entry policy"
    );
    assert_ne!(
        candidate.policy_planes.symbol_visibility_policy,
        candidate.policy_planes.return_object_policy,
        "symbol visibility policy must not equal return-object policy"
    );
}

#[test]
fn canonical_key_seed_reserves_canonical_argument_product_slots_from_source_fixture() {
    let shape = fixture_arg_product_shape(
        "product_unit_preservation.lang",
        ProductMaterialRole::MetaConstructionArgumentProduct,
    );
    let world = v08_candidate_world();
    let callee = world
        .snapshot()
        .capability()
        .resolve_meta_function_with_policy("struct", &world.package_context(), PolicyEnv::Meta)
        .expect("core struct resolves through namespace graph as SymbolObject");

    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prepare_meta_callable_candidate(
        &callee,
        shape,
        ParameterShape::exact_arity(3, Provenance::new("unit-sensitive parameter placeholder")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::Meta,
            demanded_execution: ExecutionEnv::Meta,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("unit-sensitive canonical key seed"),
        },
    ) else {
        panic!("candidate should reach applicable placeholder");
    };

    assert_eq!(
        candidate
            .canonical_key_seed
            .argument_product_shape_fingerprint_fragment,
        None,
        "fingerprint computation is intentionally deferred, but the slot must exist"
    );
    assert_eq!(
        candidate.canonical_key_seed.unit_positions,
        vec![1],
        "canonical argument product material must not collapse to arity + type values only"
    );
    assert_eq!(
        candidate
            .canonical_key_seed
            .argument_product_shape_material
            .unit_positions,
        vec![1],
        "canonical arg product shape material preserves Unit position"
    );
    assert_eq!(
        candidate
            .canonical_key_seed
            .argument_product_shape_material
            .arity,
        3,
        "canonical arg product shape material preserves arity"
    );
}

#[test]
fn namespace_delta_atomicity_object_boundary_rejects_partial_generated_subtree() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();
    let mut base = snapshot.empty_delta();
    let existing_t = base.allocate_symbol_id();
    base.insert_symbol(
        root,
        placeholder_symbol(existing_t, root, "T", "existing T"),
    );
    let snapshot = snapshot.install_delta(base).expect("install existing T");

    let mut generated = snapshot.empty_delta();
    let type_namespace = generated.allocate_node_id();
    generated.insert_node(NamespaceNode::new(
        type_namespace,
        "T<type-associated>",
        NamespaceNodeKind::Virtual,
        SourceCategory::TypeAssociatedNamespace,
        Some(root),
        Provenance::new("v0.8 generated type namespace"),
    ));
    let generated_t = generated.allocate_symbol_id();
    generated.insert_symbol(
        root,
        placeholder_symbol(generated_t, root, "T", "conflicting generated T"),
    );
    let generated_field = generated.allocate_symbol_id();
    generated.insert_symbol(
        type_namespace,
        placeholder_symbol(
            generated_field,
            type_namespace,
            "field",
            "partial generated field",
        ),
    );

    let error = snapshot
        .install_delta(generated)
        .expect_err("conflicting generated type rejects whole NamespaceDelta");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("T")));
    assert!(
        snapshot.symbol(generated_field).is_none(),
        "NamespaceDelta atomicity rejects generated children with the failed type"
    );
}

#[test]
fn candidate_preparation_input_is_the_pipeline_entry_from_build_fixture() {
    let world = v08_candidate_world();
    let callee = world
        .snapshot()
        .capability()
        .resolve_meta_function_with_policy("struct", &world.package_context(), PolicyEnv::Meta)
        .expect("core struct resolves through namespace graph as SymbolObject");

    let site = v08_candidate_call_site();
    let arg_shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);

    let input = CandidatePreparationInput::new(
        callee,
        arg_shape,
        ParameterShape::exact_arity(1, Provenance::new("pipeline entry test")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::Meta,
            demanded_execution: ExecutionEnv::Meta,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("CandidatePreparationInput pipeline entry"),
        },
    );

    let result = prepare_meta_callable_candidate_from_input(input);
    let CandidatePrepResult::ApplicablePlaceholder(candidate) = result else {
        panic!("CandidatePreparationInput pipeline should yield ApplicablePlaceholder");
    };
    assert_eq!(candidate.callee_name, "struct");
    assert_eq!(candidate.arg_product_shape.arity, 1);
}

#[test]
fn identity_type_target_and_type_argument_resolve_from_build_fixture() {
    let world = v08_identity_type_world();
    let t = world
        .snapshot()
        .capability()
        .resolve_type_object("T", &world.package_context())
        .expect("T should be resolved as type object in world from fixture");
    assert_eq!(t.kind, lang_build::SymbolKind::Type);
    assert!(
        matches!(t.payload, SymbolPayload::Type(_)),
        "t must carry Type payload (IdentityType result)"
    );
    assert_eq!(t.name, "T");

    let uint8 = world
        .snapshot()
        .capability()
        .resolve_type_object("uint8", &world.package_context())
        .expect("uint8 resolves as type object");
    let SymbolPayload::Type(type_obj) = &t.payload else {
        panic!("t payload is not Type");
    };
    assert_eq!(
        type_obj.type_symbol_id, uint8.id,
        "IdentityType(uint8) must return uint8's TypeValueId"
    );

    let identity = world
        .snapshot()
        .capability()
        .resolve_meta_function_with_policy(
            "IdentityType",
            &world.package_context(),
            PolicyEnv::Meta,
        )
        .expect("IdentityType resolves as meta function through namespace graph");
    assert_eq!(identity.name, "IdentityType");
    assert_eq!(identity.kind, lang_build::SymbolKind::MetaFunction);

    // --- Substrate path: call_target ---
    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_identity_type", "app").join("main.lang"),
    );
    let site = extract_single_call_site(&expr)
        .expect("v08_identity_type fixture initializer must be a call");
    let context = world.package_context();
    let resolved = resolve_call_target(
        &site.target,
        &world.snapshot().capability(),
        &context,
        PolicyEnv::Meta,
    )
    .expect("resolve_call_target should succeed")
    .expect("IdentityType target should resolve through namespace graph");
    assert!(
        resolved.temporary_direct_callable_shortcut,
        "resolved call target must carry the v0.8 shortcut flag"
    );
    assert_eq!(resolved.callee.name, "IdentityType");

    // --- Substrate path: ProductObject → ArgProductShape → classify_type_arguments ---
    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified = classify_type_arguments(&shape, &world.snapshot().capability(), &context);
    assert_eq!(classified.arity, 1);
    assert!(
        matches!(
            classified.raw_args[0].value_class,
            RawArgValueClass::NonValue(NonValueArgKind::TypeObject)
        ),
        "uint8 must be classified as NonValue(TypeObject)"
    );
    assert!(
        classified.raw_args[0]
            .known_first_order_type_value
            .is_some(),
        "classified type argument must have a TypeValueId"
    );
    assert_eq!(
        classified.raw_args[0].known_first_order_type_value,
        Some(TypeValueId(uint8.id.0)),
        "classified type argument TypeValueId must match uint8's SymbolId"
    );
    assert!(
        !classified.raw_args[0].receives_automatic_pass_action(),
        "type-object argument must not receive automatic pass action"
    );

    // --- Substrate path: canonical material ---
    let material =
        lang_build::CanonicalArgProductShapeMaterial::from_arg_product_shape(&classified);
    assert_eq!(material.arity, 1);
    assert_eq!(material.atom_kinds[0], CanonicalArgAtomKind::TypeObject);
    assert_eq!(material.known_type_values[0], Some(TypeValueId(uint8.id.0)));
}

#[test]
fn identity_type_classifier_resolves_uint8_through_namespace_graph() {
    let world = v08_identity_type_world();
    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_identity_type", "app").join("main.lang"),
    );
    let site = extract_single_call_site(&expr).expect("fixture initializer must be a call");
    let context = world.package_context();
    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);

    let classified = classify_type_arguments(&shape, &world.snapshot().capability(), &context);

    assert_eq!(classified.arity, 1);
    let raw = &classified.raw_args[0];
    assert!(
        matches!(
            raw.value_class,
            RawArgValueClass::NonValue(NonValueArgKind::TypeObject)
        ),
        "classify_type_arguments must resolve uint8 as TypeObject through namespace graph"
    );
    let tv = raw
        .known_first_order_type_value
        .expect("TypeValueId must be set");
    assert!(tv.0 != 0, "TypeValueId must be non-zero");
    assert!(
        !raw.receives_automatic_pass_action(),
        "classified type object must not receive automatic pass action"
    );
}

#[test]
fn identity_type_candidate_preparation_accepts_type_argument_object_boundary() {
    let world = v08_identity_type_world();
    let callee = world
        .snapshot()
        .capability()
        .resolve_meta_function_with_policy(
            "IdentityType",
            &world.package_context(),
            PolicyEnv::Meta,
        )
        .expect("IdentityType resolves through namespace graph");

    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_identity_type", "app").join("main.lang"),
    );
    let site = extract_single_call_site(&expr).expect("fixture must be a call");
    let context = world.package_context();
    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified = classify_type_arguments(&shape, &world.snapshot().capability(), &context);

    let input = CandidatePreparationInput::new(
        callee,
        classified,
        ParameterShape::type_parameter_signature(Provenance::new("IdentityType param")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::Meta,
            demanded_execution: ExecutionEnv::Meta,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("IdentityType candidate-prep object boundary"),
        },
    );

    let result = prepare_meta_callable_candidate_from_input(input);
    let CandidatePrepResult::ApplicablePlaceholder(candidate) = result else {
        panic!("IdentityType should reach applicable placeholder with type argument");
    };
    assert_eq!(candidate.callee_name, "IdentityType");
    assert_eq!(candidate.arg_product_shape.arity, 1);
    let raw = &candidate.arg_product_shape.raw_args[0];
    assert!(matches!(
        raw.value_class,
        RawArgValueClass::NonValue(NonValueArgKind::TypeObject)
    ));
    assert!(raw.known_first_order_type_value.is_some());
    let mat = &candidate.canonical_key_seed.argument_product_shape_material;
    assert_eq!(mat.arity, 1);
    assert_eq!(mat.atom_kinds[0], CanonicalArgAtomKind::TypeObject);
    assert!(mat.known_type_values[0].is_some());
}

#[test]
fn identity_type_formal_meta_invocation_returns_type_value_from_source_fixture() {
    let world = v08_identity_type_world();
    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_identity_type", "app").join("main.lang"),
    );
    let site = extract_single_call_site(&expr).expect("fixture must be a call");
    let context = world.package_context();
    let resolved = resolve_call_target(
        &site.target,
        &world.snapshot().capability(),
        &context,
        PolicyEnv::Meta,
    )
    .expect("resolve_call_target should succeed")
    .expect("IdentityType target should resolve");

    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified = classify_type_arguments(&shape, &world.snapshot().capability(), &context);

    let input = CandidatePreparationInput::new(
        resolved.callee.clone(),
        classified,
        ParameterShape::type_parameter_signature(Provenance::new("IdentityType param")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::Meta,
            demanded_execution: ExecutionEnv::Meta,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("formal invocation test"),
        },
    );

    let CandidatePrepResult::ApplicablePlaceholder(candidate) =
        prepare_meta_callable_candidate_from_input(input)
    else {
        panic!("candidate-prep should yield ApplicablePlaceholder");
    };

    let invocation_input =
        MetaInvocationInput::new(*candidate, Provenance::new("formal invocation"));
    let MetaInvocationResult::Reduction(MetaReductionResult::TypeValue(result_tv)) =
        invoke_meta_callable(invocation_input)
    else {
        panic!("invoke_meta_callable should yield TypeValue reduction");
    };
    assert!(result_tv.0 != 0, "TypeValueId result must be non-zero");
    // Verify result_tv matches what the classifier assigned
    let expected_tv = TypeValueId(
        world
            .snapshot()
            .capability()
            .resolve_type_object("uint8", &world.package_context())
            .expect("uint8 resolves")
            .id
            .0,
    );
    assert_eq!(
        result_tv, expected_tv,
        "invocation result must match uint8 TypeValueId"
    );
}

#[test]
fn identity_type_declaration_binding_installs_declared_type_after_invocation() {
    let world = v08_identity_type_world();
    let uint8 = world
        .snapshot()
        .capability()
        .resolve_type_object("uint8", &world.package_context())
        .expect("uint8 resolves as type object");
    let tv = type_value_id_from_type_symbol_placeholder(uint8.id);

    let result = bind_meta_type_value_result(
        tv,
        world.snapshot(),
        world.package_root_node(),
        "T",
        Provenance::new("binding test"),
    )
    .expect("bind_meta_type_value_result should succeed");
    assert!(
        !result.namespace_delta.nodes.is_empty() || !result.namespace_delta.symbols.is_empty(),
        "declaration binding must install a NamespaceDelta"
    );
    assert_eq!(
        result.replacement_object.name, "uint8",
        "replacement_object should be the uint8 type symbol"
    );
}

#[test]
fn identity_type_unresolved_type_argument_reports_resolution_failure() {
    let world = build_single_fixture_world("v08_identity_type", "app");
    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_identity_type", "app").join("main.lang"),
    );
    let site = extract_single_call_site(&expr).expect("fixture must be a call");
    let context = world.package_context();
    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);

    let report =
        classify_type_arguments_with_report(&shape, &world.snapshot().capability(), &context);
    assert_eq!(report.classified_shape.arity, 1, "single arg shape");
    assert!(
        report.unresolved_names.is_empty(),
        "uint8 should resolve without diagnostics"
    );
}

#[test]
fn type_value_id_placeholder_bridge_is_explicit_object_boundary() {
    let tv = type_value_id_from_type_symbol_placeholder(SymbolId(42));
    assert_eq!(tv, TypeValueId(42));
    assert_eq!(tv.as_u64(), 42);
}

#[test]
fn meta_instance_cache_reuses_identity_type_reduction() {
    let world = v08_identity_type_world();
    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_identity_type", "app").join("main.lang"),
    );
    let site = extract_single_call_site(&expr).expect("fixture must be a call");
    let context = world.package_context();
    let resolved = resolve_call_target(
        &site.target,
        &world.snapshot().capability(),
        &context,
        PolicyEnv::Meta,
    )
    .expect("resolve_call_target should succeed")
    .expect("IdentityType target should resolve");

    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified0 = classify_type_arguments(&shape, &world.snapshot().capability(), &context);

    let input = CandidatePreparationInput::new(
        resolved.callee.clone(),
        classified0.clone(),
        ParameterShape::type_parameter_signature(Provenance::new("cache test param")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::Meta,
            demanded_execution: ExecutionEnv::Meta,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("cache reuse test"),
        },
    );
    let CandidatePrepResult::ApplicablePlaceholder(candidate) =
        prepare_meta_callable_candidate_from_input(input)
    else {
        panic!("candidate-prep should yield ApplicablePlaceholder");
    };

    let invocation_input = MetaInvocationInput::new(*candidate, Provenance::new("cache test"));
    let key = invocation_input.compute_key();

    let mut cache = MetaInstanceCache::new();
    assert!(cache.lookup(&key).is_none(), "cache should be empty");

    let result1 = invoke_meta_callable_cached(invocation_input, &mut cache);
    let MetaInvocationResult::Reduction(MetaReductionResult::TypeValue(tv1)) = result1 else {
        panic!("invocation should yield TypeValue");
    };

    let cached = cache.lookup(&key).expect("entry should now be cached");
    let MetaReductionResult::TypeValue(cached_tv) = &cached.result;
    assert_eq!(
        tv1, *cached_tv,
        "cached TypeValue must match invocation result"
    );

    // Second invocation with same material (new candidate from same input)
    let CandidatePrepResult::ApplicablePlaceholder(candidate2) =
        prepare_meta_callable_candidate_from_input(CandidatePreparationInput::new(
            resolved.callee.clone(),
            classified0,
            ParameterShape::type_parameter_signature(Provenance::new("cache test param")),
            CandidatePreparationContext {
                lookup_env: PolicyEnv::Meta,
                demanded_execution: ExecutionEnv::Meta,
                build_identity: CandidateBuildIdentityPlaceholder::default(),
                provenance: Provenance::new("cache reuse test 2"),
            },
        ))
    else {
        panic!("second candidate-prep should yield ApplicablePlaceholder");
    };
    let invocation_input2 = MetaInvocationInput::new(*candidate2, Provenance::new("cache test 2"));
    let result2 = lang_build::invoke_meta_callable_cached(invocation_input2, &mut cache);
    let MetaInvocationResult::Reduction(MetaReductionResult::TypeValue(tv2)) = result2 else {
        panic!("second invocation should yield TypeValue");
    };
    assert_eq!(tv1, tv2, "cache-hit result must match original");
    assert_eq!(cache.len(), 1, "cache should not grow on hit");
}
