use lang_syntax::{
    normalize_program, parse, NormDecl, NormExpr, NormForm, NormPattern, NormPatternElem,
    NormReturnTargetSyntax,
};

fn closure(source: &str) -> lang_syntax::NormClosure {
    let parsed = parse(source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let program = normalize_program(&parsed.program);
    let [NormForm::Let(NormDecl::Let { slot, .. })] = program.forms.as_slice() else {
        panic!("declaration")
    };
    let Some(NormExpr::Closure(closure)) = slot.initializer.as_deref() else {
        panic!("closure")
    };
    closure.clone()
}

#[test]
fn serial_expressions_preserve_shape_without_inferred_completion() {
    for source in [
        "let f = () => { (); value; };",
        "let f = () => { value; (); };",
        "let f = () => { (); };",
        "let f = () => { value; };",
    ] {
        let closure = closure(source);
        assert!(closure
            .body
            .user_body()
            .unwrap()
            .forms
            .iter()
            .all(|f| matches!(f, NormForm::Expr(_))));
    }
    let closure = closure("let f = () => { () return; };");
    assert!(
        matches!(closure.body.user_body().unwrap().forms.as_slice(), [NormForm::ReturnEvent(event)] if event.target == NormReturnTargetSyntax::Omitted)
    );
}

#[test]
fn atomic_name_head_keeps_self_separate_from_binderless_argument() {
    let parsed = parse("value |> Widget { self; };");
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let program = normalize_program(&parsed.program);
    let [NormForm::Expr(NormExpr::Call { target, .. })] = program.forms.as_slice() else {
        panic!("pipe call")
    };
    let NormExpr::Closure(closure) = target.as_ref() else {
        panic!("headed callable")
    };
    let frame = closure.head.as_ref().unwrap().formal_frame();
    assert!(
        matches!(frame.written_self, Some(NormPatternElem::BindingSlot(slot)) if matches!(&slot.value_pattern, NormPattern::GeneratedSelf { owner: Some(owner), .. } if *owner == closure.semantic_owner.unwrap().id))
    );
    assert!(
        matches!(frame.explicit_parameters, [NormPatternElem::BindingSlot(slot)] if !matches!(slot.value_pattern, NormPattern::Binder { .. }))
    );
}

#[test]
fn generated_self_is_fresh_and_does_not_bind_source_self() {
    let parsed = parse("value |> Widget { self; }; value |> Widget { self; };");
    assert!(parsed.diagnostics.is_empty());
    let program = normalize_program(&parsed.program);
    let owners = program.forms.iter().map(|form| {
        let NormForm::Expr(NormExpr::Call { target, .. }) = form else { panic!("call") };
        let NormExpr::Closure(closure) = target.as_ref() else { panic!("closure") };
        let Some(NormPatternElem::BindingSlot(slot)) = closure.head.as_ref().unwrap().formal_frame().written_self else { panic!("self slot") };
        let NormPattern::GeneratedSelf { owner: Some(owner), .. } = slot.value_pattern else { panic!("fresh owner identity") };
        assert_eq!(owner, closure.semantic_owner.unwrap().id);
        assert!(matches!(closure.body.user_body().unwrap().forms.as_slice(), [NormForm::Expr(NormExpr::Name { text, .. })] if text == "self"));
        owner
    }).collect::<Vec<_>>();
    assert_ne!(owners[0], owners[1]);

    // Capture inference must still see the free body name through the shorthand.
    let closure = closure("let f = [() |> Widget { self; }]() => { value; };");
    assert!(
        matches!(&closure.head.as_ref().unwrap().captures[0].slot.value_pattern,
        NormPattern::Binder { name, .. } if name == "self")
    );
}

#[test]
fn public_policy_pair_and_choice_are_syntax_errors_in_strong_contexts() {
    for policy in [
        "runtime:compile",
        "runtime || compile",
        "const || mut",
        "(runtime || compile)",
    ] {
        for source in [
            format!("{policy} let x = value;"),
            format!("let f = (self): {policy} => {{ value; }};"),
        ] {
            assert!(!parse(&source).diagnostics.is_empty(), "accepted {source}");
        }
    }
    for source in [
        "const + compile let x = value;",
        "let x: T = value;",
        "let f = (self): compile => { value; };",
        "left || right;",
    ] {
        assert!(parse(source).diagnostics.is_empty(), "rejected {source}");
    }
}

#[test]
fn incomplete_policy_constraints_report_diagnostics() {
    for source in [
        "let f = (self): const + => { x; };",
        "let f = (self): () => { x; };",
    ] {
        assert!(!parse(source).diagnostics.is_empty(), "accepted {source}");
    }
}
#[test]
fn explicit_callable_self_position_is_not_spelling_restricted() {
    for spelling in ["receiver", "callable", "x"] {
        let parsed = parse(&format!(
            "value |> ({spelling}, <> Widget) {{ {spelling}; }};"
        ));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let program = normalize_program(&parsed.program);
        let [NormForm::Expr(NormExpr::Call { target, .. })] = program.forms.as_slice() else {
            panic!("call")
        };
        let NormExpr::Closure(closure) = target.as_ref() else {
            panic!("closure")
        };
        let frame = closure.head.as_ref().unwrap().formal_frame();
        assert!(
            matches!(frame.written_self, Some(NormPatternElem::BindingSlot(slot))
        if matches!(&slot.value_pattern, NormPattern::Binder { name, .. } if name == spelling))
        );
        assert_eq!(frame.explicit_parameters.len(), 1);
    }
}
