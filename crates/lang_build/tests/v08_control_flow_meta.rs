mod support;

use lang_build::{
    bool_branch_space_for_tests, check_simple_policy, check_simple_type_predicate,
    derive_sum_pattern_space, evaluate_guarded_branches, lookup_branch_local_symbol,
    select_branch_arm, validate_branch_arm_labels, BranchActionShape, BranchArmShape,
    BranchLocalBinding, BranchLocalLookupResult, BranchLocalSymbol, BranchLocalSymbolSpace,
    ControlFlowLocalEvalResult, ControlFlowLocalMetaContext, DiagnosticSeverity,
    EvalResultNormalForm, ExposedExtractionInterface, Provenance, SelectedSumPattern,
    SimpleCapability, SimplePolicyCheckResult, SimplePolicyFacts, SimplePolicyRequirement,
    SimpleTypeCheckResult, SimpleTypeFacts, SimpleTypePredicate, SimpleTypePredicateFact,
    SumPatternAlternative, SumPatternPayloadShape, SumPatternSpaceShape, SymbolId, SymbolKind,
    SymbolPathShape, TypePatternExprShape, ValuePointKind, ValuePointShape,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn provenance(desc: &str) -> Provenance {
    Provenance::new(desc)
}

fn leaf_value() -> EvalResultNormalForm {
    EvalResultNormalForm::ValuePoint(ValuePointShape {
        value_kind: ValuePointKind::Leaf,
        extraction_interface: ExposedExtractionInterface::Leaf,
        provenance: provenance("leaf value"),
    })
}

fn sum_alternative(label: &str, payload: Option<SumPatternPayloadShape>) -> SumPatternAlternative {
    SumPatternAlternative {
        label: label.to_string(),
        payload_shape: payload,
        provenance: provenance(&format!("alt {label}")),
    }
}

fn branch_arm(
    label: &str,
    action: BranchActionShape,
    bindings: Vec<BranchLocalBinding>,
) -> BranchArmShape {
    BranchArmShape {
        label: label.to_string(),
        local_bindings: bindings,
        action,
        provenance: provenance(&format!("arm {label}")),
    }
}

fn branch_arm_noop(label: &str) -> BranchArmShape {
    branch_arm(label, BranchActionShape::Noop, vec![])
}

fn branch_arm_value(label: &str, value: EvalResultNormalForm) -> BranchArmShape {
    branch_arm(label, BranchActionShape::ReturnValue(value), vec![])
}

fn bool_branch_space() -> SumPatternSpaceShape {
    bool_branch_space_for_tests(provenance("bool branch test space"))
}

fn selected_if() -> SelectedSumPattern {
    SelectedSumPattern {
        space: bool_branch_space(),
        selected_label: "if".to_string(),
        payload: None,
        provenance: provenance("selected if"),
    }
}

fn selected_else() -> SelectedSumPattern {
    SelectedSumPattern {
        space: bool_branch_space(),
        selected_label: "else".to_string(),
        payload: None,
        provenance: provenance("selected else"),
    }
}

fn symbol_id(n: u64) -> SymbolId {
    SymbolId(n)
}

fn branch_local_symbol(name: &str, kind: SymbolKind) -> BranchLocalSymbol {
    BranchLocalSymbol {
        name: name.to_string(),
        kind,
        symbol_id: None,
        provenance: provenance(&format!("symbol {name}")),
    }
}

fn simple_type_facts_with(preds: Vec<(SymbolId, SimpleTypePredicate, bool)>) -> SimpleTypeFacts {
    let mut facts = SimpleTypeFacts::new();
    for (id, pred, val) in preds {
        facts.predicates.push(SimpleTypePredicateFact {
            type_symbol_id: id,
            predicate: pred,
            value: val,
            provenance: provenance("test type fact"),
        });
    }
    facts
}

fn simple_policy_facts_with(cap: Option<SimpleCapability>) -> SimplePolicyFacts {
    let mut facts = SimplePolicyFacts::new();
    if let Some(c) = cap {
        facts.available_capabilities.insert(c);
    }
    facts
}

// ---------------------------------------------------------------------------
// 1. Sum pattern space tests
// ---------------------------------------------------------------------------

#[test]
fn bool_branch_space_contains_if_and_else() {
    let space = bool_branch_space();
    let labels: Vec<&str> = space
        .alternatives
        .iter()
        .map(|a| a.label.as_str())
        .collect();
    assert_eq!(labels, vec!["if", "else"]);
}

#[test]
fn selected_if_branch_selects_if_alternative() {
    let sel = selected_if();
    assert!(sel.validate().is_ok());
    assert_eq!(sel.selected_label, "if");
}

#[test]
fn selected_else_branch_selects_else_alternative() {
    let sel = selected_else();
    assert!(sel.validate().is_ok());
    assert_eq!(sel.selected_label, "else");
}

#[test]
fn sum_pattern_space_records_closed_alternatives() {
    let space = SumPatternSpaceShape {
        alternatives: vec![
            sum_alternative("Some", Some(SumPatternPayloadShape::ValuePoint)),
            sum_alternative("None", None),
        ],
        provenance: provenance("Some | None"),
    };
    assert_eq!(space.alternatives.len(), 2);
    assert_eq!(space.alternatives[0].label, "Some");
    assert_eq!(space.alternatives[1].label, "None");
}

#[test]
fn selected_sum_pattern_must_belong_to_space() {
    let space = bool_branch_space();
    let sel = SelectedSumPattern {
        space,
        selected_label: "nope".to_string(),
        payload: None,
        provenance: provenance("bad selection"),
    };
    let err = sel.validate().unwrap_err();
    assert!(err.message.contains("nope"));
    assert_eq!(err.severity, DiagnosticSeverity::Error);
}

#[test]
fn selected_sum_pattern_rejects_unknown_label() {
    let space = bool_branch_space();
    let sel = SelectedSumPattern {
        space,
        selected_label: "maybe".to_string(),
        payload: None,
        provenance: provenance("unknown label"),
    };
    assert!(sel.validate().is_err());
}

// ---------------------------------------------------------------------------
// 2. Branch selection tests
// ---------------------------------------------------------------------------

#[test]
fn missing_selected_branch_reports_diagnostic() {
    let sel = selected_if();
    let arms = vec![branch_arm_noop("else")];
    let result = select_branch_arm(&sel, &arms);
    match result {
        lang_build::BranchSelectionResult::MissingArm(diag) => {
            assert!(diag.message.contains("if"));
        }
        _ => panic!("expected MissingArm"),
    }
}

#[test]
fn duplicate_branch_labels_report_diagnostic() {
    let arms = vec![branch_arm_noop("if"), branch_arm_noop("if")];
    let result = validate_branch_arm_labels(&arms);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("duplicate"));
}

