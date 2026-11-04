use lang_syntax::{normalize_program, parse, NormExpr, NormForm, NormOrigin, NormProductElem};

fn expr(source: &str) -> NormExpr {
    let parsed = parse(source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let program = normalize_program(&parsed.program);
    let [NormForm::Expr(expr)] = program.forms.as_slice() else {
        panic!("one expression expected")
    };
    expr.clone()
}

#[test]
fn empty_delimiters_have_zero_entries_but_comma_holes_keep_their_spans() {
    for (source, entries, units) in [
        ("()", 0, 0),
        ("(,)", 2, 2),
        ("(,,)", 3, 3),
        ("(a,)", 2, 1),
        ("(,a)", 2, 1),
        ("(a,,b)", 3, 1),
    ] {
        let NormExpr::Product(product) = expr(source) else {
            panic!("Product expected: {source}")
        };
        assert_eq!(product.elements.len(), entries, "{source}");
        let mut actual_units = 0;
        for element in &product.elements {
            if let NormProductElem::Unit { origin } = element {
                actual_units += 1;
                let NormOrigin::Source(span) = origin else {
                    panic!("comma-hole provenance must remain source-backed")
                };
                assert_eq!(&source[span.byte_start..span.byte_end], ",");
            }
        }
        assert_eq!(actual_units, units, "{source}");
    }
}

#[test]
fn empty_inputs_and_continuations_do_not_synthesize_self_or_unit() {
    for (source, entries) in [
        ("() |> f", 0),
        ("()f", 0),
        ("() |> f ()", 0),
        ("x |> f ()", 1),
        ("() |> f (x)", 1),
        ("obj..method()", 1),
        ("obj[]", 1),
    ] {
        let NormExpr::Call { source: input, .. } = expr(source) else {
            panic!("Call expected: {source}")
        };
        assert_eq!(input.elements.len(), entries, "{source}");
        assert!(input
            .elements
            .iter()
            .all(|e| !matches!(e, NormProductElem::Unit { .. })));
    }
}

#[test]
fn nested_empty_product_is_not_replaced_by_a_unit_entry() {
    let NormExpr::Product(product) = expr("((),)") else {
        panic!("Product expected")
    };
    let [NormProductElem::Expr(NormExpr::Product(empty)), NormProductElem::Unit { .. }] =
        product.elements.as_slice()
    else {
        panic!("nested empty Product and actual trailing comma hole expected")
    };
    assert!(empty.elements.is_empty());
}
