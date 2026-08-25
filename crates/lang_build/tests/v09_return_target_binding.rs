use lang_build::{
    elaborate_return_targets_in_program, elaborate_return_targets_in_returnable_closure,
    elaborate_return_targets_in_returnable_closure_with_resolver, BoundReturnEvent,
    ExplicitReturnTargetResolution, PreservedReturnReason, ResolvedReturnTarget, ResolverCode,
    ReturnFrameOwner, ReturnTargetBindingReport,
};
use lang_syntax::{
    NormClosure, NormDecl, NormExpr, NormForm, NormLiteralKind, NormOrigin, NormPattern,
    NormPatternElem, NormProgram, NormReturnEvent, NormReturnTargetSyntax, NormRule, Span,
};

mod support;

fn normalize_source(source: &str) -> NormProgram {
    let parsed = lang_syntax::parse(source);
    assert!(
        parsed.diagnostics.is_empty(),
        "unexpected parse diagnostics:\n{}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    lang_syntax::normalize_program(&parsed.program)
}

fn closure_initializer(source: &str) -> NormClosure {
    let normalized = normalize_source(source);
    match normalized.forms.as_slice() {
        [NormForm::Let(NormDecl::Let { slot, .. })] => match slot.initializer.as_deref() {
            Some(NormExpr::Closure(closure)) => closure.clone(),
            other => panic!("expected closure initializer, got {other:#?}"),
        },
        other => panic!("expected single let closure declaration, got {other:#?}"),
    }
}

fn bind_closure(source: &str) -> ReturnTargetBindingReport {
    let closure = closure_initializer(source);
    elaborate_return_targets_in_returnable_closure(
        &closure,
        ReturnFrameOwner::SourceCallable {
            symbol_id: None,
            name: Some("f".to_string()),
        },
    )
}

fn bind_closure_with_own_self_identity(source: &str) -> ReturnTargetBindingReport {
    let closure = closure_initializer(source);
    let callable_owner = closure
        .semantic_owner
        .expect("normalized closure has callable owner")
        .id;
    elaborate_return_targets_in_returnable_closure_with_resolver(
        &closure,
        ReturnFrameOwner::SourceCallable {
            symbol_id: None,
            name: Some("f".to_string()),
        },
        &move |_target: &NormExpr| ExplicitReturnTargetResolution::CallableSelf(callable_owner),
    )
}

fn active_frame_id(event: &BoundReturnEvent) -> usize {
    match event.resolved_target {
        ResolvedReturnTarget::ActiveFrame(frame_id) => frame_id.0,
        ResolvedReturnTarget::DiagnosticTarget => panic!("expected active frame"),
    }
}

fn is_int_literal(expr: &NormExpr, expected: &str) -> bool {
    matches!(
        expr,
        NormExpr::Literal {
            kind: NormLiteralKind::Int,
            text,
            ..
        } if text == expected
    )
}

#[test]
fn implicit_return_binds_to_nearest_active_return_frame() {
    let report = bind_closure(
        r#"
let f = (self, x: int): runtime -> r: int => {
    x return;
};
"#,
    );

    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    assert_eq!(report.frames.len(), 1);
    assert_eq!(report.frames[0].return_slot.name.as_deref(), Some("r"));
    assert!(
        report.frames[0].callable_self_owner.is_some(),
        "return Self is anchored to the callable-local lexical owner"
    );
    assert_eq!(
        report.frames[0]
            .self_identity
            .as_ref()
            .unwrap()
            .display_name
            .as_deref(),
        Some("self")
    );
    assert_eq!(report.bound_events.len(), 1);
    assert_eq!(
        active_frame_id(&report.bound_events[0]),
        report.frames[0].frame_id.0
    );
    assert_eq!(
        report.bound_events[0].unresolved_target,
        lang_build::UnresolvedReturnTargetForm::ImplicitNearest
    );
}

#[test]
fn extraction_return_frame_preserves_the_complete_result_pattern() {
    let report = bind_closure(
        r#"
let f = (): runtime -> (r first, d second) => {
    value return;
};
"#,
    );

    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    let return_slot = &report.frames[0].return_slot;
    assert_eq!(
        return_slot.name, None,
        "a product return has no single slot name"
    );
    let slot = return_slot
        .binding_slot
        .as_ref()
        .expect("return target keeps the complete normalized binding slot");
    let NormPattern::Product { elements, .. } = &slot.value_pattern else {
        panic!("expected preserved product return Pattern");
    };
    assert_eq!(elements.len(), 2);
    assert!(matches!(
        &elements[0],
        NormPatternElem::BindingSlot(first)
            if matches!(&first.value_pattern, NormPattern::Sequence { .. })
    ));
    assert!(matches!(
        &elements[1],
        NormPatternElem::BindingSlot(second)
            if matches!(&second.value_pattern, NormPattern::Sequence { .. })
    ));
}

#[test]
fn return_outside_returnable_body_is_diagnostic() {
    let normalized = normalize_source("1 return;");
    let report = elaborate_return_targets_in_program(&normalized);

    assert!(report.bound_events.is_empty());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        Some(ResolverCode::ReturnOutsideReturnableContext)
    );
    assert!(report.diagnostics[0].provenance.is_some());
}

