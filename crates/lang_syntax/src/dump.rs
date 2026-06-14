use crate::{
    ArgPackAst, ArgPackRole, AtomAst, AtomKind, DeclAnnotationAst, Diagnostic, DiagnosticCode,
    ExprAst, ExprKind, FormAst, LetAst, LetAttrAst, LetBinderAst, OperatorExprKind, PipeExprAst,
    ProgramAst, SegmentAst, SegmentElementAst, Symbol, Token, TokenKind, TriviaKind,
    TypeObjectAnnotationAst,
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
        LetBinderAst::Extract {
            deduce, skeleton, ..
        } => {
            line(output, indent, "Extract");
            line(output, indent + 1, "deduce:");
            dump_deduce_list(output, deduce, indent + 2);
            line(output, indent + 1, "skeleton:");
            dump_canonical_skeleton(output, skeleton, indent + 2);
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

fn dump_deduce_list(output: &mut String, deduce: &crate::DeduceListAst, indent: usize) {
    line(output, indent, "DeduceList");
    line(output, indent + 1, "binders:");
    for binder in &deduce.binders {
        dump_binder_decl(output, binder, indent + 2);
    }
}

fn dump_binder_decl(output: &mut String, binder: &crate::BinderDeclAst, indent: usize) {
    line(
        output,
        indent,
        &format!("BinderDecl name={}", binder.name.text),
    );
    line(output, indent + 1, "annotation:");
    match &binder.annotation {
        Some(annotation) => dump_type_object_annotation(output, annotation, indent + 2),
        None => line(output, indent + 2, "None"),
    }
}

fn dump_canonical_skeleton(
    output: &mut String,
    skeleton: &crate::CanonicalSkeletonAst,
    indent: usize,
) {
    match skeleton {
        crate::CanonicalSkeletonAst::Segment { elements, .. } => {
            line(output, indent, "CanonicalSegment");
            line(output, indent + 1, "elements:");
            for elem in elements {
                dump_canonical_skeleton(output, elem, indent + 2);
            }
        }
        crate::CanonicalSkeletonAst::ArgPack { elements, .. } => {
            line(output, indent, "CanonicalArgPack");
            line(output, indent + 1, "elements:");
            for elem in elements {
                dump_canonical_skeleton(output, elem, indent + 2);
            }
        }
        crate::CanonicalSkeletonAst::Wildcard { .. } => {
            line(output, indent, "CanonicalWildcard _");
        }
        crate::CanonicalSkeletonAst::Name { name, role, .. } => {
            line(
                output,
                indent,
                &format!(
                    "CanonicalName role={} name={}",
                    canonical_name_role_label(*role),
                    name.text
                ),
            );
        }
        crate::CanonicalSkeletonAst::Path { names, .. } => {
            line(output, indent, "CanonicalPath");
            line(output, indent + 1, "names:");
            for name in names {
                line(output, indent + 2, &name.text);
            }
        }
        crate::CanonicalSkeletonAst::Literal { text, .. } => {
            line(
                output,
                indent,
                &format!("CanonicalLiteral \"{}\"", escape_text(text)),
            );
        }
        crate::CanonicalSkeletonAst::Error(error) => {
            line(
                output,
                indent,
                &format!("Error \"{}\"", escape_text(&error.message)),
            );
        }
    }
}

fn canonical_name_role_label(role: crate::CanonicalNameRole) -> &'static str {
    match role {
        crate::CanonicalNameRole::Hole => "Hole",
        crate::CanonicalNameRole::NodeName => "NodeName",
        crate::CanonicalNameRole::Unknown => "Unknown",
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
        SegmentElementAst::OperatorExpr(op_expr) => {
            line(output, indent, "OperatorExpr");
            dump_operator_expr(output, op_expr, indent + 1);
        }
        SegmentElementAst::ArgPack(argpack) => dump_argpack(output, argpack, indent),
    }
}

fn dump_operator_expr(output: &mut String, op_expr: &crate::OperatorExprAst, indent: usize) {
    match &op_expr.kind {
        OperatorExprKind::Atom(atom) => {
            line(output, indent, "Atom");
            dump_atom(output, atom, indent + 1);
        }
        OperatorExprKind::OperatorSugar {
            operator,
            fixity,
            args,
            ..
        } => {
            line(
                output,
                indent,
                &format!(
                    "OperatorSugar fixity={} operator=\"{}\"",
                    operator_fixity_label(*fixity),
                    escape_text(&operator.spelling)
                ),
            );
            match args.as_slice() {
                [arg] => {
                    line(output, indent + 1, "arg:");
                    dump_operator_expr(output, arg, indent + 2);
                }
                [lhs, rhs] => {
                    line(output, indent + 1, "lhs:");
                    dump_operator_expr(output, lhs, indent + 2);
                    line(output, indent + 1, "rhs:");
                    dump_operator_expr(output, rhs, indent + 2);
                }
                _ => {
                    line(output, indent + 1, "args:");
                    for arg in args {
                        dump_operator_expr(output, arg, indent + 2);
                    }
                }
            }
        }
        OperatorExprKind::Path { base, names, .. } => {
            line(output, indent, "Path");
            line(output, indent + 1, "base:");
            dump_operator_expr(output, base, indent + 2);
            line(output, indent + 1, "names:");
            for selector in names {
                dump_selector(output, selector, indent + 2);
            }
        }
        OperatorExprKind::MemberSugar {
            object, selector, ..
        } => {
            line(output, indent, "MemberSugar");
            line(output, indent + 1, "object:");
            dump_operator_expr(output, object, indent + 2);
            line(output, indent + 1, "selector:");
            dump_selector(output, selector, indent + 2);
        }
        OperatorExprKind::DoubleDotSugar {
            object,
            selector,
            args,
            ..
        } => {
            line(output, indent, "DoubleDotSugar");
            line(output, indent + 1, "object:");
            dump_operator_expr(output, object, indent + 2);
            line(output, indent + 1, "selector:");
            dump_selector(output, selector, indent + 2);
            line(output, indent + 1, "args:");
            dump_argpack(output, args, indent + 2);
        }
        OperatorExprKind::Error(error) => {
            line(
                output,
                indent,
                &format!("Error \"{}\"", escape_text(&error.message)),
            );
        }
    }
}

fn operator_fixity_label(fixity: crate::OperatorFixity) -> &'static str {
    match fixity {
        crate::OperatorFixity::Prefix => "Prefix",
        crate::OperatorFixity::Postfix => "Postfix",
        crate::OperatorFixity::Binary => "Binary",
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
            for selector in names {
                dump_selector(output, selector, indent + 2);
            }
        }
        AtomKind::MemberSugar { object, selector } => {
            line(output, indent, "MemberSugar");
            line(output, indent + 1, "object:");
            dump_atom(output, object, indent + 2);
            line(output, indent + 1, "selector:");
            dump_selector(output, selector, indent + 2);
        }
        AtomKind::DoubleDotSugar {
            object,
            selector,
            args,
        } => {
            line(output, indent, "DoubleDotSugar");
            line(output, indent + 1, "object:");
            dump_atom(output, object, indent + 2);
            line(output, indent + 1, "selector:");
            dump_selector(output, selector, indent + 2);
            line(output, indent + 1, "args:");
            dump_argpack(output, args, indent + 2);
        }
        AtomKind::Closure(closure) => dump_closure(output, closure, indent),
        AtomKind::Error(error) => {
            line(
                output,
                indent,
                &format!("Error \"{}\"", escape_text(&error.message)),
            );
        }
    }
}

