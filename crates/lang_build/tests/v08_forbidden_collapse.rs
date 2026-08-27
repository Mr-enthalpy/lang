//! Forbidden-collapse tests: prevent future implementations from compressing
//! the v0.8 substrate boundary objects into simpler-but-incorrect shapes.
//!
//! These tests verify structural separations that must hold until the full
//! generic type system, overload resolver, pattern engine, and meta invocation
//! engine are implemented.

mod support;

use support::*;

use lang_build::{
    compute_legacy_meta_instance_digest, ArgProductShape, CandidateBuildIdentityPlaceholder,
    CandidatePrepDeferredReason, CandidatePrepResult, CandidatePreparationContext,
    CanonicalArgAtomKind, ExecutionEnv, FlattenedProductInvariant, FlattenedProductObject,
    ForwardedValue, MetaInstanceCache, MetaInvocationInput, MetaInvocationValue, NonValueArgKind,
    ParameterShape, PolicyEnv, PreparedCallableCandidate, ProductAtom, ProductMaterialRole,
    Provenance, RawArgShape, RawArgValueClass, ReturnViewShape, SymbolId, TypeValueId,
};

/// Unit positions must remain in the canonical argument material and not be
/// collapsed into arity-only or arity-plus-type-symbols-only data.
///
/// Future implementations must not claim that the canonical key depends only
/// on arity and type-value list without also recording where Units sit.
#[test]
fn canonical_arg_material_does_not_collapse_unit_positions() {
    let shape = fixture_arg_product_shape(
        "product_unit_preservation.lang",
        ProductMaterialRole::MetaConstructionArgumentProduct,
    );
    let material = lang_build::CanonicalArgProductShapeMaterial::from_arg_product_shape(&shape);

    assert_eq!(material.arity, 3);
    assert_eq!(
        material.unit_positions,
        vec![1],
        "unit position must be preserved in canonical material"
    );
    assert_eq!(
        material.atom_kinds[1],
        CanonicalArgAtomKind::ProductUnit,
        "ProductUnit must be recorded as its own atom kind"
    );
    assert!(
        material.atom_kinds.len() == material.arity as usize,
        "atom_kinds length must match arity"
    );
}

/// CandidatePrepResult is before formal meta invocation.
///
/// The enum variants (ApplicablePlaceholder, Deferred, Diagnostic) must not
/// be mistaken for InvocationResult or MetaExpansionResult. This test
/// confirms that candidate-prep may defer on body-entry policy without
/// returning a meta execution result.
#[test]
fn candidate_prep_does_not_execute_meta_invocation() {
    let world = v08_candidate_world();
    let field_symbol = world
        .namespace_projection()
        .capability()
        .resolve_field_function("field::ref::T", &world.package_context())
        .expect("generated ref field function resolves through namespace graph");

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
            provenance: Provenance::new("forbidden collapse: candidate prep != meta invocation"),
        },
    );

    match &result {
        CandidatePrepResult::Deferred { reason, .. } => {
            assert_eq!(
                *reason,
                CandidatePrepDeferredReason::BodyEntryPolicyMismatch
            );
        }
        CandidatePrepResult::ApplicablePlaceholder(_) => {
            panic!("meta execution on runtime-only body must defer, not apply")
        }
        CandidatePrepResult::Diagnostic(_) => {
            panic!("meta execution on runtime-only body must defer, not diagnose")
        }
    }
    // Confirm CandidatePrepResult is NOT InvocationResult / MetaExpansionResult
    // (compile-time type guarantee; runtime assertion above proves behavior).
}

/// Product flattening must not cross Expression barriers.
///
/// A call used as a product element must remain an opaque Expression atom,
/// not expose its inner source product.
#[test]
fn product_shape_does_not_cross_expression_barrier() {
    let barrier_shape = fixture_arg_product_shape(
        "product_expression_barrier.lang",
        ProductMaterialRole::CallableArgumentProduct,
    );

    assert_eq!(
        barrier_shape.flattened.atoms.len(),
        2,
        "barrier product ((a, b) |> f, c) must yield two atoms, not three"
    );
    // The first atom must be an Expression, not a Name.
    assert!(
        matches!(
            barrier_shape.flattened.atoms[0],
            lang_build::ProductAtom::Expression { .. }
        ),
        "first atom after an Expression barrier must remain an Expression barrier"
    );

    // Contrast: the non-barrier product ((a, b), c) yields three atoms.
    let no_barrier_shape = fixture_arg_product_shape(
        "product_exposed_left.lang",
        ProductMaterialRole::CallableArgumentProduct,
    );
    assert_eq!(
        no_barrier_shape.flattened.atoms.len(),
        3,
        "non-barrier product ((a, b), c) must flatten to three atoms"
    );
}

