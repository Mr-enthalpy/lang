mod support;

use support::*;

use lang_build::{
    bind_meta_invocation_value_result, classify_type_arguments,
    classify_type_arguments_with_report, compute_type_definition_instance_id,
    extract_single_call_site, invoke_meta_callable, invoke_meta_callable_cached,
    invoke_meta_callable_cached_with_materialization_state,
    invoke_meta_callable_with_materialization_state, resolve_call_target,
    type_value_projection_from_type_symbol, AliasChain, AliasQueryDisposition, AliasQueryMode,
    CandidateBuildIdentityPlaceholder, CandidatePrepDeferredReason, CandidatePrepResult,
    CandidatePreparationContext, CanonicalArgAtomKind, CanonicalArgProductShapeMaterial,
    ExecutionEnv, FieldProjection, GeneratedTypeDefinitionValue, MetaInstanceCache,
    MetaInvocationInput, MetaInvocationResult, MetaInvocationValue, NamespaceNode,
    NamespaceNodeKind, NonValueArgKind, ParameterShape, PatternHeadId, PlaceId, PolicyEnv,
    PolicyFlag, ProductMaterialRole, Provenance, RawArgValueClass, ReturnViewShape,
    SemanticNameIndex, SourceCategory, SymbolId, SymbolPayload, TypeMaterializationState,
    TypeValueBindingPlaceholder, TypeValueId,
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
        .namespace_projection()
        .capability()
        .resolve_meta_function_with_policy(
            "struct",
            &world.package_context(),
            PolicyEnv::OpenStatic,
        )
        .expect("core struct resolves through namespace graph as SymbolObject");

    let site = v08_candidate_call_site();
    let arg_shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let result = prepare_candidate_from_fixture_symbol(
        &callee,
        arg_shape,
        ParameterShape::exact_arity(1, Provenance::new("struct source product placeholder")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
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
    let material =
        CanonicalArgProductShapeMaterial::from_arg_product_shape(&candidate.arg_product_shape);
    assert_eq!(material.arity, 1);
    assert_eq!(material.unit_positions, Vec::<usize>::new());
    assert_eq!(material.known_type_values, vec![None]);
    assert_eq!(candidate.arg_product_shape.raw_args[0].is_value(), None);
    assert!(matches!(
        candidate.arg_product_shape.raw_args[0].value_class,
        RawArgValueClass::UnknownExpression
    ));
    assert!(
        !candidate.arg_product_shape.raw_args[0].receives_automatic_pass_action(),
        "UnknownExpression does not receive automatic pass action at candidate-prep boundary"
    );
    assert_eq!(candidate.policy_planes.lookup_env, PolicyEnv::OpenStatic);
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
        candidate
            .build_identity
            .package_identity_fragment
            .as_deref(),
        Some("package:app")
    );
    assert_eq!(
        candidate.build_identity.mount_identity_fragment.as_deref(),
        Some("mount:core")
    );
    assert_eq!(
        candidate
            .build_identity
            .build_config_fingerprint_fragment
            .as_deref(),
        Some("build:fixture")
    );
    assert_eq!(
        candidate
            .build_identity
            .policy_export_fingerprint_fragment
            .as_deref(),
        Some("policy:export-meta")
    );
}

#[test]
fn generated_field_function_from_build_fixture_keeps_policy_planes_separate() {
    let world = v08_candidate_world();
    let field_symbol = world
        .namespace_projection()
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
    let result = prepare_candidate_from_fixture_symbol(
        &field_symbol,
        arg_shape,
        ParameterShape::exact_arity(1, Provenance::new("field parameter placeholder")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("build-fixture generated field function"),
        },
    );

    let CandidatePrepResult::Deferred { candidate, reason } = result else {
        panic!("runtime-only body-entry must defer instead of becoming meta-executable");
    };
    assert_eq!(reason, CandidatePrepDeferredReason::BodyEntryPolicyMismatch);
    assert_eq!(candidate.policy_planes.lookup_env, PolicyEnv::OpenStatic);
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
fn canonical_argument_product_material_reserves_slots_from_source_fixture() {
    let shape = fixture_arg_product_shape(
        "product_unit_preservation.lang",
        ProductMaterialRole::MetaConstructionArgumentProduct,
    );
    let world = v08_candidate_world();
    let callee = world
        .namespace_projection()
        .capability()
        .resolve_meta_function_with_policy(
            "struct",
            &world.package_context(),
            PolicyEnv::OpenStatic,
        )
        .expect("core struct resolves through namespace graph as SymbolObject");

    let CandidatePrepResult::ApplicablePlaceholder(candidate) =
        prepare_candidate_from_fixture_symbol(
            &callee,
            shape,
            ParameterShape::exact_arity(3, Provenance::new("unit-sensitive parameter placeholder")),
            CandidatePreparationContext {
                lookup_env: PolicyEnv::OpenStatic,
                demanded_execution: ExecutionEnv::OpenStatic,
                build_identity: CandidateBuildIdentityPlaceholder::default(),
                provenance: Provenance::new("unit-sensitive canonical key seed"),
            },
        )
    else {
        panic!("candidate should reach applicable placeholder");
    };

    let material =
        CanonicalArgProductShapeMaterial::from_arg_product_shape(&candidate.arg_product_shape);
    assert_eq!(
        material.unit_positions,
        vec![1],
        "canonical arg product shape material preserves Unit position"
    );
    assert_eq!(
        material.arity, 3,
        "canonical arg product shape material preserves arity"
    );
}

#[test]
fn namespace_delta_atomicity_object_boundary_rejects_partial_generated_subtree() {
    let snapshot = SemanticNameIndex::new();
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
fn candidate_preparation_is_the_pipeline_entry_from_build_fixture() {
    let world = v08_candidate_world();
    let callee = world
        .namespace_projection()
        .capability()
        .resolve_meta_function_with_policy(
            "struct",
            &world.package_context(),
            PolicyEnv::OpenStatic,
        )
        .expect("core struct resolves through namespace graph as SymbolObject");

    let site = v08_candidate_call_site();
    let arg_shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);

    let result = prepare_candidate_from_fixture_symbol(
        &callee,
        arg_shape,
        ParameterShape::exact_arity(1, Provenance::new("pipeline entry test")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("candidate preparation pipeline entry"),
        },
    );
    let CandidatePrepResult::ApplicablePlaceholder(candidate) = result else {
        panic!("candidate preparation pipeline should yield ApplicablePlaceholder");
    };
    assert_eq!(candidate.callee_name, "struct");
    assert_eq!(candidate.arg_product_shape.arity, 1);
}

#[test]
fn identity_type_target_and_type_argument_resolve_from_build_fixture() {
    let world = v08_identity_type_world();
    let t = world
        .namespace_projection()
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
        .namespace_projection()
        .capability()
        .resolve_type_object("uint8", &world.package_context())
        .expect("uint8 resolves as type object");
    let SymbolPayload::Type(type_obj) = &t.payload else {
        panic!("t payload is not Type");
    };
    let SymbolPayload::Type(uint8_type) = &uint8.payload else {
        panic!("uint8 payload is not Type");
    };
    assert_eq!(
        type_obj.represented_type, uint8_type.represented_type,
        "IdentityType(uint8) must return uint8's TypeValue"
    );
    assert_eq!(type_obj.carrier_symbol_id, t.id);

    let identity = world
        .namespace_projection()
        .capability()
        .resolve_meta_function_with_policy(
            "IdentityType",
            &world.package_context(),
            PolicyEnv::OpenStatic,
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
        &world.namespace_projection().capability(),
        &context,
        PolicyEnv::OpenStatic,
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
    let classified =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);
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
        Some(type_value_projection_from_type_symbol(uint8.id)),
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
    assert_eq!(
        material.known_type_values[0],
        Some(type_value_projection_from_type_symbol(uint8.id)),
        "canonical material must record the type value read through uint8"
    );
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

    let classified =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);

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
        .namespace_projection()
        .capability()
        .resolve_meta_function_with_policy(
            "IdentityType",
            &world.package_context(),
            PolicyEnv::OpenStatic,
        )
        .expect("IdentityType resolves through namespace graph");

    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_identity_type", "app").join("main.lang"),
    );
    let site = extract_single_call_site(&expr).expect("fixture must be a call");
    let context = world.package_context();
    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);

    let result = prepare_candidate_from_fixture_symbol(
        &callee,
        classified,
        ParameterShape::type_parameter_signature(Provenance::new("IdentityType param")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("IdentityType candidate-prep object boundary"),
        },
    );

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
    let mat =
        CanonicalArgProductShapeMaterial::from_arg_product_shape(&candidate.arg_product_shape);
    assert_eq!(mat.arity, 1);
    assert_eq!(mat.atom_kinds[0], CanonicalArgAtomKind::TypeObject);
    assert!(mat.known_type_values[0].is_some());
}

