use crate::ExprAst;

use super::{form::Parser, pipe::parse_pipe_expr};

pub fn parse_expr_until(
    parser: &mut Parser<'_>,
    stop: impl FnMut(&mut Parser<'_>) -> bool,
) -> ExprAst {
    parse_pipe_expr(parser, stop)
}