// ---------------------------------------------------------------------------
// Object-boundary placeholder tests: widened canonical atom kinds, RawArgShape
// refinement, canonical material + refinement linkage, alias query surface.
// These are NOT source semantic tests; classification is constructed directly.
// ---------------------------------------------------------------------------

/// Object-boundary test: `CanonicalArgAtomKind` must distinguish all future
/// non-value atom kinds so that later type/rank/meta/pattern classifiers can
/// write into canonical material without structural rework.
#[test]
fn canonical_arg_material_distinguishes_future_non_value_atom_kinds_object_boundary() {
    let shape = build_mixed_classification_shape();
    let material = lang_build::CanonicalArgProductShapeMaterial::from_arg_product_shape(&shape);

    assert_eq!(material.arity, 9);
    assert_eq!(
        material.atom_kinds,
        vec![
            CanonicalArgAtomKind::ExpressionBarrier,
            CanonicalArgAtomKind::ResolvedValue,
            CanonicalArgAtomKind::TypeObject,
            CanonicalArgAtomKind::RankObject,
            CanonicalArgAtomKind::NamespaceObject,
            CanonicalArgAtomKind::MetaObject,
            CanonicalArgAtomKind::PatternObject,
            CanonicalArgAtomKind::ProductUnit,
            CanonicalArgAtomKind::Unsupported,
        ],
        "every future non-value atom kind must have a distinct CanonicalArgAtomKind variant"
    );
}

/// Object-boundary test: RawArgShape refinement preserves provenance and the
/// automatic-pass-action boundary.
#[test]
fn raw_arg_shape_refinement_preserves_provenance_and_pass_boundary_object_boundary() {
    let arg = RawArgShape::from_product_atom(
        3,
        &ProductAtom::Unit {
            provenance: provenance("u"),
        },
    );
    // Override value_class to UnknownExpression to simulate an unresolved slot.
    let arg = arg.with_value_class(RawArgValueClass::UnknownExpression);

    assert!(!arg.receives_automatic_pass_action());
    assert_eq!(arg.is_value(), None);

    let refined = arg.clone().as_non_value(NonValueArgKind::TypeObject);
    assert_eq!(
        refined.index, 3,
        "index must be preserved through refinement"
    );
    assert_eq!(
        refined.provenance.description, arg.provenance.description,
        "provenance must be preserved through refinement"
    );
    assert_eq!(refined.is_value(), Some(false));
    assert!(
        !refined.receives_automatic_pass_action(),
        "NonValue(TypeObject) must not receive automatic pass action"
    );

    let value = arg.clone().as_resolved_value();
    assert_eq!(value.index, 3);
    assert_eq!(value.provenance.description, arg.provenance.description);
    assert_eq!(value.is_value(), Some(true));
    assert!(
        value.receives_automatic_pass_action(),
        "Value must receive automatic pass action after positive classification"
    );

    let with_tv = value.with_known_first_order_type_value(TypeValueId(5));
    assert_eq!(with_tv.known_first_order_type_value, Some(TypeValueId(5)));
    assert_eq!(with_tv.index, 3);
    assert_eq!(with_tv.provenance.description, arg.provenance.description);
}

