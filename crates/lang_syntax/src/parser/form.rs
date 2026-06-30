use crate::{
    Diagnostic, DiagnosticCode, ErrorAst, ExprAst, ExprKind, FormAst, ProgramAst, ReturnEventAst,
    ReturnTargetAst, Span, Symbol, Token, TokenKind,
};

use super::{cursor::Cursor, expr::parse_expr_until, let_stmt::parse_let_form};

pub struct Parser<'tokens> {
    pub cursor: Cursor<'tokens>,
    diagnostics: Vec<Diagnostic>,
    nesting_depth: usize,
    diagnostic_gates: Vec<Vec<Diagnostic>>,
}

impl<'tokens> Parser<'tokens> {
    pub fn new(tokens: &'tokens [Token], diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            cursor: Cursor::new(tokens),
            diagnostics,
            nesting_depth: 0,
            diagnostic_gates: Vec::new(),
        }
    }

    pub fn parse_program(&mut self) -> ProgramAst {
        let start = self.cursor.current_span();
        let mut forms = Vec::new();

        while !self.cursor.at_eof() {
            if self.cursor.consume_symbol(Symbol::Semicolon).is_some() {
                continue;
            }

            forms.push(self.parse_form());
            self.cursor.consume_form_boundary();
        }

        let end = self.cursor.current_span();
        ProgramAst {
            forms,
            span: start.join(end),
        }
    }

    pub fn finish(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn parse_form(&mut self) -> FormAst {
        if self.cursor.at_name("let") {
            return parse_let_form(self, None);
        }

        let expr = parse_expr_until(self, |parser| {
            parser.cursor.at_name("let")
                || parser.is_form_boundary()
                || parser.cursor.at_name("return")
        });
        if self.cursor.at_name("let") {
            return parse_let_form(self, Some(expr));
        }

        // Implicit return: `<value> return`
        if self.cursor.at_name("return") {
            let return_span = self.cursor.peek_non_trivia().span;
            if is_empty_expression(&expr) {
                self.error(
                    DiagnosticCode::ReturnRequiresValue,
                    "return requires a value expression; use `() return` for unit return",
                    return_span,
                );
                self.cursor.bump_non_trivia(); // consume 'return'
                self.cursor.consume_form_boundary();
                return FormAst::Error(
                    self.error_ast("return requires a value expression", return_span),
                );
            }
            self.cursor.bump_non_trivia();
            let span = expr.span.join(return_span);
            self.cursor.consume_form_boundary();
            return FormAst::ReturnEvent(ReturnEventAst {
                value: Box::new(expr),
                target: ReturnTargetAst::ImplicitNearest { span: return_span },
                span,
            });
        }

        // Explicit return: `<value> |> (Self return)`
        if let Some(return_event) = try_extract_explicit_return(&expr) {
            self.cursor.consume_form_boundary();
            return FormAst::ReturnEvent(return_event);
        }

        // Detect `return` used in invalid expression position
        if expression_contains_name(&expr, "return") {
            self.error(
                DiagnosticCode::ReturnExpressionNotAllowed,
                "return is only allowed as a block terminal form",
                expr.span,
            );
            return FormAst::Expr(expr);
        }

        FormAst::Expr(expr)
    }

    pub fn is_form_boundary(&mut self) -> bool {
        self.cursor.is_form_boundary()
    }

    pub fn is_alias_rhs_boundary(&mut self) -> bool {
        self.is_alias_rhs_hard_boundary()
    }

    pub fn is_alias_rhs_hard_boundary(&mut self) -> bool {
        matches!(
            self.cursor.peek_non_trivia().kind,
            TokenKind::Eof | TokenKind::Symbol(Symbol::Semicolon | Symbol::RBrace)
        )
    }

    pub fn enter_nesting(&mut self) {
        self.nesting_depth += 1;
    }

    pub fn leave_nesting(&mut self) {
        self.nesting_depth = self.nesting_depth.saturating_sub(1);
    }

    pub fn error(&mut self, code: DiagnosticCode, message: impl Into<String>, span: Span) {
        let diag = Diagnostic::new(code, message, span);
        if let Some(gate) = self.diagnostic_gates.last_mut() {
            gate.push(diag);
        } else {
            self.diagnostics.push(diag);
        }
    }

    pub fn gate_diagnostics(&mut self) {
        self.diagnostic_gates.push(Vec::new());
    }

    pub fn ungate_keep_diagnostics(&mut self) {
        if let Some(mut diagnostics) = self.diagnostic_gates.pop() {
            if let Some(parent) = self.diagnostic_gates.last_mut() {
                parent.append(&mut diagnostics);
            } else {
                self.diagnostics.append(&mut diagnostics);
            }
        }
    }

    pub fn ungate_drop_diagnostics(&mut self) {
        self.diagnostic_gates.pop();
    }

    pub fn error_ast(&self, message: impl Into<String>, span: Span) -> ErrorAst {
        ErrorAst {
            message: message.into(),
            span,
        }
    }

    pub fn unexpected_current(&mut self) {
        let token = self.cursor.bump_non_trivia();
        self.error(
            DiagnosticCode::UnexpectedToken,
            format!("unexpected token `{}`", token.text),
            token.span,
        );
    }

    pub fn recover_to_form_boundary(&mut self) {
        while !self.is_form_boundary() {
            self.cursor.bump_non_trivia();
        }
    }

    pub fn recover_to_paren_close(&mut self) {
        while !self.cursor.at_eof()
            && !self.cursor.at_symbol(Symbol::RParen)
            && !self.is_form_boundary()
        {
            self.cursor.bump_non_trivia();
        }
        if self.cursor.at_symbol(Symbol::RParen) {
            self.cursor.bump_non_trivia();
        }
    }
}

