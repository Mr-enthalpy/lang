mod support;

use lang_build::{
    construct_field_value, construct_owner_value, constructed_question_view, leaf_value,
    placeholder_field_constructor_head, question_view_peels, ConstructorHead, FieldProjection,
    ForwardedValue, MetaInvocationValue, MetaValueTarget, Provenance, ReturnViewShape,
};

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

fn bounded_symbol() -> lang_build::SymbolId {
    lang_build::SymbolId(10)
}

fn provenance(desc: &str) -> Provenance {
    Provenance::new(desc)
}

// --- Core roundtrip tests ---

#[test]
fn owner_value_one_step_question_view_exposes_payload() {
    let inner_payload = leaf_value(
        leaf_meta_value("inner_payload", uint8_symbol()),
        provenance("inner_payload"),
    );
    let field_pat = construct_field_value(
        bounded_symbol(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        inner_payload.clone(),
        provenance("(1 inner::TB)"),
    );
    let owner = construct_owner_value(
        bounded_symbol(),
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
        bounded_symbol(),
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
        bounded_symbol(),
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
        bounded_symbol(),
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
        bounded_symbol(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        payload,
        provenance("1 inner::TB"),
    );
    let owner = construct_owner_value(
        bounded_symbol(),
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
        bounded_symbol(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        inner_payload.clone(),
        provenance("1 inner::TB"),
    );
    let owner = construct_owner_value(
        bounded_symbol(),
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
        bounded_symbol(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        leaf.clone(),
        provenance("x inner::TB"),
    );
    assert!(question_view_peels(&field_pat));

    let owner = construct_owner_value(bounded_symbol(), field_pat, provenance("x inner::TB TB"));
    assert!(question_view_peels(&owner));
}

#[test]
fn constructor_head_is_extractable() {
    let leaf = leaf_value(leaf_meta_value("x", uint8_symbol()), provenance("x"));
    assert!(leaf.constructor_head().is_none());

    let field_pat = construct_field_value(
        bounded_symbol(),
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
        bounded_symbol(),
        "inner",
        uint8_symbol(),
        FieldProjection::Value,
    );
    match &head {
        ConstructorHead::Field {
            owner_type_symbol_id,
            field_name,
            field_type_symbol_id,
            projection,
        } => {
            assert_eq!(*owner_type_symbol_id, bounded_symbol());
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
        bounded_symbol(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        leaf,
        provenance("1 inner::TB"),
    );
    let owner = construct_owner_value(bounded_symbol(), field_pat, provenance("(1 inner::TB) TB"));
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
        bounded_symbol(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        payload,
        provenance("original field-pat"),
    );
    let peeled = constructed_question_view(&field_pat);

    // Reconstruct with different provenance
    let reconstructed = construct_field_value(
        bounded_symbol(),
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
        bounded_symbol(),
        "inner".to_string(),
        uint8_symbol(),
        FieldProjection::Value,
        payload,
        provenance("fp"),
    );
    let owner = construct_owner_value(
        bounded_symbol(),
        field_pat.clone(),
        provenance("owner orig"),
    );
    let peeled = constructed_question_view(&owner);

    let reconstructed =
        construct_owner_value(bounded_symbol(), peeled, provenance("owner reconst"));

    assert_eq!(reconstructed, owner);
    assert!(!reconstructed.exact_eq_with_provenance(&owner));
}