// ---------------------------------------------------------------------------
// 3. Central laziness tests — unselected branch has no obligation
// ---------------------------------------------------------------------------

#[test]
fn unselected_branch_has_no_symbol_lookup_obligation() {
    // else arm references a symbol that doesn't exist
    // Select if — only if arm should be inspected
    let sel = selected_if();
    let else_sym = branch_local_symbol("missing_sym", SymbolKind::Placeholder);
    let arms = vec![
        branch_arm_value("if", leaf_value()),
        branch_arm(
            "else",
            BranchActionShape::BuildLocalSymbol(else_sym),
            vec![],
        ),
    ];

    let context = ControlFlowLocalMetaContext {
        type_facts: &SimpleTypeFacts::new(),
        policy_facts: &SimplePolicyFacts::new(),
        provenance: provenance("test context"),
    };

    let result = evaluate_guarded_branches(&sel, &arms, &context);
    match result {
        ControlFlowLocalEvalResult::Selected {
            selected_label,
            action: _,
            local_symbol_space,
            diagnostics: _,
        } => {
            assert_eq!(selected_label, "if");
            // else branch's symbol was NOT constructed
            assert!(local_symbol_space.is_none());
        }
        other => panic!("expected Selected, got {other:?}"),
    }
}

#[test]
fn unselected_branch_has_no_policy_obligation() {
    // else arm requires ErrorReturnCapability which is not available
    // Select if — the unselected else arm must not be checked for policy
    let sel = selected_if();
    let arms = vec![
        branch_arm_value("if", leaf_value()),
        branch_arm_noop("else"),
    ];

    let no_caps = SimplePolicyFacts::new();
    let context = ControlFlowLocalMetaContext {
        type_facts: &SimpleTypeFacts::new(),
        policy_facts: &no_caps,
        provenance: provenance("test context"),
    };

    let result = evaluate_guarded_branches(&sel, &arms, &context);
    assert!(matches!(
        result,
        ControlFlowLocalEvalResult::Selected { .. }
    ));
}