/// Object-boundary test: canonical material must reflect refined RawArgShape
/// value classes, not collapse everything to ExpressionBarrier.
#[test]
fn canonical_material_reflects_refined_raw_arg_kinds_object_boundary() {
    let shape = build_mixed_classification_shape();
    let material = lang_build::CanonicalArgProductShapeMaterial::from_arg_product_shape(&shape);

    let kinds: Vec<CanonicalArgAtomKind> = material.atom_kinds;
    assert_eq!(
        kinds[1],
        CanonicalArgAtomKind::ResolvedValue,
        "refined Value must become ResolvedValue"
    );
    assert_eq!(
        kinds[2],
        CanonicalArgAtomKind::TypeObject,
        "refined NonValue(TypeObject) must become TypeObject"
    );
    assert_eq!(
        kinds[5],
        CanonicalArgAtomKind::MetaObject,
        "refined NonValue(MetaObject) must become MetaObject"
    );
    assert_eq!(
        kinds[7],
        CanonicalArgAtomKind::ProductUnit,
        "refined NonValue(ProductUnit) must become ProductUnit"
    );
    assert_eq!(
        kinds[8],
        CanonicalArgAtomKind::Unsupported,
        "refined Unsupported must become Unsupported"
    );
}

// ---------------------------------------------------------------------------
// Helpers for object-boundary classification tests
// ---------------------------------------------------------------------------

fn provenance(desc: &str) -> Provenance {
    Provenance::new(desc)
}

fn build_mixed_classification_shape() -> ArgProductShape {
    let raw_args = vec![
        raw_arg(0, RawArgValueClass::UnknownExpression),
        raw_arg(1, RawArgValueClass::Value),
        raw_arg(2, RawArgValueClass::NonValue(NonValueArgKind::TypeObject)),
        raw_arg(3, RawArgValueClass::NonValue(NonValueArgKind::RankObject)),
        raw_arg(
            4,
            RawArgValueClass::NonValue(NonValueArgKind::NamespaceObject),
        ),
        raw_arg(5, RawArgValueClass::NonValue(NonValueArgKind::MetaObject)),
        raw_arg(
            6,
            RawArgValueClass::NonValue(NonValueArgKind::PatternObject),
        ),
        raw_arg(7, RawArgValueClass::NonValue(NonValueArgKind::ProductUnit)),
        raw_arg(
            8,
            RawArgValueClass::Unsupported {
                summary: "unsupported test material".to_string(),
            },
        ),
    ];
    let arity = raw_args.len();
    let provenance = Provenance::new("object-boundary mixed classification shape");
    // atoms are not inspected here; fill with Units
    let mut atoms = Vec::with_capacity(arity);
    for _ in 0..arity {
        atoms.push(ProductAtom::Unit {
            provenance: provenance.clone(),
        });
    }
    ArgProductShape {
        flattened: FlattenedProductObject {
            atoms,
            provenance: provenance.clone(),
            invariant: FlattenedProductInvariant {
                no_direct_product_atom_remains: true,
            },
        },
        arity,
        raw_args,
        provenance,
    }
}

fn raw_arg(index: usize, value_class: RawArgValueClass) -> RawArgShape {
    RawArgShape {
        index,
        value_class,
        explicit_pass_mode: None,
        known_type_symbol_id: None,
        known_type_pattern_name: None,
        known_first_order_type_value: None,
        known_type_member_view: None,
        known_type_carrier_place: None,
        known_complete_type_observation: None,
        known_type_observation: None,
        known_semantic_value: None,
        known_value_mode: None,
        provenance: Provenance::new("object-boundary placeholder"),
    }
}

// ---------------------------------------------------------------------------
// Round 5: IdentityType + ParameterArgRequirement
// ---------------------------------------------------------------------------

/// Type-argument check: `UnknownExpression` and `Value` arguments must not
/// satisfy a `ParameterShape` requiring `TypeObject`.
#[test]
fn identity_type_rejects_unclassified_or_non_type_argument() {
    // UnknownExpression should be rejected by TypeObject requirement
    let unknown_shape = shape_with_class(RawArgValueClass::UnknownExpression);
    let result = prepare_type_signature_candidate(unknown_shape);
    assert!(
        !matches!(result, CandidatePrepResult::ApplicablePlaceholder(_)),
        "UnknownExpression must not satisfy TypeObject requirement"
    );

    // Value should be rejected by TypeObject requirement
    let value_shape = shape_with_class(RawArgValueClass::Value);
    let result = prepare_type_signature_candidate(value_shape);
    assert!(
        !matches!(result, CandidatePrepResult::ApplicablePlaceholder(_)),
        "Value must not satisfy TypeObject requirement"
    );
}

