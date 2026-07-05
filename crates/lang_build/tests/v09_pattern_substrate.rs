mod support;

use lang_build::{
    construct_field_value, construct_owner_value, constructed_question_view, leaf_value,
    nav_component_name, placeholder_field_constructor_head, question_view_peels,
    ConstructionInstanceId, ConstructorHead, FieldProjection, ForwardedValue, LocalPatternPlaceId,
    MetaInvocationValue, MetaValueTarget, PatternExpectation, PatternFieldMaterialization,
    PatternHeadId, PatternHeadOrigin, PatternHeadRegistry, PatternLookupInput,
    PatternMaterializationContext, Provenance, ReturnViewShape,
};
use lang_syntax::{NormOrigin, NormRule, Span};

fn leaf_meta_value(name: &str, symbol_id: lang_build::SymbolId) -> MetaInvocationValue {
    MetaInvocationValue::ForwardedValue(ForwardedValue {
        target: MetaValueTarget::TypeSymbol(symbol_id),
        return_view: ReturnViewShape::Leaf,
        provenance: Provenance::new(name),
    })
}

fn uint8_symbol() -> lang_build::SymbolId {
    lang_build::SymbolId(1)
}

fn bounded_head() -> PatternHeadId {
    PatternHeadId(10)
}

fn inner_head() -> PatternHeadId {
    PatternHeadId(11)
}

fn provenance(desc: &str) -> Provenance {
    Provenance::new(desc)
}

fn norm_origin() -> NormOrigin {
    NormOrigin::Generated {
        rule: NormRule::Unsupported,
        span: Span::at(0, 1, 1),
    }
}

// --- PatternHeadId materialization and bounded lookup tests ---

#[test]
fn same_struct_name_in_different_materialization_contexts_have_distinct_pattern_heads() {
    let mut registry = PatternHeadRegistry::new();
    let global = registry.allocate_owner_head(
        PatternMaterializationContext::Global {
            symbol_id: lang_build::SymbolId(100),
        },
        "name",
        provenance("global name"),
    );
    let namespace = registry.allocate_owner_head(
        PatternMaterializationContext::Namespace {
            namespace_symbol_id: lang_build::SymbolId(200),
            symbol_id: lang_build::SymbolId(100),
        },
        "name",
        provenance("namespace name"),
    );
    let local = registry.allocate_owner_head(
        PatternMaterializationContext::Local {
            place_id: LocalPatternPlaceId(1),
        },
        "name",
        provenance("local name"),
    );

    assert_ne!(global, namespace);
    assert_ne!(global, local);
    assert_ne!(namespace, local);
    assert_eq!(registry.get(global).unwrap().display_name, "name");
    assert_eq!(registry.get(namespace).unwrap().display_name, "name");
    assert_eq!(registry.get(local).unwrap().display_name, "name");
}

#[test]
fn same_display_name_different_local_places_are_distinct() {
    let mut registry = PatternHeadRegistry::new();
    let local1 = registry.allocate_owner_head(
        PatternMaterializationContext::Local {
            place_id: LocalPatternPlaceId(1),
        },
        "name",
        provenance("local1"),
    );
    let local2 = registry.allocate_owner_head(
        PatternMaterializationContext::Local {
            place_id: LocalPatternPlaceId(2),
        },
        "name",
        provenance("local2"),
    );

    assert_ne!(local1, local2);
}

#[test]
fn generated_pattern_head_uses_construction_identity() {
    let mut registry = PatternHeadRegistry::new();
    let generated = registry.allocate_generated_head(
        ConstructionInstanceId(88),
        "generated",
        provenance("generated"),
    );

    assert_eq!(
        registry.get(generated).unwrap().origin,
        PatternHeadOrigin::Generated {
            construction_instance_id: ConstructionInstanceId(88)
        }
    );
}

