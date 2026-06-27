mod support;

use std::fs;

use support::*;

use lang_build::{
    prepare_meta_callable_candidate, AliasChain, AliasQueryDisposition, AliasQueryMode,
    CandidateBuildIdentityPlaceholder, CandidatePrepDeferredReason, CandidatePrepResult,
    CandidatePreparationContext, CompilationWorld, ExecutionEnv, NamespaceGraphSnapshot,
    NamespaceNode, NamespaceNodeKind, ParameterShape, PlaceId, PolicyEnv, PolicyFlag, Provenance,
    RawArgValueClass, SourceCategory, SymbolId, TypeValueBindingPlaceholder, TypeValueId,
};
use lang_syntax::{NormDecl, NormExpr, NormForm, NormProduct};

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
fn candidate_prep_uses_graph_resolved_symbolobject_and_arg_product_shape_from_source_fixture() {
    let world = v08_candidate_world();
    let callee = world
        .snapshot()
        .capability()
        .resolve_meta_function_with_policy("struct", &world.package_context(), PolicyEnv::Meta)
        .expect("core struct resolves through namespace graph as SymbolObject");

    let arg_shape = candidate_fixture_arg_shape();
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
            provenance: Provenance::new("v0.8 source-fixture candidate preparation"),
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
fn generated_field_function_from_source_fixture_keeps_policy_planes_separate() {
    let world = v08_candidate_world();
    let field_symbol = world
        .snapshot()
        .capability()
        .resolve_field_function("field::ref::T", &world.package_context())
        .expect("generated ref field function resolves through namespace graph");
    let result = prepare_meta_callable_candidate(
        &field_symbol,
        candidate_fixture_arg_shape(),
        ParameterShape::exact_arity(1, Provenance::new("field parameter placeholder")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::Meta,
            demanded_execution: ExecutionEnv::Meta,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("source-fixture generated field function"),
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
fn canonical_key_seed_reserves_canonical_argument_product_slots_from_source_fixture() {
    let product =
        product_fixture_call_source("product_unit_preservation.lang", "v08 canonical product");
    let arg_shape = lang_build::ProductObject::from_norm_product(
        product,
        lang_build::ProductMaterialRole::MetaConstructionArgumentProduct,
    )
    .to_arg_product_shape();
    let world = v08_candidate_world();
    let callee = world
        .snapshot()
        .capability()
        .resolve_meta_function_with_policy("struct", &world.package_context(), PolicyEnv::Meta)
        .expect("core struct resolves through namespace graph as SymbolObject");

    let CandidatePrepResult::ApplicablePlaceholder(candidate) = prepare_meta_callable_candidate(
        &callee,
        arg_shape,
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

fn v08_candidate_world() -> CompilationWorld {
    build_single_fixture_world("v08_candidate", "app")
}

fn candidate_fixture_arg_shape() -> lang_build::ArgProductShape {
    lang_build::ProductObject::from_norm_product(
        candidate_fixture_call_source(),
        lang_build::ProductMaterialRole::MetaConstructionArgumentProduct,
    )
    .to_arg_product_shape()
}

fn candidate_fixture_call_source() -> NormProduct {
    let path = fixture_source_root("v08_candidate", "app").join("main.lang");
    let source = fs::read_to_string(path).expect("read v0.8 candidate fixture");
    let parsed = lang_syntax::parse(&source);
    assert!(
        parsed.diagnostics.is_empty(),
        "unexpected parse diagnostics:\n{}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    let normalized = lang_syntax::normalize_program(&parsed.program);
    match normalized.forms.as_slice() {
        [NormForm::Let(NormDecl::Let { slot, .. })] => match slot.initializer.as_deref() {
            Some(NormExpr::Call { source, .. }) => source.clone(),
            other => panic!("expected candidate fixture initializer call, got {other:#?}"),
        },
        other => panic!("expected one let declaration in candidate fixture, got {other:#?}"),
    }
}

fn product_fixture_call_source(name: &str, provenance: &str) -> NormProduct {
    let path = fixture_root().join("v08").join(name);
    let source = fs::read_to_string(path).expect("read v0.8 product fixture");
    let parsed = lang_syntax::parse(&source);
    assert!(
        parsed.diagnostics.is_empty(),
        "unexpected parse diagnostics:\n{}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    let normalized = lang_syntax::normalize_program(&parsed.program);
    match normalized.forms.as_slice() {
        [NormForm::Expr(NormExpr::Call { source, .. })] => source.clone(),
        other => panic!("expected one normalized call expression for {provenance}, got {other:#?}"),
    }
}
