use lang_syntax::{NormForm, NormProgram, NormReturnEvent};

use crate::model::{Diagnostic, Provenance, ResolverCode};

pub struct ControlFlowEndReport {
    pub terminal: Option<ControlFlowTerminal>,
    pub diagnostics: Vec<Diagnostic>,
}

pub enum ControlFlowTerminal {
    TailValue(lang_syntax::NormExpr),
    ReturnEvent(NormReturnEvent),
}

pub fn compute_control_flow_end_report(program: &NormProgram) -> ControlFlowEndReport {
    let mut terminal = None;
    let mut diagnostics = Vec::new();
    let mut seen_terminal = false;

    for form in &program.forms {
        if seen_terminal {
            diagnostics.push(statement_after_terminal_diagnostic(form));
            continue;
        }

        match form {
            // Transitional: generated closures and some legacy paths still
            // produce NormForm::Expr in body blocks. Treated as TailValue
            // for compatibility. Remove after all paths emit TailValue.
            NormForm::Expr(expr) => {
                terminal = Some(ControlFlowTerminal::TailValue(expr.clone()));
                seen_terminal = true;
            }
            NormForm::TailValue(expr) => {
                terminal = Some(ControlFlowTerminal::TailValue(expr.clone()));
                seen_terminal = true;
            }
            NormForm::ReturnEvent(return_ev) => {
                terminal = Some(ControlFlowTerminal::ReturnEvent(return_ev.clone()));
                seen_terminal = true;
            }
            NormForm::Let(_) | NormForm::Alias(_) | NormForm::Error(_) => {}
        }
    }

    ControlFlowEndReport {
        terminal,
        diagnostics,
    }
}

fn statement_after_terminal_diagnostic(form: &NormForm) -> Diagnostic {
    let provenance = match form {
        NormForm::Let(_) | NormForm::Alias(_) => {
            Provenance::from_norm_origin("statement after terminal block form", &form_origin(form))
        }
        NormForm::Expr(_) | NormForm::TailValue(_) => {
            Provenance::from_norm_origin("statement after terminal block form", &form_origin(form))
        }
        NormForm::ReturnEvent(_) => {
            Provenance::from_norm_origin("statement after terminal block form", &form_origin(form))
        }
        NormForm::Error(_) => {
            Provenance::from_norm_origin("statement after terminal block form", &form_origin(form))
        }
    };

    Diagnostic::hard_error(
        "statement after terminal block form in selected meta body",
        Some(provenance),
    )
    .with_code(ResolverCode::UnsupportedSelectedMetaBody)
}

fn form_origin(form: &NormForm) -> lang_syntax::NormOrigin {
    match form {
        NormForm::Let(decl) | NormForm::Alias(decl) => match decl {
            lang_syntax::NormDecl::Let { origin, .. }
            | lang_syntax::NormDecl::Alias { origin, .. } => origin.clone(),
            lang_syntax::NormDecl::Error(error) => error.origin.clone(),
        },
        NormForm::Expr(expr) | NormForm::TailValue(expr) => expr_origin(expr),
        NormForm::ReturnEvent(return_ev) => return_ev.origin.clone(),
        NormForm::Error(error) => error.origin.clone(),
    }
}

fn expr_origin(expr: &lang_syntax::NormExpr) -> lang_syntax::NormOrigin {
    match expr {
        lang_syntax::NormExpr::Call { origin, .. }
        | lang_syntax::NormExpr::Product(lang_syntax::NormProduct { origin, .. })
        | lang_syntax::NormExpr::Name { origin, .. }
        | lang_syntax::NormExpr::Literal { origin, .. }
        | lang_syntax::NormExpr::Nav { origin, .. }
        | lang_syntax::NormExpr::OperatorTarget { origin, .. }
        | lang_syntax::NormExpr::Unsupported { origin, .. } => origin.clone(),
        lang_syntax::NormExpr::Closure(closure) => closure.origin.clone(),
        lang_syntax::NormExpr::Error(error) => error.origin.clone(),
    }
}