#[test]
fn struct_materialization_boundary_allocates_owner_and_field_heads() {
    let mut registry = PatternHeadRegistry::new();
    let materialized = registry
        .materialize_struct_pattern_heads(
            PatternMaterializationContext::Global {
                symbol_id: lang_build::SymbolId(90),
            },
            "TB",
            [PatternFieldMaterialization {
                field_name: "inner".to_string(),
                field_type_symbol_id: uint8_symbol(),
                projection: FieldProjection::Value,
                provenance: provenance("inner field"),
            }],
            provenance("TB materialization"),
        )
        .expect("struct pattern heads materialize");

    let inner = materialized.field_heads[0].1;
    assert_eq!(
        registry.lookup_child(materialized.owner_head, "inner"),
        Some(inner)
    );
    assert_eq!(
        registry.get(inner).unwrap().origin,
        PatternHeadOrigin::Field {
            owner_head: materialized.owner_head,
            field_name: "inner".to_string(),
            field_type_symbol_id: uint8_symbol(),
            projection: FieldProjection::Value,
        }
    );
}

#[test]
fn field_head_is_owner_scoped() {
    let mut registry = PatternHeadRegistry::new();
    let owner_a = registry.allocate_owner_head(
        PatternMaterializationContext::Global {
            symbol_id: lang_build::SymbolId(10),
        },
        "Bounded",
        provenance("owner A"),
    );
    let owner_b = registry.allocate_owner_head(
        PatternMaterializationContext::Global {
            symbol_id: lang_build::SymbolId(11),
        },
        "Bounded",
        provenance("owner B"),
    );
    let inner_a = registry
        .allocate_field_head(
            owner_a,
            "inner",
            uint8_symbol(),
            FieldProjection::Value,
            provenance("inner A"),
        )
        .expect("inner A");
    let inner_b = registry
        .allocate_field_head(
            owner_b,
            "inner",
            uint8_symbol(),
            FieldProjection::Value,
            provenance("inner B"),
        )
        .expect("inner B");

    assert_ne!(inner_a, inner_b);
    assert_eq!(registry.lookup_child(owner_a, "inner"), Some(inner_a));
    assert_eq!(registry.lookup_child(owner_b, "inner"), Some(inner_b));
}

#[test]
fn bare_pattern_name_resolves_under_current_extraction_scope() {
    let mut registry = PatternHeadRegistry::new();
    let owner = registry.allocate_owner_head(
        PatternMaterializationContext::Global {
            symbol_id: lang_build::SymbolId(10),
        },
        "TB",
        provenance("TB"),
    );
    let inner = registry
        .allocate_field_head(
            owner,
            "inner",
            uint8_symbol(),
            FieldProjection::Value,
            provenance("inner::TB"),
        )
        .expect("inner::TB");

    let resolved = registry
        .resolve_pattern_lookup(PatternLookupInput::AutoName {
            name: "inner".to_string(),
            current_scope: owner,
            expectation: PatternExpectation::ExtractionChild,
            provenance: provenance("bare inner"),
        })
        .expect("bounded child lookup");
    assert_eq!(resolved, inner);
}

#[test]
fn explicit_terminated_nav_does_not_use_extraction_scope() {
    let mut registry = PatternHeadRegistry::new();
    let owner = registry.allocate_owner_head(
        PatternMaterializationContext::Global {
            symbol_id: lang_build::SymbolId(10),
        },
        "TB",
        provenance("TB"),
    );
    let inner = registry
        .allocate_field_head(
            owner,
            "inner",
            uint8_symbol(),
            FieldProjection::Value,
            provenance("inner::TB"),
        )
        .expect("inner::TB");

    let err = registry
        .resolve_pattern_lookup(PatternLookupInput::ExplicitNav {
            components: vec![nav_component_name("inner", norm_origin())],
            explicit_terminated: true,
            current_scope: Some(owner),
            expectation: PatternExpectation::PatternHead,
            provenance: provenance("inner::"),
        })
        .expect_err("explicit nav must not fall back to bounded child");
    assert_eq!(err.code, Some(lang_build::ResolverCode::Unresolved));
    assert_ne!(
        registry.lookup_explicit_path(&["inner".to_string()]),
        Some(inner)
    );
}

