use std::collections::BTreeSet;

use crate::{
    model::{
        CoreMetaFunction, MetaFunctionObject, NamespaceNode, NamespaceNodeId, NamespaceNodeKind,
        PolicyMetadata, PolicySet, Provenance, SemanticNameDelta, SourceCategory, SymbolId,
        SymbolKind, SymbolObject, SymbolPayload, TypeObject, VerificationPrimitive,
    },
    policy_metadata,
    policy_pair::{
        PatternComponentPolicy, PolicyPair, PolicyStage, StageSet, ValueComponentPolicy,
        ValuePresence,
    },
    policy_set_export_meta, policy_set_export_meta_runtime, policy_set_meta,
    policy_set_meta_runtime,
    semantic_name_index::{namespace_symbol, BuildError, SemanticNameIndex},
};

pub const CORE_NAMESPACE: &str = "core";

/// One declared core callable registration fact.
///
/// The bootstrap produces this roster directly so the semantic world is
/// populated from the declaration itself; the compilation world no longer
/// scans graph `SymbolPayload::MetaFunction` payloads and re-projects them
/// through the flat legacy `PolicySet` carrier.
pub(crate) struct CoreCallableRegistration {
    pub(crate) namespace: NamespaceNodeId,
    pub(crate) name: String,
    pub(crate) backing: SymbolId,
    pub(crate) primitive: CoreMetaFunction,
    pub(crate) function_policy: PolicyPair,
    pub(crate) result_policy: PolicyPair,
    pub(crate) return_shape: crate::ReturnShape,
    pub(crate) visibility: Option<crate::NamespaceVisibility>,
    pub(crate) provenance: Provenance,
}

/// One declared core type registration fact.
///
/// The bootstrap spells the canonical PolicyPair next to the graph payload
/// so the semantic world is populated from the declaration itself; the
/// compilation world no longer rescans graph `SymbolPayload::Type` payloads
/// through the flat legacy `PolicySet` projection
/// (`sync_semantic_type_values` is deleted).
pub(crate) struct CoreTypeRegistration {
    pub(crate) namespace: NamespaceNodeId,
    pub(crate) name: String,
    pub(crate) binding: SymbolId,
    pub(crate) represented_type: crate::TypeValueId,
    pub(crate) associated_namespace: NamespaceNodeId,
    pub(crate) policy: PolicyPair,
    pub(crate) provenance: Provenance,
}

pub(crate) fn install_core_bootstrap(
    snapshot: &SemanticNameIndex,
) -> Result<
    (
        SemanticNameIndex,
        NamespaceNodeId,
        Vec<CoreCallableRegistration>,
        Vec<CoreTypeRegistration>,
    ),
    BuildError,
> {
    let mut delta = snapshot.empty_delta();
    let mut core_callables = Vec::new();
    let mut core_types = Vec::new();
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
        &mut core_callables,
        core_node,
        "struct",
        CoreMetaFunction::Struct,
        Provenance::new("core meta-function `struct`"),
        policy_set_export_meta(),
    );
    insert_meta_function(
        &mut delta,
        &mut core_callables,
        core_node,
        "assert",
        CoreMetaFunction::Assert,
        Provenance::new("core meta-function `assert`"),
        policy_set_export_meta(),
    );
    insert_meta_function(
        &mut delta,
        &mut core_callables,
        core_node,
        "IdentityType",
        CoreMetaFunction::IdentityType,
        Provenance::new("core meta-function `IdentityType`"),
        policy_set_export_meta(),
    );
    insert_meta_function(
        &mut delta,
        &mut core_callables,
        core_node,
        "UnaryConstructionPrototype",
        CoreMetaFunction::UnaryConstructionPrototype,
        Provenance::new("core meta-function `UnaryConstructionPrototype`"),
        policy_set_export_meta(),
    );
    insert_verification_namespace(&mut delta, &mut core_callables, core_node);

    for name in [
        "type",
        "symbol",
        "namespace",
        "uint8",
        "ref",
        "share",
        "integer",
        "real",
        "character",
        "lifetime",
        "uint16",
        "uint32",
        "float32",
    ] {
        insert_core_type(
            &mut delta,
            &mut core_types,
            core_node,
            name,
            Provenance::new(format!("core type symbol `{name}`")),
            policy_set_export_meta_runtime(),
        );
    }

    snapshot
        .install_delta(delta)
        .map(|snapshot| (snapshot, core_node, core_callables, core_types))
        .map_err(BuildError::from)
}

