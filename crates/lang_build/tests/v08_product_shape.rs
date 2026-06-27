use std::{fs, path::PathBuf};

use lang_build::{
    NonValueArgKind, ProductAtom, ProductMaterialRole, ProductObject, RawArgValueClass,
};
use lang_syntax::{NormExpr, NormForm, NormProduct};

#[test]
fn exposed_product_nodes_flatten_left_to_right_from_source_fixture() {
    let flattened = flatten_fixture_call_source("product_exposed_left.lang");
    assert_eq!(atom_labels(&flattened.atoms), ["a", "b", "c"]);
    assert!(flattened.invariant.no_direct_product_atom_remains);

    let flattened = flatten_fixture_call_source("product_exposed_right.lang");
    assert_eq!(atom_labels(&flattened.atoms), ["a", "b", "c"]);
}

#[test]
fn expression_barrier_blocks_product_flattening_from_source_fixture() {
    let source = fixture_call_source("product_expression_barrier.lang");
    let flattened = flatten(source);
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
fn unit_positions_and_raw_arg_non_value_boundary_are_preserved_from_source_fixture() {
    let product = fixture_call_source("product_unit_preservation.lang");
    let flattened = flatten(product.clone());
    assert_eq!(atom_labels(&flattened.atoms), ["a", "Unit", "b"]);
    assert!(flattened.atoms[1].provenance().span.is_some());

    let arg_shape = ProductObject::from_norm_product(
        product,
        ProductMaterialRole::MetaConstructionArgumentProduct,
    )
    .to_arg_product_shape();
    assert_eq!(arg_shape.arity, 3);
    assert!(matches!(
        arg_shape.raw_args[1].value_class,
        RawArgValueClass::NonValue(NonValueArgKind::ProductUnit)
    ));
    assert_eq!(arg_shape.raw_args[1].is_value(), Some(false));
    assert!(
        !arg_shape.raw_args[1].receives_automatic_pass_action(),
        "ProductUnit does not receive automatic pass action at candidate-prep boundary"
    );
}

fn flatten_fixture_call_source(name: &str) -> lang_build::FlattenedProductObject {
    flatten(fixture_call_source(name))
}

fn flatten(product: NormProduct) -> lang_build::FlattenedProductObject {
    ProductObject::from_norm_product(product, ProductMaterialRole::CallableArgumentProduct)
        .flatten()
}

fn fixture_call_source(name: &str) -> NormProduct {
    match fixture_expr(name) {
        NormExpr::Call { source, .. } => source,
        other => panic!("expected fixture `{name}` to normalize to a call, got {other:#?}"),
    }
}

fn fixture_expr(name: &str) -> NormExpr {
    let source = fs::read_to_string(fixture_path(name)).expect("read v0.8 product fixture");
    let parsed = lang_syntax::parse(&source);
    assert!(
        parsed.diagnostics.is_empty(),
        "unexpected parse diagnostics for {name}:\n{}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    let normalized = lang_syntax::normalize_program(&parsed.program);
    match normalized.forms.as_slice() {
        [NormForm::Expr(expr)] => expr.clone(),
        other => panic!("expected one normalized expression in `{name}`, got {other:#?}"),
    }
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/v08")
        .join(name)
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