#[test]
fn nested_unmaterialized_closure_return_does_not_bind_to_outer_frame() {
    let report = bind_closure(
        r#"
let f = (self): runtime -> r: int => {
    let g = () => {
        1 return;
    };
    2 return;
};
"#,
    );

    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    assert_eq!(report.frames.len(), 1);
    assert_eq!(report.bound_events.len(), 1);
    assert!(is_int_literal(&report.bound_events[0].value, "2"));
    assert_eq!(
        active_frame_id(&report.bound_events[0]),
        report.frames[0].frame_id.0
    );
    assert_eq!(report.preserved_unbound_events.len(), 1);
    assert_eq!(
        report.preserved_unbound_events[0].reason,
        PreservedReturnReason::UnmaterializedClosureLiteral
    );
    assert!(is_int_literal(
        &report.preserved_unbound_events[0].event.value,
        "1"
    ));
}

#[test]
fn closure_literal_inside_return_value_is_preserved_not_bound_to_outer_frame() {
    let report = bind_closure(
        r#"
let f = (self): runtime -> r: _ => {
    () => {
        1 return;
    } return;
};
"#,
    );

    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    assert_eq!(report.frames.len(), 1);
    assert_eq!(report.bound_events.len(), 1);
    assert!(matches!(report.bound_events[0].value, NormExpr::Closure(_)));
    assert_eq!(
        active_frame_id(&report.bound_events[0]),
        report.frames[0].frame_id.0
    );
    assert_eq!(report.preserved_unbound_events.len(), 1);
    assert_eq!(
        report.preserved_unbound_events[0].reason,
        PreservedReturnReason::UnmaterializedClosureLiteral
    );
    assert!(is_int_literal(
        &report.preserved_unbound_events[0].event.value,
        "1"
    ));
}

#[test]
fn explicit_self_return_matches_active_self_frame() {
    let report = bind_closure_with_own_self_identity(
        r#"
let f = (self): runtime -> r: int => {
    1 |> (self return);
};
"#,
    );

    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    assert_eq!(report.bound_events.len(), 1);
    assert_eq!(
        active_frame_id(&report.bound_events[0]),
        report.frames[0].frame_id.0
    );
    assert!(matches!(
        &report.bound_events[0].unresolved_target,
        lang_build::UnresolvedReturnTargetForm::Explicit(NormExpr::Name { text, .. })
            if text == "self"
    ));
}