/// Object-boundary test: explicit type-object identity and
/// `as_resolved_value_with_value_type` carry distinct `value_class` and
/// pass-action boundaries.
#[test]
fn raw_arg_shape_typed_refinement_helpers_distinguish_type_object_from_value_type() {
    let arg = raw_arg(0, RawArgValueClass::UnknownExpression);

    let type_arg = arg
        .clone()
        .as_type_object_with_identity(SymbolId(5), TypeValueId(50));
    assert!(matches!(
        type_arg.value_class,
        RawArgValueClass::NonValue(NonValueArgKind::TypeObject)
    ));
    assert_eq!(type_arg.known_type_symbol_id, Some(SymbolId(5)));
    assert_eq!(type_arg.known_first_order_type_value, Some(TypeValueId(50)));
    assert_eq!(type_arg.is_value(), Some(false));
    assert!(
        !type_arg.receives_automatic_pass_action(),
        "type-object argument must not receive automatic pass action"
    );

    let value_arg = arg.as_resolved_value_with_value_type(TypeValueId(7));
    assert_eq!(value_arg.value_class, RawArgValueClass::Value);
    assert_eq!(value_arg.known_first_order_type_value, Some(TypeValueId(7)));
    assert_eq!(value_arg.is_value(), Some(true));
    assert!(
        value_arg.receives_automatic_pass_action(),
        "value argument must receive automatic pass action"
    );
}

fn shape_with_class(value_class: RawArgValueClass) -> ArgProductShape {
    let raw_args = vec![RawArgShape {
        index: 0,
        value_class,
        explicit_pass_mode: None,
        known_type_symbol_id: None,
        known_type_pattern_name: None,
        known_first_order_type_value: None,
        known_type_member_view: None,
        known_type_carrier_place: None,
        known_complete_type_observation: None,
        known_type_observation: None,
        known_semantic_value: None,
        known_value_mode: None,
        provenance: Provenance::new("rejection test shape"),
    }];
    let atoms = vec![ProductAtom::Unit {
        provenance: Provenance::new("rejection test atom"),
    }];
    ArgProductShape {
        flattened: FlattenedProductObject {
            atoms,
            provenance: Provenance::new("rejection test"),
            invariant: FlattenedProductInvariant {
                no_direct_product_atom_remains: true,
            },
        },
        arity: 1,
        raw_args,
        provenance: Provenance::new("rejection test"),
    }
}

fn prepare_type_signature_candidate(shape: ArgProductShape) -> CandidatePrepResult {
    let placeholder_callee = lang_build::SymbolObject::placeholder(
        SymbolId(100),
        "test_callee",
        lang_build::SymbolKind::MetaFunction,
        lang_build::SourceCategory::DeclaredSymbol,
        None,
        Provenance::new("rejection test callee"),
    );
    lang_build::prepare_meta_callable_candidate_with_declared_planes(
        &placeholder_callee,
        lang_build::CallableCandidateKind::MetaFunction,
        None,
        empty_policy_metadata(),
        empty_policy_metadata(),
        shape,
        ParameterShape::type_parameter_signature(Provenance::new("rejection test param")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("rejection test"),
        },
    )
}

/// `CandidatePrepResult::ApplicablePlaceholder` is not meta invocation.
///
/// Candidate prep must not return TypeValueId, must not install NamespaceDelta,
/// and must not produce InvocationResult.
#[test]
fn candidate_preparation_does_not_return_meta_invocation_result() {
    let world = v08_candidate_world();
    let callee = world
        .namespace_projection()
        .capability()
        .resolve_meta_function_with_policy(
            "struct",
            &world.package_context(),
            PolicyEnv::OpenStatic,
        )
        .expect("struct resolves");

    let site = v08_candidate_call_site();
    let shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);

    let result = prepare_candidate_from_fixture_symbol(
        &callee,
        shape,
        ParameterShape::exact_arity(1, Provenance::new("struct arity")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::OpenStatic,
            demanded_execution: ExecutionEnv::OpenStatic,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("forbidden: candidate prep != invocation"),
        },
    );

    // CandidatePrepResult is NOT InvocationResult (compile-time type guarantee).
    // Runtime: assert it is ApplicablePlaceholder — which has no TypeValueId,
    // no NamespaceDelta, no declared symbol.
    let CandidatePrepResult::ApplicablePlaceholder(candidate) = result else {
        panic!("struct candidate-prep should yield ApplicablePlaceholder");
    };
    assert!(
        candidate.arg_product_shape.raw_args[0]
            .known_first_order_type_value
            .is_none(),
        "candidate-prep must not assign TypeValueId"
    );
}

