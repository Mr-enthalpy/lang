use lang_syntax::{NormForm, NormOrigin, NormProgram, NormReturnEvent};

pub struct ControlFlowEndReport {
    pub terminal: Option<ControlFlowTerminal>,
    pub diagnostics: Vec<ControlFlowEndDiagnostic>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_expressions_do_not_terminate_but_explicit_unit_return_does() {
        for source in ["(); value;", "value; ();", "(); let x = value;"] {
            let parsed = lang_syntax::parse(source);
            assert!(parsed.diagnostics.is_empty());
            let report =
                compute_control_flow_end_report(&lang_syntax::normalize_program(&parsed.program));
            assert!(report.terminal.is_none());
            assert!(report.diagnostics.is_empty());
        }
        let parsed = lang_syntax::parse("() return; value;");
        let report =
            compute_control_flow_end_report(&lang_syntax::normalize_program(&parsed.program));
        assert!(matches!(
            report.terminal,
            Some(ControlFlowTerminal::ReturnEvent(_))
        ));
        assert_eq!(report.diagnostics.len(), 1);
    }
}

#[derive(Debug)]
pub enum ControlFlowTerminal {
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
        NormForm::ReturnEvent(return_ev) => return_ev.origin.clone(),
        NormForm::Expr(expr) => expr_origin(expr),
        NormForm::Error(error) => error.origin.clone(),
    }
}

fn expr_origin(expr: &lang_syntax::NormExpr) -> lang_syntax::NormOrigin {
    match expr {
        lang_syntax::NormExpr::InterpretationFlip { origin, .. }
        | lang_syntax::NormExpr::PolicyLet { origin, .. }
        | lang_syntax::NormExpr::Call { origin, .. }
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
