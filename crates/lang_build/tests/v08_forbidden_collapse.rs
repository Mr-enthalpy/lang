//! Forbidden-collapse tests: prevent future implementations from compressing
//! the v0.8 substrate boundary objects into simpler-but-incorrect shapes.
//!
//! These tests verify structural separations that must hold until the full
//! generic type system, overload resolver, pattern engine, and meta invocation
//! engine are implemented.

mod support;

use support::*;

use lang_build::{
    prepare_meta_callable_candidate, prepare_meta_callable_candidate_from_input, AliasChain,
    AliasCycleDetectionState, AliasQueryDisposition, AliasQueryMode, AliasQueryRequest,
    AliasWritableBoundary, ArgProductShape, CandidateBuildIdentityPlaceholder,
    CandidatePrepDeferredReason, CandidatePrepResult, CandidatePreparationContext,
    CandidatePreparationInput, CanonicalArgAtomKind, ExecutionEnv, FlattenedProductInvariant,
    FlattenedProductObject, NonValueArgKind, ParameterShape, PolicyEnv, ProductAtom,
    ProductMaterialRole, Provenance, RawArgShape, RawArgValueClass, SymbolId, TypeValueId,
};

/// Unit positions must remain in the canonical argument material and not be
/// collapsed into arity-only or arity-plus-type-values-only data.
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
/// be mistaken for MetaReductionResult or MetaExpansionResult. This test
/// confirms that candidate-prep may defer on body-entry policy without
/// returning a meta execution result.
#[test]
fn candidate_prep_does_not_execute_meta_invocation() {
    let world = v08_candidate_world();
    let field_symbol = world
        .snapshot()
        .capability()
        .resolve_field_function("field::ref::T", &world.package_context())
        .expect("generated ref field function resolves through namespace graph");

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
    // Confirm CandidatePrepResult is NOT MetaReductionResult / MetaExpansionResult
    // (compile-time type guarantee; runtime assertion above proves behavior).
}