// ---------------------------------------------------------------------------
// Round 7: canonical fingerprint + cache structure tests
// ---------------------------------------------------------------------------

/// Canonical fingerprint must distinguish different Unit positions.
#[test]
fn canonical_fingerprint_distinguishes_unit_positions() {
    let key_left = key_for_shape_with_units(&vec![0]);
    let key_right = key_for_shape_with_units(&vec![1]);
    assert_ne!(
        key_left.value, key_right.value,
        "different Unit positions must produce different fingerprints"
    );
}

/// Canonical fingerprint must distinguish unresolved from typed atoms.
#[test]
fn canonical_fingerprint_distinguishes_expression_barrier_from_type_object() {
    let key_barrier = key_for_single_arg(CanonicalArgAtomKind::ExpressionBarrier);
    let key_typed = key_for_single_arg(CanonicalArgAtomKind::TypeObject);
    assert_ne!(
        key_barrier.value, key_typed.value,
        "ExpressionBarrier vs TypeObject must produce different fingerprints"
    );
}

/// Canonical fingerprint must distinguish different TypeValues.
#[test]
fn canonical_fingerprint_distinguishes_type_values() {
    let key_a = key_for_type_value_arg(lang_build::TypeValueId(1));
    let key_b = key_for_type_value_arg(lang_build::TypeValueId(2));
    assert_ne!(key_a.value, key_b.value);
}

/// Canonical fingerprint must not include binding name.
#[test]
fn canonical_fingerprint_excludes_declaration_binding_name() {
    let key_a =
        key_for_single_arg_with_provenance(CanonicalArgAtomKind::TypeObject, "binding context A");
    let key_b =
        key_for_single_arg_with_provenance(CanonicalArgAtomKind::TypeObject, "binding context B");
    assert_eq!(
        key_a.value, key_b.value,
        "same semantic material must yield same key regardless of context"
    );
    assert_eq!(
        key_a, key_b,
        "keys with different provenance but same canonical material must be equal"
    );
}

/// Legacy meta digest equality must ignore provenance.
#[test]
fn legacy_meta_digest_equality_ignores_provenance() {
    let key_a = key_for_type_value_arg_with_provenance(lang_build::TypeValueId(5), "provenance A");
    let key_b = key_for_type_value_arg_with_provenance(lang_build::TypeValueId(5), "provenance B");

    assert_eq!(key_a, key_b, "key equality must ignore provenance");
    assert_eq!(
        key_a.cmp(&key_b),
        std::cmp::Ordering::Equal,
        "key ordering must ignore provenance"
    );

    let key_c = key_for_type_value_arg_with_provenance(lang_build::TypeValueId(6), "provenance A");
    assert_ne!(
        key_a, key_c,
        "different TypeValue must produce different key"
    );
}

/// Cache stores invocation value, not NamespaceDelta.
#[test]
fn meta_instance_cache_stores_invocation_value_not_namespace_delta() {
    let mut cache = MetaInstanceCache::new();
    let key = lang_build::compute_meta_invocation_material_key(
        lang_build::MetaCallableIdentity {
            selected_function_value: lang_build::SemanticValueId(50),
            selected_call_entry: lang_build::SemanticValueId(51),
        },
        lang_build::CanonicalValueAddr(52),
        Provenance::new("structural cache key"),
    );
    cache.insert(
        key.clone(),
        MetaInvocationValue::ForwardedValue(ForwardedValue {
            type_value: lang_build::TypeValueId(5),
            type_observation: lang_build::CanonicalTypeObservation::Detached(
                lang_build::TypeValueId(5),
            ),
            return_view: ReturnViewShape::Leaf,
            provenance: Provenance::new("test cache insert"),
        }),
        Provenance::new("test cache insert"),
    );
    let cached = cache.lookup(&key).expect("cache entry should be found");
    assert!(matches!(
        cached.result,
        MetaInvocationValue::ForwardedValue(_)
    ));
    // MetaInstanceCache does not expose NamespaceDelta — compile-time guarantee.
    assert_eq!(cache.len(), 1);
}

