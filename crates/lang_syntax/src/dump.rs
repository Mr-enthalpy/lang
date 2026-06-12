use crate::{
    ArgPackAst, ArgPackRole, AtomAst, AtomKind, DeclAnnotationAst, Diagnostic, DiagnosticCode,
    ExprAst, ExprKind, FormAst, LetAst, LetAttrAst, LetBinderAst, PipeExprAst, ProgramAst,
    SegmentAst, SegmentElementAst, Symbol, Token, TokenKind, TriviaKind, TypeObjectAnnotationAst,
};

pub fn dump_tokens(tokens: &[Token]) -> String {
    let mut output = String::new();

    for token in tokens {
        output.push_str(&format!(
            "{} \"{}\" @ {}:{} [{}..{}]\n",
            token_kind_label(&token.kind),
            escape_text(&token.text),
            token.span.line,
            token.span.column,
            token.span.byte_start,
            token.span.byte_end
        ));
    }

    output
}

pub fn dump_diagnostics(diagnostics: &[Diagnostic]) -> String {
    let mut output = String::new();

    for diagnostic in diagnostics {
        output.push_str(&format!(
            "error {} \"{}\" @ {}:{} [{}..{}]\n",
            diagnostic_code_label(diagnostic.code),
            escape_text(&diagnostic.message),
            diagnostic.span.line,
            diagnostic.span.column,
            diagnostic.span.byte_start,
            diagnostic.span.byte_end
        ));
    }

    output
}

pub fn dump_ast(program: &ProgramAst) -> String {
    let mut output = String::new();
    line(&mut output, 0, "Program");

    for form in &program.forms {
        dump_form(&mut output, form, 1);
    }

    output
}

fn dump_form(output: &mut String, form: &FormAst, indent: usize) {
    match form {
        FormAst::Let(let_ast) => dump_let(output, let_ast, indent),
        FormAst::Expr(expr) => {
            line(output, indent, "ExprForm");
            dump_expr(output, expr, indent + 1);
        }
        FormAst::Error(error) => {
            line(
                output,
                indent,
                &format!("Error \"{}\"", escape_text(&error.message)),
            );
        }
    }
}

fn dump_let(output: &mut String, let_ast: &LetAst, indent: usize) {
    line(output, indent, "Let");
    line(output, indent + 1, "attrs:");
    for attr in &let_ast.attrs {
        match attr {
            LetAttrAst::Guard => line(output, indent + 2, "Guard"),
        }
    }
    line(output, indent + 1, "binder:");
    dump_binder(output, &let_ast.binder, indent + 2);
    line(output, indent + 1, "with:");
    for dep in &let_ast.with_deps {
        line(output, indent + 2, &dep.text);
    }
    line(output, indent + 1, "value:");
    dump_expr(output, &let_ast.value, indent + 2);
}

fn dump_binder(output: &mut String, binder: &LetBinderAst, indent: usize) {
    match binder {
        LetBinderAst::Simple {
            name, annotation, ..
        } => {
            line(output, indent, &format!("Simple name={}", name.text));
            line(output, indent + 1, "annotation:");
            dump_decl_annotation(output, annotation, indent + 2);
        }
        LetBinderAst::Error(error) => {
            line(
                output,
                indent,
                &format!("Error \"{}\"", escape_text(&error.message)),
            );
        }
    }
}

fn dump_decl_annotation(output: &mut String, annotation: &DeclAnnotationAst, indent: usize) {
    match annotation {
        DeclAnnotationAst::Bare(expr) => {
            line(output, indent, "Bare");
            dump_expr(output, expr, indent + 1);
        }
        DeclAnnotationAst::TypeObjectWithRank {
            type_object_annotation,
            rank_annotation,
            ..
        } => {
            line(output, indent, "TypeObjectWithRank");
            line(output, indent + 1, "type_object:");
            dump_type_object_annotation(output, type_object_annotation, indent + 2);
            line(output, indent + 1, "rank:");
            dump_expr(output, rank_annotation, indent + 2);
        }
        DeclAnnotationAst::Error(error) => {
            line(
                output,
                indent,
                &format!("Error \"{}\"", escape_text(&error.message)),
            );
        }
    }
}

fn dump_type_object_annotation(
    output: &mut String,
    annotation: &TypeObjectAnnotationAst,
    indent: usize,
) {
    match annotation {
        TypeObjectAnnotationAst::Expr(expr) => {
            line(output, indent, "Expr");
            dump_expr(output, expr, indent + 1);
        }
        TypeObjectAnnotationAst::Hole { .. } => line(output, indent, "Hole _"),
    }
}

fn dump_expr(output: &mut String, expr: &ExprAst, indent: usize) {
    line(output, indent, "Expr");
    match &expr.kind {
        ExprKind::Pipe(pipe) => dump_pipe(output, pipe, indent + 1),
        ExprKind::Error(error) => {
            line(
                output,
                indent + 1,
                &format!("Error \"{}\"", escape_text(&error.message)),
            );
        }
    }
}

fn dump_pipe(output: &mut String, pipe: &PipeExprAst, indent: usize) {
    line(output, indent, "Pipe");
    for segment in &pipe.segments {
        dump_segment(output, segment, indent + 1);
    }
}

