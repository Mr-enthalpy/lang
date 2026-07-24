use lang_syntax::{
    normalize_and_validate_patterns, normalize_program, parse, validate_normalized_patterns,
    BindingPatternAst, CanonicalSkeletonAst, ClosureBodyAst, ClosurePlacementAst, DiagnosticCode,
    ExprKind, FormAst, NormAnnotation, NormBindingSlot, NormClosureBody, NormClosurePlacement,
    NormDecl, NormExpr, NormForm, NormNavComponent, NormOrigin, NormPattern, NormPatternElem,
    NormPolicyAtom, NormProductElem, NormRule, NormValuePolicyPattern, OperatorExprKind,
    SegmentElementAst, Span,
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

    for expected_strategy in ["default", "delete"] {
        let fixture = format!("let f = () -> r => {expected_strategy} {{ r }};");
        let closure = normalized_closure(&fixture);
        assert!(
            matches!(
                closure.body,
                NormClosureBody::NamedBlock {
                    ref strategy,
                    ..
                } if strategy == expected_strategy
            ),
            "`Name {{ ... }}` must be selected before the bare default/delete forms: {fixture}"
        );
    }
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
        NormPattern::Sequence { .. }
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
        "let f = () require obj[[cap] => { cap }] [[s]] { value };",
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
fn complete_strategy_annotation_does_not_steal_bracket_call_capture_closures() {
    for fixture in [
        "let x = obj[[cap] => { cap }];",
        "let x = ()[[cap] => { cap }];",
        "let x = (a + b)[[cap] => { cap }];",
    ] {
        let output = parse(fixture);
        assert!(
            output.diagnostics.is_empty(),
            "{fixture}\n{}",
            lang_syntax::dump_diagnostics(&output.diagnostics)
        );
        let dump = lang_syntax::dump_ast(&output.program);
        assert!(dump.contains("BracketCallSugar"), "{fixture}\n{dump}");
        assert!(dump.contains("Closure Ordinary"), "{fixture}\n{dump}");
        assert!(dump.contains("CaptureClause"), "{fixture}\n{dump}");
        assert!(
            !dump.contains("OverloadStrategy"),
            "ordinary bracket-call payload must not become strategy metadata: {fixture}\n{dump}"
        );
    }
}

#[test]
fn capture_items_normalize_to_let_shaped_bindings() {
    for (fixture, expected) in [
        ("let f = [x]() => { x };", "x"),
        ("let f = [x x]() => { x };", "x"),
        ("let f = [x y z]() => { x };", "x"),
        ("let f = [x let]() => { x };", "x"),
        ("let f = [(x, x) |> x]() => { x };", "x"),
    ] {
        let closure = normalized_closure(fixture);
        let capture = &closure.head.as_ref().unwrap().captures[0];
        assert!(capture.slot.has_let, "{fixture}");
        assert!(capture.slot.initializer.is_none(), "{fixture}");
        assert!(
            matches!(&capture.slot.value_pattern, NormPattern::Binder { name, .. } if name == expected),
            "{fixture}: {:#?}",
            capture.slot.value_pattern
        );
    }

    for fixture in [
        "let f = [(x, y) |> z]() => { z };",
        "let f = [(x, y) |> x]() => { x };",
        "let f = [(1, 2) |> make]() => { value };",
    ] {
        let closure = normalized_closure(fixture);
        assert!(
            matches!(
                closure.head.as_ref().unwrap().captures[0]
                    .slot
                    .value_pattern,
                NormPattern::Error(_)
            ),
            "capture shorthand must retain an inference error: {fixture}"
        );
    }

    for fixture in [
        "let f = [let out = x y]() => { out };",
        "let f = [out = x y]() => { out };",
    ] {
        let closure = normalized_closure(fixture);
        let capture = &closure.head.as_ref().unwrap().captures[0];
        assert!(matches!(
            &capture.slot.value_pattern,
            NormPattern::Binder { name, .. } if name == "out"
        ));
        assert!(capture.slot.policy.is_none());
    }

    let closure = normalized_closure("let f = [runtime let out = x]() => { out };");
    let capture = &closure.head.as_ref().unwrap().captures[0];
    assert!(capture.slot.policy.is_some());
    assert!(matches!(
        &capture.slot.value_pattern,
        NormPattern::Binder { name, .. } if name == "out"
    ));

    let closure = normalized_closure("let f = [runtime let <T> out: T with {} = x]() => { out };");
    let capture = &closure.head.as_ref().unwrap().captures[0];
    assert!(capture.slot.policy.is_some());
    assert_eq!(capture.slot.deduce.len(), 1);
    assert!(capture.slot.annotation.is_some());
    assert!(capture.slot.with_clause.is_some());

    let output = parse("let f = [let x === y]() => { x };");
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::InvalidAliasPosition),
        "capture binding must not import form-level alias-let"
    );
}

