use lang_build::{
    attach_type_definition_pattern_heads, attach_type_definition_pattern_heads_with_context,
    bind_meta_invocation_value_result_with_materialization_state,
    compute_type_definition_instance_id, CanonicalArgProductShapeMaterial, FieldSignatureMaterial,
    GeneratedFieldDefinition, GeneratedTypeDefinitionValue, LocalPatternPlaceId,
    NamespaceGraphSnapshot, NamespaceNode, NamespaceNodeId, NamespaceNodeKind, PatternHeadId,
    PatternHeadOrigin, PatternMaterializationContext, Provenance, ReturnSlotSemantics,
    ReturnViewShape, SourceCategory, SymbolId, SymbolObject, SymbolPayload,
    TypeDefinitionIdentityMaterial, TypeDefinitionInstanceId, TypeMaterializationState,
};

fn provenance(label: &str) -> Provenance {
    Provenance::new(label)
}

fn generated_struct_value(
    type_definition_id: TypeDefinitionInstanceId,
) -> GeneratedTypeDefinitionValue {
    let field_provenance = provenance("field x");
    GeneratedTypeDefinitionValue {
        type_definition_id,
        identity_material: TypeDefinitionIdentityMaterial {
            callee_symbol_id: SymbolId(1),
            canonical_args: CanonicalArgProductShapeMaterial {
                arity: 0,
                unit_positions: Vec::new(),
                atom_kinds: Vec::new(),
                known_type_symbols: Vec::new(),
            },
            field_signature_material: vec![FieldSignatureMaterial {
                field_name: "x".to_string(),
                field_type_symbol_id: SymbolId(50),
                field_index: 0,
                provenance: field_provenance.clone(),
            }],
            return_slot_semantics: ReturnSlotSemantics::Generate,
            build_identity_fragment: None,
            policy_export_fingerprint_fragment: None,
            provenance: provenance("identity"),
        },
        fields: vec![GeneratedFieldDefinition {
            name: "x".to_string(),
            type_symbol_id: SymbolId(50),
            index: 0,
            pattern_head: None,
            provenance: field_provenance,
        }],
        pattern_heads: None,
        return_view: ReturnViewShape::Leaf,
        type_pattern_expr: None,
        sum_pattern_space: None,
        provenance: provenance("generated struct"),
    }
}

fn generated_struct_value_for_binding() -> GeneratedTypeDefinitionValue {
    let mut value = generated_struct_value(TypeDefinitionInstanceId(0));
    value.type_definition_id = compute_type_definition_instance_id(&value.identity_material);
    value
}

fn owner_head(value: &GeneratedTypeDefinitionValue) -> PatternHeadId {
    value
        .pattern_heads
        .as_ref()
        .expect("pattern heads attached")
        .owner_head
}

fn stripped(mut value: GeneratedTypeDefinitionValue) -> GeneratedTypeDefinitionValue {
    value.pattern_heads = None;
    for field in &mut value.fields {
        field.pattern_head = None;
    }
    value
}

fn install_test_namespace(
    snapshot: NamespaceGraphSnapshot,
    name: &str,
) -> (NamespaceGraphSnapshot, SymbolId, NamespaceNodeId) {
    let root = snapshot.root_node();
    let mut delta = snapshot.empty_delta();
    let namespace_node_id = delta.allocate_node_id();
    let namespace_symbol_id = delta.allocate_symbol_id();
    delta.insert_node(NamespaceNode::new(
        namespace_node_id,
        format!("{name}<namespace>"),
        NamespaceNodeKind::Virtual,
        SourceCategory::DeclaredSymbol,
        Some(root),
        provenance("test namespace node"),
    ));
    delta.insert_symbol(
        root,
        SymbolObject::namespace(
            namespace_symbol_id,
            name,
            namespace_node_id,
            NamespaceNodeKind::Virtual,
            SourceCategory::DeclaredSymbol,
            Some(root),
            provenance("test namespace symbol"),
        ),
    );
    let snapshot = snapshot
        .install_delta(delta)
        .expect("test namespace installs");
    (snapshot, namespace_symbol_id, namespace_node_id)
}

fn type_payload(symbol: &SymbolObject) -> &lang_build::TypeObject {
    match &symbol.payload {
        SymbolPayload::Type(type_object) => type_object,
        other => panic!("expected Type payload, got {other:?}"),
    }
}

