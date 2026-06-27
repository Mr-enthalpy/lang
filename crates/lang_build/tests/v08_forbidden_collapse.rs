//! Forbidden-collapse tests: prevent future implementations from compressing
//! the v0.8 substrate boundary objects into simpler-but-incorrect shapes.
//!
//! These tests verify structural separations that must hold until the full
//! generic type system, overload resolver, pattern engine, and meta invocation
//! engine are implemented.

mod support;

use support::*;

use lang_build::{
    prepare_meta_callable_candidate, AliasChain, AliasQueryDisposition, AliasQueryMode,
    CandidateBuildIdentityPlaceholder, CandidatePrepDeferredReason, CandidatePrepResult,
    CandidatePreparationContext, CanonicalArgAtomKind, ExecutionEnv, ParameterShape, PolicyEnv,
    ProductMaterialRole, Provenance, SymbolId,
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