#[test]
fn capture_inference_ignores_locally_bound_names_and_binds_simultaneously() {
    let closure = normalized_closure("let f = [{ let local = outer; local }]() => { value };");
    assert!(matches!(
        &closure.head.as_ref().unwrap().captures[0]
            .slot
            .value_pattern,
        NormPattern::Binder { name, .. } if name == "outer"
    ));

    let closure = normalized_closure("let f = [let x = outer, let y = outer]() => { value };");
    let captures = &closure.head.as_ref().unwrap().captures;
    assert_eq!(captures.len(), 2);
    assert!(captures.iter().all(|capture| {
        matches!(
            &capture.initializer,
            NormExpr::Name { text, .. } if text == "outer"
        )
    }));
}

#[test]
fn deduce_keeps_capture_slot_open_unless_complete_strategy_tail_is_present() {
    for fixture in [
        "let f = [[cap] => { cap }] () => { value };",
        "let f = <T> [[cap] => { cap }] () => { value };",
    ] {
        let captured = normalized_closure(fixture);
        assert_eq!(captured.placement, NormClosurePlacement::Ordinary);
        let [capture] = captured.head.as_ref().unwrap().captures.as_slice() else {
            panic!("expected one capture: {fixture}");
        };
        assert!(matches!(
            &capture.slot.value_pattern,
            NormPattern::Binder { name, .. } if name == "cap"
        ));
        assert!(matches!(captured.body, NormClosureBody::Block(_)));
    }

    let strategy = normalized_closure("let f = <T> [[s]] { value };");
    assert_eq!(strategy.placement, NormClosurePlacement::InPlace);
    assert!(strategy.head.as_ref().unwrap().captures.is_empty());
    assert!(matches!(
        strategy.body,
        NormClosureBody::NamedBlock { ref strategy, .. } if strategy == "s"
    ));
}

