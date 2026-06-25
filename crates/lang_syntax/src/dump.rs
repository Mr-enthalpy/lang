use crate::{
    AliasBinderAst, AnnotationTermAst, AtomAst, AtomKind, BinderNameAst, BindingAnnotationAst,
    BindingPatternAst, BindingSlotAst, Diagnostic, DiagnosticCode, EntityRefAst, ExprAst, ExprKind,
    FormAst, HeadClauseAst, LetAliasAst, LetAst, OperatorExprKind, PipeExprAst, ProductElementAst,
    ProductExprAst, ProgramAst, SegmentAst, SegmentElementAst, Symbol, Token, TokenKind,
    TriviaKind, WithClauseKind,
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
        FormAst::AliasLet(alias) => dump_alias_let(output, alias, indent),
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
    line(output, indent + 1, "slot:");
    dump_binding_slot(output, &let_ast.slot, indent + 2);
}

fn dump_binding_slot(output: &mut String, slot: &BindingSlotAst, indent: usize) {
    line(output, indent, &format!("BindingSlot let={}", slot.has_let));
    if let Some(policy) = &slot.policy {
        line(output, indent + 1, "policy:");
        dump_expr(output, policy, indent + 2);
    }
    line(output, indent + 1, "deduce:");
    match &slot.deduce {
        Some(deduce) => dump_deduce_list(output, deduce, indent + 2),
        None => line(output, indent + 2, "None"),
    }
    line(output, indent + 1, "pattern:");
    dump_binding_pattern(output, &slot.pattern, indent + 2);
    line(output, indent + 1, "annotation:");
    match &slot.annotation {
        Some(annotation) => dump_binding_annotation(output, annotation, indent + 2),
        None => line(output, indent + 2, "None"),
    }
    line(output, indent + 1, "with_clause:");
    dump_with_clause(output, &slot.with_clause, indent + 2);
    line(output, indent + 1, "initializer:");
    match &slot.initializer {
        Some(initializer) => dump_expr(output, initializer, indent + 2),
        None => line(output, indent + 2, "None"),
    }
}

fn dump_binding_pattern(output: &mut String, pattern: &BindingPatternAst, indent: usize) {
    match pattern {
        BindingPatternAst::Binder(name) => {
            line(output, indent, "Binder");
            dump_binder_name(output, name, indent + 1);
        }
        BindingPatternAst::Product(product) => {
            line(output, indent, "ProductExtract");
            dump_product_extract(output, product, indent + 1);
        }
        BindingPatternAst::Skeleton(skeleton) => {
            line(output, indent, "PatternSkeleton");
            dump_canonical_skeleton(output, skeleton, indent + 1);
        }
        BindingPatternAst::Error(error) => {
            line(
                output,
                indent,
                &format!("Error \"{}\"", escape_text(&error.message)),
            );
        }
    }
}

fn dump_with_clause(
    output: &mut String,
    with_clause: &Option<crate::WithClauseAst>,
    indent: usize,
) {
    match with_clause {
        None => line(output, indent, "None"),
        Some(with_clause) => match &with_clause.kind {
            WithClauseKind::Empty => line(output, indent, "Empty"),
            WithClauseKind::Items { items } => {
                line(output, indent, "Items");
                for item in items {
                    line(output, indent + 1, &item.text);
                }
            }
            WithClauseKind::Error(error) => line(
                output,
                indent,
                &format!("Error \"{}\"", escape_text(&error.message)),
            ),
        },
    }
}

fn dump_alias_let(output: &mut String, alias: &LetAliasAst, indent: usize) {
    line(output, indent, "LetAlias");
    if let Some(policy) = &alias.policy {
        line(output, indent + 1, "policy:");
        dump_expr(output, policy, indent + 2);
    }
    line(output, indent + 1, "binder:");
    dump_alias_binder(output, &alias.binder, indent + 2);
    line(output, indent + 1, "target:");
    dump_entity_ref(output, &alias.target, indent + 2);
}

fn dump_alias_binder(output: &mut String, binder: &AliasBinderAst, indent: usize) {
    match binder {
        AliasBinderAst::Name(name) => line(output, indent, &format!("Name {}", name.text)),
        AliasBinderAst::Operator(operator) => {
            line(output, indent, &format!("Operator {}", operator.spelling))
        }
        AliasBinderAst::Error(error) => {
            line(
                output,
                indent,
                &format!("Error \"{}\"", escape_text(&error.message)),
            );
        }
    }
}

fn dump_entity_ref(output: &mut String, entity_ref: &EntityRefAst, indent: usize) {
    line(output, indent, "EntityRef");
    line(output, indent + 1, "components:");
    for component in &entity_ref.components {
        dump_nav_component(output, component, indent + 2);
    }
}