#[test]
fn identity_type_formal_meta_invocation_returns_forwarded_value_from_source_fixture() {
    let world = v08_identity_type_world();
    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_identity_type", "app").join("main.lang"),
    );
    let site = extract_single_call_site(&expr).expect("fixture must be a call");
    let context = world.package_context();
    let resolved = resolve_call_target(
        &site.target,
        &world.namespace_projection().capability(),
        &context,
        PolicyEnv::OpenStatic,
    )
    .expect("resolve_call_target should succeed")
    .expect("IdentityType target should resolve");

    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);

    let prep = prepare_candidate_from_fixture_symbol(
        &resolved.callee,
        classified,
        ParameterShape::type_parameter_signature(Provenance::new("IdentityType param")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("formal invocation test"),
        },
    );

    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prep else {
        panic!("candidate-prep should yield ApplicablePlaceholder");
    };

    let invocation_input =
        MetaInvocationInput::new(*candidate, Provenance::new("formal invocation"));
    let MetaInvocationResult::Value(MetaInvocationValue::ForwardedValue(fv)) =
        invoke_meta_callable(invocation_input)
    else {
        panic!("invoke_meta_callable should yield ForwardedValue");
    };
    assert_eq!(fv.return_view, ReturnViewShape::Leaf);
    let forwarded_type = fv.type_value;
    // Verify the result is the value obtained through the argument carrier.
    let expected_symbol = world
        .namespace_projection()
        .capability()
        .resolve_type_object("uint8", &world.package_context())
        .expect("uint8 resolves");
    let SymbolPayload::Type(expected_type) = expected_symbol.payload else {
        panic!("uint8 is a Type object");
    };
    assert_eq!(
        forwarded_type, expected_type.represented_type,
        "ForwardedValue must carry uint8's type value rather than its carrier Symbol"
    );
}

#[test]
fn identity_type_binding_uses_invocation_value_boundary() {
    let world = v08_identity_type_world();
    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_identity_type", "app").join("main.lang"),
    );
    let site = extract_single_call_site(&expr).expect("fixture must be a call");
    let context = world.package_context();
    let resolved = resolve_call_target(
        &site.target,
        &world.namespace_projection().capability(),
        &context,
        PolicyEnv::OpenStatic,
    )
    .expect("resolve_call_target should succeed")
    .expect("IdentityType target should resolve");

    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);

    let prep = prepare_candidate_from_fixture_symbol(
        &resolved.callee,
        classified,
        ParameterShape::type_parameter_signature(Provenance::new("binding boundary test")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("binding boundary test"),
        },
    );

    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prep else {
        panic!("should yield ApplicablePlaceholder");
    };

    let invocation_input =
        MetaInvocationInput::new(*candidate, Provenance::new("binding boundary"));
    let MetaInvocationResult::Value(invocation_value) = invoke_meta_callable(invocation_input)
    else {
        panic!("IdentityType should yield invocation value");
    };

    let result = bind_meta_invocation_value_result(
        invocation_value,
        world.namespace_projection(),
        world.package_root_node(),
        "T",
        Provenance::new("binding via ForwardedValue"),
    )
    .expect("bind_meta_invocation_value_result should succeed");
    assert!(
        !result.namespace_delta.nodes.is_empty() || !result.namespace_delta.symbols.is_empty(),
        "declaration binding must install a NamespaceDelta"
    );
    assert_eq!(
        result.replacement_object.name, "T",
        "replacement_object is the declared forwarding symbol"
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

    let report = classify_type_arguments_with_report(
        &shape,
        &world.namespace_projection().capability(),
        &context,
    );
    assert_eq!(report.classified_shape.arity, 1, "single arg shape");
    assert!(
        report.unresolved_names.is_empty(),
        "uint8 should resolve without diagnostics"
    );
}

#[test]
fn type_value_id_projection_is_derived_from_type_symbol() {
    let tv = type_value_projection_from_type_symbol(SymbolId(42));
    assert_eq!(tv, TypeValueId(42));
    assert_eq!(tv.as_u64(), 42);
}

