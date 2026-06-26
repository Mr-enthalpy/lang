use crate::{
    graph::{namespace_symbol, BuildError, NamespaceGraphSnapshot},
    model::{
        CoreMetaFunction, MetaFunctionObject, NamespaceDelta, NamespaceNode, NamespaceNodeId,
        NamespaceNodeKind, PolicyMetadata, PolicySet, Provenance, SourceCategory, SymbolKind,
        SymbolObject, SymbolPayload, TypeObject,
    },
    policy_set_export_meta, policy_set_export_meta_runtime,
};

pub const CORE_NAMESPACE: &str = "core";

pub fn install_core_bootstrap(
    snapshot: &NamespaceGraphSnapshot,
) -> Result<(NamespaceGraphSnapshot, NamespaceNodeId), BuildError> {
    let mut delta = snapshot.empty_delta();
    let core_provenance = Provenance::new("compiler-seeded core package");
    let core_node = namespace_symbol(
        &mut delta,
        snapshot.root_node(),
        CORE_NAMESPACE,
        NamespaceNodeKind::Declared,
        SourceCategory::CoreBootstrap,
        core_provenance,
    );

    for symbol in delta.symbols.values_mut() {
        if symbol.kind == SymbolKind::Namespace && symbol.name == CORE_NAMESPACE {
            symbol.policy_metadata.policy_set = policy_set_export_meta_runtime();
        }
    }

    insert_meta_function(
        &mut delta,
        core_node,
        "struct",
        CoreMetaFunction::Struct,
        Provenance::new("core meta-function `struct`"),
        policy_set_export_meta(),
    );
    insert_meta_function(
        &mut delta,
        core_node,
        "assert",
        CoreMetaFunction::Assert,
        Provenance::new("core meta-function `assert`"),
        policy_set_export_meta(),
    );

    for name in [
        "type",
        "namespace",
        "uint8",
        "ref",
        "share",
        "uint16",
        "uint32",
        "float32",
    ] {
        insert_core_type(
            &mut delta,
            core_node,
            name,
            Provenance::new(format!("core type symbol `{name}`")),
            policy_set_export_meta_runtime(),
        );
    }

    snapshot
        .install_delta(delta)
        .map(|snapshot| (snapshot, core_node))
        .map_err(BuildError::from)
}

fn insert_meta_function(
    delta: &mut NamespaceDelta,
    parent: NamespaceNodeId,
    name: &str,
    primitive: CoreMetaFunction,
    provenance: Provenance,
    policy_set: PolicySet,
) {
    let symbol_id = delta.allocate_symbol_id();
    let mut symbol = SymbolObject::placeholder(
        symbol_id,
        name,
        SymbolKind::MetaFunction,
        SourceCategory::CoreBootstrap,
        Some(parent),
        provenance,
    );
    symbol.policy_metadata.policy_set = policy_set;
    symbol.payload = SymbolPayload::MetaFunction(MetaFunctionObject {
        function_symbol_id: symbol_id,
        primitive,
        function_policy: PolicyMetadata::default(),
        body_entry_policy: PolicyMetadata::default(),
        return_object_policy: PolicyMetadata::default(),
    });
    delta.insert_symbol(parent, symbol);
}

pub(crate) fn insert_core_type(
    delta: &mut NamespaceDelta,
    parent: NamespaceNodeId,
    name: &str,
    provenance: Provenance,
    policy_set: PolicySet,
) {
    let symbol_id = delta.allocate_symbol_id();
    let associated_node = delta.allocate_node_id();
    delta.insert_node(NamespaceNode::new(
        associated_node,
        format!("{name}<type-associated>"),
        NamespaceNodeKind::Virtual,
        SourceCategory::TypeAssociatedNamespace,
        Some(parent),
        provenance.clone(),
    ));

    let mut symbol = SymbolObject::placeholder(
        symbol_id,
        name,
        SymbolKind::Type,
        SourceCategory::CoreBootstrap,
        Some(parent),
        provenance.clone(),
    );
    symbol.policy_metadata.policy_set = policy_set;
    symbol.node_kind = Some(NamespaceNodeKind::Virtual);
    symbol.payload = SymbolPayload::Type(TypeObject {
        type_symbol_id: symbol_id,
        fields: Vec::new(),
        field_names: Vec::new(),
        field_type_symbol_ids: Vec::new(),
        type_associated_namespace: Some(associated_node),
        provenance,
        generation_origin: Some("core bootstrap".to_string()),
        layout_slot: None,
        abi_slot: None,
    });
    delta.insert_symbol(parent, symbol);
}