#[test]
fn first_written_formal_is_self_even_when_its_name_is_not_self() {
    let report = bind_closure_with_own_self_identity(
        r#"
let f = (callable, x): runtime -> r: int => {
    x |> (callable return);
};
"#,
    );

    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    assert_eq!(
        report.frames[0]
            .self_identity
            .as_ref()
            .unwrap()
            .display_name
            .as_deref(),
        Some("callable")
    );
    assert_eq!(report.bound_events.len(), 1);
    assert_eq!(
        active_frame_id(&report.bound_events[0]),
        report.frames[0].frame_id.0
    );
}

#[test]
fn explicit_self_return_does_not_fall_back_to_nearest_without_matching_self() {
    let closure = closure_initializer(
        r#"
let f = (): runtime -> r: int => {
    1 |> (self return);
};
"#,
    );
    let report = elaborate_return_targets_in_returnable_closure_with_resolver(
        &closure,
        ReturnFrameOwner::AnonymousClosure,
        &|_target: &NormExpr| ExplicitReturnTargetResolution::NotActive,
    );

    assert!(report.bound_events.is_empty());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        Some(ResolverCode::ReturnTargetNotActive)
    );
    assert!(report.diagnostics[0].provenance.is_some());
}

#[test]
fn unsupported_explicit_return_target_form_is_diagnostic() {
    let origin = NormOrigin::Generated {
        rule: NormRule::Unsupported,
        span: Span::at(0, 1, 1),
    };
    let normalized = NormProgram {
        forms: vec![NormForm::ReturnEvent(NormReturnEvent {
            value: NormExpr::Name {
                text: "x".to_string(),
                origin: origin.clone(),
            },
            target: NormReturnTargetSyntax::Explicit(NormExpr::Literal {
                kind: NormLiteralKind::Int,
                text: "1".to_string(),
                origin: origin.clone(),
            }),
            origin,
        })],
        origin: NormOrigin::Generated {
            rule: NormRule::Unsupported,
            span: Span::at(0, 1, 1),
        },
    };

    let closure = NormClosure {
        semantic_owner: None,
        placement: lang_syntax::NormClosurePlacement::Ordinary,
        head: None,
        body: lang_syntax::NormClosureBody::Block(normalized),
        origin: NormOrigin::Generated {
            rule: NormRule::Unsupported,
            span: Span::at(0, 1, 1),
        },
    };
    let report = elaborate_return_targets_in_returnable_closure_with_resolver(
        &closure,
        ReturnFrameOwner::AnonymousClosure,
        &|_target: &NormExpr| ExplicitReturnTargetResolution::Unsupported,
    );

    assert!(report.bound_events.is_empty());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].code,
        Some(ResolverCode::UnsupportedReturnTargetForm)
    );
    assert!(report.diagnostics[0].provenance.is_some());
}

#[test]
fn unresolved_explicit_target_is_preserved_instead_of_matched_by_spelling() {
    let report = bind_closure(
        r#"
let f = (self): runtime -> r: int => {
    1 |> (self return);
};
"#,
    );

    assert!(report.diagnostics.is_empty());
    assert!(report.bound_events.is_empty());
    assert_eq!(report.preserved_unbound_events.len(), 1);
    assert_eq!(
        report.preserved_unbound_events[0].reason,
        PreservedReturnReason::SemanticTargetResolutionRequired
    );
}

#[test]
fn resolved_callable_owner_not_target_spelling_selects_the_frame() {
    let report = bind_closure_with_own_self_identity(
        r#"
let f = (written_self): runtime -> r: int => {
    1 |> (completely_different_text return);
};
"#,
    );

    assert!(report.diagnostics.is_empty());
    assert_eq!(report.bound_events.len(), 1);
    assert_eq!(
        active_frame_id(&report.bound_events[0]),
        report.frames[0].frame_id.0
    );
}

#[test]
fn top_level_return_reports_structured_diagnostic_through_build_pipeline() {
    let error = support::build_fixture_error("v09_return_outside", "app");
    assert_eq!(error.diagnostics.len(), 1);
    assert_eq!(
        error.diagnostics[0].code,
        Some(ResolverCode::ReturnOutsideReturnableContext)
    );
    assert!(error.diagnostics[0].provenance.is_some());
}