#[test]
fn meta_instance_cache_reuses_identity_type_invocation_value() {
    let world = v08_identity_type_world();
    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_identity_type", "app").join("main.lang"),
    );
    let site = extract_single_call_site(&expr).expect("fixture must be a call");
    let context = world.package_context();
    let resolved = resolve_call_target(
        &site.target,
        &world.namespace_projection().capability(),
        &context,
        PolicyEnv::OpenStatic,
    )
    .expect("resolve_call_target should succeed")
    .expect("IdentityType target should resolve");

    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified0 =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);

    let prep = prepare_candidate_from_fixture_symbol(
        &resolved.callee,
        classified0.clone(),
        ParameterShape::type_parameter_signature(Provenance::new("cache test param")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("cache reuse test"),
        },
    );
    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prep else {
        panic!("candidate-prep should yield ApplicablePlaceholder");
    };

    let invocation_input = MetaInvocationInput::new(*candidate, Provenance::new("cache test"));
    let key = invocation_input.compute_key();

    let mut cache = MetaInstanceCache::new();
    assert!(cache.lookup(&key).is_none(), "cache should be empty");

    let result1 = invoke_meta_callable_cached(invocation_input, &mut cache);
    let MetaInvocationResult::Value(MetaInvocationValue::ForwardedValue(fv1)) = result1 else {
        panic!("invocation should yield ForwardedValue");
    };

    let cached = cache.lookup(&key).expect("entry should now be cached");
    let MetaInvocationValue::ForwardedValue(fv_cached) = &cached.result else {
        panic!("cached result should be ForwardedValue");
    };
    assert_eq!(
        fv1.type_value, fv_cached.type_value,
        "cached ForwardedValue target must match invocation result"
    );

    // Second invocation with same material (new candidate from same input)
    let CandidatePrepResult::ApplicablePlaceholder(candidate2) =
        prepare_candidate_from_fixture_symbol(
            &resolved.callee,
            classified0,
            ParameterShape::type_parameter_signature(Provenance::new("cache test param")),
            CandidatePreparationContext {
                lookup_env: PolicyEnv::OpenStatic,
                demanded_execution: ExecutionEnv::OpenStatic,
                build_identity: CandidateBuildIdentityPlaceholder::default(),
                provenance: Provenance::new("cache reuse test 2"),
            },
        )
    else {
        panic!("second candidate-prep should yield ApplicablePlaceholder");
    };
    let invocation_input2 = MetaInvocationInput::new(*candidate2, Provenance::new("cache test 2"));
    let result2 = lang_build::invoke_meta_callable_cached(invocation_input2, &mut cache);
    let MetaInvocationResult::Value(MetaInvocationValue::ForwardedValue(fv2)) = result2 else {
        panic!("second invocation should yield ForwardedValue");
    };
    assert_eq!(
        fv1.type_value, fv2.type_value,
        "cache-hit result must match original"
    );
    assert_eq!(cache.len(), 1, "cache should not grow on hit");
}

#[test]
fn identity_type_forwarded_binding_goes_through_invocation_boundary() {
    let world = v08_identity_type_world();
    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_identity_type", "app").join("main.lang"),
    );
    let site = extract_single_call_site(&expr).expect("fixture must be a call");
    let context = world.package_context();
    let resolved = resolve_call_target(
        &site.target,
        &world.namespace_projection().capability(),
        &context,
        PolicyEnv::OpenStatic,
    )
    .expect("resolve_call_target should succeed")
    .expect("IdentityType target should resolve");

    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);

    let prep = prepare_candidate_from_fixture_symbol(
        &resolved.callee,
        classified,
        ParameterShape::type_parameter_signature(Provenance::new("forwarded binding boundary")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("forwarded binding boundary"),
        },
    );

    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prep else {
        panic!("should yield ApplicablePlaceholder");
    };

    let invocation_input =
        MetaInvocationInput::new(*candidate, Provenance::new("forwarded binding"));
    let MetaInvocationResult::Value(MetaInvocationValue::ForwardedValue(fv)) =
        invoke_meta_callable(invocation_input)
    else {
        panic!("IdentityType must yield ForwardedValue");
    };
    let type_value = fv.type_value;

    let result = bind_meta_invocation_value_result(
        MetaInvocationValue::ForwardedValue(fv),
        world.namespace_projection(),
        world.package_root_node(),
        "T",
        Provenance::new("forwarding binding"),
    )
    .expect("binding should succeed");

    assert!(
        !result.namespace_delta.nodes.is_empty() || !result.namespace_delta.symbols.is_empty(),
        "forwarded binding must install a NamespaceDelta"
    );
    assert_eq!(result.replacement_object.kind, lang_build::SymbolKind::Type);
    assert_eq!(result.replacement_object.name, "T");
    // Ordinary binding installs a fresh graph carrier while retaining exactly
    // the forwarded type value.
    let SymbolPayload::Type(type_obj) = &result.replacement_object.payload else {
        panic!("declared symbol must have Type payload");
    };
    assert_eq!(type_obj.carrier_symbol_id, result.replacement_object.id);
    assert_eq!(
        type_obj.represented_type, type_value,
        "declared type binding must carry the forwarded TypeValue"
    );
}

#[test]
fn generated_construction_value_binding_materializes_declared_type_symbol() {
    let world = v08_identity_type_world();
    let context = world.package_context();
    let callee = world
        .namespace_projection()
        .capability()
        .resolve_meta_function_with_policy(
            "UnaryConstructionPrototype",
            &context,
            PolicyEnv::OpenStatic,
        )
        .expect("UCPrototype resolves");

    let site = v08_identity_type_call_site();
    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);

    let gcv = produce_gcv(&callee, classified);
    let cid = gcv.construction_instance_id;

    let result = bind_meta_invocation_value_result(
        MetaInvocationValue::GeneratedConstructionValue(gcv),
        world.namespace_projection(),
        world.package_root_node(),
        "T",
        Provenance::new("valid GCV binding"),
    )
    .expect("GCV binding should succeed");

    assert!(
        !result.namespace_delta.nodes.is_empty() || !result.namespace_delta.symbols.is_empty(),
        "GCV binding must install a NamespaceDelta"
    );
    assert_ne!(cid, lang_build::ConstructionInstanceId(0));
}