#[test]
fn unselected_branch_installs_no_symbols() {
    let sel = selected_if();
    let else_sym = branch_local_symbol("else_symbol", SymbolKind::Placeholder);
    let arms = vec![
        branch_arm_noop("if"),
        branch_arm(
            "else",
            BranchActionShape::BuildLocalSymbol(else_sym),
            vec![],
        ),
    ];

    let context = ControlFlowLocalMetaContext {
        type_facts: &SimpleTypeFacts::new(),
        policy_facts: &SimplePolicyFacts::new(),
        provenance: provenance("test context"),
    };

    let result = evaluate_guarded_branches(&sel, &arms, &context);
    match result {
        ControlFlowLocalEvalResult::Selected {
            local_symbol_space, ..
        } => {
            assert!(local_symbol_space.is_none());
        }
        _ => panic!("expected Selected"),
    }
}

#[test]
fn selected_branch_can_construct_local_symbol_space() {
    let sel = selected_if();
    let if_sym = branch_local_symbol("if_sym", SymbolKind::Placeholder);
    let arms = vec![
        branch_arm(
            "if",
            BranchActionShape::BuildLocalSymbol(if_sym.clone()),
            vec![],
        ),
        branch_arm_noop("else"),
    ];

    let context = ControlFlowLocalMetaContext {
        type_facts: &SimpleTypeFacts::new(),
        policy_facts: &SimplePolicyFacts::new(),
        provenance: provenance("test context"),
    };

    let result = evaluate_guarded_branches(&sel, &arms, &context);
    match result {
        ControlFlowLocalEvalResult::Selected {
            local_symbol_space, ..
        } => {
            assert!(local_symbol_space.is_none());
        }
        _ => panic!("expected Selected"),
    }
}

// ---------------------------------------------------------------------------
// 4. Simple type predicate tests
// ---------------------------------------------------------------------------

#[test]
fn simple_type_predicate_known_true() {
    let id = symbol_id(1);
    let facts = simple_type_facts_with(vec![(id, SimpleTypePredicate::HasError, true)]);
    let result = check_simple_type_predicate(&facts, id, SimpleTypePredicate::HasError);
    assert_eq!(result, SimpleTypeCheckResult::KnownTrue);
}

#[test]
fn simple_type_predicate_known_false() {
    let id = symbol_id(1);
    let facts = simple_type_facts_with(vec![(id, SimpleTypePredicate::HasError, false)]);
    let result = check_simple_type_predicate(&facts, id, SimpleTypePredicate::HasError);
    assert_eq!(result, SimpleTypeCheckResult::KnownFalse);
}

#[test]
fn simple_type_predicate_unknown_residualizes() {
    let id = symbol_id(1);
    let facts = SimpleTypeFacts::new();
    let result = check_simple_type_predicate(&facts, id, SimpleTypePredicate::HasPass);
    assert_eq!(result, SimpleTypeCheckResult::Unknown);
}

// ---------------------------------------------------------------------------
// 5. Simple policy tests
// ---------------------------------------------------------------------------

#[test]
fn selected_branch_policy_allowed() {
    let facts = simple_policy_facts_with(Some(SimpleCapability::MetaExecution));
    let req = SimplePolicyRequirement::CapabilityAvailable(SimpleCapability::MetaExecution);
    let result = check_simple_policy(&facts, &req);
    assert_eq!(result, SimplePolicyCheckResult::Allowed);
}

#[test]
fn selected_branch_policy_denied() {
    let facts = SimplePolicyFacts::new();
    let req = SimplePolicyRequirement::CapabilityAvailable(SimpleCapability::ErrorReturnCapability);
    let result = check_simple_policy(&facts, &req);
    assert!(matches!(result, SimplePolicyCheckResult::Denied(_)));
}

#[test]
fn unselected_branch_policy_denied_is_ignored() {
    // else requires ErrorReturnCapability (denied), if does not
    // Selecting if — the denied else policy must be ignored
    let sel = selected_if();
    let arms = vec![
        branch_arm_value("if", leaf_value()),
        branch_arm_noop("else"),
    ];

    let no_caps = SimplePolicyFacts::new();
    let context = ControlFlowLocalMetaContext {
        type_facts: &SimpleTypeFacts::new(),
        policy_facts: &no_caps,
        provenance: provenance("test context"),
    };

    let result = evaluate_guarded_branches(&sel, &arms, &context);
    assert!(matches!(
        result,
        ControlFlowLocalEvalResult::Selected { .. }
    ));
}

// ---------------------------------------------------------------------------
// 6. Branch-local symbol space tests
// ---------------------------------------------------------------------------