#[test]
fn generated_fallback_attaches_generated_type_definition_origin() {
    let mut state = TypeMaterializationState::default();
    let type_definition_id = TypeDefinitionInstanceId(101);
    let value = attach_type_definition_pattern_heads(
        generated_struct_value(type_definition_id),
        &mut state,
        provenance("attach generated fallback"),
    )
    .expect("generated fallback attachment succeeds");

    let owner_head = owner_head(&value);
    assert_eq!(
        state.pattern_heads.get(owner_head).unwrap().origin,
        PatternHeadOrigin::GeneratedTypeDefinition { type_definition_id }
    );
    assert!(state.pattern_heads.lookup_child(owner_head, "x").is_some());
    assert_ne!(
        state.pattern_heads.get(owner_head).unwrap().display_name,
        ""
    );
}

#[test]
fn explicit_global_registry_context_attaches_global_owner_origin() {
    let mut state = TypeMaterializationState::default();
    let symbol_id = SymbolId(20);
    let value = attach_type_definition_pattern_heads_with_context(
        generated_struct_value(TypeDefinitionInstanceId(102)),
        &mut state,
        PatternMaterializationContext::Global { symbol_id },
        "Name",
        provenance("attach global"),
    )
    .expect("global attachment succeeds");

    let owner_head = owner_head(&value);
    assert_eq!(
        state.pattern_heads.get(owner_head).unwrap().origin,
        PatternHeadOrigin::GlobalBinding { symbol_id }
    );
    assert_eq!(
        state.pattern_heads.get(owner_head).unwrap().display_name,
        "Name"
    );
    assert_eq!(
        value.fields[0].pattern_head,
        state.pattern_heads.lookup_child(owner_head, "x")
    );
}

#[test]
fn explicit_namespace_registry_context_attaches_namespace_owner_origin() {
    let mut state = TypeMaterializationState::default();
    let symbol_id = SymbolId(30);
    let namespace_a = SymbolId(300);
    let namespace_b = SymbolId(301);

    let value_a = attach_type_definition_pattern_heads_with_context(
        generated_struct_value(TypeDefinitionInstanceId(103)),
        &mut state,
        PatternMaterializationContext::Namespace {
            namespace_symbol_id: namespace_a,
            symbol_id,
        },
        "Name",
        provenance("attach namespace a"),
    )
    .expect("namespace attachment succeeds");
    let value_b = attach_type_definition_pattern_heads_with_context(
        generated_struct_value(TypeDefinitionInstanceId(103)),
        &mut state,
        PatternMaterializationContext::Namespace {
            namespace_symbol_id: namespace_b,
            symbol_id,
        },
        "Name",
        provenance("attach namespace b"),
    )
    .expect("namespace attachment succeeds");

    let owner_a = owner_head(&value_a);
    let owner_b = owner_head(&value_b);
    assert_ne!(owner_a, owner_b);
    assert_eq!(
        state.pattern_heads.get(owner_a).unwrap().origin,
        PatternHeadOrigin::NamespaceBinding {
            namespace_symbol_id: namespace_a,
            symbol_id,
        }
    );
    assert_eq!(
        state.pattern_heads.get(owner_b).unwrap().origin,
        PatternHeadOrigin::NamespaceBinding {
            namespace_symbol_id: namespace_b,
            symbol_id,
        }
    );
}

#[test]
fn same_display_spelling_different_contexts_have_distinct_identities() {
    let mut state = TypeMaterializationState::default();
    let type_definition_id = TypeDefinitionInstanceId(104);
    let global = attach_type_definition_pattern_heads_with_context(
        generated_struct_value(type_definition_id),
        &mut state,
        PatternMaterializationContext::Global {
            symbol_id: SymbolId(40),
        },
        "Name",
        provenance("attach global"),
    )
    .expect("global attachment succeeds");
    let namespace = attach_type_definition_pattern_heads_with_context(
        generated_struct_value(type_definition_id),
        &mut state,
        PatternMaterializationContext::Namespace {
            namespace_symbol_id: SymbolId(400),
            symbol_id: SymbolId(40),
        },
        "Name",
        provenance("attach namespace"),
    )
    .expect("namespace attachment succeeds");
    let generated = attach_type_definition_pattern_heads(
        generated_struct_value(type_definition_id),
        &mut state,
        provenance("attach generated"),
    )
    .expect("generated attachment succeeds");

    let global_owner = owner_head(&global);
    let namespace_owner = owner_head(&namespace);
    let generated_owner = owner_head(&generated);
    assert_ne!(global_owner, namespace_owner);
    assert_ne!(global_owner, generated_owner);
    assert_ne!(namespace_owner, generated_owner);
    assert_eq!(
        state.pattern_heads.get(global_owner).unwrap().display_name,
        "Name"
    );
    assert_eq!(
        state
            .pattern_heads
            .get(namespace_owner)
            .unwrap()
            .display_name,
        "Name"
    );
    assert_eq!(
        state
            .pattern_heads
            .get(generated_owner)
            .unwrap()
            .display_name,
        "generated-type-definition-104"
    );
}