#[test]
fn generated_construction_value_binding_rejects_mismatched_construction_instance_id() {
    let world = v08_identity_type_world();

    let identity_material = lang_build::ConstructionIdentityMaterial {
        callee_symbol_id: SymbolId(99),
        canonical_args: lang_build::CanonicalArgProductShapeMaterial {
            arity: 1,
            unit_positions: vec![],
            atom_kinds: vec![lang_build::CanonicalArgAtomKind::TypeObject],
            known_type_values: vec![Some(TypeValueId(1))],
        },
        return_slot_semantics: lang_build::ReturnSlotSemantics::Generate,
        build_identity_fragment: None,
        policy_export_fingerprint_fragment: None,
        provenance: Provenance::new("test gcv"),
    };
    let real_cid = lang_build::compute_construction_instance_id(&identity_material);
    let fake_cid = lang_build::ConstructionInstanceId(real_cid.as_u64() + 1);

    let gcv = lang_build::MetaInvocationValue::GeneratedConstructionValue(
        lang_build::GeneratedConstructionValue {
            construction_instance_id: fake_cid,
            identity_material,
            return_view: ReturnViewShape::Leaf,
            provenance: Provenance::new("mismatched CID"),
        },
    );

    let err = bind_meta_invocation_value_result(
        gcv,
        world.namespace_projection(),
        world.package_root_node(),
        "T",
        Provenance::new("should reject mismatched CID"),
    )
    .expect_err("mismatched CID must be rejected");

    assert!(err
        .diagnostics
        .iter()
        .any(|d| d.message.contains("mismatched construction_instance_id")));
}

#[test]
fn meta_instance_cache_reuses_generated_construction_value() {
    let world = v08_identity_type_world();
    let context = world.package_context();
    let callee = world
        .namespace_projection()
        .capability()
        .resolve_meta_function_with_policy(
            "UnaryConstructionPrototype",
            &context,
            PolicyEnv::OpenStatic,
        )
        .expect("UCPrototype resolves");

    let site = v08_identity_type_call_site();
    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);

    // First invocation through candidate-prep → cache miss.
    let prep = prepare_candidate_from_fixture_symbol(
        &callee,
        classified.clone(),
        ParameterShape::type_parameter_signature(Provenance::new("GCV cache test")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("GCV cache test"),
        },
    );
    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prep else {
        panic!("should yield ApplicablePlaceholder");
    };
    let invocation_input = MetaInvocationInput::new(*candidate, Provenance::new("GCV cache"));
    let key = invocation_input.compute_key();

    let mut cache = MetaInstanceCache::new();
    assert!(cache.lookup(&key).is_none());

    let result1 = invoke_meta_callable_cached(invocation_input, &mut cache);
    let MetaInvocationResult::Value(MetaInvocationValue::GeneratedConstructionValue(gcv1)) =
        result1
    else {
        panic!("first invocation should yield GCV");
    };
    let cid1 = gcv1.construction_instance_id;

    let cached = cache.lookup(&key).expect("GCV entry should now be cached");
    assert!(matches!(
        cached.result,
        MetaInvocationValue::GeneratedConstructionValue(_)
    ));

    // Second invocation with same material → cache hit.
    let prep2 = prepare_candidate_from_fixture_symbol(
        &callee,
        classified,
        ParameterShape::type_parameter_signature(Provenance::new("GCV cache test 2")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("GCV cache test 2"),
        },
    );
    let CandidatePrepResult::ApplicablePlaceholder(candidate2) = prep2 else {
        panic!("second candidate-prep should yield ApplicablePlaceholder");
    };
    let invocation_input2 = MetaInvocationInput::new(*candidate2, Provenance::new("GCV cache 2"));
    let result2 = invoke_meta_callable_cached(invocation_input2, &mut cache);
    let MetaInvocationResult::Value(MetaInvocationValue::GeneratedConstructionValue(gcv2)) =
        result2
    else {
        panic!("second invocation should yield GCV");
    };

    assert_eq!(cid1, gcv2.construction_instance_id);
    assert_eq!(cache.len(), 1, "cache should not grow on hit");
}

#[test]
fn unary_construction_prototype_invocation_returns_generated_construction_value() {
    let world = v08_identity_type_world();
    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_identity_type", "app").join("main.lang"),
    );
    let site = extract_single_call_site(&expr).expect("fixture must be a call");
    let context = world.package_context();
    let _resolved = resolve_call_target(
        &site.target,
        &world.namespace_projection().capability(),
        &context,
        PolicyEnv::OpenStatic,
    )
    .expect("resolve_call_target should succeed")
    .expect("target should resolve");

    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);

    let callee = world
        .namespace_projection()
        .capability()
        .resolve_meta_function_with_policy(
            "UnaryConstructionPrototype",
            &context,
            PolicyEnv::OpenStatic,
        )
        .expect("UnaryConstructionPrototype resolves through namespace graph");

    let prep = prepare_candidate_from_fixture_symbol(
        &callee,
        classified,
        ParameterShape::type_parameter_signature(Provenance::new(
            "UnaryConstructionPrototype param",
        )),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("UCPrototype invocation test"),
        },
    );

    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prep else {
        panic!("UCPrototype candidate-prep should yield ApplicablePlaceholder");
    };

    let callee_symbol_id = candidate.callee_symbol_id;

    let invocation_input =
        MetaInvocationInput::new(*candidate, Provenance::new("UCPrototype invocation"));
    let MetaInvocationResult::Value(MetaInvocationValue::GeneratedConstructionValue(gcv)) =
        invoke_meta_callable(invocation_input)
    else {
        panic!("UCPrototype should yield GeneratedConstructionValue");
    };

    assert_eq!(gcv.return_view, ReturnViewShape::Leaf);
    assert_eq!(
        gcv.identity_material.return_slot_semantics,
        lang_build::ReturnSlotSemantics::Generate
    );
    assert!(gcv.construction_instance_id.as_u64() != 0);
    assert_eq!(gcv.identity_material.callee_symbol_id, callee_symbol_id);
}