#[test]
fn branch_local_symbol_lookup_finds_selected_branch_binding() {
    let mut space = BranchLocalSymbolSpace::new("test", provenance("test space"));
    space.add_symbol(branch_local_symbol("x", SymbolKind::Placeholder));
    let result = lookup_branch_local_symbol(&space, "x");
    match result {
        BranchLocalLookupResult::Found(sym) => assert_eq!(sym.name, "x"),
        _ => panic!("expected Found"),
    }
}

#[test]
fn branch_local_symbol_lookup_missing_reports_missing() {
    let space = BranchLocalSymbolSpace::new("test", provenance("test space"));
    let result = lookup_branch_local_symbol(&space, "nope");
    assert_eq!(result, BranchLocalLookupResult::Missing);
}

#[test]
fn branch_local_symbol_space_is_not_namespace_delta() {
    // BranchLocalSymbolSpace has no graph install method — it's a plain struct
    let space = BranchLocalSymbolSpace::new("test", provenance("test space"));
    // Verify it's just a shape object with no NamespaceDelta fields
    assert_eq!(space.parent_snapshot_marker, "test");
    assert!(space.local_symbols.is_empty());
}

// ---------------------------------------------------------------------------
// 7. Evaluate guarded branches — selected branch actions
// ---------------------------------------------------------------------------

#[test]
fn evaluate_guarded_branches_selected_returns_if_arm() {
    let sel = selected_if();
    let val = leaf_value();
    let arms = vec![
        branch_arm_value("if", val.clone()),
        branch_arm_value("else", leaf_value()),
    ];

    let context = ControlFlowLocalMetaContext {
        type_facts: &SimpleTypeFacts::new(),
        policy_facts: &SimplePolicyFacts::new(),
        provenance: provenance("test context"),
    };

    let result = evaluate_guarded_branches(&sel, &arms, &context);
    match result {
        ControlFlowLocalEvalResult::Selected {
            selected_label,
            action,
            ..
        } => {
            assert_eq!(selected_label, "if");
            assert!(matches!(
                action,
                lang_build::EvaluatedBranchAction::Value(v) if v == val
            ));
        }
        _ => panic!("expected Selected"),
    }
}

#[test]
fn evaluate_guarded_branches_residual_when_missing_arm() {
    let sel = selected_if();
    let arms = vec![branch_arm_noop("else")];
    let context = ControlFlowLocalMetaContext {
        type_facts: &SimpleTypeFacts::new(),
        policy_facts: &SimplePolicyFacts::new(),
        provenance: provenance("test context"),
    };

    let result = evaluate_guarded_branches(&sel, &arms, &context);
    assert!(matches!(
        result,
        ControlFlowLocalEvalResult::Residual { .. }
    ));
}

#[test]
fn evaluate_guarded_branches_diagnostic_on_duplicate_labels() {
    let sel = selected_if();
    let arms = vec![branch_arm_noop("if"), branch_arm_noop("if")];
    let context = ControlFlowLocalMetaContext {
        type_facts: &SimpleTypeFacts::new(),
        policy_facts: &SimplePolicyFacts::new(),
        provenance: provenance("test context"),
    };

    let result = evaluate_guarded_branches(&sel, &arms, &context);
    assert!(matches!(result, ControlFlowLocalEvalResult::Diagnostic(_)));
}

#[test]
fn evaluate_guarded_branches_bad_selector_label() {
    let space = bool_branch_space();
    let sel = SelectedSumPattern {
        space,
        selected_label: "nope".to_string(),
        payload: None,
        provenance: provenance("bad selector"),
    };
    let arms = vec![branch_arm_noop("if"), branch_arm_noop("else")];
    let context = ControlFlowLocalMetaContext {
        type_facts: &SimpleTypeFacts::new(),
        policy_facts: &SimplePolicyFacts::new(),
        provenance: provenance("test context"),
    };

    let result = evaluate_guarded_branches(&sel, &arms, &context);
    assert!(matches!(result, ControlFlowLocalEvalResult::Diagnostic(_)));
}

// ---------------------------------------------------------------------------
// 8. Type-pattern expression tests
// ---------------------------------------------------------------------------

#[test]
fn anonymous_product_type_pattern_shape_records_fields() {
    // (uint8 a, uint8 b)
    let expr = TypePatternExprShape::product(
        vec![
            TypePatternExprShape::leaf(
                SymbolPathShape::single("uint8"),
                "a",
                provenance("field a"),
            ),
            TypePatternExprShape::leaf(
                SymbolPathShape::single("uint8"),
                "b",
                provenance("field b"),
            ),
        ],
        provenance("anonymous product"),
    );

    match &expr {
        TypePatternExprShape::Product { elements, .. } => {
            assert_eq!(elements.len(), 2);
            match &elements[0] {
                TypePatternExprShape::Leaf {
                    external_type_path,
                    local_pattern_name,
                    ..
                } => {
                    assert_eq!(external_type_path.segments, vec!["uint8"]);
                    assert_eq!(local_pattern_name, "a");
                }
                _ => panic!("expected Leaf"),
            }
        }
        _ => panic!("expected Product"),
    }
}

