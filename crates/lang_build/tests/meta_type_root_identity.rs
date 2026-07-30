//! Canonical meta-type root identity.
//!
//! `MetaTypeRoot = MetaCallableIdentity + Normalize(Arguments)` and
//! `TypeValue = (OuterMetaInstanceRoot, NormalizedStructBody)`.  The
//! `meta_type_roots` fixture declares two source meta functions `f` and `g`
//! with byte-identical bodies (`let r = (t inner) |> struct; r;`), so their
//! generated struct bodies normalize to the same body material.  The body is
//! shared material only:
//!
//! ```text
//! Root(f(uint8)) != Root(g(uint8))
//! Body(f(uint8)) =  Body(g(uint8))
//! ```

mod support;

use std::collections::BTreeSet;

use lang_build::{
    compute_canonical_meta_instance_key, extract_single_call_site, CanonicalValueAddr,
    ClusterSymbolResult, CompilationWorld, InvocationOutcome, MetaCallableIdentity,
    MetaInstanceKey, MetaInstanceRoot, NamespaceNodeId, OrdinaryInvocationContext,
    PatternComponentPolicy, PolicyPair, PolicyStage, Provenance, SemanticOwnerKind,
    SemanticValueId, SemanticWorld, StageSet, SymbolId, TypeDefinitionInstanceId, TypeValueId,
    ValueComponentPolicy, ValueMutability, ValuePresence,
};
use support::{build_single_fixture_world, initializer_from_source};

fn invoke_meta(
    world: &mut CompilationWorld,
    spelling: &str,
    provenance: &str,
) -> ClusterSymbolResult {
    let initializer = initializer_from_source(spelling);
    let call_site = extract_single_call_site(&initializer).expect("normalized call");
    let result = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&[ValueMutability::Const]),
            Provenance::new(provenance),
        )
        .expect("source meta callable is selected through the ordinary spine");
    let InvocationOutcome::ClusterSymbol(meta) = result else {
        panic!("meta-declared source callable returns a cluster construction");
    };
    meta
}

fn unique_type_member_root(
    world: &CompilationWorld,
    meta: &ClusterSymbolResult,
) -> (lang_build::PatternValueId, lang_build::TypeValueId) {
    assert_eq!(
        meta.construction.member_views.len(),
        1,
        "one self-rooted construction produces the cluster's unique type member"
    );
    let view = &meta.construction.member_views[0];
    assert!(view.value.is_none(), "constructed type members are pure-P");
    let type_value = world
        .semantic_world()
        .type_for_pattern(view.pattern)
        .expect("the unique type member pattern is paired with a canonical TypeValue");
    (view.pattern, type_value)
}

/// Same meta function + same normalized arguments + same body: the second
/// invocation reuses the same canonical TypeValue root and pattern.
#[test]
fn same_meta_same_args_reuses_one_canonical_type_root() {
    let mut world = build_single_fixture_world("meta_type_roots", "app");
    let first = invoke_meta(&mut world, "let A: type = uint8 f;", "root identity f #1");
    let second = invoke_meta(&mut world, "let B: type = uint8 f;", "root identity f #2");

    let (first_pattern, first_type) = unique_type_member_root(&world, &first);
    let (second_pattern, second_type) = unique_type_member_root(&world, &second);
    assert_eq!(
        first_type, second_type,
        "f(uint8) has exactly one canonical TypeValue root"
    );
    assert_eq!(
        first_pattern, second_pattern,
        "f(uint8) has exactly one canonical MetaInstance-owned pattern"
    );
    assert_eq!(
        first.generated_types[0].canonical_type,
        Some(first_type),
        "the generated body carries the canonical root annotation"
    );
    assert_eq!(
        first.generated_types[0].canonical_type,
        second.generated_types[0].canonical_type
    );
}