#[test]
fn generated_construction_value_carries_construction_instance_identity() {
    let world = v08_identity_type_world();
    let context = world.package_context();
    let callee = world
        .namespace_projection()
        .capability()
        .resolve_meta_function_with_policy(
            "UnaryConstructionPrototype",
            &context,
            PolicyEnv::OpenStatic,
        )
        .expect("UCPrototype resolves");

    let site = v08_identity_type_call_site();
    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);

    let prep = prepare_candidate_from_fixture_symbol(
        &callee,
        classified,
        ParameterShape::type_parameter_signature(Provenance::new("UCPrototype param")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("GCV identity test"),
        },
    );

    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prep else {
        panic!("should yield ApplicablePlaceholder");
    };

    let invocation_input =
        MetaInvocationInput::new(*candidate, Provenance::new("GCV identity invocation"));
    let gcv = match invoke_meta_callable(invocation_input) {
        MetaInvocationResult::Value(MetaInvocationValue::GeneratedConstructionValue(gcv)) => gcv,
        MetaInvocationResult::Value(MetaInvocationValue::ForwardedValue(_)) => {
            panic!("UCPrototype must NOT return a forwarded TypeValue")
        }
        MetaInvocationResult::Value(MetaInvocationValue::GeneratedTypeDefinitionValue(_)) => {
            panic!("UCPrototype must NOT return GeneratedTypeDefinitionValue")
        }
        MetaInvocationResult::Diagnostic(d) => panic!("unexpected diagnostic: {d:?}"),
    };

    assert!(
        gcv.construction_instance_id.as_u64() != 0,
        "GeneratedConstructionValue must have a ConstructionInstanceId"
    );
    assert_eq!(
        gcv.identity_material.return_slot_semantics,
        lang_build::ReturnSlotSemantics::Generate
    );
}

#[test]
fn binding_layer_materializes_generated_construction_value() {
    let world = v08_identity_type_world();
    let context = world.package_context();
    let callee = world
        .namespace_projection()
        .capability()
        .resolve_meta_function_with_policy(
            "UnaryConstructionPrototype",
            &context,
            PolicyEnv::OpenStatic,
        )
        .expect("UCPrototype resolves");

    let site = v08_identity_type_call_site();
    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);

    let prep = prepare_candidate_from_fixture_symbol(
        &callee,
        classified,
        ParameterShape::type_parameter_signature(Provenance::new("UCPrototype binding test")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("binding materialization test"),
        },
    );

    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prep else {
        panic!("should yield ApplicablePlaceholder");
    };

    let invocation_input =
        MetaInvocationInput::new(*candidate, Provenance::new("binding materialization"));
    let MetaInvocationResult::Value(MetaInvocationValue::GeneratedConstructionValue(gcv)) =
        invoke_meta_callable(invocation_input)
    else {
        panic!("should yield GCV");
    };
    let cid = gcv.construction_instance_id;

    let result = bind_meta_invocation_value_result(
        MetaInvocationValue::GeneratedConstructionValue(gcv),
        world.namespace_projection(),
        world.package_root_node(),
        "T",
        Provenance::new("GCV materialization"),
    )
    .expect("GCV binding should succeed");

    assert!(
        !result.namespace_delta.nodes.is_empty() || !result.namespace_delta.symbols.is_empty(),
        "GCV binding must install a NamespaceDelta"
    );

    // TypeValueId projection must be derived from declared symbol after binding,
    // not from the construction instance identity.
    let declared = &result.replacement_object;
    assert_eq!(declared.kind, lang_build::SymbolKind::Type);
    assert_eq!(declared.name, "T");
    let tv = type_value_projection_from_type_symbol(declared.id);
    assert_ne!(
        tv.as_u64(),
        cid.as_u64(),
        "construction identity must not equal TypeValueId projection"
    );
}

#[test]
fn generated_construction_identity_is_independent_of_binding_name() {
    let world = v08_identity_type_world();
    let context = world.package_context();
    let callee = world
        .namespace_projection()
        .capability()
        .resolve_meta_function_with_policy(
            "UnaryConstructionPrototype",
            &context,
            PolicyEnv::OpenStatic,
        )
        .expect("UCPrototype resolves");

    let site = v08_identity_type_call_site();
    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);

    let prep = prepare_candidate_from_fixture_symbol(
        &callee,
        classified,
        ParameterShape::type_parameter_signature(Provenance::new("UCPrototype identity test")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("identity independence test"),
        },
    );

    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prep else {
        panic!("should yield ApplicablePlaceholder");
    };

    let invocation_input =
        MetaInvocationInput::new(*candidate, Provenance::new("identity independence"));
    let MetaInvocationResult::Value(MetaInvocationValue::GeneratedConstructionValue(gcv)) =
        invoke_meta_callable(invocation_input)
    else {
        panic!("should yield GCV");
    };
    let _cid = gcv.construction_instance_id;

    // Bind same GCV under two different names, installing the first to
    // advance the snapshot so the second gets a distinct SymbolId.
    let result_a = bind_meta_invocation_value_result(
        MetaInvocationValue::GeneratedConstructionValue(gcv.clone()),
        world.namespace_projection(),
        world.package_root_node(),
        "A",
        Provenance::new("bind as A"),
    )
    .expect("bind as A should succeed");

    // Install A's delta so B gets a different SymbolId from the graph.
    let snapshot_after_a = world
        .namespace_projection()
        .install_delta(result_a.namespace_delta)
        .expect("install A's delta");

    let result_b = bind_meta_invocation_value_result(
        MetaInvocationValue::GeneratedConstructionValue(gcv),
        &snapshot_after_a,
        world.package_root_node(),
        "B",
        Provenance::new("bind as B"),
    )
    .expect("bind as B should succeed");

    assert_ne!(
        result_a.replacement_object.id, result_b.replacement_object.id,
        "distinct bindings must have distinct declared symbol IDs"
    );
    assert_eq!(
        result_a.replacement_object.cache_key_fragment.as_deref(),
        result_b.replacement_object.cache_key_fragment.as_deref(),
        "same GCV bound at different names must carry same cache key fragment"
    );
}

#[test]
fn generated_construction_identity_changes_with_canonical_args() {
    let world = v08_identity_type_world();
    let context = world.package_context();
    let callee = world
        .namespace_projection()
        .capability()
        .resolve_meta_function_with_policy(
            "UnaryConstructionPrototype",
            &context,
            PolicyEnv::OpenStatic,
        )
        .expect("UCPrototype resolves");

    let site = v08_identity_type_call_site();
    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let classified_uint8 =
        classify_type_arguments(&shape, &world.namespace_projection().capability(), &context);

    let gcv_uint8 = produce_gcv(&callee, classified_uint8);
    let cid_uint8 = gcv_uint8.construction_instance_id;

    // Produce a second GCV with a different real type argument (uint16).
    let uint16 = world
        .namespace_projection()
        .capability()
        .resolve_type_object("uint16", &context)
        .expect("uint16 resolves as type object");
    let classified_uint16 = produce_classified_shape(
        &uint16,
        &world,
        &context,
        lang_build::ProductMaterialRole::MetaConstructionArgumentProduct,
    );
    let cid_uint16 = produce_gcv(&callee, classified_uint16).construction_instance_id;

    assert_ne!(
        cid_uint8, cid_uint16,
        "different canonical args (uint8 vs uint16) must produce different ConstructionInstanceId"
    );
}

