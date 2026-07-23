use lang_syntax::{
    normalize_and_validate, normalize_program, parse, validate_normalized_patterns,
    BindingPatternAst, ClosureBodyAst, ClosurePlacementAst, ExprKind, FormAst, NormAnnotation,
    NormBindingSlot, NormClosureBody, NormClosurePlacement, NormDecl, NormExpr, NormForm,
    NormOrigin, NormPattern, NormPatternElem, NormRule, OperatorExprKind, SegmentElementAst, Span,
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
    normalized_initializer_at(source, 0)
}

fn normalized_initializer_at(source: &str, index: usize) -> NormExpr {
    let output = parsed(source);
    let program = normalize_program(&output.program);
    let Some(NormForm::Let(NormDecl::Let { slot, .. })) = program.forms.get(index) else {
        panic!(
            "expected let form at index {index}, got {:#?}",
            program.forms
        );
    };
    slot.initializer
        .as_deref()
        .expect("let fixture has initializer")
        .clone()
}

fn normalized_initializer_with_diagnostics(source: &str) -> (NormExpr, lang_syntax::ParseOutput) {
    let output = parse(source);
    let program = normalize_program(&output.program);
    let [NormForm::Let(NormDecl::Let { slot, .. })] = program.forms.as_slice() else {
        panic!("expected one let form, got {:#?}", program.forms);
    };
    (
        slot.initializer
            .as_deref()
            .expect("let fixture has initializer")
            .clone(),
        output,
    )
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
    let (invalid_message_expr, _) =
        normalized_initializer_with_diagnostics("let f = () -> r => (reason) delete;");
    assert!(matches!(invalid_message_expr, NormExpr::Error(_)));

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
    assert_eq!(escaped.placement, NormClosurePlacement::InPlace);
    assert!(matches!(
        escaped.body,
        NormClosureBody::NamedBlock { ref strategy, .. } if strategy == "prefer_named"
    ));
    assert!(matches!(
        escaped.head.as_ref().unwrap().returns.as_ref().unwrap().value_pattern,
        NormPattern::Binder { ref name, .. } if name == "r"
    ));

    let legacy = normalized_closure("let f = () -> r name { r };");
    assert_eq!(legacy.placement, NormClosurePlacement::InPlace);
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
fn double_bracket_strategy_uses_one_closure_head_boundary_in_every_context() {
    for fixture in [
        "let f = () [[s]] { value };",
        "let f = () : compile [[s]] { value };",
        "let f = () require C [[s]] { value };",
        "let f = <T> [[s]] { value };",
        "let f = <T>() [[s]] { value };",
        "let f = <T>() -> r [[s]] { value };",
    ] {
        let closure = normalized_closure(fixture);
        assert_eq!(
            closure.placement,
            NormClosurePlacement::InPlace,
            "{fixture}"
        );
        assert!(
            matches!(
                closure.body,
                NormClosureBody::NamedBlock { ref strategy, .. } if strategy == "s"
            ),
            "{fixture}"
        );
    }

    let output = parsed("value |> () [[s]] { value };");
    let normalized = normalize_program(&output.program);
    let dump = lang_syntax::dump_norm_program(&normalized);
    assert!(dump.contains("Closure placement=InPlace"), "{dump}");
    assert!(dump.contains("UserBody strategy=Named(s)"), "{dump}");
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
    assert_eq!(closure.placement, NormClosurePlacement::InPlace);
    assert!(matches!(
        closure.origin,
        NormOrigin::Generated {
            rule: NormRule::DotClosureLowering,
            ..
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
            if is_generated_closure(closure, NormRule::DotClosureLowering)
    ));

    let direct_member = normalized_initializer("let x = object..push(value);");
    let NormExpr::Call { target, .. } = direct_member else {
        panic!("double-dot must remain direct call sugar");
    };
    assert!(matches!(
        target.as_ref(),
        NormExpr::Closure(closure)
            if is_generated_closure(closure, NormRule::DoubleDotLowering)
    ));
}

#[test]
fn dot_closure_has_no_pipe_or_product_binding_privilege() {
    for (dot_source, bound_source) in [
        (
            "let x = items |> .push value;",
            "let d = .push; let x = items |> d value;",
        ),
        (
            "let x = items |> .push (value);",
            "let d = .push; let x = items |> d (value);",
        ),
    ] {
        let dot = normalized_initializer(dot_source);
        let bound = normalized_initializer_at(bound_source, 1);
        assert_eq!(
            expression_binding_shape(&dot),
            expression_binding_shape(&bound),
            "replacing `.push` with an ordinary bound expression must preserve the general call spine"
        );
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
            if is_generated_closure(closure, NormRule::DotClosureLowering)
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
    let lang_syntax::AtomKind::Closure(closure) = &atom.kind else {
        panic!("expected closure");
    };
    assert_eq!(closure.placement, ClosurePlacementAst::Ordinary);
    assert!(matches!(closure.body, ClosureBodyAst::Block(_)));
    let params = closure.head.as_ref().unwrap().params.as_ref().unwrap();
    let lang_syntax::ProductExtractElementAst::Slot(slot) = &params.extract.elements[0] else {
        panic!("expected parameter slot");
    };
    assert!(matches!(
        &slot.pattern,
        BindingPatternAst::Pack { inner, .. }
            if matches!(inner.as_ref(), BindingPatternAst::Binder(_))
    ));
}

#[test]
fn closure_placement_is_independent_of_head_presence_and_strategy() {
    for fixture in [
        "let f = { value };",
        "let f = () -> r { r };",
        "let f = () -> r [[prefer_named]] { r };",
    ] {
        assert_eq!(
            normalized_closure(fixture).placement,
            NormClosurePlacement::InPlace,
            "{fixture}"
        );
    }

    for fixture in [
        "let f = () -> r => { r };",
        "let f = () -> r => prefer_named { r };",
        "let f = () -> r => default;",
        "let f = () -> r => delete;",
    ] {
        assert_eq!(
            normalized_closure(fixture).placement,
            NormClosurePlacement::Ordinary,
            "{fixture}"
        );
    }

    for fixture in [
        "value |> () -> r { r };",
        "value |> () -> r [[prefer_named]] { r };",
    ] {
        let output = parse(fixture);
        assert!(
            output.diagnostics.is_empty(),
            "headed in-place closure supplies its own extraction head in `{fixture}`:\n{}",
            lang_syntax::dump_diagnostics(&output.diagnostics)
        );
    }
}

#[test]
fn in_place_closure_rejects_capture_environment() {
    for fixture in ["let f = [x] { x };", "let f = [x]() -> r { r };"] {
        let (expr, output) = normalized_initializer_with_diagnostics(fixture);
        assert!(output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("cannot have a capture list")));
        assert!(
            matches!(expr, NormExpr::Error(_)),
            "invalid in-place capture must remain an error expression: {fixture}"
        );
    }
}

#[test]
fn malformed_callable_tails_never_normalize_as_user_bodies() {
    for fixture in [
        "let f = () -> r => (reason) delete;",
        "let f = () -> r => strategy;",
        "let f = () -> r [[strategy { r };",
    ] {
        let (expr, output) = normalized_initializer_with_diagnostics(fixture);
        assert!(!output.diagnostics.is_empty(), "{fixture}");
        assert!(
            matches!(expr, NormExpr::Error(_)),
            "malformed callable tail must not become an executable body: {fixture}"
        );
    }
}

#[test]
fn global_pack_validation_visits_every_binding_slot_context() {
    for fixture in [
        "let (...x, ...y) = value;",
        "let f = () -> r => { let (...x, ...y) = value; r };",
        "let f = (...x, ...y) -> r => { r };",
        "let f = () -> (...x, ...y) => { value };",
        "let f = (outer, (inner, ...x, ...y)) -> r => { r };",
    ] {
        let output = parse(fixture);
        let normalized = normalize_program(&output.program);
        assert!(
            validate_normalized_patterns(&normalized).is_err(),
            "global normalized Pattern validation must reject `{fixture}`"
        );
    }

    let origin = NormOrigin::Source(Span::new(0, 1, 1, 1));
    let pack = |name: &str| NormPattern::Pack {
        inner: Box::new(NormPattern::Binder {
            name: name.to_string(),
            origin: origin.clone(),
        }),
        origin: origin.clone(),
    };
    let annotation = NormAnnotation {
        pattern: NormPattern::Sequence {
            elements: vec![pack("x"), pack("y")],
            origin: origin.clone(),
        },
        origin: origin.clone(),
    };
    let program = lang_syntax::NormProgram {
        forms: vec![NormForm::Let(NormDecl::Let {
            slot: NormBindingSlot {
                policy: None,
                has_let: true,
                deduce: Vec::new(),
                value_pattern: NormPattern::Binder {
                    name: "value".to_string(),
                    origin: origin.clone(),
                },
                annotation: Some(annotation),
                with_clause: None,
                initializer: None,
                origin: origin.clone(),
            },
            origin: origin.clone(),
        })],
        origin,
    };
    assert!(
        validate_normalized_patterns(&program).is_err(),
        "Sequence and annotation levels must use the same one-pack validator"
    );

    let valid = parse("let (head, ...rest) = value;");
    assert!(normalize_and_validate(&valid.program).is_ok());

    let invalid = parse("let (...x, ...y) = value;");
    let failure = normalize_and_validate(&invalid.program)
        .expect_err("downstream handoff must reject an invalid normalized Pattern");
    assert!(!failure.pattern_errors.is_empty());
}

fn expression_binding_shape(expr: &NormExpr) -> String {
    match expr {
        NormExpr::Call { source, target, .. } => format!(
            "call([{}],{})",
            source
                .elements
                .iter()
                .map(|element| match element {
                    lang_syntax::NormProductElem::Expr(expr) => expression_binding_shape(expr),
                    lang_syntax::NormProductElem::Unit { .. } => "unit".to_string(),
                })
                .collect::<Vec<_>>()
                .join(","),
            expression_binding_shape(target)
        ),
        NormExpr::Product(product) => format!(
            "product([{}])",
            product
                .elements
                .iter()
                .map(|element| match element {
                    lang_syntax::NormProductElem::Expr(expr) => expression_binding_shape(expr),
                    lang_syntax::NormProductElem::Unit { .. } => "unit".to_string(),
                })
                .collect::<Vec<_>>()
                .join(",")
        ),
        NormExpr::Closure(closure)
            if is_generated_closure(closure, NormRule::DotClosureLowering) =>
        {
            "$field".to_string()
        }
        NormExpr::Name { text, .. } if text == "d" => "$field".to_string(),
        NormExpr::Name { text, .. } => format!("name({text})"),
        NormExpr::Literal { text, .. } => format!("literal({text})"),
        NormExpr::Nav { .. } => "nav".to_string(),
        NormExpr::Closure(_) => "closure".to_string(),
        NormExpr::OperatorTarget { spelling, .. } => format!("operator({spelling})"),
        NormExpr::Error(_) => "error".to_string(),
        NormExpr::Unsupported { .. } => "unsupported".to_string(),
    }
}

fn is_generated_closure(closure: &lang_syntax::NormClosure, rule: NormRule) -> bool {
    matches!(
        closure.origin,
        NormOrigin::Generated {
            rule: actual,
            ..
        } if actual == rule
    )
}
