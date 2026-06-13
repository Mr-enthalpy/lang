use crate::{Diagnostic, DiagnosticCode, ErrorAst, FormAst, ProgramAst, Span, Symbol, Token};

use super::{cursor::Cursor, expr::parse_expr_until, let_stmt::parse_let};

pub struct Parser<'tokens> {
    pub cursor: Cursor<'tokens>,
    diagnostics: Vec<Diagnostic>,
    nesting_depth: usize,
}

impl<'tokens> Parser<'tokens> {
    pub fn new(tokens: &'tokens [Token], diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            cursor: Cursor::new(tokens),
            diagnostics,
            nesting_depth: 0,
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
            FormAst::Let(parse_let(self))
        } else {
            FormAst::Expr(parse_expr_until(self, |parser| parser.is_form_boundary()))
        }
    }

    pub fn is_form_boundary(&mut self) -> bool {
        if self.nesting_depth == 0 && self.cursor.has_newline_trivia_ahead() {
            return true;
        }
        self.cursor.is_form_boundary()
    }

    pub fn enter_nesting(&mut self) {
        self.nesting_depth += 1;
    }

    pub fn leave_nesting(&mut self) {
        self.nesting_depth = self.nesting_depth.saturating_sub(1);
    }

    pub fn error(&mut self, code: DiagnosticCode, message: impl Into<String>, span: Span) {
        self.diagnostics.push(Diagnostic::new(code, message, span));
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
