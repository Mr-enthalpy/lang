mod support;

use support::*;

use lang_build::{
    NonValueArgKind, ProductAtom, ProductMaterialRole, ProductObject, RawArgValueClass,
};
use lang_syntax::NormExpr;

#[test]
fn exposed_product_nodes_flatten_left_to_right_from_source_fixture() {
    let shape = fixture_arg_product_shape(
        "product_exposed_left.lang",
        ProductMaterialRole::CallableArgumentProduct,
    );
    assert_eq!(atom_labels(&shape.flattened.atoms), ["a", "b", "c"]);
    assert!(shape.flattened.invariant.no_direct_product_atom_remains);

    let shape = fixture_arg_product_shape(
        "product_exposed_right.lang",
        ProductMaterialRole::CallableArgumentProduct,
    );
    assert_eq!(atom_labels(&shape.flattened.atoms), ["a", "b", "c"]);
}

#[test]
fn expression_barrier_blocks_product_flattening_from_source_fixture() {
    let site = fixture_call_site("product_expression_barrier.lang");
    let product = ProductObject::from_norm_product(
        site.source_product.clone(),
        ProductMaterialRole::CallableArgumentProduct,
    );
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
fn nested_call_source_local_flatten_and_expression_barrier_are_not_contradictory() {
    let shape_left = fixture_arg_product_shape(
        "product_exposed_left.lang",
        ProductMaterialRole::CallableArgumentProduct,
    );
    assert_eq!(
        atom_labels(&shape_left.flattened.atoms),
        ["a", "b", "c"],
        "((a, b), c) |> f: inner product flattens into the call source"
    );

    let shape_barrier = fixture_arg_product_shape(
        "product_expression_barrier.lang",
        ProductMaterialRole::CallableArgumentProduct,
    );
    assert_eq!(
        atom_labels(&shape_barrier.flattened.atoms),
        ["Call", "c"],
        "((a, b) |> f, c) |> g: inner call is an Expression barrier, outer sees Call + c"
    );
}

#[test]
fn unit_positions_and_raw_arg_non_value_boundary_are_preserved_from_source_fixture() {
    let shape = fixture_arg_product_shape(
        "product_unit_preservation.lang",
        ProductMaterialRole::MetaConstructionArgumentProduct,
    );
    assert_eq!(atom_labels(&shape.flattened.atoms), ["a", "Unit", "b"]);
    assert!(shape.flattened.atoms[1].provenance().span.is_some());

    assert_eq!(shape.arity, 3);
    assert!(matches!(
        shape.raw_args[1].value_class,
        RawArgValueClass::NonValue(NonValueArgKind::ProductUnit)
    ));
    assert_eq!(shape.raw_args[1].is_value(), Some(false));
    assert!(
        !shape.raw_args[1].receives_automatic_pass_action(),
        "ProductUnit does not receive automatic pass action at candidate-prep boundary"
    );
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
            ProductAtom::Unsupported { summary, .. } => format!("Unsupported({summary})"),
        })
        .collect()
}