fn dump_selector(output: &mut String, selector: &crate::SelectorAst, indent: usize) {
    match selector {
        crate::SelectorAst::Text(name) => line(output, indent, &format!("TextName {}", name.text)),
        crate::SelectorAst::Numeric(num) => {
            line(output, indent, &format!("NumericName {}", num.text))
        }
    }
}

fn dump_closure(output: &mut String, closure: &crate::ClosureAst, indent: usize) {
    match closure {
        crate::ClosureAst::Inline(inner) => {
            line(output, indent, "Closure Inline");
            if let Some(head) = &inner.head {
                dump_fn_head_prefix(output, head, indent + 1);
            }
            dump_body_block(output, &inner.body, indent + 1);
        }
        crate::ClosureAst::Explicit(inner) => {
            line(output, indent, "Closure Explicit");
            if let Some(head) = &inner.head {
                dump_fn_head_prefix(output, head, indent + 1);
            }
            dump_body_block(output, &inner.body, indent + 1);
        }
    }
}

fn dump_body_block(output: &mut String, body: &crate::BodyBlockAst, indent: usize) {
    line(output, indent, "BodyBlock");
    line(output, indent + 1, "forms:");
    for form in &body.forms {
        dump_form(output, form, indent + 2);
    }
}

fn dump_fn_head_prefix(output: &mut String, head: &crate::FnHeadPrefixAst, indent: usize) {
    line(output, indent, "FnHeadPrefix");
    if let Some(deduce) = &head.deduce {
        line(output, indent + 1, "deduce:");
        dump_deduce_list(output, deduce, indent + 2);
    }
    if let Some(captures) = &head.captures {
        line(output, indent + 1, "captures:");
        dump_capture_clause(output, captures, indent + 2);
    }
    if let Some(params) = &head.params {
        line(output, indent + 1, "params:");
        dump_param_clause(output, params, indent + 2);
    }
    if let Some(trait_expr) = &head.fn_item_trait {
        line(output, indent + 1, "fn_item_trait:");
        dump_expr(output, trait_expr, indent + 2);
    }
    if let Some(returns) = &head.returns {
        line(output, indent + 1, "returns:");
        dump_return_clause(output, returns, indent + 2);
    }
}

