mod support;

use support::*;

use lang_build::{NonValueArgKind, ProductAtom, ProductSyntaxMaterial, RawArgValueClass};
use lang_syntax::NormExpr;

#[test]
fn exposed_product_nodes_flatten_left_to_right_from_source_fixture() {
    let shape = fixture_arg_product_shape("product_exposed_left.lang");
    assert_eq!(atom_labels(&shape.flattened.atoms), ["a", "b", "c"]);
    assert!(shape.flattened.invariant.no_direct_product_atom_remains);

    let shape = fixture_arg_product_shape("product_exposed_right.lang");
    assert_eq!(atom_labels(&shape.flattened.atoms), ["a", "b", "c"]);
}

#[test]
fn expression_barrier_blocks_product_flattening_from_source_fixture() {
    let site = fixture_call_site("product_expression_barrier.lang");
    let product = ProductSyntaxMaterial::from_norm_product(site.source_product.clone());
    let flattened = product.flatten();
    assert_eq!(atom_labels(&flattened.atoms), ["Call", "c"]);

    let ProductAtom::Expression {
        expr: NormExpr::Call { source, .. },
        ..
    } = &flattened.atoms[0]
    else {
        panic!("first atom should remain an opaque call Expression barrier");
    };
    assert_eq!(
        source.elements.len(),
        2,
        "Expression barrier must preserve the inner call source product"
    );
}

#[test]
fn call_input_exposes_available_standard_packaging_and_keeps_expression_barriers() {
    let shape_left = fixture_arg_product_shape("product_exposed_left.lang");
    assert_eq!(
        atom_labels(&shape_left.flattened.atoms),
        ["a", "b", "c"],
        "((a, b), c) |> f: inner product flattens into the call source"
    );

    let shape_barrier = fixture_arg_product_shape("product_expression_barrier.lang");
    assert_eq!(
        atom_labels(&shape_barrier.flattened.atoms),
        ["Call", "c"],
        "((a, b) |> f, c) |> g: inner call is an Expression barrier, outer sees Call + c"
    );
}

#[test]
fn unit_positions_and_raw_arg_non_value_boundary_are_preserved_from_source_fixture() {
    let shape = fixture_arg_product_shape("product_unit_preservation.lang");
    assert_eq!(atom_labels(&shape.flattened.atoms), ["a", "Unit", "b"]);
    assert!(shape.flattened.atoms[1].provenance().span.is_some());

    assert_eq!(shape.arity, 3);
    assert!(matches!(
        shape.raw_args[1].value_class,
        RawArgValueClass::NonValue(NonValueArgKind::ProductUnit)
    ));
    assert_eq!(shape.raw_args[1].is_value(), Some(false));
}

#[test]
fn normalized_call_site_exposes_product_material_before_arg_shape_from_source_fixture() {
    let site = fixture_call_site("product_unit_preservation.lang");
    let product_material = site.source_product_material();
    assert_eq!(product_material.original, site.source_product);
    assert!(product_material
        .provenance
        .description
        .contains("ProductSyntaxMaterial"));
    let shape = product_material.to_arg_product_shape();
    assert_eq!(shape.arity, 3);
    assert!(matches!(
        shape.raw_args[1].value_class,
        RawArgValueClass::NonValue(NonValueArgKind::ProductUnit)
    ));
}

fn source_arg_shape(source: &str) -> lang_build::ArgProductShape {
    let parsed = lang_syntax::parse(source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let program = lang_syntax::normalize_program(&parsed.program);
    let [lang_syntax::NormForm::Expr(expr)] = program.forms.as_slice() else {
        panic!("one expression expected")
    };
    lang_build::extract_single_call_site(expr)
        .unwrap()
        .to_arg_product_shape()
}

#[test]
fn empty_products_expose_zero_arguments_without_filtering_comma_hole_units() {
    for (source, expected) in [
        ("() |> f", vec![]),
        ("()f", vec![]),
        ("() |> f ()", vec![]),
        ("(()) |> f", vec![]),
        ("x |> f ()", vec!["x"]),
        ("() |> f (x)", vec!["x"]),
        ("obj..method()", vec!["obj"]),
        ("obj[]", vec!["obj"]),
        ("obj[()]", vec!["obj"]),
        ("((),) |> f", vec!["Unit"]),
        ("(,) |> f", vec!["Unit", "Unit"]),
        ("obj..method(,)", vec!["obj", "Unit", "Unit"]),
        ("obj[,]", vec!["obj", "Unit", "Unit"]),
    ] {
        let shape = source_arg_shape(source);
        assert_eq!(atom_labels(&shape.flattened.atoms), expected, "{source}");
        assert_eq!(shape.arity, expected.len(), "{source}");
        for (index, label) in expected.iter().enumerate() {
            if *label == "Unit" {
                assert!(matches!(
                    shape.raw_args[index].value_class,
                    RawArgValueClass::NonValue(NonValueArgKind::ProductUnit)
                ));
            }
        }
    }
}

#[test]
fn empty_input_does_not_erase_an_expression_barrier_or_other_product_entries() {
    let shape = source_arg_shape("(() |> f) |> g");
    assert_eq!(atom_labels(&shape.flattened.atoms), ["Call"]);
    assert_eq!(shape.arity, 1);
    let ProductAtom::Expression {
        expr: NormExpr::Call { source, .. },
        ..
    } = &shape.flattened.atoms[0]
    else {
        panic!("the inner call must remain an opaque expression")
    };
    assert!(source.elements.is_empty());

    let shape = source_arg_shape("(((),a),((),b)) |> f");
    assert_eq!(atom_labels(&shape.flattened.atoms), ["a", "b"]);
    assert_eq!(shape.arity, 2);
}

#[test]
fn empty_and_single_unit_call_inputs_have_distinct_canonical_identity() {
    let base = lang_build::CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let mut world = base.semantic_world().clone();
    let mut address = |source| {
        let shape = source_arg_shape(source);
        world
            .canonical_arguments_product_address(&shape.raw_args, &shape.flattened.atoms)
            .unwrap()
    };
    let empty = address("() |> f");
    assert_eq!(empty, address("(()) |> f"));
    let one_unit = address("((),) |> f");
    assert_ne!(empty, one_unit);
    assert_ne!(one_unit, address("(,) |> f"));
}

fn atom_labels(atoms: &[ProductAtom]) -> Vec<String> {
    atoms
        .iter()
        .map(|atom| match atom {
            ProductAtom::Expression {
                expr: NormExpr::Name { text, .. },
                ..
            } => text.clone(),
            ProductAtom::Expression {
                expr: NormExpr::Call { .. },
                ..
            } => "Call".to_string(),
            ProductAtom::Expression { .. } => "Expr".to_string(),
            ProductAtom::Unit { .. } => "Unit".to_string(),
            ProductAtom::SemanticValue { value, .. } => {
                format!("SemanticValue({})", value.0)
            }
            ProductAtom::Unsupported { summary, .. } => format!("Unsupported({summary})"),
        })
        .collect()
}
