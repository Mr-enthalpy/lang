use lang_build::{
    pack_operand_is_admissible, solve_parameter_product_relation, CallableOwnerPlacement,
    LocalCallableIdentity, OverloadArgShape, PackOperandClass, PackageId, PatternLayerOrder,
    PatternRelationContext, SemanticOwnerGraph, SpecificityTuple,
};
use lang_syntax::{
    normalize_program, validate_pack_pattern_element_level, NormBindingSlot, NormDecl, NormExpr,
    NormForm, NormOrigin, NormPattern, NormPatternElem, Span,
};

fn origin() -> NormOrigin {
    NormOrigin::Source(Span::new(0, 0, 1, 1))
}

fn slot(pattern: NormPattern) -> NormPatternElem {
    NormPatternElem::BindingSlot(NormBindingSlot {
        policy: None,
        has_let: false,
        deduce: Vec::new(),
        value_pattern: pattern,
        annotation: None,
        with_clause: None,
        initializer: None,
        origin: origin(),
    })
}

fn pack(name: &str) -> NormPatternElem {
    slot(NormPattern::Pack {
        inner: Box::new(NormPattern::Binder {
            name: name.to_string(),
            origin: origin(),
        }),
        origin: origin(),
    })
}

fn arg(index: usize) -> OverloadArgShape {
    OverloadArgShape {
        top_pattern_name: Some(format!("arg{index}")),
        type_symbol_id: None,
        value_type: None,
        pattern_value: None,
        type_core_observation: None,
        complete_type_observation: None,
        effective_view: None,
        semantic_value: None,
        is_value: false,
        provenance: lang_build::Provenance::new(format!("arg {index}")),
    }
}

fn source_parameter(source: &str) -> NormPatternElem {
    source_closure(source)
        .head
        .as_ref()
        .unwrap()
        .formal_frame()
        .explicit_parameters[0]
        .clone()
}

fn source_closure(source: &str) -> lang_syntax::NormClosure {
    let parsed = lang_syntax::parse(source);
    assert!(
        parsed.diagnostics.is_empty(),
        "{}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    let normalized = normalize_program(&parsed.program);
    let [NormForm::Let(NormDecl::Let { slot, .. })] = normalized.forms.as_slice() else {
        panic!("expected one let declaration");
    };
    let Some(NormExpr::Closure(closure)) = slot.initializer.as_deref() else {
        panic!("expected closure initializer");
    };
    closure.clone()
}

fn relation_context<'a>(closure: &lang_syntax::NormClosure) -> PatternRelationContext<'a> {
    let mut owners = SemanticOwnerGraph::new();
    let package = owners.package_root(PackageId(1), "pack-test");
    let callable = owners.callable(
        package,
        LocalCallableIdentity(1),
        CallableOwnerPlacement::Ordinary,
    );
    PatternRelationContext::for_source_callable(closure, callable, None)
        .expect("normalized source callable has a Pattern root")
}

#[test]
fn normalized_pack_validation_is_per_structural_level() {
    let duplicate = vec![pack("left"), pack("right")];
    let error = validate_pack_pattern_element_level(&duplicate)
        .expect_err("two packs at one normalized level must fail");
    assert_eq!(error.pack_count, 2);

    let nested = NormPattern::Product {
        elements: vec![
            slot(NormPattern::Product {
                elements: vec![pack("inner")],
                origin: origin(),
            }),
            pack("outer"),
        ],
        origin: origin(),
    };
    lang_syntax::validate_pack_pattern_layers(&nested)
        .expect("different normalized levels may each contain one pack");

    let same_level_nesting = NormPattern::Pack {
        inner: Box::new(NormPattern::Pack {
            inner: Box::new(NormPattern::Binder {
                name: "args".to_string(),
                origin: origin(),
            }),
            origin: origin(),
        }),
        origin: origin(),
    };
    assert_eq!(
        lang_syntax::validate_pack_pattern_layers(&same_level_nesting)
            .expect_err("adjacent packs do not create a new structural level")
            .pack_count,
        2
    );
}

#[test]
fn pack_binding_captures_the_remainder_without_counting_its_length() {
    let closure = source_closure("let f = (self, ...args) -> r => { r };");
    let params = closure
        .head
        .as_ref()
        .unwrap()
        .formal_frame()
        .explicit_parameters;
    let context = relation_context(&closure);
    let two = solve_parameter_product_relation(params, &[arg(0), arg(1)], &context).unwrap();
    let two_hundred_args = (0..200).map(arg).collect::<Vec<_>>();
    let two_hundred =
        solve_parameter_product_relation(params, &two_hundred_args, &context).unwrap();
    assert_eq!(two.specificity, two_hundred.specificity);
    assert_eq!(two.specificity.explicit_pack_match_count, 1);
    assert_eq!(two.named_pack_bindings()["args"].len(), 2);
    assert_eq!(two_hundred.named_pack_bindings()["args"].len(), 200);
}

#[test]
fn bare_product_pack_is_not_a_structured_match() {
    let source = source_parameter("let f = (self, ...(a, b)) -> r => { r };");
    let _ = source;

    let parsed = lang_syntax::parse("let f = (self, ...(a, b)) -> r => { r };");
    let invalid = lang_syntax::normalize_and_validate_patterns(&parsed.program)
        .expect_err("bare Product Pack operand must fail normalized Pattern validation");
    assert!(invalid.pattern_errors.iter().any(|error| matches!(
        error,
        lang_syntax::PatternValidationError::NonCanonicalPackOperand { .. }
    )));
}

#[test]
fn pack_operand_admissibility_depends_on_layer_order_and_stable_top_mode() {
    assert!(pack_operand_is_admissible(
        PatternLayerOrder::Ordered,
        PackOperandClass::WholeRemainderBinder,
    ));
    assert!(pack_operand_is_admissible(
        PatternLayerOrder::Unordered,
        PackOperandClass::WholeRemainderBinder,
    ));
    assert!(pack_operand_is_admissible(
        PatternLayerOrder::Ordered,
        PackOperandClass::Structured {
            stable_top_mode: true,
        },
    ));
    assert!(!pack_operand_is_admissible(
        PatternLayerOrder::Ordered,
        PackOperandClass::Structured {
            stable_top_mode: false,
        },
    ));
    assert!(!pack_operand_is_admissible(
        PatternLayerOrder::Unordered,
        PackOperandClass::Structured {
            stable_top_mode: true,
        },
    ));
}

#[test]
fn node_class_evidence_orders_explicit_above_pack_above_discards() {
    let base = SpecificityTuple {
        max_depth: 1,
        sum_depth: 1,
        ..SpecificityTuple::default()
    };
    let explicit = SpecificityTuple {
        non_discard_explicit_node_count: 1,
        ..base
    };
    let explicit_pack = SpecificityTuple {
        explicit_pack_match_count: 1,
        ..base
    };
    let discard = SpecificityTuple {
        explicit_discard_count: 1,
        ..base
    };
    let pack_discard = SpecificityTuple {
        pack_discard_count: 1,
        ..base
    };

    assert!(explicit > explicit_pack);
    assert!(explicit_pack > discard);
    assert!(discard > pack_discard);
}