#[test]
fn canonical_sequence_accepts_pack_as_a_direct_pattern_child() {
    let output = parsed("let <T> head ...rest T = value;");
    let normalized = normalize_program(&output.program);
    let [NormForm::Let(NormDecl::Let { slot, .. })] = normalized.forms.as_slice() else {
        panic!("expected one let declaration");
    };
    let NormPattern::Sequence { elements, .. } = &slot.value_pattern else {
        panic!("canonical Pattern sequence must normalize as NormPattern::Sequence");
    };
    assert!(matches!(
        elements.as_slice(),
        [
            NormPattern::Skeleton { .. },
            NormPattern::Pack { inner, .. },
            NormPattern::HoleRef { name, .. }
        ] if matches!(inner.as_ref(), NormPattern::Binder { name, .. } if name == "rest")
            && name == "T"
    ));

    for fixture in ["let <T> head ...x ...y T = value;", "let ......x = value;"] {
        let output = parse(fixture);
        assert!(
            output.diagnostics.is_empty(),
            "parser must preserve Pack shape without normalized-level diagnostics: {fixture}\n{}",
            lang_syntax::dump_diagnostics(&output.diagnostics)
        );
        assert!(
            validate_normalized_patterns(&normalize_program(&output.program)).is_err(),
            "normalized Pattern validator must reject {fixture}"
        );
    }
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
fn pack_is_pattern_only_and_normalized_validation_owns_layer_cardinality() {
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
    assert!(
        duplicate.diagnostics.is_empty(),
        "the parser must preserve all syntactically formed packs without claiming normalized levels"
    );
    assert!(
        validate_normalized_patterns(&normalize_program(&duplicate.program)).is_err(),
        "normalized Pattern validation must reject duplicate packs at one level"
    );

    let nested_without_level = parse("let f = (......args) -> r => { r };");
    assert!(
        nested_without_level.diagnostics.is_empty(),
        "directly nested pack syntax must reach the normalized Pattern validator"
    );
    assert!(
        validate_normalized_patterns(&normalize_program(&nested_without_level.program)).is_err(),
        "the normalized validator must reject directly nested packs"
    );

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
fn bare_product_pack_is_preserved_raw_but_rejected_after_p_normalization() {
    let output = parsed("let ...(a, b) = value;");
    let [FormAst::Let(let_ast)] = output.program.forms.as_slice() else {
        panic!("expected let declaration");
    };
    assert!(matches!(
        &let_ast.slot.pattern,
        BindingPatternAst::Pack { inner, .. }
            if matches!(
                inner.as_ref(),
                BindingPatternAst::Skeleton(CanonicalSkeletonAst::ProductExtract {
                    elements,
                    ..
                }) if elements.len() == 2
            )
    ));

    let normalized = normalize_program(&output.program);
    let [NormForm::Let(NormDecl::Let { slot, .. })] = normalized.forms.as_slice() else {
        panic!("expected normalized let declaration");
    };
    assert!(matches!(
        &slot.value_pattern,
        NormPattern::Pack { inner, .. }
            if matches!(inner.as_ref(), NormPattern::Product { .. })
    ));
    let errors = validate_normalized_patterns(&normalized)
        .expect_err("a bare Product cannot manufacture a stable Pack operand boundary");
    assert!(errors.iter().any(|error| matches!(
        error,
        lang_syntax::PatternValidationError::NonCanonicalPackOperand { .. }
    )));
}

#[test]
fn explicitly_headed_structured_pack_survives_as_a_semantic_candidate() {
    let output = parsed("let ...((a, b) pair) = value;");
    let normalized = normalize_program(&output.program);
    let [NormForm::Let(NormDecl::Let { slot, .. })] = normalized.forms.as_slice() else {
        panic!("expected normalized let declaration");
    };
    assert!(matches!(
        &slot.value_pattern,
        NormPattern::Pack { inner, .. }
            if matches!(inner.as_ref(), NormPattern::Sequence { .. })
    ));
    validate_normalized_patterns(&normalized)
        .expect("an explicitly headed structured operand is not a bare Product");
}

#[test]
fn same_deduce_list_telescope_binds_later_annotation_to_earlier_hole() {
    let output = parsed("let <A, B: A> x = value;");
    let normalized = normalize_program(&output.program);
    let slot = binding_slot_at(&normalized.forms[0]);
    let [a, b] = slot.deduce.as_slice() else {
        panic!("expected two telescope binders");
    };
    assert_eq!(
        annotation_hole_target(b.annotation.as_ref().expect("B annotation")),
        a.id,
        "the later B annotation must target the preceding A binder"
    );
}

#[test]
fn nested_deduce_lists_form_a_left_to_right_telescope_with_exact_ids() {
    let output = parsed("let <A> (let <B: A> (let <C: A, D: B> x: D)) = value;");
    let normalized = normalize_program(&output.program);
    let [NormForm::Let(NormDecl::Let { slot: outer, .. })] = normalized.forms.as_slice() else {
        panic!("expected outer let");
    };
    let outer_a = outer.deduce[0].id;
    let NormPattern::Product { elements, .. } = &outer.value_pattern else {
        panic!("expected outer extraction Product");
    };
    let [NormPatternElem::BindingSlot(middle)] = elements.as_slice() else {
        panic!("expected middle binding slot");
    };
    let middle_b = middle.deduce[0].id;
    assert!(matches!(
        middle.deduce[0]
            .annotation
            .as_ref()
            .map(|annotation| &annotation.pattern),
        Some(NormPattern::HoleRef { target, name, .. })
            if *target == outer_a && name == "A"
    ));

    let NormPattern::Product { elements, .. } = &middle.value_pattern else {
        panic!("expected middle extraction Product");
    };
    let [NormPatternElem::BindingSlot(inner)] = elements.as_slice() else {
        panic!("expected inner binding slot");
    };
    assert_eq!(inner.deduce.len(), 2);
    let inner_c = inner.deduce[0].id;
    let inner_d = inner.deduce[1].id;
    assert_ne!(outer_a, middle_b);
    assert_ne!(middle_b, inner_c);
    assert_ne!(inner_c, inner_d);
    assert!(matches!(
        inner.deduce[0]
            .annotation
            .as_ref()
            .map(|annotation| &annotation.pattern),
        Some(NormPattern::HoleRef { target, name, .. })
            if *target == outer_a && name == "A"
    ));
    assert!(matches!(
        inner.deduce[1]
            .annotation
            .as_ref()
            .map(|annotation| &annotation.pattern),
        Some(NormPattern::HoleRef { target, name, .. })
            if *target == middle_b && name == "B"
    ));
    assert!(matches!(
        inner.annotation.as_ref().map(|annotation| &annotation.pattern),
        Some(NormPattern::HoleRef { target, name, .. })
            if *target == inner_d && name == "D"
    ));
}

#[test]
fn raw_nested_canonical_roles_keep_all_active_deduce_lists() {
    let output = parsed("let <A> (let <B> A B) = value;");
    let [FormAst::Let(let_ast)] = output.program.forms.as_slice() else {
        panic!("expected let");
    };
    let BindingPatternAst::Product(product) = &let_ast.slot.pattern else {
        panic!("expected outer Product");
    };
    let [lang_syntax::ProductExtractElementAst::Slot(inner)] = product.elements.as_slice() else {
        panic!("expected inner slot");
    };
    let BindingPatternAst::Skeleton(CanonicalSkeletonAst::Segment { elements, .. }) =
        &inner.pattern
    else {
        panic!("expected canonical sequence");
    };
    assert!(matches!(
        elements.as_slice(),
        [
            CanonicalSkeletonAst::Name {
                role: lang_syntax::CanonicalNameRole::Hole,
                ..
            },
            CanonicalSkeletonAst::Name {
                role: lang_syntax::CanonicalNameRole::Hole,
                ..
            }
        ]
    ));
}

#[test]
fn raw_callable_deduce_syntax_keeps_postfix_binding_slot_annotations() {
    let output = parsed(
        "let f = <A>[let c: A = source](x: A) -> r: A => {
            let y: A = x;
            let g = <B: A>(z: B) -> q: A => {
                let w: A = z;
                w
            };
            y
        };",
    );
    let dump = lang_syntax::dump_ast(&output.program);
    assert!(
        dump.matches("AnnotationExpr").count() >= 6,
        "capture, parameter, return, and body slots must preserve postfix annotations:\n{dump}"
    );
    assert!(
        dump.matches("Name A").count() >= 5,
        "Raw AST must preserve annotation spellings without preclassifying them as types:\n{dump}"
    );
}

#[test]
fn raw_capture_and_return_hole_roles_normalize_to_the_exact_head_binder() {
    fn raw_pattern_marks_hole(pattern: &BindingPatternAst, expected: &str) -> bool {
        fn skeleton_marks_hole(skeleton: &CanonicalSkeletonAst, expected: &str) -> bool {
            match skeleton {
                CanonicalSkeletonAst::Segment { elements, .. } => elements
                    .iter()
                    .any(|element| skeleton_marks_hole(element, expected)),
                CanonicalSkeletonAst::Pack { inner, .. } => skeleton_marks_hole(inner, expected),
                CanonicalSkeletonAst::ProductExtract { elements, .. } => {
                    elements.iter().any(|element| match element {
                        lang_syntax::CanonicalProductElementAst::Skeleton(skeleton) => {
                            skeleton_marks_hole(skeleton, expected)
                        }
                        lang_syntax::CanonicalProductElementAst::Unit { .. } => false,
                    })
                }
                CanonicalSkeletonAst::Name { name, role, .. } => {
                    name.text == expected && *role == lang_syntax::CanonicalNameRole::Hole
                }
                CanonicalSkeletonAst::Wildcard { .. }
                | CanonicalSkeletonAst::NavPath { .. }
                | CanonicalSkeletonAst::Literal { .. }
                | CanonicalSkeletonAst::Error(_) => false,
            }
        }

        matches!(
            pattern,
            BindingPatternAst::Skeleton(skeleton)
                if skeleton_marks_hole(skeleton, expected)
        )
    }

    fn norm_pattern_targets(pattern: &NormPattern, expected: lang_syntax::HoleBinderId) -> bool {
        match pattern {
            NormPattern::HoleRef { target, .. } => *target == expected,
            NormPattern::Sequence { elements, .. } => elements
                .iter()
                .any(|element| norm_pattern_targets(element, expected)),
            NormPattern::Pack { inner, .. } => norm_pattern_targets(inner, expected),
            NormPattern::Product { elements, .. } => elements.iter().any(|element| match element {
                NormPatternElem::Pattern(pattern) => norm_pattern_targets(pattern, expected),
                NormPatternElem::BindingSlot(slot) => {
                    norm_pattern_targets(&slot.value_pattern, expected)
                }
                NormPatternElem::Unit { .. } => false,
            }),
            NormPattern::BindingSlot { slot, .. } => {
                norm_pattern_targets(&slot.value_pattern, expected)
            }
            NormPattern::Binder { .. }
            | NormPattern::OperatorBinder { .. }
            | NormPattern::Unit { .. }
            | NormPattern::AnonymousHole { .. }
            | NormPattern::Name { .. }
            | NormPattern::Literal { .. }
            | NormPattern::Nav { .. }
            | NormPattern::Skeleton { .. }
            | NormPattern::Error(_)
            | NormPattern::Unsupported { .. } => false,
        }
    }

    let output = parsed("let f = <A>[let c A = source]() -> A r => { r };");
    let [FormAst::Let(let_ast)] = output.program.forms.as_slice() else {
        panic!("expected let declaration");
    };
    let initializer = let_ast.slot.initializer.as_ref().expect("initializer");
    let ExprKind::Pipe(pipe) = &initializer.kind else {
        panic!("expected closure expression shell");
    };
    let SegmentElementAst::OperatorExpr(operator) = &pipe.segments[0].elements[0] else {
        panic!("expected closure atom");
    };
    let OperatorExprKind::Atom(atom) = &operator.kind else {
        panic!("expected closure atom");
    };
    let lang_syntax::AtomKind::Closure(raw_closure) = &atom.kind else {
        panic!("expected closure");
    };
    let raw_head = raw_closure.head.as_ref().expect("raw head");
    let raw_captures = raw_head.captures.as_ref().expect("raw captures");
    let [lang_syntax::CaptureItemAst::Explicit {
        slot: raw_capture, ..
    }] = raw_captures.items.as_slice()
    else {
        panic!("expected explicit raw capture");
    };
    assert!(raw_pattern_marks_hole(&raw_capture.pattern, "A"));
    assert!(raw_pattern_marks_hole(
        &raw_head.returns.as_ref().expect("raw return").slot.pattern,
        "A"
    ));

    let normalized = normalize_program(&output.program);
    let outer_slot = binding_slot_at(&normalized.forms[0]);
    let NormExpr::Closure(closure) = outer_slot
        .initializer
        .as_deref()
        .expect("normalized initializer")
    else {
        panic!("expected normalized closure");
    };
    let head = closure.head.as_ref().expect("normalized head");
    let head_a = head.deduce[0].id;
    assert!(norm_pattern_targets(
        &head.captures[0].slot.value_pattern,
        head_a
    ));
    assert!(norm_pattern_targets(
        &head
            .returns
            .as_ref()
            .expect("normalized return")
            .value_pattern,
        head_a
    ));
}

#[test]
fn deduce_telescope_rejects_duplicates_and_does_not_resolve_forward_or_self_refs() {
    for source in ["let <A, A> x = value;", "let <A> (let <A> x) = value;"] {
        let output = parsed(source);
        let invalid = normalize_and_validate_patterns(&output.program)
            .expect_err("same-list and active-scope duplicate holes must fail");
        assert!(invalid.pattern_errors.iter().any(|error| matches!(
            error,
            lang_syntax::PatternValidationError::DuplicateHole { name, .. } if name == "A"
        )));
    }

    let output = parsed("let <A: B, B> x = value;");
    let normalized = normalize_program(&output.program);
    let [NormForm::Let(NormDecl::Let { slot, .. })] = normalized.forms.as_slice() else {
        panic!("expected let");
    };
    assert!(matches!(
        slot.deduce[0]
            .annotation
            .as_ref()
            .map(|annotation| &annotation.pattern),
        Some(NormPattern::Name { name, .. }) if name == "B"
    ));

    let output = parsed("let <A: A> x = value;");
    let normalized = normalize_program(&output.program);
    let [NormForm::Let(NormDecl::Let { slot, .. })] = normalized.forms.as_slice() else {
        panic!("expected let");
    };
    assert!(matches!(
        slot.deduce[0]
            .annotation
            .as_ref()
            .map(|annotation| &annotation.pattern),
        Some(NormPattern::Name { name, .. }) if name == "A"
    ));

    let output = parsed("let <A> (let <A: A> x) = value;");
    let normalized = normalize_program(&output.program);
    let [NormForm::Let(NormDecl::Let { slot: outer, .. })] = normalized.forms.as_slice() else {
        panic!("expected outer let");
    };
    let outer_a = outer.deduce[0].id;
    let NormPattern::Product { elements, .. } = &outer.value_pattern else {
        panic!("expected outer extraction Product");
    };
    let [NormPatternElem::BindingSlot(inner)] = elements.as_slice() else {
        panic!("expected inner binding slot");
    };
    assert_eq!(inner.deduce[0].duplicate_of, Some(outer_a));
    assert!(matches!(
        inner.deduce[0]
            .annotation
            .as_ref()
            .map(|annotation| &annotation.pattern),
        Some(NormPattern::HoleRef { target, name, .. })
            if *target == outer_a && name == "A"
    ));
}

fn annotation_hole_target(annotation: &NormAnnotation) -> lang_syntax::HoleBinderId {
    let NormPattern::HoleRef { target, .. } = &annotation.pattern else {
        panic!("expected exact alpha-normalized HoleRef, got {annotation:#?}");
    };
    *target
}

fn binding_slot_at(form: &NormForm) -> &NormBindingSlot {
    let NormForm::Let(NormDecl::Let { slot, .. }) = form else {
        panic!("expected normalized let form, got {form:#?}");
    };
    slot
}

fn find_generated_closure(expr: &NormExpr, rule: NormRule) -> Option<&lang_syntax::NormClosure> {
    match expr {
        NormExpr::Closure(closure)
            if matches!(
                &closure.origin,
                NormOrigin::Generated {
                    rule: closure_rule,
                    ..
                } if *closure_rule == rule
            ) =>
        {
            Some(closure)
        }
        NormExpr::Call { source, target, .. } => source
            .elements
            .iter()
            .filter_map(|element| match element {
                NormProductElem::Expr(expr) => Some(expr),
                NormProductElem::Unit { .. } => None,
            })
            .find_map(|expr| find_generated_closure(expr, rule))
            .or_else(|| find_generated_closure(target, rule)),
        NormExpr::Product(product) => product.elements.iter().find_map(|element| match element {
            NormProductElem::Expr(expr) => find_generated_closure(expr, rule),
            NormProductElem::Unit { .. } => None,
        }),
        NormExpr::Nav { components, .. } => {
            components.iter().find_map(|component| match component {
                NormNavComponent::Group { expr, .. } => find_generated_closure(expr, rule),
                NormNavComponent::Name { .. }
                | NormNavComponent::Operator { .. }
                | NormNavComponent::Error(_) => None,
            })
        }
        NormExpr::Closure(_)
        | NormExpr::Name { .. }
        | NormExpr::Literal { .. }
        | NormExpr::OperatorTarget { .. }
        | NormExpr::Error(_)
        | NormExpr::Unsupported { .. } => None,
    }
}

#[test]
fn callable_return_annotation_uses_the_binding_slot_suffix() {
    let output = parsed("let f = <A>() -> r: A => { r };");
    let normalized = normalize_program(&output.program);
    let outer_slot = binding_slot_at(&normalized.forms[0]);
    let NormExpr::Closure(closure) = outer_slot
        .initializer
        .as_deref()
        .expect("closure initializer")
    else {
        panic!("expected closure");
    };
    let head = closure.head.as_ref().expect("closure head");
    let return_slot = head.returns.as_ref().expect("return slot");
    assert!(matches!(
        &return_slot.value_pattern,
        NormPattern::Binder { name, .. } if name == "r"
    ));
    assert_eq!(
        annotation_hole_target(
            return_slot
                .annotation
                .as_ref()
                .expect("postfix return annotation")
        ),
        head.deduce[0].id
    );

    let prefix_shaped = parsed("let g = <A>() -> A r => { r };");
    let normalized = normalize_program(&prefix_shaped.program);
    let outer_slot = binding_slot_at(&normalized.forms[0]);
    let NormExpr::Closure(closure) = outer_slot
        .initializer
        .as_deref()
        .expect("closure initializer")
    else {
        panic!("expected closure");
    };
    assert!(
        closure
            .head
            .as_ref()
            .and_then(|head| head.returns.as_ref())
            .and_then(|returns| returns.annotation.as_ref())
            .is_none(),
        "`-> A r` remains an extraction Pattern and must not be reinterpreted as a type annotation"
    );
}

#[test]
fn generated_receiver_holes_are_hygienic_inside_source_t_scope() {
    let output = parsed(
        "let f = <T>() => {
            let dot = .name;
            let negative = -x;
            let member = obj..method(x);
            dot
        };",
    );
    let normalized = normalize_program(&output.program);
    validate_normalized_patterns(&normalized)
        .expect("generated receiver holes must not redeclare source T");

    let outer_slot = binding_slot_at(&normalized.forms[0]);
    let NormExpr::Closure(outer) = outer_slot
        .initializer
        .as_deref()
        .expect("outer closure initializer")
    else {
        panic!("expected outer closure");
    };
    let source_t = outer.head.as_ref().expect("outer closure head").deduce[0].id;
    let body = outer.body.user_body().expect("outer body");

    for (index, rule) in [
        NormRule::DotClosureLowering,
        NormRule::PrefixNegativeLowering,
        NormRule::DoubleDotLowering,
    ]
    .into_iter()
    .enumerate()
    {
        let initializer = binding_slot_at(&body.forms[index])
            .initializer
            .as_deref()
            .expect("generated helper initializer");
        let generated = find_generated_closure(initializer, rule)
            .unwrap_or_else(|| panic!("missing generated {rule:?} closure"));
        let head = generated.head.as_ref().expect("generated closure head");
        let generated_t = &head.deduce[0];
        assert_eq!(generated_t.name, "T");
        assert_eq!(generated_t.duplicate_of, None);
        assert_ne!(
            generated_t.id, source_t,
            "generated display spelling T must not capture source T"
        );
        let NormPatternElem::BindingSlot(receiver) = &head.params[0] else {
            panic!("expected generated receiver slot");
        };
        assert_eq!(
            annotation_hole_target(
                receiver
                    .annotation
                    .as_ref()
                    .expect("generated receiver annotation")
            ),
            generated_t.id,
            "generated reference must follow its hygienic key"
        );
    }
}

