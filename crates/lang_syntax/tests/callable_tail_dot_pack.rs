use lang_syntax::{
    normalize_program, parse, BindingPatternAst, ClosureAst, ClosureBodyAst, ExprKind, FormAst,
    NormClosureBody, NormDecl, NormExpr, NormForm, NormPattern, NormPatternElem, NormRule,
    OperatorExprKind, SegmentElementAst,
};

fn parsed(source: &str) -> lang_syntax::ParseOutput {
    let output = parse(source);
    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics:\n{}",
        lang_syntax::dump_diagnostics(&output.diagnostics)
    );
    output
}

fn normalized_initializer(source: &str) -> NormExpr {
    let output = parsed(source);
    let program = normalize_program(&output.program);
    let [NormForm::Let(NormDecl::Let { slot, .. })] = program.forms.as_slice() else {
        panic!("expected one let form, got {:#?}", program.forms);
    };
    slot.initializer
        .as_deref()
        .expect("let fixture has initializer")
        .clone()
}

fn normalized_closure(source: &str) -> lang_syntax::NormClosure {
    let NormExpr::Closure(closure) = normalized_initializer(source) else {
        panic!("expected closure initializer");
    };
    closure
}

#[test]
fn callable_tail_normalizes_delete_default_and_named_strategy() {
    let bare_delete = normalized_closure("let f = () -> r => delete;");
    assert!(matches!(
        bare_delete.body,
        NormClosureBody::Delete(ref delete) if delete.message.is_none()
    ));

    let message_delete = normalized_closure("let f = () -> r => (\"reason\") delete;");
    assert!(matches!(
        message_delete.body,
        NormClosureBody::Delete(ref delete) if delete.message.as_deref() == Some("\"reason\"")
    ));

    let invalid_message = parse("let f = () -> r => (reason) delete;");
    assert!(invalid_message
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("string literal")));

    let defaulted = normalized_closure("let f = () -> r => default;");
    assert!(matches!(defaulted.body, NormClosureBody::Defaulted { .. }));

    let named = normalized_closure("let f = () -> r => prefer_named { r };");
    assert!(matches!(
        named.body,
        NormClosureBody::NamedBlock { ref strategy, .. } if strategy == "prefer_named"
    ));
}

#[test]
fn double_bracket_strategy_does_not_steal_the_old_return_extraction_pattern() {
    let escaped = normalized_closure("let f = () -> r [[prefer_named]] { r };");
    assert!(matches!(
        escaped.body,
        NormClosureBody::NamedBlock { ref strategy, .. } if strategy == "prefer_named"
    ));
    assert!(matches!(
        escaped.head.as_ref().unwrap().returns.as_ref().unwrap().value_pattern,
        NormPattern::Binder { ref name, .. } if name == "r"
    ));

    let legacy = normalized_closure("let f = () -> r name { r };");
    assert!(matches!(legacy.body, NormClosureBody::Block(_)));
    assert!(matches!(
        legacy
            .head
            .as_ref()
            .unwrap()
            .returns
            .as_ref()
            .unwrap()
            .value_pattern,
        NormPattern::Skeleton { .. }
    ));
}

#[test]
fn dot_name_is_a_first_class_field_function_closure() {
    let output = parsed(".push;");
    let [FormAst::Expr(expr)] = output.program.forms.as_slice() else {
        panic!("expected one expression form");
    };
    let ExprKind::Pipe(pipe) = &expr.kind else {
        panic!("expected pipe expression shell");
    };
    let SegmentElementAst::OperatorExpr(operator) = &pipe.segments[0].elements[0] else {
        panic!("expected operator-expression atom");
    };
    assert!(matches!(
        operator.kind,
        OperatorExprKind::Atom(lang_syntax::AtomAst {
            kind: lang_syntax::AtomKind::DotClosure { .. },
            ..
        })
    ));

    let program = normalize_program(&output.program);
    let [NormForm::Expr(NormExpr::Closure(closure))] = program.forms.as_slice() else {
        panic!("standalone .push must normalize to a closure");
    };
    assert!(matches!(
        closure.kind,
        lang_syntax::NormClosureKind::Generated {
            rule: NormRule::DotClosureLowering
        }
    ));
    let head = closure.head.as_ref().unwrap();
    assert_eq!(head.params.len(), 2);
    assert!(matches!(
        &head.params[1],
        NormPatternElem::BindingSlot(slot)
            if matches!(&slot.value_pattern, NormPattern::Pack { inner, .. }
                if matches!(inner.as_ref(), NormPattern::Binder { name, .. } if name == "args"))
    ));
}

#[test]
fn compact_member_sugar_calls_the_same_dot_closure_and_double_dot_survives() {
    let member = normalized_initializer("let x = object.push;");
    let NormExpr::Call { target, .. } = member else {
        panic!("object.push must normalize as a call");
    };
    assert!(matches!(
        target.as_ref(),
        NormExpr::Closure(closure)
            if matches!(closure.kind,
                lang_syntax::NormClosureKind::Generated {
                    rule: NormRule::DotClosureLowering
                })
    ));

    let direct_member = normalized_initializer("let x = object..push(value);");
    let NormExpr::Call { target, .. } = direct_member else {
        panic!("double-dot must remain direct call sugar");
    };
    assert!(matches!(
        target.as_ref(),
        NormExpr::Closure(closure)
            if matches!(closure.kind,
                lang_syntax::NormClosureKind::Generated {
                    rule: NormRule::DoubleDotLowering
                })
    ));
}