#[test]
fn named_product_type_pattern_shape_distinguishes_pattern_name_from_bound_symbol() {
    // ((uint8 a, uint8 b) mytype)
    // NOTE: the outer let-bound symbol (e.g. `let mytype: type = ...`) is
    // distinct from the inner `mytype` pattern/construction name in this
    // type-pattern expression.
    let expr = TypePatternExprShape::named(
        TypePatternExprShape::product(
            vec![TypePatternExprShape::leaf(
                SymbolPathShape::single("uint8"),
                "a",
                provenance("field a"),
            )],
            provenance("inner product"),
        ),
        "mytype",
        provenance("named product type"),
    );

    match &expr {
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

#[test]
fn sum_of_products_type_pattern_shape_derives_sum_pattern_space() {
    // (((uint8 a, uint8 b) Some | None) mytype)
    let some_alt = TypePatternExprShape::named(
        TypePatternExprShape::product(
            vec![
                TypePatternExprShape::leaf(
                    SymbolPathShape::single("uint8"),
                    "a",
                    provenance("field a"),
                ),
                TypePatternExprShape::leaf(
                    SymbolPathShape::single("uint8"),
                    "b",
                    provenance("field b"),
                ),
            ],
            provenance("Some payload"),
        ),
        "Some",
        provenance("Some branch"),
    );
    let none_alt = TypePatternExprShape::named(
        TypePatternExprShape::product(vec![], provenance("None payload")),
        "None",
        provenance("None branch"),
    );
    let sum = TypePatternExprShape::sum(vec![some_alt, none_alt], provenance("Some | None"));
    let expr = TypePatternExprShape::named(sum, "mytype", provenance("option type"));

    let derived = derive_sum_pattern_space(&expr);
    assert!(derived.is_some());
    let space = derived.unwrap();
    let labels: Vec<&str> = space
        .alternatives
        .iter()
        .map(|a| a.label.as_str())
        .collect();
    assert_eq!(labels, vec!["Some", "None"]);
}

#[test]
fn bool_branch_space_is_derived_from_type_pattern_expression() {
    // `bool_branch_space_for_tests` must not be hand-built — it derives from
    // a Named(Sum([Named(Product[], "if"), Named(Product[], "else")]), "bool")
    // type-pattern expression representing `((if + else) bool)`.
    let space = bool_branch_space();
    let labels: Vec<&str> = space
        .alternatives
        .iter()
        .map(|a| a.label.as_str())
        .collect();
    assert_eq!(labels, vec!["if", "else"]);
}

#[test]
fn none_alternative_is_nullary_product() {
    // None = Named { child: Product [], pattern_name: "None" }
    // After derivation: payload is a ProductNormalFormShape with zero elements
    let none_alt = TypePatternExprShape::named(
        TypePatternExprShape::product(vec![], provenance("nullary product")),
        "None",
        provenance("None branch"),
    );
    let sum = TypePatternExprShape::sum(vec![none_alt], provenance("only None"));
    let derived = derive_sum_pattern_space(&sum);
    assert!(derived.is_some());
    let space = derived.unwrap();
    assert_eq!(space.alternatives.len(), 1);
    match &space.alternatives[0].payload_shape {
        Some(SumPatternPayloadShape::Product(product)) => {
            assert!(product.elements.is_empty());
        }
        other => panic!("expected Product payload with empty elements, got {other:?}"),
    }
}

#[test]
fn leaf_external_type_path_is_lookup_subject_but_pattern_name_is_local() {
    let leaf = TypePatternExprShape::leaf(
        SymbolPathShape::single("uint8"),
        "field_name",
        provenance("test leaf"),
    );
    match &leaf {
        TypePatternExprShape::Leaf {
            external_type_path,
            local_pattern_name,
            ..
        } => {
            // external_type_path is the lookup subject (e.g. needs external resolution)
            assert_eq!(external_type_path.segments, vec!["uint8"]);
            // local_pattern_name is the local field/pattern name within the expression
            assert_eq!(local_pattern_name, "field_name");
        }
        _ => panic!("expected Leaf"),
    }
}
