use crate::{
    Diagnostic, DiagnosticCode, ErrorAst, FormAst, ProgramAst, Span, Symbol, Token, TokenKind,
};

use super::{cursor::Cursor, expr::parse_expr_until, let_stmt::parse_let_form};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Continuation {
    None,
    PipeRight,
    // Future: AtomSuffix, LetValue, ClosureBody, OperatorRight
    //
    // NOTE: Continuation is a phase-2 temporary parser-state enum.
    // It is not the final continuation-frame design.  When closure,
    // operator, deduce-list, and canonical-skeleton parsers land,
    // continuation state should be stack/frame-based rather than a
    // single flat enum on Parser.
}

impl Continuation {
    pub fn is_active(self) -> bool {
        !matches!(self, Continuation::None)
    }
}

pub struct Parser<'tokens> {
    pub cursor: Cursor<'tokens>,
    diagnostics: Vec<Diagnostic>,
    nesting_depth: usize,
    pub continuation: Continuation,
    diagnostic_gates: Vec<Vec<Diagnostic>>,
}

impl<'tokens> Parser<'tokens> {
    pub fn new(tokens: &'tokens [Token], diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            cursor: Cursor::new(tokens),
            diagnostics,
            nesting_depth: 0,
            continuation: Continuation::None,
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
            parse_let_form(self)
        } else {
            FormAst::Expr(parse_expr_until(self, |parser| parser.is_form_boundary()))
        }
    }

    pub fn is_form_boundary(&mut self) -> bool {
        if self.can_promote_newline_to_form_sep() {
            return true;
        }
        self.cursor.is_form_boundary()
    }

    pub fn at_top_level_newline(&self) -> bool {
        self.nesting_depth == 0 && self.cursor.has_newline_trivia_ahead()
    }

    fn can_promote_newline_to_form_sep(&self) -> bool {
        if self.nesting_depth != 0 {
            return false;
        }
        if self.continuation.is_active() {
            return false;
        }
        if !self.cursor.has_newline_trivia_ahead() {
            return false;
        }
        let cursor_index = self.cursor.current_index();
        let prev = self.cursor.peek_prev_significant(cursor_index);
        if !Self::can_end_form_token(prev) {
            return false;
        }
        let (_, next) = self.cursor.peek_at_skip_trivia(cursor_index);
        if !Self::can_start_form_token(next) {
            return false;
        }
        if Self::is_continuation_token(prev) || Self::is_continuation_token(next) {
            return false;
        }
        true
    }

    pub fn can_promote_newline_after_segment_element(&self) -> bool {
        if self.nesting_depth != 0 {
            return false;
        }
        if !self.cursor.has_newline_trivia_ahead() {
            return false;
        }
        let cursor_index = self.cursor.current_index();
        let prev = self.cursor.peek_prev_significant(cursor_index);
        if !Self::can_end_form_token(prev) {
            return false;
        }
        let (_, next) = self.cursor.peek_at_skip_trivia(cursor_index);
        if !Self::can_start_form_token(next) {
            return false;
        }
        if Self::is_continuation_token(prev) || Self::is_continuation_token(next) {
            return false;
        }
        true
    }

    pub fn is_alias_rhs_boundary(&mut self) -> bool {
        if self.nesting_depth == 0 && self.cursor.has_newline_trivia_ahead() {
            let cursor_index = self.cursor.current_index();
            let (_, next) = self.cursor.peek_at_skip_trivia(cursor_index);
            if Self::can_start_form_token(next) && !Self::is_continuation_token(next) {
                return true;
            }
        }
        matches!(
            self.cursor.peek_non_trivia().kind,
            TokenKind::Eof | TokenKind::Symbol(Symbol::Semicolon | Symbol::RBrace)
        )
    }

    fn can_end_form_token(token: &Token) -> bool {
        matches!(
            token.kind,
            TokenKind::Name
                | TokenKind::IntLiteral
                | TokenKind::StringLiteral
                | TokenKind::Symbol(Symbol::RParen)
                | TokenKind::Symbol(Symbol::RBracket)
                | TokenKind::Symbol(Symbol::RBrace)
        )
    }

    fn can_start_form_token(token: &Token) -> bool {
        matches!(
            token.kind,
            TokenKind::Name
                | TokenKind::IntLiteral
                | TokenKind::StringLiteral
                | TokenKind::Symbol(Symbol::LParen)
                | TokenKind::Symbol(Symbol::LBrace)
        )
    }

    fn is_continuation_token(token: &Token) -> bool {
        matches!(
            token.kind,
            TokenKind::Symbol(
                Symbol::PipeGreater
                    | Symbol::Dot
                    | Symbol::DotDot
                    | Symbol::ColonColon
                    | Symbol::Comma
                    | Symbol::FatArrow
                    | Symbol::ThinArrow
                    | Symbol::Equal
                    | Symbol::Colon
                    | Symbol::Less
                    | Symbol::Greater
            ) | TokenKind::Operator(_)
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
