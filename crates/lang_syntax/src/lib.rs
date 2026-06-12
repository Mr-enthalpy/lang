//! v0.1 frontend library.
//!
//! The current implementation includes the minimal lexer loop:
//! source text -> tokens + lexer diagnostics + stable token dump.
//! Parser, AST construction, and parser diagnostics are intentionally deferred.

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