#[test]
fn binding_slot_policy_precedes_its_local_deduce_scope() {
    let output = parsed("Inner let <Inner> x = value;");
    let normalized = normalize_program(&output.program);
    let slot = binding_slot_at(&normalized.forms[0]);
    let NormValuePolicyPattern::Conjunction(policy) = &slot
        .policy
        .as_ref()
        .expect("leading slot policy")
        .value_policy
    else {
        panic!("expected value policy conjunction");
    };
    assert!(matches!(
        policy.choices[0].atoms.as_slice(),
        [NormPolicyAtom::Name { text, .. }] if text == "Inner"
    ));
    assert_eq!(slot.deduce.len(), 1);

    let output = parsed("let <Outer> (Outer let <Inner> x: Inner) = value;");
    let normalized = normalize_program(&output.program);
    let outer = binding_slot_at(&normalized.forms[0]);
    let outer_id = outer.deduce[0].id;
    let NormPattern::Product { elements, .. } = &outer.value_pattern else {
        panic!("expected outer product extraction");
    };
    let [NormPatternElem::BindingSlot(inner)] = elements.as_slice() else {
        panic!("expected nested binding slot");
    };
    let NormValuePolicyPattern::Conjunction(policy) = &inner
        .policy
        .as_ref()
        .expect("nested leading policy")
        .value_policy
    else {
        panic!("expected value policy conjunction");
    };
    assert!(matches!(
        policy.choices[0].atoms.as_slice(),
        [NormPolicyAtom::HoleRef { target, text, .. }]
            if *target == outer_id && text == "Outer"
    ));
    let inner_id = inner.deduce[0].id;
    assert_eq!(
        annotation_hole_target(inner.annotation.as_ref().expect("inner annotation")),
        inner_id
    );
}

