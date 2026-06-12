use crate::{Diagnostic, DiagnosticCode, ErrorAst, FormAst, ProgramAst, Span, Symbol, Token};

use super::{cursor::Cursor, expr::parse_expr_until, let_stmt::parse_let};

pub struct Parser<'tokens> {
    pub cursor: Cursor<'tokens>,
    diagnostics: Vec<Diagnostic>,
}

impl<'tokens> Parser<'tokens> {
    pub fn new(tokens: &'tokens [Token], diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            cursor: Cursor::new(tokens),
            diagnostics,
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
            FormAst::Expr(parse_expr_until(self, |parser| {
                parser.cursor.is_form_boundary()
            }))
        }
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
        while !self.cursor.is_form_boundary() {
            self.cursor.bump_non_trivia();
        }
    }
}
