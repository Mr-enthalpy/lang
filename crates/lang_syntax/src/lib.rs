//! v0.1 frontend library placeholder.
//!
//! Real lexer, parser, AST, diagnostics, and dump modules will be added in later
//! commits. This crate exists now so the workspace can compile.

pub mod diagnostic;
pub mod dump;
pub mod lexer;
pub mod source;
pub mod span;
pub mod token;

pub use diagnostic::{Diagnostic, DiagnosticCode};
pub use dump::{dump_diagnostics, dump_tokens};
pub use lexer::{lex, LexOutput};
pub use span::Span;
pub use token::{Symbol, Token, TokenKind, TriviaKind};

pub const VERSION: &str = "0.1.0";