fn dump_binder_name(output: &mut String, name: &BinderNameAst, indent: usize) {
    match name {
        BinderNameAst::Text(name) => line(output, indent, &format!("TextName {}", name.text)),
        BinderNameAst::Operator(name) => {
            line(output, indent, &format!("OperatorName {}", name.spelling))
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
        Some(annotation) => dump_annotation_term(output, annotation, indent + 2),
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
        crate::CanonicalSkeletonAst::ProductExtract { elements, .. } => {
            line(output, indent, "CanonicalProductExtract");
            line(output, indent + 1, "elements:");
            for elem in elements {
                match elem {
                    crate::CanonicalProductElementAst::Skeleton(skeleton) => {
                        dump_canonical_skeleton(output, skeleton, indent + 2)
                    }
                    crate::CanonicalProductElementAst::Unit { .. } => {
                        line(output, indent + 2, "CanonicalProductUnit")
                    }
                }
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
        crate::CanonicalSkeletonAst::NavPath { names, .. } => {
            line(output, indent, "CanonicalNavPath");
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

fn dump_binding_annotation(output: &mut String, annotation: &BindingAnnotationAst, indent: usize) {
    match annotation {
        BindingAnnotationAst::Expr(expr) => {
            line(output, indent, "AnnotationExpr");
            dump_expr(output, expr, indent + 1);
        }
        BindingAnnotationAst::Compound { left, right, .. } => {
            line(output, indent, "AnnotationCompound");
            line(output, indent + 1, "left:");
            dump_annotation_term(output, left, indent + 2);
            line(output, indent + 1, "right:");
            dump_expr(output, right, indent + 2);
        }
        BindingAnnotationAst::Error(error) => {
            line(
                output,
                indent,
                &format!("Error \"{}\"", escape_text(&error.message)),
            );
        }
    }
}

fn dump_annotation_term(output: &mut String, annotation: &AnnotationTermAst, indent: usize) {
    match annotation {
        AnnotationTermAst::Expr(expr) => {
            line(output, indent, "AnnotationTermExpr");
            dump_expr(output, expr, indent + 1);
        }
        AnnotationTermAst::Hole { .. } => line(output, indent, "AnnotationTermHole _"),
    }
}

fn dump_expr(output: &mut String, expr: &ExprAst, indent: usize) {
    line(output, indent, "Expr");
    match &expr.kind {
        ExprKind::Pipe(pipe) => dump_pipe(output, pipe, indent + 1),
        ExprKind::Product(product) => dump_product(output, product, indent + 1),
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
        SegmentElementAst::Product(product) => dump_product(output, product, indent),
    }
}

fn dump_operator_expr(output: &mut String, op_expr: &crate::OperatorExprAst, indent: usize) {
    match &op_expr.kind {
        OperatorExprKind::Atom(atom) => {
            line(output, indent, "Atom");
            dump_atom(output, atom, indent + 1);
        }
        OperatorExprKind::Product(product) => {
            dump_product(output, product, indent);
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
        OperatorExprKind::NavPath { components, .. } => {
            line(output, indent, "NavPath");
            line(output, indent + 1, "components:");
            for component in components {
                dump_nav_component(output, component, indent + 2);
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
            dump_product(output, args, indent + 2);
        }
        OperatorExprKind::BracketCallSugar {
            object,
            operator,
            args,
            ..
        } => {
            line(
                output,
                indent,
                &format!(
                    "BracketCallSugar operator=\"{}\"",
                    escape_text(&operator.spelling)
                ),
            );
            line(output, indent + 1, "object:");
            dump_operator_expr(output, object, indent + 2);
            line(output, indent + 1, "args:");
            dump_product(output, args, indent + 2);
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

fn dump_product(output: &mut String, product: &ProductExprAst, indent: usize) {
    line(output, indent, "Product");
    line(output, indent + 1, "elements:");
    for element in &product.elements {
        match element {
            ProductElementAst::Expr(expr) => dump_expr(output, expr, indent + 2),
            ProductElementAst::Unit { .. } => line(output, indent + 2, "ProductUnit"),
        }
    }
}

fn dump_product_extract(output: &mut String, product: &crate::ProductExtractAst, indent: usize) {
    line(output, indent, "elements:");
    for element in &product.elements {
        match element {
            crate::ProductExtractElementAst::Slot(slot) => {
                dump_binding_slot(output, slot, indent + 1)
            }
            crate::ProductExtractElementAst::Unit { .. } => {
                line(output, indent + 1, "ProductExtractUnit")
            }
        }
    }
}

fn dump_atom(output: &mut String, atom: &AtomAst, indent: usize) {
    match &atom.kind {
        AtomKind::Name(name) => line(output, indent, &format!("Name {}", name.text)),
        AtomKind::IntLiteral(value) => line(output, indent, &format!("IntLiteral {}", value)),
        AtomKind::FloatLiteral(value) => line(output, indent, &format!("FloatLiteral {}", value)),
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
        AtomKind::NavPath { components } => {
            line(output, indent, "NavPath");
            line(output, indent + 1, "components:");
            for component in components {
                dump_nav_component(output, component, indent + 2);
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
            dump_product(output, args, indent + 2);
        }
        AtomKind::BracketCallSugar {
            object,
            operator,
            args,
        } => {
            line(
                output,
                indent,
                &format!(
                    "BracketCallSugar operator=\"{}\"",
                    escape_text(&operator.spelling)
                ),
            );
            line(output, indent + 1, "object:");
            dump_atom(output, object, indent + 2);
            line(output, indent + 1, "args:");
            dump_product(output, args, indent + 2);
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
    }
}

fn dump_nav_component(output: &mut String, component: &crate::NavComponentAst, indent: usize) {
    match component {
        crate::NavComponentAst::Text(name) => {
            line(output, indent, &format!("component Name {}", name.text))
        }
        crate::NavComponentAst::Operator(operator) => line(
            output,
            indent,
            &format!("component Operator {}", operator.spelling),
        ),
        crate::NavComponentAst::Group(expr) => {
            line(output, indent, "component Group");
            dump_expr(output, expr, indent + 1);
        }
        crate::NavComponentAst::Error(error) => line(
            output,
            indent,
            &format!("component Error \"{}\"", escape_text(&error.message)),
        ),
    }
}

fn dump_closure(output: &mut String, closure: &crate::ClosureAst, indent: usize) {
    match closure {
        crate::ClosureAst::InPlace(inner) => {
            line(output, indent, "Closure InPlace");
            dump_body_block(output, &inner.body, indent + 1);
        }
        crate::ClosureAst::Explicit(inner) => {
            line(output, indent, "Closure Explicit");
            dump_fn_head_prefix(output, &inner.head, indent + 1);
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
    if !head.clauses.is_empty() {
        line(output, indent + 1, "clauses:");
        for clause in &head.clauses {
            dump_head_clause(output, clause, indent + 2);
        }
    }
}

fn dump_head_clause(output: &mut String, clause: &HeadClauseAst, indent: usize) {
    match clause {
        HeadClauseAst::Require { expr, .. } => {
            line(output, indent, "Require");
            dump_expr(output, expr, indent + 1);
        }
        HeadClauseAst::Pre { expr, .. } => {
            line(output, indent, "Pre");
            dump_expr(output, expr, indent + 1);
        }
        HeadClauseAst::Post { expr, .. } => {
            line(output, indent, "Post");
            dump_expr(output, expr, indent + 1);
        }
        HeadClauseAst::LifetimePre { expr, .. } => {
            line(output, indent, "LifetimePre");
            dump_expr(output, expr, indent + 1);
        }
        HeadClauseAst::LifetimePost { expr, .. } => {
            line(output, indent, "LifetimePost");
            dump_expr(output, expr, indent + 1);
        }
        HeadClauseAst::Error(error) => {
            line(
                output,
                indent,
                &format!("Error \"{}\"", escape_text(&error.message)),
            );
        }
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
    line(output, indent + 1, "extract:");
    dump_product_extract(output, &clause.extract, indent + 2);
}

fn dump_return_clause(output: &mut String, clause: &crate::ReturnClauseAst, indent: usize) {
    line(output, indent, "ReturnClause");
    line(output, indent + 1, "slot:");
    dump_binding_slot(output, &clause.slot, indent + 2);
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
        TokenKind::FloatLiteral => "FloatLiteral".to_string(),
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
        Symbol::TripleEqual => "TripleEqual",
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
        DiagnosticCode::ExpectedBindingAnnotation => "ExpectedBindingAnnotation",
        DiagnosticCode::ExpectedEqual => "ExpectedEqual",
        DiagnosticCode::EmptyPipeSegment => "EmptyPipeSegment",
        DiagnosticCode::ExpectedNameAfterDot => "ExpectedNameAfterDot",
        DiagnosticCode::ExpectedNameAfterDoubleDot => "ExpectedNameAfterDoubleDot",
        DiagnosticCode::ExpectedProductAfterDoubleDotName => "ExpectedProductAfterDoubleDotName",
        DiagnosticCode::UnclosedParen => "UnclosedParen",
        DiagnosticCode::UnclosedBracket => "UnclosedBracket",
        DiagnosticCode::UnclosedBrace => "UnclosedBrace",
        DiagnosticCode::InvalidDeduceList => "InvalidDeduceList",
        DiagnosticCode::InvalidCanonicalSkeleton => "InvalidCanonicalSkeleton",
        DiagnosticCode::InvalidClosureHead => "InvalidClosureHead",
        DiagnosticCode::InvalidOperatorExpression => "InvalidOperatorExpression",
        DiagnosticCode::ChainedNonAssociativeOperator => "ChainedNonAssociativeOperator",
        DiagnosticCode::InvalidNavComponent => "InvalidNavComponent",
        DiagnosticCode::TopLevelComma => "TopLevelComma",
        DiagnosticCode::UnusedClosureAst => "UnusedClosureAst",
        DiagnosticCode::ExpectedAliasTarget => "ExpectedAliasTarget",
        DiagnosticCode::InvalidAliasBinder => "InvalidAliasBinder",
        DiagnosticCode::InvalidAliasPosition => "InvalidAliasPosition",
        DiagnosticCode::InvalidEntityRef => "InvalidEntityRef",
        DiagnosticCode::UnexpectedAliasRhsExpression => "UnexpectedAliasRhsExpression",
        DiagnosticCode::InvalidNumericLiteral => "InvalidNumericLiteral",
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