fn is_empty_expression(expr: &ExprAst) -> bool {
    match &expr.kind {
        ExprKind::Pipe(pipe) => pipe.segments.iter().all(|s| s.elements.is_empty()),
        ExprKind::Product(_) => false,
        ExprKind::Error(_) => true,
    }
}

fn try_extract_explicit_return(expr: &ExprAst) -> Option<ReturnEventAst> {
    let ExprKind::Pipe(pipe) = &expr.kind else {
        return None;
    };
    if pipe.segments.len() != 2 {
        return None;
    }
    let lhs_seg = &pipe.segments[0];
    let rhs_seg = &pipe.segments[1];
    if rhs_seg.elements.len() != 1 {
        return None;
    }

    let rhs_elem = &rhs_seg.elements[0];
    let (target, span) = extract_return_target_from_segment_element(rhs_elem)?;

    let value = segment_to_expr(lhs_seg);
    let span = value.span.join(span);
    Some(ReturnEventAst {
        value: Box::new(value),
        target,
        span,
    })
}

fn extract_return_target_from_segment_element(
    elem: &crate::SegmentElementAst,
) -> Option<(ReturnTargetAst, Span)> {
    let crate::SegmentElementAst::OperatorExpr(op) = elem else {
        return None;
    };
    extract_return_target_from_operator(op)
}

fn extract_return_target_from_operator(
    op: &crate::OperatorExprAst,
) -> Option<(ReturnTargetAst, Span)> {
    let crate::OperatorExprKind::Atom(atom) = &op.kind else {
        return None;
    };
    let crate::AtomKind::Group(inner_expr) = &atom.kind else {
        return None;
    };
    let crate::ExprKind::Pipe(inner_pipe) = &inner_expr.kind else {
        return None;
    };
    if inner_pipe.segments.len() != 1 {
        return None;
    }
    let inner_seg = &inner_pipe.segments[0];
    if inner_seg.elements.len() < 1 || inner_seg.elements.len() > 2 {
        return None;
    }
    let last_elem = inner_seg.elements.last().unwrap();
    if !element_is_name(last_elem, "return") {
        return None;
    }
    if inner_seg.elements.len() == 2 {
        let first_elem = &inner_seg.elements[0];
        if !element_is_name(first_elem, "Self") {
            return None;
        }
        Some((
            ReturnTargetAst::Explicit {
                target: Box::new(element_to_expr(first_elem)),
                span: element_span(first_elem),
            },
            atom.span,
        ))
    } else {
        Some((
            ReturnTargetAst::ImplicitNearest {
                span: element_span(last_elem),
            },
            atom.span,
        ))
    }
}

fn expression_contains_name(expr: &ExprAst, name: &str) -> bool {
    match &expr.kind {
        ExprKind::Pipe(pipe) => pipe.segments.iter().any(|seg| {
            seg.elements
                .iter()
                .any(|el| segment_element_contains_name(el, name))
        }),
        ExprKind::Product(prod) => prod.elements.iter().any(|el| match el {
            crate::ProductElementAst::Expr(e) => expression_contains_name(e, name),
            crate::ProductElementAst::Unit { .. } => false,
        }),
        ExprKind::Error(_) => false,
    }
}

fn segment_element_contains_name(el: &crate::SegmentElementAst, name: &str) -> bool {
    match el {
        crate::SegmentElementAst::OperatorExpr(op) => operator_expr_contains_name(op, name),
        crate::SegmentElementAst::Product(prod) => prod.elements.iter().any(|el| match el {
            crate::ProductElementAst::Expr(e) => expression_contains_name(e, name),
            crate::ProductElementAst::Unit { .. } => false,
        }),
    }
}