#[test]
fn callable_deduce_scope_covers_capture_params_policy_return_body_and_nested_callable() {
    let output = parsed(
        "let f = <A>[let c: A = source](x: A):A -> r: A => {
            let y: A = x;
            let g = <B: A>(z: B) -> q: A => {
                let w: A = z;
                w
            };
            y
        };",
    );
    let normalized = normalize_program(&output.program);
    let outer_slot = binding_slot_at(&normalized.forms[0]);
    let NormExpr::Closure(outer) = outer_slot
        .initializer
        .as_deref()
        .expect("outer closure initializer")
    else {
        panic!("expected outer closure");
    };
    let outer_head = outer.head.as_ref().expect("outer closure head");
    let outer_a = outer_head.deduce[0].id;
    assert_eq!(outer_a.local_ordinal(), 0);

    assert_eq!(
        annotation_hole_target(
            outer_head.captures[0]
                .slot
                .annotation
                .as_ref()
                .expect("capture annotation")
        ),
        outer_a
    );
    let NormPatternElem::BindingSlot(param) = &outer_head.params[0] else {
        panic!("expected parameter binding slot");
    };
    assert_eq!(
        annotation_hole_target(param.annotation.as_ref().expect("parameter annotation")),
        outer_a
    );
    let Some(call_policy) = &outer_head.call_policy else {
        panic!("expected call policy");
    };
    let NormValuePolicyPattern::Conjunction(call_policy) = &call_policy.value_policy else {
        panic!("expected conjunction call policy");
    };
    assert!(matches!(
        call_policy.choices[0].atoms.as_slice(),
        [NormPolicyAtom::HoleRef { target, text, .. }]
            if *target == outer_a && text == "A"
    ));
    assert_eq!(
        annotation_hole_target(
            outer_head
                .returns
                .as_ref()
                .and_then(|returns| returns.annotation.as_ref())
                .expect("return annotation")
        ),
        outer_a
    );

    let outer_body = outer.body.user_body().expect("outer user body");
    assert_eq!(
        annotation_hole_target(
            binding_slot_at(&outer_body.forms[0])
                .annotation
                .as_ref()
                .expect("body-local annotation")
        ),
        outer_a
    );

    let nested_slot = binding_slot_at(&outer_body.forms[1]);
    let NormExpr::Closure(nested) = nested_slot
        .initializer
        .as_deref()
        .expect("nested closure initializer")
    else {
        panic!("expected nested closure");
    };
    let nested_head = nested.head.as_ref().expect("nested closure head");
    let nested_b = nested_head.deduce[0].id;
    assert_ne!(nested_b, outer_a);
    assert_eq!(
        annotation_hole_target(
            nested_head.deduce[0]
                .annotation
                .as_ref()
                .expect("nested telescope annotation")
        ),
        outer_a
    );
    let NormPatternElem::BindingSlot(nested_param) = &nested_head.params[0] else {
        panic!("expected nested parameter");
    };
    assert_eq!(
        annotation_hole_target(
            nested_param
                .annotation
                .as_ref()
                .expect("nested parameter annotation")
        ),
        nested_b
    );
    assert_eq!(
        annotation_hole_target(
            nested_head
                .returns
                .as_ref()
                .and_then(|returns| returns.annotation.as_ref())
                .expect("nested return annotation")
        ),
        outer_a
    );
    let nested_body = nested.body.user_body().expect("nested body");
    assert_eq!(
        annotation_hole_target(
            binding_slot_at(&nested_body.forms[0])
                .annotation
                .as_ref()
                .expect("nested body annotation")
        ),
        outer_a
    );
}