fn dump_capture_clause(output: &mut String, clause: &crate::CaptureClauseAst, indent: usize) {
    line(output, indent, "CaptureClause");
    line(output, indent + 1, "items:");
    for item in &clause.items {
        dump_expr(output, &item.expr, indent + 2);
    }
}

fn dump_param_clause(output: &mut String, clause: &crate::ParamClauseAst, indent: usize) {
    line(output, indent, "ParamClause");
    line(output, indent + 1, "params:");
    for param in &clause.params {
        dump_param_item(output, param, indent + 2);
    }
}

fn dump_param_item(output: &mut String, item: &crate::ParamItemAst, indent: usize) {
    match item {
        crate::ParamItemAst::NameParam {
            name, annotation, ..
        } => {
            line(output, indent, &format!("NameParam name={}", name.text));
            line(output, indent + 1, "annotation:");
            match annotation {
                Some(a) => dump_type_object_annotation(output, a, indent + 2),
                None => line(output, indent + 2, "None"),
            }
        }
        crate::ParamItemAst::ExtractParam {
            deduce,
            skeleton,
            annotation,
            ..
        } => {
            line(output, indent, "ExtractParam");
            line(output, indent + 1, "deduce:");
            match deduce {
                Some(d) => dump_deduce_list(output, d, indent + 2),
                None => line(output, indent + 2, "None"),
            }
            line(output, indent + 1, "skeleton:");
            dump_canonical_skeleton(output, skeleton, indent + 2);
            line(output, indent + 1, "annotation:");
            match annotation {
                Some(a) => dump_type_object_annotation(output, a, indent + 2),
                None => line(output, indent + 2, "None"),
            }
        }
        crate::ParamItemAst::Error(error) => {
            line(
                output,
                indent,
                &format!("Error \"{}\"", escape_text(&error.message)),
            );
        }
    }
}

fn dump_return_clause(output: &mut String, clause: &crate::ReturnClauseAst, indent: usize) {
    line(output, indent, "ReturnClause");
    line(output, indent + 1, "binder:");
    dump_return_binder(output, &clause.binder, indent + 2);
    if let Some(constraint) = &clause.constraint {
        line(output, indent + 1, "constraint:");
        dump_expr(output, constraint, indent + 2);
    }
}

fn dump_return_binder(output: &mut String, binder: &crate::ReturnBinderAst, indent: usize) {
    match binder {
        crate::ReturnBinderAst::TypeExpr(expr) => {
            line(output, indent, "TypeExpr");
            dump_expr(output, expr, indent + 1);
        }
        crate::ReturnBinderAst::ExtractType {
            deduce, skeleton, ..
        } => {
            line(output, indent, "ExtractType");
            line(output, indent + 1, "deduce:");
            dump_deduce_list(output, deduce, indent + 2);
            line(output, indent + 1, "skeleton:");
            dump_canonical_skeleton(output, skeleton, indent + 2);
        }
        crate::ReturnBinderAst::Error(error) => {
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
        TokenKind::Operator(spelling) => {
            format!("Operator.{}", spelling.label())
        }
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
        DiagnosticCode::EmptyPipeSegment => "EmptyPipeSegment",
        DiagnosticCode::ExpectedNameAfterDot => "ExpectedNameAfterDot",
        DiagnosticCode::ExpectedNameAfterDoubleDot => "ExpectedNameAfterDoubleDot",
        DiagnosticCode::ExpectedArgPackAfterDoubleDotName => "ExpectedArgPackAfterDoubleDotName",
        DiagnosticCode::UnclosedParen => "UnclosedParen",
        DiagnosticCode::UnclosedBracket => "UnclosedBracket",
        DiagnosticCode::UnclosedBrace => "UnclosedBrace",
        DiagnosticCode::InvalidDeduceList => "InvalidDeduceList",
        DiagnosticCode::InvalidCanonicalSkeleton => "InvalidCanonicalSkeleton",
        DiagnosticCode::InvalidClosureHead => "InvalidClosureHead",
        DiagnosticCode::InvalidOperatorExpression => "InvalidOperatorExpression",
        DiagnosticCode::ChainedNonAssociativeOperator => "ChainedNonAssociativeOperator",
        DiagnosticCode::TopLevelComma => "TopLevelComma",
        DiagnosticCode::UnusedClosureAst => "UnusedClosureAst",
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