/// Different meta functions + same normalized arguments + same body:
/// distinct roots and distinct canonical TypeValues, while the normalized
/// internal body material stays equal.  The definition id alone must never
/// merge `g(uint8)` into `f(uint8)`'s root.
#[test]
fn different_meta_same_body_gets_distinct_roots_with_equal_body_material() {
    let mut world = build_single_fixture_world("meta_type_roots", "app");
    let f = invoke_meta(&mut world, "let A: type = uint8 f;", "root identity f");
    let g = invoke_meta(&mut world, "let B: type = uint8 g;", "root identity g");

    let (f_pattern, f_type) = unique_type_member_root(&world, &f);
    let (g_pattern, g_type) = unique_type_member_root(&world, &g);

    // Distinct roots: TypeValue, pattern, and root annotation all differ.
    assert_ne!(
        f_type, g_type,
        "Root(f(uint8)) != Root(g(uint8)): a shared body never merges roots"
    );
    assert_ne!(
        f_pattern, g_pattern,
        "g(uint8) must not reuse the pattern registered under MetaInstance(f, uint8)"
    );
    assert_ne!(
        f.generated_types[0].canonical_type,
        g.generated_types[0].canonical_type
    );

    // Each root is owned by its own MetaInstance.
    for (meta, pattern) in [(&f, f_pattern), (&g, g_pattern)] {
        let _ = meta;
        let owner = world
            .semantic_world()
            .pattern_owner(pattern)
            .expect("member pattern owner")
            .owner;
        assert!(matches!(
            world
                .semantic_world()
                .owners()
                .node(owner)
                .expect("owner node")
                .kind,
            SemanticOwnerKind::MetaInstance { .. }
        ));
    }

    // Equal normalized body material: the generated definitions carry the
    // same body identity and semantically equal field material, while their
    // canonical roots differ.
    let f_body = &f.generated_types[0];
    let g_body = &g.generated_types[0];
    assert_eq!(
        f_body.type_definition_id, g_body.type_definition_id,
        "Body(f(uint8)) = Body(g(uint8)): identical bodies normalize to one body id"
    );
    assert_eq!(f_body.fields.len(), g_body.fields.len());
    for (f_field, g_field) in f_body.fields.iter().zip(g_body.fields.iter()) {
        assert_eq!(f_field.name, g_field.name);
        assert_eq!(f_field.type_value, g_field.type_value);
        assert_eq!(f_field.index, g_field.index);
    }
}