#[test]
fn callable_deduce_scope_rejects_nested_active_name_redeclaration() {
    let output = parsed(
        "let f = <A>() => {
            let g = <A>() => { value };
            g
        };",
    );
    let invalid = normalize_and_validate_patterns(&output.program)
        .expect_err("nested callable may not redeclare an active hole name");
    assert!(invalid.pattern_errors.iter().any(|error| matches!(
        error,
        lang_syntax::PatternValidationError::DuplicateHole { name, .. } if name == "A"
    )));
}

#[test]
fn hole_identity_is_alpha_owner_local_and_graphs_ignore_spelling_and_offset() {
    fn closure_graph(source: &str) -> (usize, bool) {
        let normalized = normalize_program(&parsed(source).program);
        let closure_slot = binding_slot_at(
            normalized
                .forms
                .last()
                .expect("fixture has closure declaration"),
        );
        let NormExpr::Closure(closure) = closure_slot
            .initializer
            .as_deref()
            .expect("closure initializer")
        else {
            panic!("expected closure");
        };
        let head = closure.head.as_ref().expect("closure head");
        let binder = head.deduce[0].id;
        let NormPatternElem::BindingSlot(param) = &head.params[0] else {
            panic!("expected parameter");
        };
        let target =
            annotation_hole_target(param.annotation.as_ref().expect("parameter annotation"));
        (head.deduce.len(), binder == target)
    }

    let a = closure_graph("let f = <A>(x: A) => { x };");
    let x = closure_graph("let padding = value; let f = <X>(x: X) => { x };");
    assert!(a.1);
    assert!(x.1);
    assert_eq!(
        a, x,
        "compare binder/reference graph shape, never bare IDs across distinct AlphaOwners"
    );
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
    assert!(normalize_and_validate_patterns(&valid.program).is_ok());

    let invalid = parse("let (...x, ...y) = value;");
    let failure = normalize_and_validate_patterns(&invalid.program)
        .expect_err("downstream handoff must reject an invalid normalized Pattern");
    assert!(!failure.pattern_errors.is_empty());
}

#[test]
fn pattern_validation_certificate_does_not_claim_recovery_free_syntax() {
    let recovered = parse("let f = () -> r => strategy;");
    assert!(!recovered.diagnostics.is_empty());

    let validated = normalize_and_validate_patterns(&recovered.program)
        .expect("the Pattern-layer certificate is independent of recovered expression errors");
    assert!(matches!(
        validated.as_program().forms.as_slice(),
        [NormForm::Let(NormDecl::Let { slot, .. })]
            if matches!(slot.initializer.as_deref(), Some(NormExpr::Error(_)))
    ));
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
