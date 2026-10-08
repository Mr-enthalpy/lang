use lang_syntax::{NormDecl, NormExpr, NormForm, NormPattern};

fn normalize(source: &str) -> lang_syntax::NormProgram {
    let parsed = lang_syntax::parse(source);
    assert!(
        parsed.diagnostics.is_empty(),
        "{}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    lang_syntax::normalize_program(&parsed.program)
}

fn value_leaf(expr: &NormExpr, depth: usize) -> bool {
    match expr {
        NormExpr::InterpretationFlip { operand, .. } if depth > 0 => {
            structural_leaf(operand, depth - 1)
        }
        NormExpr::Name { text, .. } if depth == 0 => {
            assert_eq!(text, "value");
            false
        }
        other => panic!("expected value polarity at depth {depth}: {other:?}"),
    }
}

fn structural_leaf(pattern: &NormPattern, depth: usize) -> bool {
    match pattern {
        NormPattern::InterpretationFlip { operand, .. } if depth > 0 => {
            value_leaf(operand, depth - 1)
        }
        NormPattern::Name { name, .. } if depth == 0 => {
            assert_eq!(name, "value");
            true
        }
        other => panic!("expected structural polarity at depth {depth}: {other:?}"),
    }
}

#[test]
fn arbitrary_postfix_nesting_preserves_even_and_flips_odd_polarity() {
    for depth in [0, 1, 2, 3, 4, 17, 32] {
        let suffix = "$".repeat(depth);
        let program = normalize(&format!("value{suffix};"));
        let [NormForm::Expr(expr)] = program.forms.as_slice() else {
            panic!("one expression");
        };
        assert_eq!(value_leaf(expr, depth), depth % 2 == 1);

        let program = normalize(&format!("let destination:value{suffix} = input;"));
        let [NormForm::Let(NormDecl::Let { slot, .. })] = program.forms.as_slice() else {
            panic!("one declaration");
        };
        assert_eq!(
            structural_leaf(&slot.annotation.as_ref().unwrap().pattern, depth),
            depth % 2 == 0
        );
    }
}

#[test]
fn polarity_changes_do_not_insert_read_call_stage_or_authority_material() {
    let program = normalize("value$$$;");
    let [NormForm::Expr(expr)] = program.forms.as_slice() else {
        panic!("one expression");
    };
    // The only nodes between the source occurrence and its ordinary name are
    // the three interpretation boundaries verified by the recursive checker.
    assert!(value_leaf(expr, 3));
    let dump = lang_syntax::dump_norm_program(&program);
    assert!(!dump.contains("Call "));
    assert!(!dump.contains("PolicyLet"));
    assert!(!dump.contains("Closure"));
}

#[test]
fn nested_structure_can_return_to_value_computation_under_the_same_horizon() {
    let program = normalize("((bool a)(args Mytypefun)$)$;");
    let dump = lang_syntax::dump_norm_program(&program);
    assert!(dump.contains("InterpretationFlip V->S"));
    assert!(dump.contains("InterpretationFlip "));
    assert!(dump.contains("Name \"Mytypefun\""));
    assert!(!dump.contains("PolicyLet"));
    assert!(!dump.contains("Unsupported"), "{dump}");
}