/// Declared canonical PolicyPair coordinate for a core built-in: the value
/// stage set is spelled directly and the Pattern stage set is its static
/// projection.  Core built-ins carry no mutability restriction and are
/// always present.
pub(crate) fn core_declared_pair(stages: &[PolicyStage], _export_root: bool) -> PolicyPair {
    let mut value_stages = StageSet::new();
    let mut pattern_stages = StageSet::new();
    for &stage in stages {
        value_stages.insert(stage);
        if stage.is_static() {
            pattern_stages.insert(stage);
        }
    }
    PolicyPair {
        value: ValueComponentPolicy {
            stages: value_stages,
            mutability: BTreeSet::new(),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: pattern_stages,
        },
    }
}

/// Declared body-entry / return-object planes of one
/// core built-in, spelled at the declaration site.  The invocation spine
/// obtains these planes from the primitive identity instead of reading the
/// graph `SymbolPayload::MetaFunction` payload.
pub(crate) fn core_primitive_callable_planes(
    primitive: CoreMetaFunction,
) -> (PolicyMetadata, PolicyMetadata) {
    let return_policy = match primitive {
        CoreMetaFunction::Struct => policy_set_meta_runtime(),
        CoreMetaFunction::Assert
        | CoreMetaFunction::Verify(_)
        | CoreMetaFunction::IdentityType
        | CoreMetaFunction::UnaryConstructionPrototype => policy_set_meta(),
    };
    (
        policy_metadata(policy_set_meta()),
        policy_metadata(return_policy),
    )
}

fn insert_meta_function(
    delta: &mut SemanticNameDelta,
    core_callables: &mut Vec<CoreCallableRegistration>,
    parent: NamespaceNodeId,
    name: &str,
    primitive: CoreMetaFunction,
    provenance: Provenance,
    policy_set: PolicySet,
) {
    let symbol_id = delta.allocate_symbol_id();
    let (body_entry_policy, return_object_policy) = core_primitive_callable_planes(primitive);
    // Independent declared shape/privilege coordinates for each built-in:
    // `struct` and `identity_type` return complete type values;
    // `assert` / `verify` / `UnaryConstructionPrototype` return a single
    // ordinary value.  All core primitives are privileged built-ins (they
    // may consume raw/meta material); privilege implies nothing about the
    // shape and neither coordinate is re-derived at call time.
    let return_shape = match primitive {
        CoreMetaFunction::Struct | CoreMetaFunction::IdentityType => crate::ReturnShape::SingleType,
        CoreMetaFunction::Assert
        | CoreMetaFunction::Verify(_)
        | CoreMetaFunction::UnaryConstructionPrototype => {
            crate::ReturnShape::SingleVal(crate::PatternConstraint::Unconstrained)
        }
    };
    let mut symbol = SymbolObject::placeholder(
        symbol_id,
        name,
        SymbolKind::MetaFunction,
        SourceCategory::CoreBootstrap,
        Some(parent),
        provenance,
    );
    symbol.policy_metadata.policy_set = policy_set;
    // Declaration-boundary export fact: core callables are declared public by
    // the toolchain package, so external member views retain them and they
    // enter ordinary overload as normal candidates (no call-time bypass).
    symbol.visibility_metadata.namespace_visibility = Some(crate::NamespaceVisibility::Public);
    symbol.visibility_metadata.export_root = true;
    symbol.payload = SymbolPayload::MetaFunction(MetaFunctionObject {
        function_symbol_id: symbol_id,
        primitive: Some(primitive),
        source_callable: None,
        function_policy: policy_metadata(symbol.policy_metadata.policy_set.clone()),
        body_entry_policy,
        return_object_policy,
        return_shape,
        privilege: crate::CallablePrivilege::BuiltinPrivileged,
    });
    // Declared semantic registration fact, spelled once next to the graph
    // payload. `struct` exposes the completed type value at meta and runtime
    // while its independent body-entry plane remains meta-only; execution
    // authority is never inferred from the result view.
    core_callables.push(CoreCallableRegistration {
        namespace: parent,
        name: name.to_string(),
        backing: symbol_id,
        primitive,
        function_policy: match primitive {
            CoreMetaFunction::Struct => {
                core_declared_pair(&[PolicyStage::Meta, PolicyStage::Runtime], true)
            }
            _ => core_declared_pair(&[PolicyStage::Meta], true),
        },
        result_policy: match primitive {
            CoreMetaFunction::Struct => {
                core_declared_pair(&[PolicyStage::Meta, PolicyStage::Runtime], false)
            }
            CoreMetaFunction::Assert
            | CoreMetaFunction::Verify(_)
            | CoreMetaFunction::IdentityType
            | CoreMetaFunction::UnaryConstructionPrototype => {
                core_declared_pair(&[PolicyStage::Meta], false)
            }
        },
        return_shape,
        visibility: Some(crate::NamespaceVisibility::Public),
        provenance: symbol.provenance.clone(),
    });
    delta.insert_symbol(parent, symbol);
}

