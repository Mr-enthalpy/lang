mod support;
use support::*;

use lang_build::{ResolveExpectation, SourceCategory, Stage, SymbolPayload};

#[test]
fn directory_context_preserves_complete_types_without_structural_composition() {
    let world = build_single_fixture_world("nested_physical_namespace", "app");
    let semantic = world.semantic_world();
    let root = world.package_root_node();
    let a = semantic.child_namespace(root, "a").unwrap();
    let b = semantic.child_namespace(a, "b").unwrap();
    for (parent, name) in [(root, "a"), (a, "b")] {
        let directory = world
            .namespace_projection()
            .child_symbol(parent, name)
            .unwrap();
        assert!(matches!(directory.payload, SymbolPayload::Namespace { .. }));
        if let Some(binding) = semantic.symbol_in_namespace(parent, name) {
            assert!(binding.pure_p().is_none());
            assert!(binding.ordinary_value().is_none());
        }
    }
    let nested = semantic.symbol_in_namespace(b, "T").unwrap();
    let outer = semantic.symbol_in_namespace(root, "Root").unwrap();
    let core = semantic
        .symbol_in_namespace(world.core_node(), "uint8")
        .unwrap();
    assert_eq!(
        nested.pure_p().unwrap().complete_type,
        core.pure_p().unwrap().complete_type
    );
    assert_eq!(
        nested.pure_p().unwrap().complete_type,
        outer.pure_p().unwrap().complete_type
    );
    assert_ne!(nested.identity, outer.identity);
    assert_ne!(nested.pure_p_place(), outer.pure_p_place());
    assert!(semantic.compile_instances().next().is_none());
    assert!(semantic.symbol_in_namespace(b, "foo").is_none());
    assert!(semantic.child_namespace(b, "foo").is_none());
    assert!(semantic.child_namespace(root, "src").is_none());
}

#[test]
fn scalar_compile_results_use_canonical_inputs_without_a_nameexpr_result() {
    let world = build_single_fixture_world("compile_scalar_identity", "app");
    let semantic = world.semantic_world();
    let instances = semantic.compile_instances().collect::<Vec<_>>();
    assert_eq!(instances.len(), 3, "equal inputs reuse one instance within an owner; content and parent owner are independent axes");
    let nested_namespace = semantic
        .child_namespace(world.package_root_node(), "a")
        .unwrap();
    let nested_owner = semantic.namespace_owner(nested_namespace).unwrap();
    assert_eq!(
        instances
            .iter()
            .filter(|instance| instance.root.parent_owner == semantic.package_owner())
            .count(),
        2
    );
    assert_eq!(
        instances
            .iter()
            .filter(|instance| instance.root.parent_owner == nested_owner)
            .count(),
        1
    );
    for instance in instances {
        let result = instance
            .delivered_result()
            .expect("ordinary scalar result delivery");
        assert!(result.complete_type.is_none());
        assert!(matches!(
            semantic.value(result.value).unwrap().payload,
            lang_build::SemanticValuePayload::ConstructedLiteral { .. }
        ));
        assert!(!instance.evaluation_active());
        let self_name = semantic.symbol(instance.invoke_name()).unwrap();
        assert!(self_name.pure_p().is_none());
        assert!(self_name.ordinary_value().is_none());
    }
    let mut observation = semantic.clone();
    let mut address = |name| {
        let binding = semantic
            .symbol_in_namespace(world.package_root_node(), name)
            .unwrap();
        let value = binding.ordinary_value().unwrap();
        let record = semantic.value(value).unwrap();
        let shape =
            lang_build::ArgProductShape::from_flattened(lang_build::FlattenedProductMaterial {
                atoms: vec![lang_build::ProductAtom::SemanticValue {
                    value,
                    type_value: record.type_value,
                    mode: record.mode,
                    provenance: record.provenance.clone(),
                }],
                provenance: record.provenance.clone(),
                invariant: lang_build::FlattenedProductInvariant {
                    no_direct_product_atom_remains: true,
                },
            });
        observation
            .canonical_argument_address(&shape.raw_args[0], &shape.flattened.atoms[0])
            .unwrap()
    };
    assert_eq!(address("first"), address("same"));
    assert_ne!(address("first"), address("different"));
}

#[test]
fn type_value_binding_reuses_value_and_keeps_fresh_binding_place() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let symbol = world
        .resolve_with_expectation("T", ResolveExpectation::CoreTypeProjection)
        .expect("resolve ordinary type-value binding");
    let core_uint8 = world
        .resolve_with_expectation("uint8::core", ResolveExpectation::CoreTypeProjection)
        .expect("resolve core uint8 type");

    assert_eq!(symbol.name, "T");
    assert_eq!(symbol.source_category, SourceCategory::DeclaredSymbol);
    assert_eq!(symbol.parent, Some(world.package_root_node()));
    let view = symbol
        .policy_view
        .as_ref()
        .expect("type binding Policy view");
    assert!(view.pair.value.stage() == Some(Stage::Compile));
    assert!(view.pair.value.stage() != Some(Stage::Runtime));
    let symbol_id = symbol.id;

    let SymbolPayload::CompleteTypeProjection(type_projection) = symbol.payload else {
        panic!("expected bound Type payload");
    };

    let SymbolPayload::CompleteTypeProjection(core_type) = core_uint8.payload else {
        panic!("core uint8 is a CompleteType projection");
    };
    assert_eq!(type_projection.carrier_symbol_id, symbol_id);
    assert_eq!(type_projection.represented_type, core_type.represented_type);
    assert_ne!(symbol_id, core_uint8.id);
    assert!(type_projection.type_associated_namespace.is_some());
}