#[test]
fn meta_instance_cache_uses_structural_instance_identity_not_a_digest_channel() {
    let key = |call_entry| {
        lang_build::compute_meta_invocation_material_key(
            lang_build::MetaCallableIdentity {
                selected_function_value: lang_build::SemanticValueId(70),
                selected_call_entry: lang_build::SemanticValueId(call_entry),
            },
            lang_build::CanonicalValueAddr(72),
            Provenance::new("structural cache identity"),
        )
    };
    let first = key(71);
    let different_selected_callable = key(73);
    let mut cache = MetaInstanceCache::new();
    cache.insert(
        first.clone(),
        MetaInvocationValue::ForwardedValue(ForwardedValue {
            type_value: TypeValueId(5),
            type_observation: lang_build::CanonicalTypeObservation::Detached(TypeValueId(5)),
            return_view: ReturnViewShape::Leaf,
            provenance: Provenance::new("cached material"),
        }),
        Provenance::new("cache insert"),
    );

    assert!(cache.lookup(&first).is_some());
    assert!(
        cache.lookup(&different_selected_callable).is_none(),
        "a different selected callable coordinate can never hit the cache through digest compatibility"
    );
}

/// MetaInvocationInput primitive is derived from candidate, not caller.
#[test]
fn meta_invocation_primitive_identity_is_derived_from_candidate() {
    let candidate = bare_candidate();
    assert!(
        candidate.callee_primitive.is_some(),
        "candidate from candidate-prep must carry callee_primitive"
    );
    let input = MetaInvocationInput::new(candidate, Provenance::new("test"));
    // The primitive is owned by the selected candidate.  MetaInvocationInput
    // intentionally has no digest-derived cache-key authority.
    assert_eq!(
        input.candidate.callee_primitive,
        Some(lang_build::CoreMetaFunction::IdentityType)
    );
}

fn empty_policy_metadata() -> lang_build::PolicyMetadata {
    lang_build::PolicyMetadata {
        slots: std::collections::BTreeMap::new(),
        policy_set: lang_build::PolicySet {
            flags: std::collections::BTreeSet::new(),
        },
    }
}

fn empty_policy_planes() -> lang_build::CandidatePolicyPlanes {
    lang_build::CandidatePolicyPlanes {
        lookup_env: PolicyEnv::OpenStatic,
        symbol_visibility_policy: empty_policy_metadata(),
        demanded_execution: ExecutionEnv::OpenStatic,
        body_entry_policy: empty_policy_metadata(),
        return_object_policy: empty_policy_metadata(),
    }
}

fn digest_raw_arg(
    index: usize,
    value_class: RawArgValueClass,
    type_value: Option<TypeValueId>,
    provenance_desc: &str,
) -> RawArgShape {
    RawArgShape {
        index,
        value_class,
        explicit_pass_mode: None,
        known_type_symbol_id: None,
        known_type_pattern_name: None,
        known_first_order_type_value: type_value,
        known_type_member_view: None,
        known_type_carrier_place: None,
        known_complete_type_observation: None,
        known_type_observation: None,
        known_semantic_value: None,
        known_value_mode: None,
        provenance: Provenance::new(provenance_desc),
    }
}

fn digest_shape_from_args(raw_args: Vec<RawArgShape>, provenance_desc: &str) -> ArgProductShape {
    let arity = raw_args.len();
    let provenance = Provenance::new(provenance_desc);
    let atoms = (0..arity)
        .map(|_| ProductAtom::Unit {
            provenance: provenance.clone(),
        })
        .collect();
    ArgProductShape {
        flattened: FlattenedProductObject {
            atoms,
            provenance: provenance.clone(),
            invariant: FlattenedProductInvariant {
                no_direct_product_atom_remains: true,
            },
        },
        arity,
        raw_args,
        provenance,
    }
}

