//! Declared result-class and return-Pattern invariants.

use lang_build::{declared_result_class_from_closure, DeclaredResultClass};

use lang_syntax::{NormClosure, NormDecl, NormExpr, NormForm};

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

#[test]
fn declared_result_class_is_the_single_result_authority() {
    assert_eq!(
        declared_result_class("let f = (self, t: type): meta -> r: symbol => { r; };"),
        DeclaredResultClass::OrdinaryValue
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
