pub mod argpack;
pub mod atom;
pub mod canonical;
pub mod cursor;
pub mod deduce;
pub mod expr;
pub mod form;
pub mod let_stmt;
pub mod pipe;

use crate::{lex, Diagnostic, ProgramAst, Token};

pub struct ParseOutput {
    pub tokens: Vec<Token>,
    pub program: ProgramAst,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(source: &str) -> ParseOutput {
    let lex_output = lex(source);
    let tokens = lex_output.tokens;
    let mut parser = form::Parser::new(&tokens, lex_output.diagnostics);
    let program = parser.parse_program();
    let diagnostics = parser.finish();

    ParseOutput {
        tokens,
        program,
        diagnostics,
    }
}