fn dump_segment(output: &mut String, segment: &SegmentAst, indent: usize) {
    line(
        output,
        indent,
        &format!("Segment has_incoming={}", segment.has_incoming),
    );
    for element in &segment.elements {
        dump_segment_element(output, element, indent + 1);
    }
}

fn dump_segment_element(output: &mut String, element: &SegmentElementAst, indent: usize) {
    match element {
        SegmentElementAst::Atom(atom) => {
            line(output, indent, "Atom");
            dump_atom(output, atom, indent + 1);
        }
        SegmentElementAst::ArgPack(argpack) => dump_argpack(output, argpack, indent),
    }
}

fn dump_argpack(output: &mut String, argpack: &ArgPackAst, indent: usize) {
    line(
        output,
        indent,
        &format!("ArgPack role={}", argpack_role_label(argpack.role)),
    );
    line(output, indent + 1, "args:");
    for arg in &argpack.args {
        dump_expr(output, arg, indent + 2);
    }
}

fn dump_atom(output: &mut String, atom: &AtomAst, indent: usize) {
    match &atom.kind {
        AtomKind::Name(name) => line(output, indent, &format!("Name {}", name.text)),
        AtomKind::IntLiteral(value) => line(output, indent, &format!("IntLiteral {}", value)),
        AtomKind::StringLiteral(value) => {
            line(
                output,
                indent,
                &format!("StringLiteral \"{}\"", escape_text(value)),
            );
        }
        AtomKind::Group(expr) => {
            line(output, indent, "Group");
            dump_expr(output, expr, indent + 1);
        }
        AtomKind::Path { base, names } => {
            line(output, indent, "Path");
            line(output, indent + 1, "base:");
            dump_atom(output, base, indent + 2);
            line(output, indent + 1, "names:");
            for name in names {
                line(output, indent + 2, &name.text);
            }
        }
        AtomKind::Error(error) => {
            line(
                output,
                indent,
                &format!("Error \"{}\"", escape_text(&error.message)),
            );
        }
    }
}

fn line(output: &mut String, indent: usize, text: &str) {
    for _ in 0..indent {
        output.push_str("  ");
    }
    output.push_str(text);
    output.push('\n');
}

fn token_kind_label(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Name => "Name".to_string(),
        TokenKind::IntLiteral => "IntLiteral".to_string(),
        TokenKind::StringLiteral => "StringLiteral".to_string(),
        TokenKind::Symbol(symbol) => format!("Symbol.{}", symbol_label(*symbol)),
        TokenKind::Trivia(trivia) => format!("Trivia.{}", trivia_label(*trivia)),
        TokenKind::Invalid => "Invalid".to_string(),
        TokenKind::Eof => "Eof".to_string(),
    }
}

fn symbol_label(symbol: Symbol) -> &'static str {
    match symbol {
        Symbol::LParen => "LParen",
        Symbol::RParen => "RParen",
        Symbol::LBracket => "LBracket",
        Symbol::RBracket => "RBracket",
        Symbol::LBrace => "LBrace",
        Symbol::RBrace => "RBrace",
        Symbol::Comma => "Comma",
        Symbol::Colon => "Colon",
        Symbol::Equal => "Equal",
        Symbol::Dot => "Dot",
        Symbol::DotDot => "DotDot",
        Symbol::ColonColon => "ColonColon",
        Symbol::PipeGreater => "PipeGreater",
        Symbol::FatArrow => "FatArrow",
        Symbol::ThinArrow => "ThinArrow",
        Symbol::Less => "Less",
        Symbol::Greater => "Greater",
        Symbol::Semicolon => "Semicolon",
    }
}

fn trivia_label(trivia: TriviaKind) -> &'static str {
    match trivia {
        TriviaKind::Whitespace => "Whitespace",
        TriviaKind::LineComment => "LineComment",
        TriviaKind::BlockComment => "BlockComment",
    }
}

fn diagnostic_code_label(code: DiagnosticCode) -> &'static str {
    match code {
        DiagnosticCode::InvalidToken => "InvalidToken",
        DiagnosticCode::UnclosedString => "UnclosedString",
        DiagnosticCode::UnclosedComment => "UnclosedComment",
        DiagnosticCode::UnexpectedToken => "UnexpectedToken",
        DiagnosticCode::ExpectedName => "ExpectedName",
        DiagnosticCode::ExpectedColon => "ExpectedColon",
        DiagnosticCode::ExpectedDeclAnnotation => "ExpectedDeclAnnotation",
        DiagnosticCode::ExpectedEqual => "ExpectedEqual",
        DiagnosticCode::UnclosedParen => "UnclosedParen",
        DiagnosticCode::EmptyPipeSegment => "EmptyPipeSegment",
    }
}

fn argpack_role_label(role: ArgPackRole) -> &'static str {
    match role {
        ArgPackRole::SourcePack => "SourcePack",
        ArgPackRole::InsertPack => "InsertPack",
        ArgPackRole::RightTargetSubsegment => "RightTargetSubsegment",
        ArgPackRole::Unknown => "Unknown",
    }
}

fn escape_text(text: &str) -> String {
    let mut escaped = String::new();

    for ch in text.chars() {
        match ch {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            _ => escaped.push(ch),
        }
    }

    escaped
}