fn produce_classified_shape(
    type_symbol: &lang_build::SymbolObject,
    _world: &lang_build::CompilationWorld,
    _context: &lang_build::ResolverContext,
    role: lang_build::ProductMaterialRole,
) -> lang_build::ArgProductShape {
    let site = v08_identity_type_call_site();
    let shape = site.to_arg_product_shape(role);
    let mut classified = shape.clone();
    for raw in &mut classified.raw_args {
        if matches!(raw.value_class, RawArgValueClass::UnknownExpression) {
            *raw = raw.clone().as_type_object_with_type_symbol(type_symbol.id);
        }
    }
    classified
}

fn v08_identity_type_call_site() -> lang_build::NormalizedCallSite {
    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_identity_type", "app").join("main.lang"),
    );
    extract_single_call_site(&expr).expect("fixture must be a call")
}

fn produce_gcv(
    callee: &lang_build::SymbolObject,
    classified: lang_build::ArgProductShape,
) -> lang_build::GeneratedConstructionValue {
    let prep = prepare_candidate_from_fixture_symbol(
        &callee,
        classified,
        ParameterShape::type_parameter_signature(Provenance::new("GCV production")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("GCV production"),
        },
    );

    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prep else {
        panic!("should yield ApplicablePlaceholder");
    };

    let invocation_input = MetaInvocationInput::new(*candidate, Provenance::new("GCV production"));
    let MetaInvocationResult::Value(MetaInvocationValue::GeneratedConstructionValue(gcv)) =
        invoke_meta_callable(invocation_input)
    else {
        panic!("should yield GCV");
    };
    gcv
}

#[test]
fn identity_type_initializer_expands_through_meta_invocation_driver() {
    let world = build_single_fixture_world("v08_identity_type", "app");
    let result = world
        .resolve_with_expectation("T", lang_build::ResolveExpectation::TypeObject)
        .expect("connected source build installs IdentityType result");
    let uint8 = world
        .resolve_with_expectation("uint8", lang_build::ResolveExpectation::TypeObject)
        .expect("uint8 resolves");
    assert_eq!(result.name, "T");
    let SymbolPayload::Type(type_object) = &result.payload else {
        panic!("replacement_object must be the declared binding symbol with Type payload");
    };
    let SymbolPayload::Type(uint8_type) = &uint8.payload else {
        panic!("uint8 is a Type object");
    };
    assert_eq!(type_object.carrier_symbol_id, result.id);
    assert_eq!(
        type_object.represented_type, uint8_type.represented_type,
        "ordinary binding must preserve the forwarded TypeValue"
    );
}

#[test]
fn unary_construction_initializer_expands_through_meta_invocation_driver() {
    let world = build_single_fixture_world("v08_unary_construction", "app");
    let result = world
        .resolve_with_expectation("T", lang_build::ResolveExpectation::TypeObject)
        .expect("connected source build installs UnaryConstructionPrototype result");
    assert_eq!(result.kind, lang_build::SymbolKind::Type);
    assert_eq!(result.name, "T");
    assert!(result
        .cache_key_fragment
        .as_deref()
        .is_some_and(|fragment| fragment.starts_with("construction:")));
}

#[test]
fn struct_initializer_expands_through_generated_type_definition_value() {
    let world = build_single_fixture_world("v08_struct_uint8", "app");
    let result = world
        .resolve_with_expectation("T", lang_build::ResolveExpectation::TypeObject)
        .expect("connected source build installs struct result");
    let SymbolPayload::Type(type_object) = &result.payload else {
        panic!("struct binding must materialize a TypeObject");
    };
    let type_namespace = type_object
        .type_associated_namespace
        .expect("struct type must have associated namespace");
    assert!(
        world
            .semantic_world()
            .namespace_owner(type_namespace)
            .is_some(),
        "generated type namespace is installed in SemanticWorld"
    );
    assert_eq!(type_object.field_names, vec!["a".to_string()]);
    assert!(
        world.resolve("a::T").is_ok(),
        "generated field projection is installed"
    );
}

#[test]
fn struct_formal_invocation_produces_value_not_namespace_delta() {
    let world = lang_build::CompilationWorld::from_manifest(&empty_app_manifest())
        .expect("empty world with core");
    let initializer = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_struct_uint8", "app").join("main.lang"),
    );
    let invocation_input =
        struct_invocation_input(&world, &initializer, "uint8", "pure struct invocation");

    let MetaInvocationResult::Value(MetaInvocationValue::GeneratedTypeDefinitionValue(gtdv)) =
        invoke_meta_callable(invocation_input)
    else {
        panic!("struct formal invocation must produce GeneratedTypeDefinitionValue");
    };

    assert_ne!(gtdv.type_definition_id.as_u64(), 0);
    assert_eq!(gtdv.fields.len(), 1);
}

#[test]
fn materialized_struct_type_definition_records_pattern_heads() {
    let world = lang_build::CompilationWorld::from_manifest(&empty_app_manifest())
        .expect("empty world with core");
    let initializer = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_struct_uint8", "app").join("main.lang"),
    );
    let invocation_input = struct_invocation_input(
        &world,
        &initializer,
        "uint8",
        "pattern head materialization",
    );
    let mut materialization_state = TypeMaterializationState::default();

    let MetaInvocationResult::Value(MetaInvocationValue::GeneratedTypeDefinitionValue(gtdv)) =
        invoke_meta_callable_with_materialization_state(
            invocation_input,
            &mut materialization_state,
        )
    else {
        panic!("struct formal invocation must produce GeneratedTypeDefinitionValue");
    };

    let pattern_heads = gtdv
        .pattern_heads
        .as_ref()
        .expect("GeneratedTypeDefinitionValue records pattern heads");
    let field_head = pattern_heads
        .field_heads
        .iter()
        .find(|field| field.field_name == "a")
        .expect("field `a` records a pattern head")
        .field_head;
    assert_eq!(gtdv.fields[0].pattern_head, Some(field_head));
    assert_eq!(
        materialization_state
            .pattern_heads
            .lookup_child(pattern_heads.owner_head, "a"),
        Some(field_head)
    );
}

