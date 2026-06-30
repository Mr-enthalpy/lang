use lang_syntax::{NormForm, NormOrigin, NormProgram, NormReturnEvent};

pub struct ControlFlowEndReport {
    pub terminal: Option<ControlFlowTerminal>,
    pub diagnostics: Vec<ControlFlowEndDiagnostic>,
}

#[derive(Debug)]
pub enum ControlFlowTerminal {
    TailValue(lang_syntax::NormExpr),
    ReturnEvent(NormReturnEvent),
}

#[derive(Debug)]
pub enum ControlFlowEndDiagnostic {
    StatementAfterTerminal { origin: NormOrigin },
}

pub fn compute_control_flow_end_report(program: &NormProgram) -> ControlFlowEndReport {
    let mut terminal = None;
    let mut diagnostics = Vec::new();
    let mut seen_terminal = false;

    for form in &program.forms {
        if seen_terminal {
            diagnostics.push(ControlFlowEndDiagnostic::StatementAfterTerminal {
                origin: form_origin(form),
            });
            continue;
        }

        match form {
            NormForm::TailValue(expr) => {
                terminal = Some(ControlFlowTerminal::TailValue(expr.clone()));
                seen_terminal = true;
            }
            NormForm::ReturnEvent(return_ev) => {
                terminal = Some(ControlFlowTerminal::ReturnEvent(return_ev.clone()));
                seen_terminal = true;
            }
            NormForm::Let(_) | NormForm::Alias(_) | NormForm::Expr(_) | NormForm::Error(_) => {}
        }
    }

    ControlFlowEndReport {
        terminal,
        diagnostics,
    }
}

fn form_origin(form: &NormForm) -> NormOrigin {
    match form {
        NormForm::Let(decl) | NormForm::Alias(decl) => match decl {
            lang_syntax::NormDecl::Let { origin, .. }
            | lang_syntax::NormDecl::Alias { origin, .. } => origin.clone(),
            lang_syntax::NormDecl::Error(error) => error.origin.clone(),
        },
        NormForm::TailValue(expr) => expr_origin(expr),
        NormForm::ReturnEvent(return_ev) => return_ev.origin.clone(),
        NormForm::Expr(expr) => expr_origin(expr),
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
