//! Declared result-class and return-Pattern invariants.

mod support;

use lang_build::{
    declared_result_class_from_closure, validate_declared_result_class, DeclaredResultClass,
    PatternComponentPolicy, PolicyPair, PolicyStage, Provenance, StageSet, ValueComponentPolicy,
    ValuePresence,
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

fn declared_result_class(source: &str) -> DeclaredResultClass {
    declared_result_class_from_closure(&closure_initializer(source))
        .expect("the return slot declares a result class")
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
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: stage_set(pattern_stages),
        },
    }
}

#[test]
fn declared_result_class_is_the_single_result_authority() {
    assert_eq!(
        declared_result_class("let f = (self, t: type): meta -> r: symbol => { r; };"),
        DeclaredResultClass::ClusterSymbol
    );
    assert_eq!(
        declared_result_class("let f = (self, t: type): meta -> let r: type => { r; };"),
        DeclaredResultClass::CompleteType
    );
    assert_eq!(
        declared_result_class(
            "let f = (self, _ uint8: type): compile -> let result: uint8 => { result; };"
        ),
        DeclaredResultClass::OrdinaryValue
    );
    assert_eq!(
        declared_result_class("let f = (self): compile -> _: unit => { self; };"),
        DeclaredResultClass::Unit
    );
}

#[test]
fn return_pattern_does_not_define_result_class() {
    let constrained = declared_result_class(
        "let f = (self, _ uint8: type): compile -> let result: uint8 => { result; };",
    );
    let unconstrained = declared_result_class("let f = (self): compile => { self; };");
    assert_eq!(constrained, DeclaredResultClass::OrdinaryValue);
    assert_eq!(unconstrained, DeclaredResultClass::OrdinaryValue);
}

#[test]
fn unit_result_requires_the_underscore_binder_spelling() {
    let error = declared_result_class_from_closure(&closure_initializer(
        "let f = (self): compile -> r: unit => { self; };",
    ))
    .expect_err("a named binder with a unit annotation is rejected");
    assert!(error.message.contains("_: unit"));
}

#[test]
fn cluster_symbol_result_requires_a_pure_meta_domain() {
    let provenance = Provenance::new("ClusterSymbol result Policy validation");
    let meta = p2(&[PolicyStage::Meta], &[PolicyStage::Meta]);
    let compile = p2(&[PolicyStage::Compile], &[PolicyStage::Compile]);
    let meta_compile = p2(
        &[PolicyStage::Meta, PolicyStage::Compile],
        &[PolicyStage::Meta, PolicyStage::Compile],
    );
    assert!(
        validate_declared_result_class(DeclaredResultClass::ClusterSymbol, &meta, &provenance)
            .is_ok()
    );
    assert!(validate_declared_result_class(
        DeclaredResultClass::ClusterSymbol,
        &compile,
        &provenance
    )
    .is_err());
    assert!(validate_declared_result_class(
        DeclaredResultClass::ClusterSymbol,
        &meta_compile,
        &provenance
    )
    .is_err());
}

#[test]
fn non_cluster_result_classes_do_not_acquire_a_policy_derived_category() {
    let provenance = Provenance::new("result class remains declaration-owned");
    let meta = p2(&[PolicyStage::Meta], &[PolicyStage::Meta]);
    let compile = p2(&[PolicyStage::Compile], &[PolicyStage::Compile]);
    for result_class in [
        DeclaredResultClass::Unit,
        DeclaredResultClass::CompleteType,
        DeclaredResultClass::OrdinaryValue,
    ] {
        assert!(validate_declared_result_class(result_class.clone(), &meta, &provenance).is_ok());
        assert!(validate_declared_result_class(result_class, &compile, &provenance).is_ok());
    }
}

#[test]
fn invalid_cluster_symbol_policy_fails_at_declaration() {
    let error = build_fixture_error("result_class_validation_error", "app");
    assert!(error.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("requires a pure meta result P2")));
}