fn digest_candidate_for_shape(
    shape: ArgProductShape,
    provenance_desc: &str,
) -> PreparedCallableCandidate {
    PreparedCallableCandidate {
        callee_symbol_id: SymbolId(99),
        callee_name: "test".to_string(),
        callee_primitive: None,
        callable_kind: lang_build::CallableCandidateKind::MetaFunction,
        arg_product_shape: shape,
        parameter_shape: ParameterShape::deferred(Provenance::new(provenance_desc)),
        policy_planes: empty_policy_planes(),
        build_identity: CandidateBuildIdentityPlaceholder::default(),
        provenance: Provenance::new(provenance_desc),
    }
}

fn value_class_for_atom_kind(kind: CanonicalArgAtomKind) -> RawArgValueClass {
    match kind {
        CanonicalArgAtomKind::ExpressionBarrier => RawArgValueClass::UnknownExpression,
        CanonicalArgAtomKind::ResolvedValue => RawArgValueClass::Value,
        CanonicalArgAtomKind::TypeObject => RawArgValueClass::NonValue(NonValueArgKind::TypeObject),
        CanonicalArgAtomKind::RankObject => RawArgValueClass::NonValue(NonValueArgKind::RankObject),
        CanonicalArgAtomKind::NamespaceObject => {
            RawArgValueClass::NonValue(NonValueArgKind::NamespaceObject)
        }
        CanonicalArgAtomKind::MetaObject => RawArgValueClass::NonValue(NonValueArgKind::MetaObject),
        CanonicalArgAtomKind::PatternObject => {
            RawArgValueClass::NonValue(NonValueArgKind::PatternObject)
        }
        CanonicalArgAtomKind::ProductUnit => {
            RawArgValueClass::NonValue(NonValueArgKind::ProductUnit)
        }
        CanonicalArgAtomKind::Unsupported => RawArgValueClass::Unsupported {
            summary: "unsupported digest material".to_string(),
        },
    }
}

fn key_for_single_arg(kind: CanonicalArgAtomKind) -> lang_build::CanonicalFingerprint {
    key_for_single_arg_with_provenance(kind, "test key material")
}

fn key_for_type_value_arg(type_value: lang_build::TypeValueId) -> lang_build::CanonicalFingerprint {
    key_for_type_value_arg_with_provenance(type_value, "test key material")
}

fn key_for_shape_with_units(unit_positions: &[usize]) -> lang_build::CanonicalFingerprint {
    let desc = "unit position digest material";
    let raw_args = (0..3usize)
        .map(|index| {
            let value_class = if unit_positions.contains(&index) {
                RawArgValueClass::NonValue(NonValueArgKind::ProductUnit)
            } else {
                RawArgValueClass::UnknownExpression
            };
            digest_raw_arg(index, value_class, None, desc)
        })
        .collect();
    let candidate = digest_candidate_for_shape(digest_shape_from_args(raw_args, desc), desc);
    compute_legacy_meta_instance_digest(&candidate)
}

fn bare_candidate() -> PreparedCallableCandidate {
    let mut candidate = digest_candidate_for_shape(
        digest_shape_from_args(Vec::new(), "bare candidate"),
        "bare candidate",
    );
    candidate.callee_symbol_id = SymbolId(1);
    candidate.callee_name = "bare".to_string();
    candidate.callee_primitive = Some(lang_build::CoreMetaFunction::IdentityType);
    candidate
}

fn key_for_single_arg_with_provenance(
    kind: CanonicalArgAtomKind,
    provenance_desc: &str,
) -> lang_build::CanonicalFingerprint {
    let raw_args = vec![digest_raw_arg(
        0,
        value_class_for_atom_kind(kind),
        None,
        provenance_desc,
    )];
    let candidate = digest_candidate_for_shape(
        digest_shape_from_args(raw_args, provenance_desc),
        provenance_desc,
    );
    compute_legacy_meta_instance_digest(&candidate)
}

fn key_for_type_value_arg_with_provenance(
    type_value: lang_build::TypeValueId,
    provenance_desc: &str,
) -> lang_build::CanonicalFingerprint {
    let raw_args = vec![digest_raw_arg(
        0,
        RawArgValueClass::NonValue(NonValueArgKind::TypeObject),
        Some(type_value),
        provenance_desc,
    )];
    let candidate = digest_candidate_for_shape(
        digest_shape_from_args(raw_args, provenance_desc),
        provenance_desc,
    );
    compute_legacy_meta_instance_digest(&candidate)
}