fn operator_expr_contains_name(op: &crate::OperatorExprAst, name: &str) -> bool {
    match &op.kind {
        crate::OperatorExprKind::Atom(atom) => match &atom.kind {
            crate::AtomKind::Name(n) => &n.text == name,
            crate::AtomKind::Group(expr) => expression_contains_name(expr, name),
            crate::AtomKind::NavPath { components, .. } => components.iter().any(|c| match c {
                crate::NavComponentAst::Text(n) => &n.text == name,
                crate::NavComponentAst::Group(expr) => expression_contains_name(expr, name),
                _ => false,
            }),
            _ => false,
        },
        crate::OperatorExprKind::Product(prod) => prod.elements.iter().any(|el| match el {
            crate::ProductElementAst::Expr(e) => expression_contains_name(e, name),
            crate::ProductElementAst::Unit { .. } => false,
        }),
        crate::OperatorExprKind::OperatorSugar { args, .. } => args
            .iter()
            .any(|arg| operator_expr_contains_name(arg, name)),
        crate::OperatorExprKind::MemberSugar { object, .. }
        | crate::OperatorExprKind::DoubleDotSugar { object, .. }
        | crate::OperatorExprKind::BracketCallSugar { object, .. } => {
            operator_expr_contains_name(object, name)
        }
        crate::OperatorExprKind::NavPath { components, .. } => components.iter().any(|c| match c {
            crate::NavComponentAst::Text(n) => &n.text == name,
            crate::NavComponentAst::Group(expr) => expression_contains_name(expr, name),
            _ => false,
        }),
        crate::OperatorExprKind::Error(_) => false,
    }
}

fn element_is_name(el: &crate::SegmentElementAst, name: &str) -> bool {
    match el {
        crate::SegmentElementAst::OperatorExpr(op) => match &op.kind {
            crate::OperatorExprKind::Atom(atom) => {
                matches!(&atom.kind, crate::AtomKind::Name(n) if &n.text == name)
            }
            _ => false,
        },
        _ => false,
    }
}

fn element_to_expr(el: &crate::SegmentElementAst) -> ExprAst {
    match el {
        crate::SegmentElementAst::OperatorExpr(op) => ExprAst {
            kind: ExprKind::Pipe(crate::PipeExprAst {
                segments: vec![crate::SegmentAst {
                    elements: vec![el.clone()],
                    has_incoming: false,
                    span: op.span,
                }],
                span: op.span,
            }),
            span: op.span,
        },
        crate::SegmentElementAst::Product(prod) => ExprAst {
            kind: ExprKind::Product(prod.clone()),
            span: prod.span,
        },
    }
}

fn element_span(el: &crate::SegmentElementAst) -> Span {
    match el {
        crate::SegmentElementAst::OperatorExpr(op) => op.span,
        crate::SegmentElementAst::Product(prod) => prod.span,
    }
}

fn segment_to_expr(seg: &crate::SegmentAst) -> ExprAst {
    if seg.elements.is_empty() {
        return ExprAst {
            kind: ExprKind::Error(crate::ErrorAst {
                message: "empty segment".to_string(),
                span: seg.span,
            }),
            span: seg.span,
        };
    }
    if seg.elements.len() == 1 {
        let span = match &seg.elements[0] {
            crate::SegmentElementAst::OperatorExpr(_op) => seg.span,
            crate::SegmentElementAst::Product(prod) => prod.span,
        };
        return ExprAst {
            kind: ExprKind::Pipe(crate::PipeExprAst {
                segments: vec![seg.clone()],
                span,
            }),
            span,
        };
    } else {
        ExprAst {
            kind: ExprKind::Pipe(crate::PipeExprAst {
                segments: vec![seg.clone()],
                span: seg.span,
            }),
            span: seg.span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser_with_eof() -> Parser<'static> {
        let tokens = Box::leak(Box::new([Token::new(
            TokenKind::Eof,
            Span::at(0, 1, 1),
            "",
        )]));
        Parser::new(tokens, Vec::new())
    }

    #[test]
    fn nested_diagnostic_gates_keep_into_parent_and_drop_parent() {
        let mut parser = parser_with_eof();
        let span = Span::at(0, 1, 1);

        parser.gate_diagnostics();
        parser.error(DiagnosticCode::UnexpectedToken, "outer", span);
        parser.gate_diagnostics();
        parser.error(DiagnosticCode::UnexpectedToken, "inner", span);
        parser.ungate_keep_diagnostics();
        parser.ungate_drop_diagnostics();

        assert!(parser.finish().is_empty());
    }

    #[test]
    fn nested_diagnostic_gates_drop_inner_and_keep_outer() {
        let mut parser = parser_with_eof();
        let span = Span::at(0, 1, 1);

        parser.gate_diagnostics();
        parser.error(DiagnosticCode::UnexpectedToken, "outer", span);
        parser.gate_diagnostics();
        parser.error(DiagnosticCode::UnexpectedToken, "inner", span);
        parser.ungate_drop_diagnostics();
        parser.ungate_keep_diagnostics();

        let diagnostics = parser.finish();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "outer");
    }
}