#[test]
fn stripped_values_reattach_under_explicit_registry_context() {
    let mut state = TypeMaterializationState::default();
    let type_definition_id = TypeDefinitionInstanceId(105);
    let generated = attach_type_definition_pattern_heads(
        generated_struct_value(type_definition_id),
        &mut state,
        provenance("attach generated"),
    )
    .expect("generated attachment succeeds");

    // Cacheable generated type-definition values strip concrete PatternHeadId
    // material. The low-level helper may reattach them under an explicitly
    // supplied transitional registry context.
    let replayable = stripped(generated);
    let global = attach_type_definition_pattern_heads_with_context(
        replayable.clone(),
        &mut state,
        PatternMaterializationContext::Global {
            symbol_id: SymbolId(50),
        },
        "Name",
        provenance("reattach global"),
    )
    .expect("global reattachment succeeds");
    let namespace = attach_type_definition_pattern_heads_with_context(
        replayable,
        &mut state,
        PatternMaterializationContext::Namespace {
            namespace_symbol_id: SymbolId(500),
            symbol_id: SymbolId(50),
        },
        "Name",
        provenance("reattach namespace"),
    )
    .expect("namespace reattachment succeeds");

    assert_eq!(
        state.pattern_heads.get(owner_head(&global)).unwrap().origin,
        PatternHeadOrigin::GlobalBinding {
            symbol_id: SymbolId(50),
        }
    );
    assert_eq!(
        state
            .pattern_heads
            .get(owner_head(&namespace))
            .unwrap()
            .origin,
        PatternHeadOrigin::NamespaceBinding {
            namespace_symbol_id: SymbolId(500),
            symbol_id: SymbolId(50),
        }
    );
}

#[test]
fn local_context_uses_place_identity_not_rendered_path_identity() {
    let mut state = TypeMaterializationState::default();
    let place_id = LocalPatternPlaceId(7);
    let value = attach_type_definition_pattern_heads_with_context(
        generated_struct_value(TypeDefinitionInstanceId(106)),
        &mut state,
        PatternMaterializationContext::Local { place_id },
        "Name",
        provenance("attach local"),
    )
    .expect("local attachment succeeds");

    let owner_head = owner_head(&value);
    assert_eq!(
        state.pattern_heads.get(owner_head).unwrap().origin,
        PatternHeadOrigin::LocalMaterialization {
            place_id,
            display_name: "Name".to_string(),
        }
    );
    assert_eq!(
        state.pattern_heads.get(owner_head).unwrap().display_name,
        "Name"
    );
    assert_ne!(
        state.pattern_heads.get(owner_head).unwrap().display_name,
        "Name::__inner_ns::Self"
    );
}

#[test]
fn binding_generated_type_at_root_uses_generated_fallback() {
    let snapshot = NamespaceGraphSnapshot::new();
    let mut state = TypeMaterializationState::default();
    let value = generated_struct_value_for_binding();
    let type_definition_id = value.type_definition_id;
    let expansion = bind_meta_invocation_value_result_with_materialization_state(
        lang_build::MetaInvocationValue::GeneratedTypeDefinitionValue(value),
        &snapshot,
        snapshot.root_node(),
        "Name",
        provenance("bind root generated type"),
        &mut state,
    )
    .expect("root binding succeeds");

    let type_object = type_payload(&expansion.replacement_object);
    let owner_head = type_object
        .owner_pattern_head
        .expect("binding attaches owner pattern head");
    assert_eq!(type_object.type_symbol_id, expansion.replacement_object.id);
    assert_eq!(
        state.pattern_heads.get(owner_head).unwrap().origin,
        PatternHeadOrigin::GeneratedTypeDefinition { type_definition_id }
    );
    assert_eq!(
        type_object.fields[0].pattern_head,
        state.pattern_heads.lookup_child(owner_head, "x")
    );
    assert_eq!(
        type_object
            .extraction_interface
            .as_ref()
            .expect("struct binding exposes extraction interface")
            .owner_pattern_head,
        Some(owner_head)
    );
}

