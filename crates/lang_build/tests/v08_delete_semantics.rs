use lang_build::meta_body::{
    check_closure_body_delete_legality, evaluate_selected_meta_closure_body,
    selected_meta_delete_diagnostic, ClosureBodyExecutionEnv, SelectedMetaBodyEvaluation,
};
use lang_build::{DiagnosticSeverity, Provenance};
use lang_syntax::{NormClosureBody, NormDeleteBody, NormExpr, NormLiteralKind, NormOrigin, Span};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn provenance(desc: &str) -> Provenance {
    Provenance::new(desc)
}

fn block_body() -> NormClosureBody {
    NormClosureBody::Block(lang_syntax::NormProgram {
        forms: vec![],
        origin: NormOrigin::Source(Span::new(0, 0, 1, 1)),
    })
}

fn delete_body(msg: &str) -> NormClosureBody {
    NormClosureBody::Delete(NormDeleteBody {
        message: Box::new(NormExpr::Literal {
            kind: NormLiteralKind::String,
            text: format!("\"{msg}\""),
            origin: NormOrigin::Source(Span::new(0, 0, 1, 1)),
        }),
        origin: NormOrigin::Source(Span::new(0, 0, 1, 1)),
    })
}

fn delete_body_non_string() -> NormClosureBody {
    NormClosureBody::Delete(NormDeleteBody {
        message: Box::new(NormExpr::Name {
            text: "not_a_literal".to_string(),
            origin: NormOrigin::Source(Span::new(0, 0, 1, 1)),
        }),
        origin: NormOrigin::Source(Span::new(0, 0, 1, 1)),
    })
}

// ---------------------------------------------------------------------------
// Legality tests
// ---------------------------------------------------------------------------

#[test]
fn delete_body_is_legal_in_meta_execution_env() {
    let body = delete_body("reason");
    let result =
        check_closure_body_delete_legality(&body, ClosureBodyExecutionEnv::Meta, provenance("t"));
    assert!(result.is_ok());
}

#[test]
fn delete_body_is_rejected_in_runtime_execution_env() {
    let body = delete_body("reason");
    let result = check_closure_body_delete_legality(
        &body,
        ClosureBodyExecutionEnv::Runtime,
        provenance("t"),
    );
    assert!(result.is_err());
    let diag = result.unwrap_err();
    assert!(diag.message.contains("meta-executed"));
    assert_eq!(diag.severity, DiagnosticSeverity::Error);
}

#[test]
fn block_body_is_legal_in_meta_execution_env() {
    let body = block_body();
    let result =
        check_closure_body_delete_legality(&body, ClosureBodyExecutionEnv::Meta, provenance("t"));
    assert!(result.is_ok());
}

#[test]
fn block_body_is_legal_in_runtime_execution_env() {
    let body = block_body();
    let result = check_closure_body_delete_legality(
        &body,
        ClosureBodyExecutionEnv::Runtime,
        provenance("t"),
    );
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// Selected meta evaluation tests
// ---------------------------------------------------------------------------

#[test]
fn delete_body_produces_static_diagnostic_when_selected_meta_body_is_evaluated() {
    let body = delete_body("cannot combine bare if");
    let result = evaluate_selected_meta_closure_body(&body, provenance("t"));
    match result {
        SelectedMetaBodyEvaluation::DeleteDiagnostic(diag) => {
            assert_eq!(diag.severity, DiagnosticSeverity::Error);
            assert!(diag.message.contains("meta delete:"));
            assert!(diag.message.contains("cannot combine bare if"));
        }
        _ => panic!("expected DeleteDiagnostic"),
    }
}

#[test]
fn delete_body_diagnostic_uses_string_literal_message() {
    let body = delete_body("bare if residual");
    let diag = selected_meta_delete_diagnostic(
        match &body {
            NormClosureBody::Delete(d) => d,
            _ => panic!("expected Delete"),
        },
        provenance("t"),
    );
    assert!(diag.message.contains("bare if residual"));
}

#[test]
fn delete_body_does_not_produce_value() {
    // Delete is not a MetaInvocationValue variant — it produces a
    // Diagnostic. The evaluate function proves this.
    let body = delete_body("msg");
    let result = evaluate_selected_meta_closure_body(&body, provenance("t"));
    assert!(matches!(
        result,
        SelectedMetaBodyEvaluation::DeleteDiagnostic(_)
    ));
}

#[test]
fn non_string_delete_message_is_diagnostic() {
    let body = delete_body_non_string();
    let diag = selected_meta_delete_diagnostic(
        match &body {
            NormClosureBody::Delete(d) => d,
            _ => panic!("expected Delete"),
        },
        provenance("t"),
    );
    assert!(diag.message.contains("string literal"));
    assert_eq!(diag.severity, DiagnosticSeverity::Error);
}

#[test]
fn block_body_deferred_by_selected_meta_evaluation() {
    let body = block_body();
    let result = evaluate_selected_meta_closure_body(&body, provenance("t"));
    assert_eq!(result, SelectedMetaBodyEvaluation::DeferredBlock);
}
