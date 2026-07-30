//! Return-shape ontology acceptance tests.
//!
//! `CallableSemantics = P1 × P2 × ReturnShape × Privilege`: the return
//! shape is elaborated once from the return-slot annotation, independent
//! of the Policy stage, and `Validate(P2, ReturnShape)` is a legality
//! relation — never a derivation in either direction.

mod support;

use std::collections::BTreeSet;

use lang_build::{
    declared_return_shape_from_closure, validate_return_shape, PatternComponentPolicy,
    PatternConstraint, PolicyPair, PolicyStage, Provenance, ReturnShape, StageSet,
    ValueComponentPolicy, ValuePresence,
};
use lang_syntax::{NormClosure, NormDecl, NormExpr, NormForm};

use support::build_fixture_error;

fn closure_initializer(source: &str) -> NormClosure {
    let parsed = lang_syntax::parse(source);
    assert!(
        parsed.diagnostics.is_empty(),
        "unexpected parse diagnostics:\n{}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    let normalized = lang_syntax::normalize_program(&parsed.program);
    match normalized.forms.as_slice() {
        [NormForm::Let(NormDecl::Let { slot, .. })] => match slot.initializer.as_deref() {
            Some(NormExpr::Closure(closure)) => closure.clone(),
            other => panic!("expected closure initializer, got {other:#?}"),
        },
        other => panic!("expected single let closure declaration, got {other:#?}"),
    }
}

fn shape_of(source: &str) -> ReturnShape {
    declared_return_shape_from_closure(&closure_initializer(source))
        .expect("return-slot annotation elaborates to a shape")
}

fn p2(value_stages: &[PolicyStage], pattern_stages: &[PolicyStage]) -> PolicyPair {
    let stage_set = |stages: &[PolicyStage]| {
        let mut set = StageSet::new();
        for stage in stages {
            set.insert(*stage);
        }
        set
    };
    PolicyPair {
        value: ValueComponentPolicy {
            stages: stage_set(value_stages),
            mutability: BTreeSet::new(),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: stage_set(pattern_stages),
        },
    }
}

#[test]
fn return_slot_annotations_elaborate_to_shapes() {
    assert_eq!(
        shape_of("let f = (self, t: type): meta -> r: symbol => { r; };"),
        ReturnShape::ClusterSymbol
    );
    assert_eq!(
        shape_of("let f = (self, t: type): meta -> let r: type => { r; };"),
        ReturnShape::SingleType
    );
    assert_eq!(
        shape_of("let f = (self, _ uint8: type): compile -> let result: uint8 => { result; };"),
        ReturnShape::SingleVal(PatternConstraint::Constrained)
    );
    assert_eq!(
        shape_of("let f = (self): compile => { self; };"),
        ReturnShape::SingleVal(PatternConstraint::Unconstrained)
    );
}

#[test]
fn unit_shape_requires_the_underscore_binder_spelling() {
    // `_` occupies the leftmost slot so `unit` cannot be misread as the
    // leftmost to-be-extracted name of an extraction shorthand.
    assert_eq!(
        shape_of("let f = (self): compile -> _: unit => { self; };"),
        ReturnShape::Unit
    );
    let error = declared_return_shape_from_closure(&closure_initializer(
        "let f = (self): compile -> r: unit => { self; };",
    ))
    .expect_err("a named binder with a `unit` annotation is rejected");
    assert!(
        error.message.contains("_: unit"),
        "diagnostic explains the required spelling: {}",
        error.message
    );
}

#[test]
fn validate_is_a_legality_relation_over_the_single_position_criterion() {
    let provenance = Provenance::new("shape validation test");
    let meta = p2(&[PolicyStage::Meta], &[PolicyStage::Meta]);
    let compile = p2(&[PolicyStage::Compile], &[PolicyStage::Compile]);
    // ClusterSymbol: plural values at ONE position — requires meta P2.
    assert!(validate_return_shape(ReturnShape::ClusterSymbol, &meta, &provenance).is_ok());
    assert!(validate_return_shape(ReturnShape::ClusterSymbol, &compile, &provenance).is_err());
    // Single-position shapes are legal under both.
    for shape in [
        ReturnShape::Unit,
        ReturnShape::SingleType,
        ReturnShape::SingleVal(PatternConstraint::Unconstrained),
        ReturnShape::SingleVal(PatternConstraint::Constrained),
    ] {
        assert!(validate_return_shape(shape, &meta, &provenance).is_ok());
        assert!(validate_return_shape(shape, &compile, &provenance).is_ok());
    }
}

#[test]
fn cluster_symbol_requires_a_pure_meta_domain() {
    let provenance = Provenance::new("mixed meta domain rejection");
    let meta_compile = p2(
        &[PolicyStage::Meta, PolicyStage::Compile],
        &[PolicyStage::Meta, PolicyStage::Compile],
    );
    let meta_runtime = p2(
        &[PolicyStage::Meta, PolicyStage::Runtime],
        &[PolicyStage::Meta],
    );
    assert!(validate_return_shape(ReturnShape::ClusterSymbol, &meta_compile, &provenance).is_err());
    assert!(validate_return_shape(ReturnShape::ClusterSymbol, &meta_runtime, &provenance).is_err());
}

#[test]
fn cluster_symbol_shape_without_meta_p2_fails_at_declaration() {
    let error = build_fixture_error("s4_shape_validate_error", "app");
    assert!(
        error.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("requires a pure meta result P2")),
        "declaration-time Validate(P2, ReturnShape) rejects `-> r: symbol` under compile:\n{:#?}",
        error.diagnostics
    );
}
