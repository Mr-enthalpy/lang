use lang_build::{
    NonValueArgKind, ProductAtom, ProductMaterialRole, ProductObject, RawArgValueClass,
};
use lang_syntax::{NormExpr, NormOrigin, NormProduct, NormProductElem, Span};

#[test]
fn exposed_product_nodes_flatten_left_to_right() {
    let product = norm_product(
        10,
        vec![
            elem(expr_product(
                11,
                vec![elem(name(1, "a")), elem(name(2, "b"))],
            )),
            elem(name(3, "c")),
        ],
    );
    let flattened = flatten(product);
    assert_eq!(atom_labels(&flattened.atoms), ["a", "b", "c"]);
    assert!(flattened.invariant.no_direct_product_atom_remains);

    let product = norm_product(
        20,
        vec![
            elem(name(4, "a")),
            elem(expr_product(
                21,
                vec![elem(name(5, "b")), elem(name(6, "c"))],
            )),
        ],
    );
    let flattened = flatten(product);
    assert_eq!(atom_labels(&flattened.atoms), ["a", "b", "c"]);
}

#[test]
fn expression_barrier_blocks_product_flattening() {
    let call = norm_call(10, norm_product(11, vec![elem(name(1, "a"))]), name(2, "f"));
    let product = norm_product(
        12,
        vec![
            elem(call),
            elem(expr_product(
                13,
                vec![elem(name(3, "b")), elem(name(4, "c"))],
            )),
        ],
    );
    let flattened = flatten(product);
    assert_eq!(atom_labels(&flattened.atoms), ["Call", "b", "c"]);

    let call_with_product_source = norm_call(
        20,
        norm_product(21, vec![elem(name(5, "a")), elem(name(6, "b"))]),
        name(7, "f"),
    );
    let product = norm_product(22, vec![elem(call_with_product_source), elem(name(8, "c"))]);
    let flattened = flatten(product);
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
        "Expression barrier must preserve the call source product"
    );
}

#[test]
fn unit_positions_and_raw_arg_non_value_boundary_are_preserved() {
    let product = norm_product(
        30,
        vec![
            elem(expr_product(31, vec![elem(name(1, "a")), unit(2)])),
            elem(name(3, "b")),
        ],
    );
    let flattened = flatten(product.clone());
    assert_eq!(atom_labels(&flattened.atoms), ["a", "Unit", "b"]);
    assert_eq!(flattened.atoms[1].provenance().span, Some(span(2)));

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
    assert!(!arg_shape.raw_args[1].receives_automatic_pass_action());

    let product = norm_product(
        40,
        vec![elem(expr_product(41, vec![unit(4)])), elem(name(5, "a"))],
    );
    let flattened = flatten(product);
    assert_eq!(atom_labels(&flattened.atoms), ["Unit", "a"]);
}

fn flatten(product: NormProduct) -> lang_build::FlattenedProductObject {
    ProductObject::from_norm_product(product, ProductMaterialRole::CallableArgumentProduct)
        .flatten()
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

fn norm_product(index: usize, elements: Vec<NormProductElem>) -> NormProduct {
    NormProduct {
        elements,
        origin: origin(index),
    }
}

fn expr_product(index: usize, elements: Vec<NormProductElem>) -> NormExpr {
    NormExpr::Product(norm_product(index, elements))
}

fn elem(expr: NormExpr) -> NormProductElem {
    NormProductElem::Expr(expr)
}

fn unit(index: usize) -> NormProductElem {
    NormProductElem::Unit {
        origin: origin(index),
    }
}

fn name(index: usize, text: &str) -> NormExpr {
    NormExpr::Name {
        text: text.to_string(),
        origin: origin(index),
    }
}

fn norm_call(index: usize, source: NormProduct, target: NormExpr) -> NormExpr {
    NormExpr::Call {
        source,
        target: Box::new(target),
        origin: origin(index),
    }
}

fn origin(index: usize) -> NormOrigin {
    NormOrigin::Source(span(index))
}

fn span(index: usize) -> Span {
    Span::new(index, index + 1, 1, index + 1)
}