fn static_type_pair() -> PolicyPair {
    let mut stages = StageSet::new();
    stages.insert(PolicyStage::Meta);
    stages.insert(PolicyStage::Compile);
    PolicyPair {
        value: ValueComponentPolicy {
            stages: stages.clone(),
            mutability: BTreeSet::new(),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy { stages },
    }
}

/// B4 — `MetaRootKey = function identity + normalized arguments`;
/// body material never participates in root identity.  Same function, same
/// arguments, different body: the root stays the same, and the follow-up is
/// the idempotence/conflict split — equal body replays the root, a
/// conflicting body is a hard construction conflict, never a second root.
#[test]
fn same_root_conflicting_body_is_a_conflict_never_a_second_root() {
    let mut world = SemanticWorld::new("unit");
    world.bind_package_namespace(NamespaceNodeId(0));
    let policy = static_type_pair();
    let provenance = Provenance::new("b4 root vs body");
    let (_meta_function, type_object, _pattern) = world
        .register_type_symbol(
            NamespaceNodeId(0),
            "type",
            SymbolId(1),
            TypeValueId(0),
            TypeValueId(0),
            None,
            policy.clone(),
            provenance.clone(),
        )
        .expect("type-rank symbol registers in the unit world");
    // The instance root binds the selected function object VALUE identity;
    // the placement parent is a declaration-environment owner only.
    let root = MetaInstanceRoot {
        meta_callable: MetaCallableIdentity {
            selected_function_value: type_object,
            selected_call_entry: SemanticValueId(type_object.as_u64() + 500),
        },
        placement_parent: world.package_owner(),
    };
    let key = MetaInstanceKey {
        callable: root.meta_callable,
        arguments: CanonicalValueAddr(1),
        provenance: provenance.clone(),
    };
    let body_a = TypeDefinitionInstanceId(11);
    let body_b = TypeDefinitionInstanceId(22);
    let pattern = lang_build::CanonicalPatternValue::Atom(lang_build::CanonicalPatternAtom::Type(
        lang_build::CanonicalTypeObservation::Detached(lang_build::TypeValueId(11)),
    ));

    // Root allocation, then idempotent reuse under equal body material.
    let first = world
        .install_generated_type_value(
            &root,
            key.clone(),
            body_a,
            pattern.clone(),
            policy.clone(),
            provenance.clone(),
        )
        .expect("first installation allocates the root")
        .expect("the type rank is installed");
    let replay = world
        .install_generated_type_value(
            &root,
            key.clone(),
            body_a,
            pattern.clone(),
            policy.clone(),
            provenance.clone(),
        )
        .expect("equal body under the same root is idempotent reuse")
        .expect("the type rank is installed");
    assert_eq!(
        first, replay,
        "same root + same body replays the identical (value, pattern, type) triple"
    );

    // Same root, conflicting body: hard error, and no second root.
    let conflict = world.install_generated_type_value(
        &root,
        key.clone(),
        body_b,
        pattern.clone(),
        policy.clone(),
        provenance.clone(),
    );
    let Err(diagnostic) = conflict else {
        panic!("a conflicting body under one root must be a construction conflict");
    };
    assert!(
        diagnostic.message.contains("meta construction conflict"),
        "the conflict diagnostic names the rule: {}",
        diagnostic.message
    );

    // The failed conflict never disturbed or split the original root.
    let after = world
        .install_generated_type_value(&root, key, body_a, pattern, policy, provenance)
        .expect("the original body still replays after the rejected conflict")
        .expect("the type rank is installed");
    assert_eq!(
        first, after,
        "a rejected conflicting body never allocates a second root"
    );
}

/// Two distinct meta function VALUES under one
/// carrier Symbol get distinct instance roots even for identical normalized
/// arguments and identical bodies.  The carrier Symbol never keys the root.
#[test]
fn distinct_meta_callables_under_one_carrier_symbol_get_distinct_roots() {
    let mut world = SemanticWorld::new("unit");
    world.bind_package_namespace(NamespaceNodeId(0));
    let policy = static_type_pair();
    let provenance = Provenance::new("two vals one symbol");
    let (_carrier, first_val, _pattern) = world
        .register_type_symbol(
            NamespaceNodeId(0),
            "type",
            SymbolId(1),
            TypeValueId(0),
            TypeValueId(0),
            None,
            policy.clone(),
            provenance.clone(),
        )
        .expect("type-rank symbol registers in the unit world");
    // Two distinct function object values hosted by ONE carrier Symbol.
    let root_a = MetaInstanceRoot {
        meta_callable: MetaCallableIdentity {
            selected_function_value: first_val,
            selected_call_entry: SemanticValueId(first_val.as_u64() + 500),
        },
        placement_parent: world.package_owner(),
    };
    let root_b = MetaInstanceRoot {
        meta_callable: MetaCallableIdentity {
            selected_function_value: SemanticValueId(first_val.as_u64() + 1000),
            selected_call_entry: SemanticValueId(first_val.as_u64() + 1500),
        },
        placement_parent: world.package_owner(),
    };
    // Identical normalized arguments for both invocations.
    let key_a = MetaInstanceKey {
        callable: root_a.meta_callable,
        arguments: CanonicalValueAddr(1),
        provenance: provenance.clone(),
    };
    let key_b = MetaInstanceKey {
        callable: root_b.meta_callable,
        arguments: CanonicalValueAddr(1),
        provenance: provenance.clone(),
    };
    let body = TypeDefinitionInstanceId(11);
    let pattern = lang_build::CanonicalPatternValue::Atom(lang_build::CanonicalPatternAtom::Type(
        lang_build::CanonicalTypeObservation::Detached(lang_build::TypeValueId(11)),
    ));

    let a = world
        .install_generated_type_value(
            &root_a,
            key_a,
            body,
            pattern.clone(),
            policy.clone(),
            provenance.clone(),
        )
        .expect("first val installs its own root")
        .expect("the type rank is installed");
    let b = world
        .install_generated_type_value(&root_b, key_b, body, pattern, policy, provenance)
        .expect("an equal body under a DIFFERENT meta callable is never a conflict")
        .expect("the type rank is installed");
    assert_ne!(
        a.2, b.2,
        "two meta vals under one carrier Symbol keep distinct canonical roots"
    );
    assert_ne!(
        a.1, b.1,
        "two meta vals under one carrier Symbol keep distinct root patterns"
    );
}

/// The canonical meta instance key is
/// built from the selected function value identity and the canonical address
/// of the whole argument Product: normalization-equivalent literal spellings
/// share one key, argument order is significant (the invocation parentheses
/// are a Product), and neither formal binder names nor any declaration
/// SymbolId can enter it.
#[test]
fn source_meta_key_normalizes_arguments_and_carries_no_formal_names() {
    use lang_build::{canonical_literal_norm, CanonicalNormForm, CanonicalProductConstructor};
    use lang_syntax::NormLiteralKind;

    let mut world = SemanticWorld::new("unit");
    let selected = MetaCallableIdentity {
        selected_function_value: SemanticValueId(7),
        selected_call_entry: SemanticValueId(70),
    };
    let provenance = Provenance::new("norm key");

    // Normalization-equivalent spellings intern to one address …
    let dec = world.intern_canonical_value(canonical_literal_norm(NormLiteralKind::Int, "4096"));
    let hex = world.intern_canonical_value(canonical_literal_norm(NormLiteralKind::Int, "0x1000"));
    assert_eq!(dec, hex, "Addr(4096) = Addr(0x1000)");
    // … so the argument Products — and the instance keys — agree as well.
    let args_dec = world.intern_canonical_value(CanonicalNormForm::Product {
        constructor: CanonicalProductConstructor::CallParentheses,
        members: vec![dec],
    });
    let args_hex = world.intern_canonical_value(CanonicalNormForm::Product {
        constructor: CanonicalProductConstructor::CallParentheses,
        members: vec![hex],
    });
    assert_eq!(
        args_dec, args_hex,
        "Addr(Product(4096)) = Addr(Product(0x1000))"
    );
    let key_dec = compute_canonical_meta_instance_key(selected, args_dec, provenance.clone());
    let key_hex = compute_canonical_meta_instance_key(selected, args_hex, provenance.clone());
    assert_eq!(key_dec, key_hex);

    // A different normalized argument changes the key.
    let other = world.intern_canonical_value(canonical_literal_norm(NormLiteralKind::Int, "2"));
    let args_other = world.intern_canonical_value(CanonicalNormForm::Product {
        constructor: CanonicalProductConstructor::CallParentheses,
        members: vec![other],
    });
    assert_ne!(
        key_dec,
        compute_canonical_meta_instance_key(selected, args_other, provenance.clone())
    );

    // The invocation parentheses are a Product, so the argument tuple is
    // order-sensitive at the top level: swapping two distinct arguments
    // produces a different Product address and a different key.
    let ab = world.intern_canonical_value(CanonicalNormForm::Product {
        constructor: CanonicalProductConstructor::CallParentheses,
        members: vec![dec, other],
    });
    let ba = world.intern_canonical_value(CanonicalNormForm::Product {
        constructor: CanonicalProductConstructor::CallParentheses,
        members: vec![other, dec],
    });
    assert_ne!(ab, ba, "Product argument order is positional identity");
    assert_ne!(
        compute_canonical_meta_instance_key(selected, ab, provenance.clone()),
        compute_canonical_meta_instance_key(selected, ba, provenance.clone())
    );

    // A different selected function value changes the key even for equal
    // normalized arguments (distinct meta vals never share a root).
    assert_ne!(
        key_dec,
        compute_canonical_meta_instance_key(
            MetaCallableIdentity {
                selected_function_value: SemanticValueId(8),
                selected_call_entry: SemanticValueId(70),
            },
            args_dec,
            provenance.clone()
        )
    );

    // One function value exposing two distinct `()` call entries is TWO
    // distinct meta callables: the selected call entry is a
    // structural coordinate of the key.
    assert_ne!(
        key_dec,
        compute_canonical_meta_instance_key(
            MetaCallableIdentity {
                selected_function_value: SemanticValueId(7),
                selected_call_entry: SemanticValueId(71),
            },
            args_dec,
            provenance.clone()
        )
    );

    // The key stores its structural coordinates directly — equality is
    // defined on them, never through a digest.
    assert_eq!(key_dec.callable, selected);
    assert_eq!(key_dec.arguments, args_dec);

    // Opaque fresh addresses never merge: material without a stable normal
    // form can only under-merge.
    let fresh_a = world.fresh_opaque_canonical_address();
    let fresh_b = world.fresh_opaque_canonical_address();
    assert_ne!(fresh_a, fresh_b);
    let args_fresh_a = world.intern_canonical_value(CanonicalNormForm::Product {
        constructor: CanonicalProductConstructor::CallParentheses,
        members: vec![fresh_a],
    });
    let args_fresh_b = world.intern_canonical_value(CanonicalNormForm::Product {
        constructor: CanonicalProductConstructor::CallParentheses,
        members: vec![fresh_b],
    });
    assert_ne!(
        compute_canonical_meta_instance_key(selected, args_fresh_a, provenance.clone()),
        compute_canonical_meta_instance_key(selected, args_fresh_b, provenance)
    );
}

/// Already-materialized simple literal values normalize by
/// CONTENT, not by value identity: two distinct `SimpleLiteral` semantic
/// values with normalization-equivalent spellings share one canonical
/// argument address (and merge with the un-materialized literal spelling),
/// while content-free `PlainValue` material stays identity-opaque.
#[test]
fn materialized_simple_literals_normalize_by_content_not_identity() {
    use lang_build::{canonical_literal_norm, ProductAtom, RawArgShape};
    use lang_syntax::NormLiteralKind;

    let mut world = SemanticWorld::new("unit");
    world.bind_package_namespace(NamespaceNodeId(0));
    let policy = static_type_pair();
    let provenance = Provenance::new("literal content");
    world
        .register_type_symbol(
            NamespaceNodeId(0),
            "t",
            SymbolId(1),
            TypeValueId(0),
            TypeValueId(0),
            None,
            policy.clone(),
            provenance.clone(),
        )
        .expect("type-rank symbol registers in the unit world");

    let address_of = |world: &mut SemanticWorld, value| {
        let atom = ProductAtom::SemanticValue {
            value,
            type_value: TypeValueId(0),
            mutability: ValueMutability::Const,
            provenance: provenance.clone(),
        };
        let raw = RawArgShape::from_product_atom(0, &atom);
        world
            .canonical_argument_address(&raw, &atom)
            .expect("acyclic Val2 normalizes")
    };

    // Two DISTINCT materialized literal values, normalization-equivalent
    // spellings: one canonical address.
    let dec = world
        .install_simple_literal_value(
            TypeValueId(0),
            policy.clone(),
            NormLiteralKind::Int,
            "4096",
            provenance.clone(),
        )
        .expect("literal value installs against the registered type");
    let hex = world
        .install_simple_literal_value(
            TypeValueId(0),
            policy.clone(),
            NormLiteralKind::Int,
            "0x1000",
            provenance.clone(),
        )
        .expect("literal value installs against the registered type");
    assert_ne!(dec, hex, "two installations are two distinct Val1");
    let dec_addr = address_of(&mut world, dec);
    let hex_addr = address_of(&mut world, hex);
    assert_eq!(
        dec_addr, hex_addr,
        "Norm(Val1, P) is content normalization, never OpaqueValue(id)"
    );

    // The materialized content stays SEPARATE from the un-materialized
    // literal spelling: `Norm(Val1, P)` is a pair, and the materialized
    // value's named-type P differs from the spelling's intrinsic literal
    // P.  Different content separates as always.
    let spelling =
        world.intern_canonical_value(canonical_literal_norm(NormLiteralKind::Int, "4'096"));
    assert_ne!(
        dec_addr, spelling,
        "same Val1 content under different P keeps different addresses"
    );
    let other = world
        .install_simple_literal_value(
            TypeValueId(0),
            policy.clone(),
            NormLiteralKind::Int,
            "2",
            provenance.clone(),
        )
        .expect("literal value installs against the registered type");
    assert_ne!(dec_addr, address_of(&mut world, other));

    // Content-free plain values keep the identity-stable opaque form: one
    // value re-normalizes to one address, two values never merge.
    let plain_a = world
        .install_plain_value(TypeValueId(0), policy.clone(), provenance.clone())
        .expect("plain value installs against the registered type");
    let plain_b = world
        .install_plain_value(TypeValueId(0), policy, provenance.clone())
        .expect("plain value installs against the registered type");
    let plain_a_addr = address_of(&mut world, plain_a);
    assert_eq!(plain_a_addr, address_of(&mut world, plain_a));
    assert_ne!(plain_a_addr, address_of(&mut world, plain_b));
}