#[test]
fn explicit_nav_path_does_not_receive_extraction_completion() {
    let mut registry = PatternHeadRegistry::new();
    let owner = registry.allocate_owner_head(
        PatternMaterializationContext::Global {
            symbol_id: lang_build::SymbolId(10),
        },
        "TB",
        provenance("TB"),
    );
    let inner = registry
        .allocate_field_head(
            owner,
            "inner",
            uint8_symbol(),
            FieldProjection::Value,
            provenance("inner::TB"),
        )
        .expect("inner::TB");

    let err = registry
        .resolve_pattern_lookup(PatternLookupInput::ExplicitNav {
            components: vec![
                nav_component_name("inner", norm_origin()),
                nav_component_name("Other", norm_origin()),
            ],
            explicit_terminated: false,
            current_scope: Some(owner),
            expectation: PatternExpectation::PatternHead,
            provenance: provenance("inner::Other"),
        })
        .expect_err("explicit nav path must not receive bounded completion");
    assert_eq!(err.code, Some(lang_build::ResolverCode::Unresolved));
    assert_ne!(
        registry.lookup_explicit_path(&["inner".to_string(), "Other".to_string()]),
        Some(inner)
    );
}

#[test]
fn explicit_registered_path_resolves_without_bounded_scope() {
    let mut registry = PatternHeadRegistry::new();
    let owner = registry.allocate_owner_head(
        PatternMaterializationContext::Global {
            symbol_id: lang_build::SymbolId(10),
        },
        "TB",
        provenance("TB"),
    );
    let field_head = registry
        .allocate_field_head(
            owner,
            "inner",
            uint8_symbol(),
            FieldProjection::Value,
            provenance("inner::TB"),
        )
        .expect("field head");
    let explicit_head = registry.allocate_external_forward_head(
        lang_build::SymbolId(99),
        "inner",
        provenance("explicit inner"),
    );
    registry
        .register_explicit_path(["inner"], explicit_head, provenance("register inner::"))
        .expect("explicit path registers");

    let resolved = registry
        .resolve_pattern_lookup(PatternLookupInput::ExplicitNav {
            components: vec![nav_component_name("inner", norm_origin())],
            explicit_terminated: true,
            current_scope: Some(owner),
            expectation: PatternExpectation::PatternHead,
            provenance: provenance("inner::"),
        })
        .expect("explicit nav resolves registered path");

    assert_eq!(resolved, explicit_head);
    assert_ne!(resolved, field_head);
}

#[test]
fn duplicate_explicit_path_is_conflict() {
    let mut registry = PatternHeadRegistry::new();
    let first = registry.allocate_external_forward_head(
        lang_build::SymbolId(1),
        "inner",
        provenance("first"),
    );
    let second = registry.allocate_external_forward_head(
        lang_build::SymbolId(2),
        "inner",
        provenance("second"),
    );
    registry
        .register_explicit_path(["inner"], first, provenance("register first"))
        .expect("first registration");

    let err = registry
        .register_explicit_path(["inner"], second, provenance("register second"))
        .expect_err("different head for same explicit path is a conflict");
    assert_eq!(
        err.code,
        Some(lang_build::ResolverCode::PatternHeadConflict)
    );
}

#[test]
fn duplicate_field_name_under_same_owner_is_conflict() {
    let mut registry = PatternHeadRegistry::new();
    let owner = registry.allocate_owner_head(
        PatternMaterializationContext::Global {
            symbol_id: lang_build::SymbolId(10),
        },
        "TB",
        provenance("TB"),
    );
    registry
        .allocate_field_head(
            owner,
            "inner",
            uint8_symbol(),
            FieldProjection::Value,
            provenance("inner::TB"),
        )
        .expect("first field");

    let err = registry
        .allocate_field_head(
            owner,
            "inner",
            lang_build::SymbolId(2),
            FieldProjection::Value,
            provenance("conflicting inner::TB"),
        )
        .expect_err("same owner + same name + different material conflicts");
    assert_eq!(
        err.code,
        Some(lang_build::ResolverCode::PatternHeadConflict)
    );
}