fn insert_verification_namespace(
    delta: &mut SemanticNameDelta,
    core_callables: &mut Vec<CoreCallableRegistration>,
    core_node: NamespaceNodeId,
) {
    let node_id = delta.allocate_node_id();
    let symbol_id = delta.allocate_symbol_id();
    let provenance = Provenance::new("core verification namespace `verify`");
    delta.insert_node(NamespaceNode::new(
        node_id,
        "verify",
        NamespaceNodeKind::Declared,
        SourceCategory::CoreBootstrap,
        Some(core_node),
        provenance.clone(),
    ));

    let mut symbol = SymbolObject::namespace(
        symbol_id,
        "verify",
        node_id,
        NamespaceNodeKind::Declared,
        SourceCategory::CoreBootstrap,
        Some(core_node),
        provenance,
    );
    symbol.policy_metadata.policy_set = policy_set_export_meta();
    symbol.payload = SymbolPayload::VerificationNamespace { node: node_id };
    delta.insert_symbol(core_node, symbol);

    for (name, primitive) in [
        ("exists", VerificationPrimitive::Exists),
        ("not_exists", VerificationPrimitive::NotExists),
        ("resolves_as", VerificationPrimitive::ResolvesAs),
        ("not_resolves", VerificationPrimitive::NotResolves),
        ("kind", VerificationPrimitive::Kind),
        ("namespace_kind", VerificationPrimitive::NamespaceKind),
        ("field_names", VerificationPrimitive::FieldNames),
        ("has_field", VerificationPrimitive::HasField),
        ("field_projection", VerificationPrimitive::FieldProjection),
        ("field_owner", VerificationPrimitive::FieldOwner),
        ("field_type", VerificationPrimitive::FieldType),
        ("policy", VerificationPrimitive::Policy),
        ("not_policy", VerificationPrimitive::NotPolicy),
        ("body_entry_policy", VerificationPrimitive::BodyEntryPolicy),
        (
            "not_body_entry_policy",
            VerificationPrimitive::NotBodyEntryPolicy,
        ),
        ("return_policy", VerificationPrimitive::ReturnPolicy),
        ("not_return_policy", VerificationPrimitive::NotReturnPolicy),
    ] {
        insert_meta_function(
            delta,
            core_callables,
            node_id,
            name,
            CoreMetaFunction::Verify(primitive),
            Provenance::new(format!("core verification operation `verify::{name}`")),
            policy_set_export_meta(),
        );
    }
}

pub(crate) fn insert_core_type(
    delta: &mut SemanticNameDelta,
    core_types: &mut Vec<CoreTypeRegistration>,
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
    // Declaration-boundary export fact, mirroring `insert_meta_function`:
    // core type symbols are public members of the toolchain package.
    symbol.visibility_metadata.namespace_visibility = Some(crate::NamespaceVisibility::Public);
    symbol.visibility_metadata.export_root = true;
    symbol.node_kind = Some(NamespaceNodeKind::Virtual);
    // Core lookup indices come from a type registry namespace disjoint from
    // graph Symbol allocation.  Their opaque representation is provisional;
    // the only hard property here is that no SymbolId conversion defines type
    // identity.
    let represented_type = crate::TypeValueId((1u64 << 62) | core_types.len() as u64);
    // Declared semantic registration fact: core type carriers are declared
    // `export meta runtime`, spelled as the canonical pair directly.
    core_types.push(CoreTypeRegistration {
        namespace: parent,
        name: name.to_string(),
        binding: symbol_id,
        represented_type,
        associated_namespace: associated_node,
        policy: core_declared_pair(&[PolicyStage::Meta, PolicyStage::Runtime], true),
        provenance: provenance.clone(),
    });
    symbol.payload = SymbolPayload::Type(TypeObject {
        carrier_symbol_id: symbol_id,
        represented_type,
        owner_pattern_head: None,
        fields: Vec::new(),
        field_names: Vec::new(),
        field_type_values: Vec::new(),
        field_type_symbol_ids: Vec::new(),
        type_associated_namespace: Some(associated_node),
        extraction_interface: None,
        provenance,
        generation_origin: Some("core bootstrap".to_string()),
        layout_slot: None,
        abi_slot: None,
    });
    delta.insert_symbol(parent, symbol);
}
