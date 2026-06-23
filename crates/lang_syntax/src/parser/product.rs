use crate::{DiagnosticCode, ExprAst, ExprKind, ProductElementAst, ProductExprAst, Span, Symbol};

use super::{form::Parser, pipe::parse_pipe_expr};

pub fn parse_product_expr(parser: &mut Parser<'_>) -> ProductExprAst {
    parse_delimited_product_expr(
        parser,
        Symbol::LParen,
        Symbol::RParen,
        DiagnosticCode::UnclosedParen,
        "unclosed parentheses",
    )
}

pub fn parse_bracket_product_expr(parser: &mut Parser<'_>) -> ProductExprAst {
    parse_delimited_product_expr(
        parser,
        Symbol::LBracket,
        Symbol::RBracket,
        DiagnosticCode::UnclosedBracket,
        "unclosed brackets",
    )
}

fn parse_delimited_product_expr(
    parser: &mut Parser<'_>,
    open: Symbol,
    close: Symbol,
    unclosed: DiagnosticCode,
    unclosed_message: &str,
) -> ProductExprAst {
    let open_token = parser
        .cursor
        .consume_symbol(open)
        .expect("parse_delimited_product_expr called at opening delimiter");

    parser.enter_nesting();

    let mut elements = Vec::new();
    let mut expect_element = true;

    while !at_product_end(parser, close) {
        if parser.cursor.at_symbol(Symbol::Comma) {
            let comma = parser.cursor.bump_non_trivia();
            if expect_element {
                elements.push(ProductElementAst::Unit { span: comma.span });
            }
            expect_element = true;
            continue;
        }

        let expr = parse_pipe_expr(parser, |p| {
            p.cursor.at_symbol(Symbol::Comma) || p.cursor.at_symbol(close)
        });
        elements.push(ProductElementAst::Expr(expr));

        if let Some(comma) = parser.cursor.consume_symbol(Symbol::Comma) {
            expect_element = true;
            if at_product_end(parser, close) {
                elements.push(ProductElementAst::Unit { span: comma.span });
                break;
            }
        } else {
            break;
        }
    }

    let end = if let Some(close_token) = parser.cursor.consume_symbol(close) {
        close_token.span
    } else {
        let span = parser.cursor.current_span();
        parser.error(unclosed, unclosed_message, open_token.span);
        span
    };

    let span = open_token.span.join(end);
    parser.leave_nesting();
    ProductExprAst { elements, span }
}

fn at_product_end(parser: &mut Parser<'_>, close: Symbol) -> bool {
    parser.cursor.at_eof() || parser.cursor.at_symbol(close) || parser.is_form_boundary()
}

pub fn product_expr(product: ProductExprAst) -> ExprAst {
    ExprAst {
        span: product.span,
        kind: ExprKind::Product(product),
    }
}

pub fn error_expr(parser: &Parser<'_>, message: &str, span: Span) -> ExprAst {
    ExprAst {
        kind: ExprKind::Error(parser.error_ast(message, span)),
        span,
    }
}