#[test]
fn auto_name_with_non_extraction_expectation_is_rejected() {
    let mut registry = PatternHeadRegistry::new();
    let owner = registry.allocate_owner_head(
        PatternMaterializationContext::Global {
            symbol_id: lang_build::SymbolId(10),
        },
        "TB",
        provenance("TB"),
    );
    registry
        .allocate_field_head(
            owner,
            "inner",
            uint8_symbol(),
            FieldProjection::Value,
            provenance("inner::TB"),
        )
        .expect("field head");

    let err = registry
        .resolve_pattern_lookup(PatternLookupInput::AutoName {
            name: "inner".to_string(),
            current_scope: owner,
            expectation: PatternExpectation::PatternHead,
            provenance: provenance("bare inner as PatternHead"),
        })
        .expect_err("AutoName is only supported as ExtractionChild");
    assert_eq!(
        err.code,
        Some(lang_build::ResolverCode::UnsupportedPatternExpectation)
    );
}

#[test]
fn same_display_name_different_owner_head_values_are_not_equal() {
    let mut registry = PatternHeadRegistry::new();
    let owner1 = registry.allocate_owner_head(
        PatternMaterializationContext::Global {
            symbol_id: lang_build::SymbolId(31),
        },
        "Bounded",
        provenance("Bounded owner1"),
    );
    let owner2 = registry.allocate_owner_head(
        PatternMaterializationContext::Global {
            symbol_id: lang_build::SymbolId(32),
        },
        "Bounded",
        provenance("Bounded owner2"),
    );
    let payload = leaf_value(leaf_meta_value("1", uint8_symbol()), provenance("1"));

    let value1 = construct_owner_value(owner1, payload.clone(), provenance("owner1 value"));
    let value2 = construct_owner_value(owner2, payload, provenance("owner2 value"));

    assert_eq!(registry.get(owner1).unwrap().display_name, "Bounded");
    assert_eq!(registry.get(owner2).unwrap().display_name, "Bounded");
    assert_ne!(value1, value2);
}

#[test]
fn constructor_reconstruction_roundtrip_uses_pattern_head_identity() {
    let mut registry = PatternHeadRegistry::new();
    let owner = registry.allocate_owner_head(
        PatternMaterializationContext::Global {
            symbol_id: lang_build::SymbolId(40),
        },
        "TB",
        provenance("TB"),
    );
    let field = registry
        .allocate_field_head(
            owner,
            "inner",
            uint8_symbol(),
            FieldProjection::Value,
            provenance("inner::TB"),
        )
        .expect("inner::TB");

    let payload = leaf_value(leaf_meta_value("1", uint8_symbol()), provenance("1"));
    let field_value = construct_field_value(
        owner,
        field,
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        payload.clone(),
        provenance("1 inner::TB"),
    );
    let owner_value =
        construct_owner_value(owner, field_value.clone(), provenance("(1 inner::TB) TB"));

    assert_eq!(constructed_question_view(&owner_value), field_value);
    assert_eq!(constructed_question_view(&field_value), payload);

    let reconstructed_field = construct_field_value(
        owner,
        field,
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        constructed_question_view(&field_value),
        provenance("reconstructed field"),
    );
    let reconstructed_owner = construct_owner_value(
        owner,
        reconstructed_field,
        provenance("reconstructed owner"),
    );

    assert_eq!(reconstructed_owner, owner_value);
}

// --- Core roundtrip tests ---