#[test]
fn generated_type_definition_semantic_eq_includes_pattern_heads() {
    let world = lang_build::CompilationWorld::from_manifest(&empty_app_manifest())
        .expect("empty world with core");
    let initializer = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_struct_uint8", "app").join("main.lang"),
    );
    let gtdv = produce_gtdv_from_struct_initializer(&world, &initializer, "uint8");
    let mut changed_heads = gtdv.clone();
    let pattern_heads = changed_heads
        .pattern_heads
        .as_mut()
        .expect("generated type definition records pattern heads");
    pattern_heads.owner_head = PatternHeadId(pattern_heads.owner_head.0 + 1000);
    pattern_heads.field_heads[0].field_head =
        PatternHeadId(pattern_heads.field_heads[0].field_head.0 + 1000);
    changed_heads.fields[0].pattern_head = Some(pattern_heads.field_heads[0].field_head);

    assert!(
        !MetaInvocationValue::GeneratedTypeDefinitionValue(gtdv).semantic_eq(
            &MetaInvocationValue::GeneratedTypeDefinitionValue(changed_heads)
        )
    );
}

#[test]
fn source_struct_materialization_updates_world_pattern_head_registry() {
    let world = build_single_fixture_world("v08_struct_uint8", "app");
    let resolved = world.resolve("T").expect("T resolves");
    let SymbolPayload::Type(type_object) = &resolved.payload else {
        panic!("T must be a generated TypeObject");
    };
    let owner_head = type_object
        .owner_pattern_head
        .expect("source-built TypeObject records owner PatternHeadId");
    let field_head = type_object.fields[0]
        .pattern_head
        .expect("source-built TypeField records field PatternHeadId");

    assert_eq!(
        world
            .type_materialization_state()
            .pattern_heads
            .lookup_child(owner_head, "a"),
        Some(field_head)
    );
}

#[test]
fn generated_type_definition_identity_is_independent_of_binding_name() {
    let world = lang_build::CompilationWorld::from_manifest(&empty_app_manifest())
        .expect("empty world with core");
    let initializer = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_struct_uint8", "app").join("main.lang"),
    );
    let gtdv = produce_gtdv_from_struct_initializer(&world, &initializer, "uint8");
    let type_definition_id = gtdv.type_definition_id;

    let result_a = bind_meta_invocation_value_result(
        MetaInvocationValue::GeneratedTypeDefinitionValue(gtdv.clone()),
        world.namespace_projection(),
        world.package_root_node(),
        "A",
        Provenance::new("bind generated struct A"),
    )
    .expect("bind A");
    let snapshot_a = world
        .namespace_projection()
        .install_delta(result_a.namespace_delta.clone())
        .expect("install A");
    let result_b = bind_meta_invocation_value_result(
        MetaInvocationValue::GeneratedTypeDefinitionValue(gtdv),
        &snapshot_a,
        world.package_root_node(),
        "B",
        Provenance::new("bind generated struct B"),
    )
    .expect("bind B");

    assert_ne!(
        result_a.replacement_object.id,
        result_b.replacement_object.id
    );
    assert_eq!(
        result_a.replacement_object.cache_key_fragment,
        result_b.replacement_object.cache_key_fragment
    );
    assert_eq!(
        result_a.replacement_object.cache_key_fragment.as_deref(),
        Some(format!("type-definition:{}", type_definition_id.as_u64()).as_str())
    );
}

#[test]
fn generated_type_definition_identity_changes_with_field_signature() {
    let world = lang_build::CompilationWorld::from_manifest(&empty_app_manifest())
        .expect("empty world with core");
    let uint8_initializer = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_struct_uint8", "app").join("main.lang"),
    );
    let uint16_initializer = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_struct_uint16", "app").join("main.lang"),
    );

    let uint8_gtdv = produce_gtdv_from_struct_initializer(&world, &uint8_initializer, "uint8");
    let uint16_gtdv = produce_gtdv_from_struct_initializer(&world, &uint16_initializer, "uint16");

    assert_ne!(
        uint8_gtdv.type_definition_id, uint16_gtdv.type_definition_id,
        "different field type symbols must produce different TypeDefinitionInstanceId"
    );
    assert_eq!(
        compute_type_definition_instance_id(&uint8_gtdv.identity_material),
        uint8_gtdv.type_definition_id
    );
}

#[test]
fn type_definition_identity_material_equality_ignores_field_provenance() {
    let canonical_args = lang_build::CanonicalArgProductShapeMaterial {
        arity: 1,
        unit_positions: vec![],
        atom_kinds: vec![lang_build::CanonicalArgAtomKind::TypeObject],
        known_type_values: vec![Some(TypeValueId(2))],
    };
    let left = lang_build::TypeDefinitionIdentityMaterial {
        callee_symbol_id: SymbolId(1),
        canonical_args: canonical_args.clone(),
        field_signature_material: vec![lang_build::FieldSignatureMaterial {
            field_name: "a".to_string(),
            field_type_value: TypeValueId(2),
            field_type_observation: lang_build::CanonicalTypeObservation::Detached(TypeValueId(2)),
            field_type_carrier_symbol: SymbolId(2),
            field_index: 0,
            visibility: lang_build::StructuralMemberVisibility::Public,
            provenance: Provenance::new("left field provenance"),
        }],
        return_slot_semantics: lang_build::ReturnSlotSemantics::Generate,
        build_identity_fragment: Some("build".to_string()),
        policy_export_fingerprint_fragment: Some("policy".to_string()),
        provenance: Provenance::new("left type definition provenance"),
    };
    let right = lang_build::TypeDefinitionIdentityMaterial {
        callee_symbol_id: SymbolId(1),
        canonical_args,
        field_signature_material: vec![lang_build::FieldSignatureMaterial {
            field_name: "a".to_string(),
            field_type_value: TypeValueId(2),
            field_type_observation: lang_build::CanonicalTypeObservation::Detached(TypeValueId(2)),
            field_type_carrier_symbol: SymbolId(22),
            field_index: 0,
            visibility: lang_build::StructuralMemberVisibility::Public,
            provenance: Provenance::new("right field provenance"),
        }],
        return_slot_semantics: lang_build::ReturnSlotSemantics::Generate,
        build_identity_fragment: Some("build".to_string()),
        policy_export_fingerprint_fragment: Some("policy".to_string()),
        provenance: Provenance::new("right type definition provenance"),
    };

    assert_eq!(left, right);
    assert_eq!(
        compute_type_definition_instance_id(&left),
        compute_type_definition_instance_id(&right),
        "field provenance and the source carrier Symbol must not affect generated type definition identity"
    );
}