/// The three alias query modes must return distinct dispositions and must not
/// collapse into a single global transparency flag.
#[test]
fn alias_query_mode_is_not_global_transparency() {
    let alias = AliasChain::new(
        SymbolId(10),
        SymbolId(20),
        Provenance::new("forbidden collapse: alias query modes"),
    );

    let d_typeval = alias.query_disposition(AliasQueryMode::TypeValueEvaluation);
    let d_callable = alias.query_disposition(AliasQueryMode::CallableLookup);
    let d_place = alias.query_disposition(AliasQueryMode::InjectionPlaceTarget);

    assert_ne!(
        d_typeval, d_callable,
        "type-value evaluation disposition must differ from callable lookup"
    );
    assert_ne!(
        d_typeval, d_place,
        "type-value evaluation disposition must differ from injection place target"
    );
    assert_ne!(
        d_callable, d_place,
        "callable lookup disposition must differ from injection place target"
    );

    assert_eq!(d_typeval, AliasQueryDisposition::FollowValueChain);
    assert_eq!(
        d_callable,
        AliasQueryDisposition::PolicyAwareSymbolResolution
    );
    assert_eq!(d_place, AliasQueryDisposition::FollowPlaceWithBoundary);
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

/// Object-boundary test: `AliasChain::query` accepts a request and returns
/// a placeholder result without performing full alias resolution.
#[test]
fn alias_query_request_drives_placeholder_result_object_boundary() {
    let chain = AliasChain::new(
        SymbolId(10),
        SymbolId(20),
        Provenance::new("alias query request test"),
    );

    for mode in [
        AliasQueryMode::TypeValueEvaluation,
        AliasQueryMode::CallableLookup,
        AliasQueryMode::InjectionPlaceTarget,
    ] {
        let request = AliasQueryRequest::new(mode, SymbolId(10), Provenance::new("test request"));
        let result = chain.query(&request);
        assert_eq!(result.disposition, chain.query_disposition(mode));
        assert_eq!(
            result.final_place, None,
            "placeholder result must not resolve final place"
        );
        assert_eq!(
            result.writable_boundary,
            AliasWritableBoundary::Unknown,
            "placeholder result must leave writable boundary unknown"
        );
        assert_eq!(
            result.cycle_detection_state,
            AliasCycleDetectionState::NotChecked,
            "placeholder result must leave cycle detection unchecked"
        );
    }

    assert!(
        !chain.creates_fresh_writable_place(),
        "alias chain must not claim to create a fresh writable place at object boundary"
    );

    // Source-symbol mismatch: conservative placeholder.
    let request = AliasQueryRequest::new(
        AliasQueryMode::TypeValueEvaluation,
        SymbolId(99),
        Provenance::new("mismatched source symbol"),
    );
    let result = chain.query(&request);
    assert_eq!(result.final_symbol, None);
    assert_eq!(result.final_value, None);
    assert_eq!(result.final_place, None);
}

/// Forbidden-collapse: `CandidatePrepResult` from the input wrapper must still
/// defer (not execute) for runtime-only body-entry policy.
#[test]
fn candidate_preparation_input_wrapper_still_does_not_execute_invocation() {
    let world = v08_candidate_world();
    let field_symbol = world
        .snapshot()
        .capability()
        .resolve_field_function("field::ref::T", &world.package_context())
        .expect("generated ref field function resolves through namespace graph");

    let site = v08_candidate_call_site();
    let arg_shape = site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);

    let input = CandidatePreparationInput::new(
        field_symbol,
        arg_shape,
        ParameterShape::exact_arity(1, Provenance::new("wrapper invocation test")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::Meta,
            demanded_execution: ExecutionEnv::Meta,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("forbidden: wrapper must not execute"),
        },
    );

    let result = prepare_meta_callable_candidate_from_input(input);
    match &result {
        CandidatePrepResult::Deferred { reason, .. } => {
            assert_eq!(
                *reason,
                CandidatePrepDeferredReason::BodyEntryPolicyMismatch
            );
        }
        CandidatePrepResult::ApplicablePlaceholder(_) => {
            panic!("wrapper must not execute meta invocation on runtime-only body")
        }
        CandidatePrepResult::Diagnostic(_) => {
            panic!("wrapper must not diagnose when body-entry policy mismatch should defer")
        }
    }
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
        known_first_order_type_value: None,
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
    let input = candidate_input(unknown_shape);
    let result = prepare_meta_callable_candidate_from_input(input);
    assert!(
        !matches!(result, CandidatePrepResult::ApplicablePlaceholder(_)),
        "UnknownExpression must not satisfy TypeObject requirement"
    );

    // Value should be rejected by TypeObject requirement
    let value_shape = shape_with_class(RawArgValueClass::Value);
    let input = candidate_input(value_shape);
    let result = prepare_meta_callable_candidate_from_input(input);
    assert!(
        !matches!(result, CandidatePrepResult::ApplicablePlaceholder(_)),
        "Value must not satisfy TypeObject requirement"
    );
}

/// Object-boundary test: `as_type_object_with_type_value` and
/// `as_resolved_value_with_value_type` carry distinct `value_class` and
/// pass-action boundaries.
#[test]
fn raw_arg_shape_typed_refinement_helpers_distinguish_type_object_from_value_type() {
    let arg = raw_arg(0, RawArgValueClass::UnknownExpression);

    let type_arg = arg.clone().as_type_object_with_type_value(TypeValueId(5));
    assert!(matches!(
        type_arg.value_class,
        RawArgValueClass::NonValue(NonValueArgKind::TypeObject)
    ));
    assert_eq!(type_arg.known_first_order_type_value, Some(TypeValueId(5)));
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
        known_first_order_type_value: None,
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

fn candidate_input(shape: ArgProductShape) -> CandidatePreparationInput {
    let placeholder_callee = lang_build::SymbolObject::placeholder(
        SymbolId(100),
        "test_callee",
        lang_build::SymbolKind::MetaFunction,
        lang_build::SourceCategory::DeclaredSymbol,
        None,
        Provenance::new("rejection test callee"),
    );
    CandidatePreparationInput::new(
        placeholder_callee,
        shape,
        ParameterShape::type_parameter_signature(Provenance::new("rejection test param")),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::Meta,
            demanded_execution: ExecutionEnv::Meta,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: Provenance::new("rejection test"),
        },
    )
}