#[test]
fn explicit_pipe_dot_closure_arguments_join_the_call_source_product() {
    for fixture in [
        "let x = items |> .push value;",
        "let x = items |> .push (value);",
    ] {
        let call = normalized_initializer(fixture);
        let NormExpr::Call { source, target, .. } = call else {
            panic!("{fixture} must normalize to one field-function call");
        };
        assert_eq!(source.elements.len(), 2, "{fixture}");
        assert!(matches!(
            target.as_ref(),
            NormExpr::Closure(closure)
                if matches!(closure.kind,
                    lang_syntax::NormClosureKind::Generated {
                        rule: NormRule::DotClosureLowering
                    })
        ));
    }
}

#[test]
fn compact_member_then_space_argument_remains_an_outer_call() {
    let outer = normalized_initializer("let x = items.push value;");
    let NormExpr::Call { source, target, .. } = outer else {
        panic!("compact member followed by an argument must remain an outer call");
    };
    assert!(matches!(
        target.as_ref(),
        NormExpr::Name { text, .. } if text == "value"
    ));
    let [lang_syntax::NormProductElem::Expr(NormExpr::Call {
        source: member_source,
        target: member_target,
        ..
    })] = source.elements.as_slice()
    else {
        panic!("outer call source must be the already-lowered `items.push`");
    };
    assert_eq!(member_source.elements.len(), 1);
    assert!(matches!(
        member_target.as_ref(),
        NormExpr::Closure(closure)
            if matches!(closure.kind,
                lang_syntax::NormClosureKind::Generated {
                    rule: NormRule::DotClosureLowering
                })
    ));
}

#[test]
fn pack_is_pattern_only_and_each_product_level_allows_one() {
    let closure = normalized_closure("let f = (val: T, ...args) -> r => { r };");
    let head = closure.head.as_ref().unwrap();
    assert!(matches!(
        &head.params[1],
        NormPatternElem::BindingSlot(slot)
            if matches!(&slot.value_pattern, NormPattern::Pack { .. })
    ));

    let nested = parsed("let f = (a, (b, ...inner), ...outer) -> r => { r };");
    assert!(nested.diagnostics.is_empty());

    let duplicate = parse("let f = (a, ...x, ...y) -> r => { r };");
    assert!(duplicate.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == lang_syntax::DiagnosticCode::MultiplePackPatternsAtSameLevel
    }));

    let nested_without_level = parse("let f = (......args) -> r => { r };");
    assert!(nested_without_level.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == lang_syntax::DiagnosticCode::MultiplePackPatternsAtSameLevel
    }));

    let rhs_spread = parse("let x = ...args;");
    assert!(
        !rhs_spread.diagnostics.is_empty(),
        "ellipsis must not acquire an RHS spread interpretation"
    );
}

#[test]
fn pack_is_available_in_every_binding_slot_context() {
    for fixture in [
        "let ...rest = value;",
        "let (head, ...rest) = value;",
        "let f = (...args) -> r => { r };",
        "let f = () -> ...result => { value };",
        "let f = () -> r => { let (head, ...rest) = value; r };",
    ] {
        let output = parse(fixture);
        assert!(
            output.diagnostics.is_empty(),
            "pack binding must be accepted in `{fixture}`:\n{}",
            lang_syntax::dump_diagnostics(&output.diagnostics)
        );
        let _normalized = normalize_program(&output.program);
    }
}

#[test]
fn raw_pack_node_wraps_the_inner_binding_pattern() {
    let output = parsed("let f = (...args) -> r => { r };");
    let [FormAst::Let(let_ast)] = output.program.forms.as_slice() else {
        panic!("expected let declaration");
    };
    let initializer = let_ast.slot.initializer.as_ref().unwrap();
    let ExprKind::Pipe(pipe) = &initializer.kind else {
        panic!("expected closure expression shell");
    };
    let SegmentElementAst::OperatorExpr(operator) = &pipe.segments[0].elements[0] else {
        panic!("expected closure atom");
    };
    let OperatorExprKind::Atom(atom) = &operator.kind else {
        panic!("expected closure atom");
    };
    let lang_syntax::AtomKind::Closure(ClosureAst::Explicit(closure)) = &atom.kind else {
        panic!("expected explicit closure");
    };
    assert!(matches!(closure.body, ClosureBodyAst::Block(_)));
    let params = closure.head.params.as_ref().unwrap();
    let lang_syntax::ProductExtractElementAst::Slot(slot) = &params.extract.elements[0] else {
        panic!("expected parameter slot");
    };
    assert!(matches!(
        &slot.pattern,
        BindingPatternAst::Pack { inner, .. }
            if matches!(inner.as_ref(), BindingPatternAst::Binder(_))
    ));
}
