mod support;

use lang_build::{
    decode_struct_associated_val2_let, decode_struct_type_pattern_expr, derive_sum_pattern_space,
    DecodedStructPattern, DiagnosticSeverity, NamespaceVisibility, PatternComponentPolicy,
    PolicyPair, PolicyStage, Provenance, StageSet, StructAssociatedVal2Contribution,
    StructLeafTypeExprShape, StructuralMemberVisibility, SymbolPathShape, TypePatternExprShape,
    ValueComponentPolicy, ValuePresence,
};
use lang_syntax::{
    norm::NormNavComponent, NormExpr, NormOperatorFixity, NormOrigin, NormProduct, NormProductElem,
    Span,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn provenance(desc: &str) -> Provenance {
    Provenance::new(desc)
}

fn fake_origin() -> NormOrigin {
    NormOrigin::Source(Span::new(0, 0, 1, 1))
}

/// Build a simple call: (single_type_expr field_name)
fn call_type_field(type_name: &str, field_name: &str) -> NormExpr {
    NormExpr::Call {
        source: NormProduct {
            elements: vec![NormProductElem::Expr(NormExpr::Name {
                text: type_name.to_string(),
                origin: fake_origin(),
            })],
            origin: fake_origin(),
        },
        target: Box::new(NormExpr::Name {
            text: field_name.to_string(),
            origin: fake_origin(),
        }),
        origin: fake_origin(),
    }
}

fn annotated_field(type_name: &str, field_name: &str, visibility: &str) -> NormExpr {
    NormExpr::Call {
        source: NormProduct {
            elements: vec![NormProductElem::Expr(NormExpr::Name {
                text: type_name.to_string(),
                origin: fake_origin(),
            })],
            origin: fake_origin(),
        },
        target: Box::new(NormExpr::Call {
            source: NormProduct {
                elements: vec![NormProductElem::Expr(NormExpr::Name {
                    text: field_name.to_string(),
                    origin: fake_origin(),
                })],
                origin: fake_origin(),
            },
            target: Box::new(NormExpr::OperatorTarget {
                spelling: format!("[[{visibility}]]"),
                fixity: NormOperatorFixity::Postfix,
                arity: 1,
                origin: fake_origin(),
            }),
            origin: fake_origin(),
        }),
        origin: fake_origin(),
    }
}

/// Build a product expression from multiple field calls.
fn product_of_fields(fields: Vec<(&str, &str)>) -> NormExpr {
    NormExpr::Product(NormProduct {
        elements: fields
            .into_iter()
            .map(|(type_name, field_name)| {
                NormProductElem::Expr(call_type_field(type_name, field_name))
            })
            .collect(),
        origin: fake_origin(),
    })
}

/// Build a Named call: `(source_expr target_name)`
fn named_call(source: NormExpr, target_name: &str) -> NormExpr {
    let source_product = match source {
        NormExpr::Product(p) => p,
        other => NormProduct {
            elements: vec![NormProductElem::Expr(other)],
            origin: fake_origin(),
        },
    };
    NormExpr::Call {
        source: source_product,
        target: Box::new(NormExpr::Name {
            text: target_name.to_string(),
            origin: fake_origin(),
        }),
        origin: fake_origin(),
    }
}

/// Build a sum expression using | operator: `(alt1 | alt2)`.
fn bar_sum(alternatives: Vec<NormExpr>) -> NormExpr {
    NormExpr::Call {
        source: NormProduct {
            elements: alternatives
                .into_iter()
                .map(NormProductElem::Expr)
                .collect(),
            origin: fake_origin(),
        },
        target: Box::new(NormExpr::OperatorTarget {
            spelling: "|".to_string(),
            fixity: NormOperatorFixity::Binary,
            arity: 2,
            origin: fake_origin(),
        }),
        origin: fake_origin(),
    }
}

/// Build a + sum expression (non-canonical — should be rejected by decoder).
fn plus_sum(alternatives: Vec<NormExpr>) -> NormExpr {
    NormExpr::Call {
        source: NormProduct {
            elements: alternatives
                .into_iter()
                .map(NormProductElem::Expr)
                .collect(),
            origin: fake_origin(),
        },
        target: Box::new(NormExpr::OperatorTarget {
            spelling: "+".to_string(),
            fixity: NormOperatorFixity::Binary,
            arity: 2,
            origin: fake_origin(),
        }),
        origin: fake_origin(),
    }
}

/// Build a Nav expression for a type path (e.g. `Vec::std`).
fn nav_expr(segments: Vec<&str>) -> NormExpr {
    NormExpr::Nav {
        components: segments
            .iter()
            .map(|s| NormNavComponent::Name {
                name: s.to_string(),
                origin: fake_origin(),
            })
            .collect(),
        explicit_terminated: false,
        origin: fake_origin(),
    }
}

/// Build an empty product expression (nullary — no elements).
fn empty_product() -> NormExpr {
    NormExpr::Product(NormProduct {
        elements: vec![],
        origin: fake_origin(),
    })
}

// ---------------------------------------------------------------------------
// 1. Anonymous product tests
// ---------------------------------------------------------------------------

#[test]
fn decode_anonymous_product_struct_expr() {
    let expr = product_of_fields(vec![("uint8", "a"), ("uint8", "b")]);
    let result = decode_struct_type_pattern_expr(&expr, provenance("test"));
    assert!(result.is_ok());
    let shape = result.unwrap();
    match shape {
        TypePatternExprShape::Product { elements, .. } => {
            assert_eq!(elements.len(), 2);
        }
        _ => panic!("expected Product"),
    }
}

#[test]
fn decode_duplicate_field_name_is_diagnostic() {
    let expr = product_of_fields(vec![("uint8", "a"), ("uint16", "a")]);
    let result = decode_struct_type_pattern_expr(&expr, provenance("test"));
    assert!(result.is_err());
    let diag = result.unwrap_err();
    assert!(diag.message.contains("duplicate"));
    assert_eq!(diag.severity, DiagnosticSeverity::Error);
}

// ---------------------------------------------------------------------------
// 2. Named product tests
// ---------------------------------------------------------------------------

#[test]
fn decode_named_product_struct_expr() {
    let inner = product_of_fields(vec![("uint8", "a"), ("uint8", "b")]);
    let expr = named_call(inner, "mytype");
    let result = decode_struct_type_pattern_expr(&expr, provenance("test"));
    assert!(result.is_ok());
    let shape = result.unwrap();
    match shape {
        TypePatternExprShape::Named {
            pattern_name,
            child,
            ..
        } => {
            assert_eq!(pattern_name, "mytype");
            assert!(matches!(
                child.as_ref(),
                TypePatternExprShape::Product { .. }
            ));
        }
        _ => panic!("expected Named"),
    }
}

// ---------------------------------------------------------------------------
// 3. Leaf tests
// ---------------------------------------------------------------------------

#[test]
fn decode_leaf_distinguishes_type_path_from_field_name() {
    let expr = call_type_field("uint8", "a");
    let result = decode_struct_type_pattern_expr(&expr, provenance("test"));
    assert!(result.is_ok());
    let shape = result.unwrap();
    match shape {
        TypePatternExprShape::Leaf {
            external_type_expr,
            local_pattern_name,
            ..
        } => {
            match external_type_expr {
                StructLeafTypeExprShape::Path(p) => {
                    assert_eq!(p.segments, vec!["uint8"]);
                }
                _ => panic!("expected Path"),
            }
            assert_eq!(local_pattern_name, "a");
        }
        _ => panic!("expected Leaf"),
    }
}

#[test]
fn decode_leaf_nav_path_as_type_expr() {
    let nav = nav_expr(vec!["Vec", "std"]);
    let expr = NormExpr::Call {
        source: NormProduct {
            elements: vec![NormProductElem::Expr(nav)],
            origin: fake_origin(),
        },
        target: Box::new(NormExpr::Name {
            text: "a".to_string(),
            origin: fake_origin(),
        }),
        origin: fake_origin(),
    };
    let result = decode_struct_type_pattern_expr(&expr, provenance("test"));
    assert!(result.is_ok());
    let shape = result.unwrap();
    match shape {
        TypePatternExprShape::Leaf {
            external_type_expr,
            local_pattern_name,
            ..
        } => {
            match external_type_expr {
                StructLeafTypeExprShape::Path(p) => {
                    assert_eq!(p.segments, vec!["Vec", "std"]);
                }
                _ => panic!("expected Path"),
            }
            assert_eq!(local_pattern_name, "a");
        }
        _ => panic!("expected Leaf"),
    }
}

#[test]
fn decode_leaf_type_expr_as_normalized_ast() {
    // When a name appears as the rightmost target in an application chain,
    // it is the local pattern/field name. The prefix is the type expression.
    // For a chain like `int Vec a`, the decoder correctly identifies `a` as
    // the local name; `int Vec` is preserved in the type-expr field.
    let int_vec_call = NormExpr::Call {
        source: NormProduct {
            elements: vec![NormProductElem::Expr(NormExpr::Name {
                text: "int".to_string(),
                origin: fake_origin(),
            })],
            origin: fake_origin(),
        },
        target: Box::new(NormExpr::Name {
            text: "Vec".to_string(),
            origin: fake_origin(),
        }),
        origin: fake_origin(),
    };
    let expr = NormExpr::Call {
        source: NormProduct {
            elements: vec![NormProductElem::Expr(int_vec_call)],
            origin: fake_origin(),
        },
        target: Box::new(NormExpr::Name {
            text: "a".to_string(),
            origin: fake_origin(),
        }),
        origin: fake_origin(),
    };
    let result = decode_struct_type_pattern_expr(&expr, provenance("test"));
    assert!(result.is_ok());
    let shape = result.unwrap();
    // Per the mechanical leaf rule, the rightmost Name in an application
    // chain is the local pattern name; the prefix is the type expression:
    //   E Name → Leaf(external_type_expr = E, local_pattern_name = Name)
    // Here `int Vec a` → Leaf(type_expr = int Vec, local_pattern_name = a).
    // The inner call `int Vec` is preserved as the type expression.
    match shape {
        TypePatternExprShape::Leaf {
            external_type_expr,
            local_pattern_name,
            ..
        } => {
            assert_eq!(local_pattern_name, "a");
            match external_type_expr {
                StructLeafTypeExprShape::NormalizedAst { .. } => {}
                _ => panic!("expected NormalizedAst for type expression"),
            }
        }
        _ => panic!("expected Leaf"),
    }
}

// ---------------------------------------------------------------------------
// 4. Sum pattern tests
// ---------------------------------------------------------------------------

#[test]
fn canonical_bar_sum_decodes_as_sum() {
    // (((uint8 a, uint8 b) Some | None) mytype) — Named wrapping Sum wrapping Named alternatives
    let expr = bar_sum(vec![
        named_call(
            product_of_fields(vec![("uint8", "a"), ("uint8", "b")]),
            "Some",
        ),
        named_call(empty_product(), "None"),
    ]);
    let result = decode_struct_type_pattern_expr(&expr, provenance("test"));
    assert!(result.is_ok());
    let shape = result.unwrap();
    match shape {
        TypePatternExprShape::Sum { alternatives, .. } => {
            assert_eq!(alternatives.len(), 2);
        }
        _ => panic!("expected Sum"),
    }
}

#[test]
fn plus_requires_pattern_combination_reduction() {
    let expr = plus_sum(vec![
        named_call(
            product_of_fields(vec![("uint8", "a"), ("uint8", "b")]),
            "Some",
        ),
        named_call(empty_product(), "None"),
    ]);
    let result = decode_struct_type_pattern_expr(&expr, provenance("test"));
    assert!(result.is_err());
    let diag = result.unwrap_err();
    assert!(diag.message.contains("pattern-combination"));
    assert!(diag.message.contains("+"));
}

#[test]
fn decode_none_is_nullary_product() {
    let none_named = named_call(empty_product(), "None");
    let expr = bar_sum(vec![none_named]);
    let result = decode_struct_type_pattern_expr(&expr, provenance("test"));
    assert!(result.is_ok());
    let shape = result.unwrap();
    match shape {
        TypePatternExprShape::Sum { alternatives, .. } => {
            assert_eq!(alternatives.len(), 1);
            match &alternatives[0] {
                TypePatternExprShape::Named {
                    pattern_name,
                    child,
                    ..
                } => {
                    assert_eq!(pattern_name, "None");
                    if let TypePatternExprShape::Product { elements, .. } = child.as_ref() {
                        assert!(elements.is_empty());
                    } else {
                        panic!("expected Product child");
                    }
                }
                _ => panic!("expected Named"),
            }
        }
        _ => panic!("expected Sum"),
    }
}

#[test]
fn decode_duplicate_alternative_name_is_diagnostic() {
    let some_alt = named_call(product_of_fields(vec![("uint8", "a")]), "Some");
    let expr = bar_sum(vec![some_alt.clone(), some_alt]);
    let result = decode_struct_type_pattern_expr(&expr, provenance("test"));
    assert!(result.is_err());
    let diag = result.unwrap_err();
    assert!(diag.message.contains("duplicate"));
}

#[test]
fn leaf_alternative_derives_sum_pattern_space() {
    // A bare Leaf (e.g. `uint8 a`) is a valid sum alternative.
    // `uint8 a | None` → Sum[Leaf(uint8,a), Named(Product[], "None")]
    let leaf_a = TypePatternExprShape::leaf(
        StructLeafTypeExprShape::Path(SymbolPathShape::single("uint8")),
        "a",
        provenance("leaf a"),
    );
    let none_alt = TypePatternExprShape::named(
        TypePatternExprShape::product(vec![], provenance("nullary")),
        "None",
        provenance("none alt"),
    );
    let sum = TypePatternExprShape::sum(vec![leaf_a, none_alt], provenance("leaf | None"));
    let derived = derive_sum_pattern_space(&sum);
    assert!(derived.is_some());
    let space = derived.unwrap();
    let labels: Vec<&str> = space
        .alternatives
        .iter()
        .map(|a| a.label.as_str())
        .collect();
    assert_eq!(labels, vec!["a", "None"]);
}

// ---------------------------------------------------------------------------
// 5. Unsupported shape test
// ---------------------------------------------------------------------------

#[test]
fn decode_unsupported_shape_is_diagnostic() {
    let expr = NormExpr::Literal {
        kind: lang_syntax::NormLiteralKind::Int,
        text: "42".to_string(),
        origin: fake_origin(),
    };
    let result = decode_struct_type_pattern_expr(&expr, provenance("test"));
    assert!(result.is_err());
    let diag = result.unwrap_err();
    assert!(diag.message.contains("unsupported"));
}

#[test]
fn bare_name_at_top_level_is_a_pure_no_value_pattern() {
    let expr = NormExpr::Name {
        text: "None".to_string(),
        origin: fake_origin(),
    };
    let shape =
        decode_struct_type_pattern_expr(&expr, provenance("test")).expect("pure Pattern decodes");
    assert!(matches!(
        &shape,
        TypePatternExprShape::Named {
            pattern_name,
            child,
            ..
        } if pattern_name == "None"
            && matches!(
                child.as_ref(),
                TypePatternExprShape::Product { elements, .. } if elements.is_empty()
            )
    ));
    assert!(shape.is_pure_pattern_without_value());
}

// ---------------------------------------------------------------------------
// 6. DecodedStructPattern wrapper test
// ---------------------------------------------------------------------------

#[test]
fn decoded_struct_pattern_wraps_type_pattern_expr() {
    let expr = product_of_fields(vec![("uint8", "a")]);
    let shape = decode_struct_type_pattern_expr(&expr, provenance("test")).unwrap();
    let decoded = DecodedStructPattern::new(shape.clone(), provenance("wrapped"));
    assert_eq!(decoded.type_pattern_expr, shape);
}

// ---------------------------------------------------------------------------
// 7. Bound symbol vs inner pattern name
// ---------------------------------------------------------------------------

#[test]
fn named_pattern_name_is_inner_construction_name_not_bound_symbol() {
    // An explicit Group `(child parent)` lifts the Group to Product and wraps
    // as Named. `(uint8 a) mytype` → Named(Product[Leaf(uint8,a)], "mytype").
    // In normalized AST, `(uint8 a)` is a Product (Group), and `(uint8 a) mytype`
    // is Call(source: [Product([Call(uint8,a)])], target: Name("mytype")).
    let inner_product = NormExpr::Product(NormProduct {
        elements: vec![NormProductElem::Expr(call_type_field("uint8", "a"))],
        origin: fake_origin(),
    });
    let expr = NormExpr::Call {
        source: NormProduct {
            elements: vec![NormProductElem::Expr(inner_product)],
            origin: fake_origin(),
        },
        target: Box::new(NormExpr::Name {
            text: "mytype".to_string(),
            origin: fake_origin(),
        }),
        origin: fake_origin(),
    };
    let result = decode_struct_type_pattern_expr(&expr, provenance("test"));
    assert!(result.is_ok());
    let shape = result.unwrap();
    match shape {
        TypePatternExprShape::Named { pattern_name, .. } => {
            assert_eq!(pattern_name, "mytype");
        }
        _ => panic!("expected Named"),
    }
}

// ---------------------------------------------------------------------------
// 8. Integration: derive sum space from decoded expression
// ---------------------------------------------------------------------------

#[test]
fn derive_sum_space_from_decoded_sum_of_products() {
    let expr = bar_sum(vec![
        named_call(empty_product(), "if"),
        named_call(empty_product(), "else"),
    ]);
    let shape = decode_struct_type_pattern_expr(&expr, provenance("test")).unwrap();
    let derived = derive_sum_pattern_space(&shape);
    assert!(derived.is_some());
    let space = derived.unwrap();
    let labels: Vec<&str> = space
        .alternatives
        .iter()
        .map(|a| a.label.as_str())
        .collect();
    assert_eq!(labels, vec!["if", "else"]);
}

#[test]
fn structural_member_visibility_is_part_of_the_decoded_struct_model() {
    let default =
        decode_struct_type_pattern_expr(&call_type_field("uint8", "ordinary"), provenance("field"))
            .unwrap();
    assert!(matches!(
        default,
        TypePatternExprShape::Leaf {
            visibility: StructuralMemberVisibility::Default,
            ..
        }
    ));

    let private = decode_struct_type_pattern_expr(
        &annotated_field("uint8", "secret", "private"),
        provenance("private field"),
    )
    .unwrap();
    assert!(matches!(
        private,
        TypePatternExprShape::Leaf {
            local_pattern_name,
            visibility: StructuralMemberVisibility::Private,
            ..
        } if local_pattern_name == "secret"
    ));

    let public = decode_struct_type_pattern_expr(
        &annotated_field("uint8", "visible", "public"),
        provenance("public field"),
    )
    .unwrap();
    assert!(matches!(
        public,
        TypePatternExprShape::Leaf {
            visibility: StructuralMemberVisibility::Public,
            ..
        }
    ));
}

fn resolved_value_pair() -> PolicyPair {
    PolicyPair {
        value: ValueComponentPolicy {
            stages: StageSet::from([PolicyStage::Runtime]),
            mutability: Default::default(),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: StageSet::from([PolicyStage::Compile]),
        },
    }
}

#[test]
fn struct_associated_let_contributes_named_val2_without_requiring_absent_pv() {
    let parsed = lang_syntax::parse("let helper = callable_value;");
    assert!(parsed.diagnostics.is_empty());
    let normalized = lang_syntax::normalize_program(&parsed.program);
    let lang_syntax::NormForm::Let(decl) = &normalized.forms[0] else {
        panic!("expected normalized let");
    };
    let value_pair = resolved_value_pair();
    let associated = decode_struct_associated_val2_let(
        decl,
        &value_pair,
        provenance("associated Val2 contribution"),
    )
    .unwrap();
    assert!(matches!(
        associated,
        StructAssociatedVal2Contribution::NamedValue {
            name,
            visibility: None,
            value_policy,
            ..
        } if name == "helper" && value_policy == value_pair
    ));
}

#[test]
fn struct_associated_val2_let_preserves_visibility_and_callable_value() {
    let parsed = lang_syntax::parse("private let fun = (self_fun, object) => { object };");
    assert!(parsed.diagnostics.is_empty());
    let normalized = lang_syntax::normalize_program(&parsed.program);
    let lang_syntax::NormForm::Let(decl) = &normalized.forms[0] else {
        panic!("expected normalized let");
    };
    let associated = decode_struct_associated_val2_let(
        decl,
        &resolved_value_pair(),
        provenance("member-like associated function"),
    )
    .unwrap();
    let StructAssociatedVal2Contribution::NamedValue {
        visibility, value, ..
    } = &associated
    else {
        panic!("expected a named Val2 value");
    };
    assert_eq!(*visibility, Some(NamespaceVisibility::Private));
    let NormExpr::Closure(closure) = value else {
        panic!("associated callable value must remain a closure");
    };
    let frame = closure
        .head
        .as_ref()
        .expect("headed associated callable")
        .formal_frame();
    assert!(matches!(
        frame.written_self,
        Some(lang_syntax::NormPatternElem::BindingSlot(slot))
            if matches!(&slot.value_pattern, lang_syntax::NormPattern::Binder { name, .. }
                if name == "self_fun")
    ));
    assert!(matches!(
        frame.explicit_parameters,
        [lang_syntax::NormPatternElem::BindingSlot(slot)]
            if matches!(&slot.value_pattern, lang_syntax::NormPattern::Binder { name, .. }
                if name == "object")
    ));
}

#[test]
fn struct_associated_unit_let_is_a_call_entry_not_a_named_closure_value() {
    let parsed = lang_syntax::parse("let () = (object) => { object };");
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let normalized = lang_syntax::normalize_program(&parsed.program);
    let lang_syntax::NormForm::Let(decl) = &normalized.forms[0] else {
        panic!("expected normalized let");
    };
    let associated = decode_struct_associated_val2_let(
        decl,
        &resolved_value_pair(),
        provenance("local call entry"),
    )
    .unwrap();
    let StructAssociatedVal2Contribution::CallEntry { implementation, .. } = &associated else {
        panic!("expected a call-entry contribution");
    };
    let NormExpr::Closure(closure) = implementation else {
        panic!("call entry implementation must remain a closure");
    };
    let frame = closure
        .head
        .as_ref()
        .expect("headed call entry")
        .formal_frame();
    assert!(matches!(
        frame.written_self,
        Some(lang_syntax::NormPatternElem::BindingSlot(slot))
            if matches!(&slot.value_pattern, lang_syntax::NormPattern::Binder { name, .. }
                if name == "object")
    ));
    assert!(frame.explicit_parameters.is_empty());
}

#[test]
fn struct_associated_val2_let_still_rejects_extraction_lhs() {
    let parsed = lang_syntax::parse("let (a, b) = value;");
    assert!(parsed.diagnostics.is_empty());
    let normalized = lang_syntax::normalize_program(&parsed.program);
    let lang_syntax::NormForm::Let(decl) = &normalized.forms[0] else {
        panic!("expected normalized let");
    };
    assert!(decode_struct_associated_val2_let(
        decl,
        &resolved_value_pair(),
        provenance("invalid associated extraction"),
    )
    .is_err());
}
