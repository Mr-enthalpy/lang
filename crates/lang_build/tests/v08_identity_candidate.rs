mod support;

use support::*;

use lang_build::{
    policy_metadata, policy_set_meta_runtime, policy_set_runtime, prepare_meta_callable_candidate,
    AliasChain, AliasQueryDisposition, AliasQueryMode, CallablePolicyMetadata,
    CandidateBuildIdentityPlaceholder, CandidatePrepDeferredReason, CandidatePrepResult,
    CandidatePreparationContext, CompilationWorld, ExecutionEnv, FieldObject, FieldProjection,
    NamespaceGraphSnapshot, NamespaceNode, NamespaceNodeKind, ParameterShape, PlaceId, PolicyEnv,
    PolicyFlag, Provenance, RawArgValueClass, SourceCategory, SymbolId, SymbolKind, SymbolObject,
    SymbolPayload, TypeValueBindingPlaceholder, TypeValueId,
};
use lang_syntax::{NormExpr, NormOrigin, NormProduct, NormProductElem, Span};

#[test]
fn type_value_binding_keeps_symbol_place_and_type_value_distinct() {
    let binding = TypeValueBindingPlaceholder::new(
        SymbolId(1),
        PlaceId(10),
        TypeValueId(20),
        Provenance::new("let T: type = uint8 placeholder"),
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
fn alias_chain_forwards_without_creating_a_fresh_writable_place() {
    let alias = AliasChain::new(
        SymbolId(2),
        SymbolId(3),
        Provenance::new("let T === uint8 placeholder"),
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
fn candidate_prep_requires_graph_resolved_symbolobject_and_arg_product_shape() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let callee = world
        .snapshot()
        .capability()
        .resolve_meta_function_with_policy("struct", &world.package_context(), PolicyEnv::Meta)
        .expect("core struct resolves through namespace graph as SymbolObject");

    let arg_shape = one_expression_arg_shape();
    let result = prepare_meta_callable_candidate(
        &callee,
        arg_shape,
        ParameterShape::exact_arity(1, Provenance::new("single type constructor parameter")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::Meta,
            demanded_execution: ExecutionEnv::Meta,
            build_identity: CandidateBuildIdentityPlaceholder {
                package_identity_fragment: Some("package:app".to_string()),
                mount_identity_fragment: Some("mount:core".to_string()),
                build_config_fingerprint_fragment: Some("build:debug-test".to_string()),
                policy_export_fingerprint_fragment: Some("policy:export-meta".to_string()),
            },
            provenance: Provenance::new("v0.8 candidate preparation"),
        },
    );

    let CandidatePrepResult::ApplicablePlaceholder(candidate) = result else {
        panic!("core struct should reach the applicable placeholder boundary");
    };
    assert_eq!(candidate.callee_symbol_id, callee.id);
    assert_eq!(candidate.arg_product_shape.arity, 1);
    assert_eq!(candidate.arg_product_shape.raw_args[0].is_value(), None);
    assert!(matches!(
        candidate.arg_product_shape.raw_args[0].value_class,
        RawArgValueClass::UnknownExpression
    ));
    // Candidate-prep must not insert pass actions before value/type/rank/meta/
    // pattern classification; UnknownExpression may become a value later.
    assert!(!candidate.arg_product_shape.raw_args[0].receives_automatic_pass_action());
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
        Some("build:debug-test")
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
fn symbol_visibility_does_not_imply_body_entry_or_return_object_policy() {
    let field_symbol = runtime_only_field_function_symbol();
    let result = prepare_meta_callable_candidate(
        &field_symbol,
        one_expression_arg_shape(),
        ParameterShape::exact_arity(1, Provenance::new("field parameter placeholder")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::Meta,
            demanded_execution: ExecutionEnv::Meta,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("meta-visible runtime-body field function"),
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
}

#[test]
fn canonical_key_seed_reserves_canonical_argument_product_slots() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let callee = world
        .snapshot()
        .capability()
        .resolve_meta_function_with_policy("struct", &world.package_context(), PolicyEnv::Meta)
        .expect("core struct resolves through namespace graph as SymbolObject");
    let product = NormProduct {
        elements: vec![
            NormProductElem::Expr(NormExpr::Name {
                text: "T".to_string(),
                origin: origin(1),
            }),
            NormProductElem::Unit { origin: origin(2) },
        ],
        origin: origin(3),
    };
    let arg_shape = lang_build::ProductObject::from_norm_product(
        product,
        lang_build::ProductMaterialRole::MetaConstructionArgumentProduct,
    )
    .to_arg_product_shape();

    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prepare_meta_callable_candidate(
        &callee,
        arg_shape,
        ParameterShape::exact_arity(2, Provenance::new("unit-sensitive parameter placeholder")),
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
}

#[test]
fn failed_v08_generated_delta_installs_no_partial_subtree() {
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

fn one_expression_arg_shape() -> lang_build::ArgProductShape {
    let product = NormProduct {
        elements: vec![NormProductElem::Expr(NormExpr::Name {
            text: "T".to_string(),
            origin: origin(1),
        })],
        origin: origin(2),
    };
    lang_build::ProductObject::from_norm_product(
        product,
        lang_build::ProductMaterialRole::MetaConstructionArgumentProduct,
    )
    .to_arg_product_shape()
}

fn runtime_only_field_function_symbol() -> SymbolObject {
    let mut symbol = SymbolObject::placeholder(
        SymbolId(100),
        "field",
        SymbolKind::FieldFunction,
        SourceCategory::GeneratedChild,
        None,
        Provenance::new("meta-visible runtime-body field function"),
    );
    symbol.policy_metadata.policy_set = policy_set_meta_runtime();
    symbol.payload = SymbolPayload::FieldFunction(FieldObject {
        owner_type_symbol_id: SymbolId(101),
        field_name: "field".to_string(),
        field_type_symbol_id: SymbolId(102),
        projection: FieldProjection::Ref,
        callable_policy: CallablePolicyMetadata {
            body_entry_policy: policy_metadata(policy_set_runtime()),
            return_object_policy: policy_metadata(policy_set_runtime()),
        },
        provenance: Provenance::new("field callable payload"),
    });
    symbol
}

fn origin(index: usize) -> NormOrigin {
    NormOrigin::Source(Span::new(index, index + 1, 1, index + 1))
}