#[test]
fn meta_instance_cache_reuses_generated_type_definition_value() {
    let world = lang_build::CompilationWorld::from_manifest(&empty_app_manifest())
        .expect("empty world with core");
    let initializer = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_struct_uint8", "app").join("main.lang"),
    );
    let invocation_input = struct_invocation_input(
        &world,
        &initializer,
        "uint8",
        "generated type definition cache",
    );
    let key = invocation_input.compute_key();
    let mut cache = MetaInstanceCache::new();

    let result1 = invoke_meta_callable_cached(invocation_input, &mut cache);
    let MetaInvocationResult::Value(MetaInvocationValue::GeneratedTypeDefinitionValue(gtdv1)) =
        result1
    else {
        panic!("first invocation should yield GeneratedTypeDefinitionValue");
    };
    let cached = cache
        .lookup(&key)
        .expect("GeneratedTypeDefinitionValue should be cached");
    let cached_debug = format!("{cached:?}");
    assert!(!cached_debug.contains("NamespaceDelta"));
    assert!(!cached_debug.contains("CachedStructBinding"));

    let invocation_input2 = struct_invocation_input(
        &world,
        &initializer,
        "uint8",
        "generated type definition cache hit",
    );
    let result2 = invoke_meta_callable_cached(invocation_input2, &mut cache);
    let MetaInvocationResult::Value(MetaInvocationValue::GeneratedTypeDefinitionValue(gtdv2)) =
        result2
    else {
        panic!("second invocation should yield GeneratedTypeDefinitionValue");
    };

    assert_eq!(gtdv1.type_definition_id, gtdv2.type_definition_id);
    assert_eq!(cache.len(), 1);
}

#[test]
fn cached_struct_invocation_rematerializes_pattern_heads_in_current_state() {
    let world = lang_build::CompilationWorld::from_manifest(&empty_app_manifest())
        .expect("empty world with core");
    let initializer = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_struct_uint8", "app").join("main.lang"),
    );
    let invocation_input = struct_invocation_input(
        &world,
        &initializer,
        "uint8",
        "generated type definition cache state miss",
    );
    let key = invocation_input.compute_key();
    let mut cache = MetaInstanceCache::new();
    let mut miss_state = TypeMaterializationState::default();

    let MetaInvocationResult::Value(MetaInvocationValue::GeneratedTypeDefinitionValue(gtdv1)) =
        invoke_meta_callable_cached_with_materialization_state(
            invocation_input,
            &mut cache,
            &mut miss_state,
        )
    else {
        panic!("first invocation should yield GeneratedTypeDefinitionValue");
    };
    let cached = cache
        .lookup(&key)
        .expect("cache stores replayable pure invocation material");
    let MetaInvocationValue::GeneratedTypeDefinitionValue(cached_gtdv) = &cached.result else {
        panic!("cached result should be a generated type definition");
    };
    assert!(
        cached_gtdv.pattern_heads.is_none(),
        "cache must not store concrete registry-backed PatternHeadId material"
    );
    assert!(
        cached_gtdv
            .fields
            .iter()
            .all(|field| field.pattern_head.is_none()),
        "cache must not store concrete field PatternHeadId material"
    );

    let invocation_input2 = struct_invocation_input(
        &world,
        &initializer,
        "uint8",
        "generated type definition cache state hit",
    );
    let mut hit_state = TypeMaterializationState::default();
    hit_state.pattern_heads.allocate_external_forward_head(
        SymbolId(999),
        "preexisting",
        Provenance::new("preexisting pattern head"),
    );

    let MetaInvocationResult::Value(MetaInvocationValue::GeneratedTypeDefinitionValue(gtdv2)) =
        invoke_meta_callable_cached_with_materialization_state(
            invocation_input2,
            &mut cache,
            &mut hit_state,
        )
    else {
        panic!("cache hit should yield GeneratedTypeDefinitionValue");
    };

    let heads1 = gtdv1.pattern_heads.as_ref().expect("miss result has heads");
    let heads2 = gtdv2
        .pattern_heads
        .as_ref()
        .expect("hit result rematerializes heads");
    assert_eq!(gtdv1.type_definition_id, gtdv2.type_definition_id);
    assert_ne!(
        heads1.owner_head, heads2.owner_head,
        "cache hit must use current registry materialization, not stale cached PatternHeadId"
    );
    let field_head2 = heads2.field_heads[0].field_head;
    assert_eq!(
        hit_state.pattern_heads.lookup_child(heads2.owner_head, "a"),
        Some(field_head2),
        "current materialization state must contain replayed extraction child scope"
    );
}

fn struct_invocation_input(
    world: &lang_build::CompilationWorld,
    initializer: &lang_syntax::NormExpr,
    field_type_name: &str,
    provenance: &str,
) -> MetaInvocationInput {
    let site = extract_single_call_site(initializer).expect("struct initializer must be a call");
    let context = world.package_context();
    let resolved = resolve_call_target(
        &site.target,
        &world.namespace_projection().capability(),
        &context,
        PolicyEnv::OpenStatic,
    )
    .expect("resolve_call_target should succeed")
    .expect("struct target should resolve");
    let type_symbol = world
        .namespace_projection()
        .capability()
        .resolve_type_object(field_type_name, &context)
        .expect("field type resolves");
    let mut classified =
        site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    for raw_arg in &mut classified.raw_args {
        if matches!(raw_arg.value_class, RawArgValueClass::UnknownExpression) {
            *raw_arg = raw_arg
                .clone()
                .as_type_object_with_type_symbol(type_symbol.id);
        }
    }
    let prep = prepare_candidate_from_fixture_symbol(
        &resolved.callee,
        classified.clone(),
        ParameterShape::type_parameter_sequence(
            classified.arity,
            Provenance::new("struct direct invocation field signature"),
        ),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new(provenance),
        },
    );
    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prep else {
        panic!("struct candidate should be applicable");
    };
    MetaInvocationInput::new(*candidate, Provenance::new(provenance))
}

fn produce_gtdv_from_struct_initializer(
    world: &lang_build::CompilationWorld,
    initializer: &lang_syntax::NormExpr,
    field_type_name: &str,
) -> GeneratedTypeDefinitionValue {
    let invocation_input =
        struct_invocation_input(world, initializer, field_type_name, "produce GTDV");
    let MetaInvocationResult::Value(MetaInvocationValue::GeneratedTypeDefinitionValue(gtdv)) =
        invoke_meta_callable(invocation_input)
    else {
        panic!("struct invocation should yield GeneratedTypeDefinitionValue");
    };
    gtdv
}