#[test]
fn owner_value_one_step_question_view_exposes_payload() {
    let inner_payload = leaf_value(
        leaf_meta_value("inner_payload", uint8_symbol()),
        provenance("inner_payload"),
    );
    let field_pat = construct_field_value(
        bounded_head(),
        inner_head(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        inner_payload.clone(),
        provenance("(1 inner::TB)"),
    );
    let owner = construct_owner_value(
        bounded_head(),
        field_pat.clone(),
        provenance("(1 inner::TB) TB"),
    );

    // Owner question view peels to field-pattern value
    let peeled = constructed_question_view(&owner);
    assert_eq!(peeled, field_pat);

    // Owner question view does NOT peel to the leaf directly
    assert_ne!(peeled, inner_payload);
}

#[test]
fn field_pattern_one_step_question_view_exposes_payload() {
    let inner_payload = leaf_value(
        leaf_meta_value("inner_payload", uint8_symbol()),
        provenance("inner_payload"),
    );
    let field_pat = construct_field_value(
        bounded_head(),
        inner_head(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        inner_payload.clone(),
        provenance("(1 inner::TB)"),
    );

    // Field-pattern question view peels to payload
    let peeled = constructed_question_view(&field_pat);
    assert_eq!(peeled, inner_payload);
}

#[test]
fn leaf_question_view_is_idempotent() {
    let leaf = leaf_value(leaf_meta_value("leaf", uint8_symbol()), provenance("leaf"));
    let peeled = constructed_question_view(&leaf);
    assert_eq!(peeled, leaf);
}

#[test]
fn field_pattern_reconstruction_roundtrip() {
    let payload = leaf_value(leaf_meta_value("1", uint8_symbol()), provenance("1"));
    let field_pat = construct_field_value(
        bounded_head(),
        inner_head(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        payload.clone(),
        provenance("1 inner::TB"),
    );

    // peels back
    let peeled = constructed_question_view(&field_pat);
    assert_eq!(peeled, payload);

    // can reconstruct
    let reconstructed = construct_field_value(
        bounded_head(),
        inner_head(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        peeled.clone(),
        provenance("reconstructed"),
    );
    // semantic equality: same constructor + payload
    assert_eq!(reconstructed, field_pat);
    // but exact object identity differs because provenance is different
    assert!(!reconstructed.exact_eq_with_provenance(&field_pat));
}

#[test]
fn owner_reconstruction_roundtrip() {
    let payload = leaf_value(leaf_meta_value("1", uint8_symbol()), provenance("1"));
    let field_pat = construct_field_value(
        bounded_head(),
        inner_head(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        payload,
        provenance("1 inner::TB"),
    );
    let owner = construct_owner_value(
        bounded_head(),
        field_pat.clone(),
        provenance("(1 inner::TB) TB"),
    );

    // peels back
    let peeled_once = constructed_question_view(&owner);
    assert_eq!(peeled_once, field_pat);

    let peeled_twice = constructed_question_view(&peeled_once);
    assert_ne!(peeled_twice, field_pat);
    assert!(matches!(
        peeled_twice,
        lang_build::ConstructedValue::Leaf { .. }
    ));
}

#[test]
fn equality_does_not_insert_question() {
    let inner_payload = leaf_value(leaf_meta_value("1", uint8_symbol()), provenance("1"));
    let field_pat = construct_field_value(
        bounded_head(),
        inner_head(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        inner_payload.clone(),
        provenance("1 inner::TB"),
    );
    let owner = construct_owner_value(
        bounded_head(),
        field_pat.clone(),
        provenance("(1 inner::TB) TB"),
    );

    // owner != inner_payload — equality does NOT insert ? automatically
    assert_ne!(owner, inner_payload);
    // field_pat != inner_payload without explicit ?
    assert_ne!(field_pat, inner_payload);
    // after explicit ?, they match
    assert_eq!(constructed_question_view(&owner), field_pat);
    assert_eq!(constructed_question_view(&field_pat), inner_payload);
}

#[test]
fn has_question_view_distinguishes_peelable_from_leaf() {
    let leaf = leaf_value(leaf_meta_value("x", uint8_symbol()), provenance("x"));
    assert!(!question_view_peels(&leaf));

    let field_pat = construct_field_value(
        bounded_head(),
        inner_head(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        leaf.clone(),
        provenance("x inner::TB"),
    );
    assert!(question_view_peels(&field_pat));

    let owner = construct_owner_value(bounded_head(), field_pat, provenance("x inner::TB TB"));
    assert!(question_view_peels(&owner));
}

#[test]
fn constructor_head_is_extractable() {
    let leaf = leaf_value(leaf_meta_value("x", uint8_symbol()), provenance("x"));
    assert!(leaf.constructor_head().is_none());

    let field_pat = construct_field_value(
        bounded_head(),
        inner_head(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        leaf,
        provenance("x inner::TB"),
    );
    let head = field_pat
        .constructor_head()
        .expect("Field has constructor head");
    assert!(matches!(head, lang_build::ConstructorHead::Field { .. }));
}

#[test]
fn struct_type_records_field_constructor_placeholder() {
    let head = placeholder_field_constructor_head(
        bounded_head(),
        inner_head(),
        "inner",
        uint8_symbol(),
        FieldProjection::Value,
    );
    match &head {
        ConstructorHead::Field {
            owner_head,
            field_head,
            field_name,
            field_type_symbol_id,
            projection,
        } => {
            assert_eq!(*owner_head, bounded_head());
            assert_eq!(*field_head, inner_head());
            assert_eq!(*field_name, "inner");
            assert_eq!(*field_type_symbol_id, uint8_symbol());
            assert_eq!(*projection, FieldProjection::Value);
        }
        _ => panic!("expected Field constructor"),
    }
}

#[test]
fn into_leaf_value_for_lowering_unwraps_payload() {
    let inner_meta = leaf_meta_value("1", uint8_symbol());
    let inner_meta_clone = inner_meta.clone();
    let leaf = leaf_value(inner_meta, provenance("1"));
    let field_pat = construct_field_value(
        bounded_head(),
        inner_head(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        leaf,
        provenance("1 inner::TB"),
    );
    let owner = construct_owner_value(bounded_head(), field_pat, provenance("(1 inner::TB) TB"));
    assert_eq!(
        owner.into_leaf_value_for_internal_lowering_only(),
        inner_meta_clone
    );
}

#[test]
fn leaf_semantic_eq_ignores_inner_meta_value_provenance() {
    let leaf1 = leaf_value(
        MetaInvocationValue::ForwardedValue(ForwardedValue {
            target: MetaValueTarget::TypeSymbol(uint8_symbol()),
            return_view: ReturnViewShape::Leaf,
            provenance: Provenance::new("leaf1 provenance"),
        }),
        provenance("outer leaf1"),
    );
    let leaf2 = leaf_value(
        MetaInvocationValue::ForwardedValue(ForwardedValue {
            target: MetaValueTarget::TypeSymbol(uint8_symbol()),
            return_view: ReturnViewShape::Leaf,
            provenance: Provenance::new("leaf2 provenance — different from leaf1"),
        }),
        provenance("outer leaf2 — also different"),
    );

    // Semantic equality: ignores provenance at both levels
    assert_eq!(leaf1, leaf2);
    // Exact object identity: both levels' provenance differ
    assert!(!leaf1.exact_eq_with_provenance(&leaf2));
}

#[test]
fn reconstructed_field_pattern_semantic_eq_original() {
    let payload = leaf_value(leaf_meta_value("1", uint8_symbol()), provenance("1"));
    let field_pat = construct_field_value(
        bounded_head(),
        inner_head(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        payload,
        provenance("original field-pat"),
    );
    let peeled = constructed_question_view(&field_pat);

    // Reconstruct with different provenance
    let reconstructed = construct_field_value(
        bounded_head(),
        inner_head(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        peeled,
        provenance("reconstructed field-pat"),
    );

    // structural equality fails because provenance differs
    assert_eq!(reconstructed, field_pat);
    // exact object identity differs because provenance differs
    assert!(!reconstructed.exact_eq_with_provenance(&field_pat));
}

#[test]
fn reconstructed_owner_semantic_eq_original() {
    let payload = leaf_value(leaf_meta_value("1", uint8_symbol()), provenance("1"));
    let field_pat = construct_field_value(
        bounded_head(),
        inner_head(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        payload,
        provenance("fp"),
    );
    let owner = construct_owner_value(bounded_head(), field_pat.clone(), provenance("owner orig"));
    let peeled = constructed_question_view(&owner);

    let reconstructed = construct_owner_value(bounded_head(), peeled, provenance("owner reconst"));

    assert_eq!(reconstructed, owner);
    assert!(!reconstructed.exact_eq_with_provenance(&owner));
}