#[test]
fn binding_destination_does_not_select_or_reroot_owner_head() {
    let snapshot = NamespaceGraphSnapshot::new();
    let (snapshot, _, namespace_a_node_id) = install_test_namespace(snapshot, "ns_a");
    let (snapshot, _, namespace_b_node_id) = install_test_namespace(snapshot, "ns_b");
    let mut state = TypeMaterializationState::default();
    let replayable = stripped(generated_struct_value_for_binding());
    let type_definition_id = replayable.type_definition_id;

    let expansion_a = bind_meta_invocation_value_result_with_materialization_state(
        lang_build::MetaInvocationValue::GeneratedTypeDefinitionValue(replayable.clone()),
        &snapshot,
        namespace_a_node_id,
        "Name",
        provenance("bind namespace a generated type"),
        &mut state,
    )
    .expect("namespace a binding succeeds");
    let snapshot = snapshot
        .install_delta(expansion_a.namespace_delta.clone())
        .expect("namespace a binding delta installs");
    let expansion_b = bind_meta_invocation_value_result_with_materialization_state(
        lang_build::MetaInvocationValue::GeneratedTypeDefinitionValue(replayable),
        &snapshot,
        namespace_b_node_id,
        "Name",
        provenance("bind namespace b generated type"),
        &mut state,
    )
    .expect("namespace b binding succeeds");

    let type_a = type_payload(&expansion_a.replacement_object);
    let type_b = type_payload(&expansion_b.replacement_object);
    let owner_a = type_a
        .owner_pattern_head
        .expect("namespace a binding attaches owner pattern head");
    let owner_b = type_b
        .owner_pattern_head
        .expect("namespace b binding attaches owner pattern head");

    assert_eq!(owner_a, owner_b);
    assert_eq!(
        state.pattern_heads.get(owner_a).unwrap().origin,
        PatternHeadOrigin::GeneratedTypeDefinition { type_definition_id }
    );
    assert_eq!(
        state.pattern_heads.get(owner_b).unwrap().origin,
        PatternHeadOrigin::GeneratedTypeDefinition { type_definition_id }
    );
    assert_eq!(
        state.pattern_heads.get(owner_a).unwrap().display_name,
        format!("generated-type-definition-{}", type_definition_id.as_u64())
    );
    assert_eq!(
        state.pattern_heads.get(owner_b).unwrap().display_name,
        format!("generated-type-definition-{}", type_definition_id.as_u64())
    );
    assert_eq!(
        type_a.fields[0].pattern_head,
        state.pattern_heads.lookup_child(owner_a, "x")
    );
    assert_eq!(
        type_b.fields[0].pattern_head,
        state.pattern_heads.lookup_child(owner_b, "x")
    );
}

#[test]
fn binding_preserves_pre_attached_provisional_owner_material() {
    let snapshot = NamespaceGraphSnapshot::new();
    let mut state = TypeMaterializationState::default();
    let provisional_symbol_id = SymbolId(900);
    let value = attach_type_definition_pattern_heads_with_context(
        generated_struct_value_for_binding(),
        &mut state,
        PatternMaterializationContext::Global {
            symbol_id: provisional_symbol_id,
        },
        "provisional-owner",
        provenance("attach provisional owner"),
    )
    .expect("provisional attachment succeeds");
    let provisional_owner = owner_head(&value);

    let expansion = bind_meta_invocation_value_result_with_materialization_state(
        lang_build::MetaInvocationValue::GeneratedTypeDefinitionValue(value),
        &snapshot,
        snapshot.root_node(),
        "Destination",
        provenance("bind pre-attached generated type"),
        &mut state,
    )
    .expect("binding preserves provisional material");

    let type_object = type_payload(&expansion.replacement_object);
    let bound_owner = type_object
        .owner_pattern_head
        .expect("binding preserves owner pattern head");
    assert_eq!(bound_owner, provisional_owner);
    assert_ne!(type_object.type_symbol_id, provisional_symbol_id);
    assert_eq!(
        state.pattern_heads.get(bound_owner).unwrap().origin,
        PatternHeadOrigin::GlobalBinding {
            symbol_id: provisional_symbol_id,
        }
    );
    assert_eq!(
        state.pattern_heads.get(bound_owner).unwrap().display_name,
        "provisional-owner"
    );
}
