use lang_syntax::{NormExpr, NormProduct, NormProductElem};

use crate::{
    extraction_view::{NamedExtractionField, NamedProductExtractionShape, TypeExtractionInterface},
    meta_candidate::{
        prepare_meta_callable_candidate_with_declared_planes, CallableCandidateKind,
        CandidateBuildIdentityPlaceholder, CandidatePrepDeferredReason, CandidatePrepResult,
        CandidatePreparationContext, ParameterShape,
    },
    meta_invocation::{
        attach_type_definition_pattern_heads, compute_type_definition_instance_id,
        GeneratedFieldDefinition, GeneratedTypeDefinitionValue, MetaInvocationInput,
        MetaInvocationValue, ReturnSlotSemantics,
    },
    model::{
        CallablePolicyMetadata, CoreMetaFunction, Diagnostic, ExecutionEnv, FieldObject,
        FieldProjection, NamespaceNode, NamespaceNodeId, NamespaceNodeKind, PolicyEnv, Provenance,
        SemanticNameDelta, SourceCategory, SymbolId, SymbolKind, SymbolObject, SymbolPayload,
        TypeField, TypeObject,
    },
    normalized_call::NormalizedCallSite,
    pattern_head::TypeMaterializationState,
    pattern_space::{StructLeafTypeExprShape, StructuralMemberVisibility, TypePatternExprShape},
    policy_metadata,
    policy_pair::NamespaceVisibility,
    policy_set_meta_runtime, policy_set_runtime,
    product_shape::{
        ArgProductShape, FlattenedProductInvariant, FlattenedProductObject, ProductAtom,
        ProductMaterialRole,
    },
    semantic_name_index::{BuildError, ResolverContext, SemanticNameIndex},
    type_argument::{classify_type_arguments_env_with_report, TypeResolutionEnv},
};

/// Result of a successful early meta expansion.
#[derive(Clone, Debug)]
pub struct MetaExpansionResult {
    pub replacement_object: SymbolObject,
    pub namespace_delta: SemanticNameDelta,
    pub diagnostics: Vec<Diagnostic>,
    pub provenance: Provenance,
}

/// Primitive-explicit variant for the canonical A-stage.
///
/// The core primitive identity comes from the semantic
/// `OrdinaryCallEntry.core_primitive`, so the invocation spine never reads
/// the graph `SymbolPayload::MetaFunction` to enter a core body.
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_resolved_core_meta_call_with_primitive(
    callee: &SymbolObject,
    primitive: CoreMetaFunction,
    site: &NormalizedCallSite,
    type_env: &dyn TypeResolutionEnv,
    resolver_context: &ResolverContext,
    lookup_env: PolicyEnv,
    demanded_execution: ExecutionEnv,
    build_identity: CandidateBuildIdentityPlaceholder,
    provenance: Provenance,
) -> Result<MetaInvocationInput, BuildError> {
    let primitive_name = match primitive {
        CoreMetaFunction::Struct => "struct",
        CoreMetaFunction::Assert => "assert",
        CoreMetaFunction::Verify(_) => "verify",
        CoreMetaFunction::IdentityType => "IdentityType",
        CoreMetaFunction::UnaryConstructionPrototype => "UnaryConstructionPrototype",
    };

    let arg_product_shape =
        site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let mut unresolved_type_names = Vec::new();
    let mut struct_decoded_pattern: Option<crate::struct_decoder::DecodedStructPattern> = None;
    let (classified_shape, parameter_shape) = match primitive {
        CoreMetaFunction::IdentityType | CoreMetaFunction::UnaryConstructionPrototype => {
            let report = classify_type_arguments_env_with_report(
                &arg_product_shape,
                type_env,
                resolver_context,
            );
            unresolved_type_names = report.unresolved_names;
            (
                report.classified_shape,
                ParameterShape::type_parameter_signature(Provenance::new(format!(
                    "{primitive_name} : type -> type signature"
                ))),
            )
        }
        CoreMetaFunction::Struct => {
            validate_struct_source_product(&site.source_product)?;
            let source_arg = NormExpr::Product(site.source_product.clone());
            let decoded_shape = crate::struct_decoder::decode_struct_type_pattern_expr(
                &source_arg,
                provenance.clone(),
            )
            .map_err(BuildError::single)?;
            let classified_shape = classify_decoded_struct_field_arguments(
                type_env,
                &decoded_shape,
                resolver_context,
                provenance.clone(),
            )?;
            struct_decoded_pattern = Some(crate::struct_decoder::DecodedStructPattern::new(
                decoded_shape,
                provenance.clone(),
            ));
            (
                classified_shape.clone(),
                ParameterShape::type_parameter_sequence(
                    classified_shape.arity,
                    Provenance::new("struct field type signature"),
                ),
            )
        }
        CoreMetaFunction::Assert => {
            return Err(BuildError::single(Diagnostic::hard_error(
                "meta hard error: direct source-level `assert` expansion is not implemented",
                Some(provenance),
            )));
        }
        CoreMetaFunction::Verify(_) => {
            return Err(BuildError::single(Diagnostic::hard_error(
                "meta hard error: source verification operations cannot be used as initializers",
                Some(provenance),
            )));
        }
    };

    // The core body-entry / return-object planes come
    // from the primitive's declared facts, not from re-reading the graph
    // `SymbolPayload::MetaFunction` payload on the invocation spine.
    let (body_entry_policy, return_object_policy) =
        crate::core::core_primitive_callable_planes(primitive);
    let candidate = match prepare_meta_callable_candidate_with_declared_planes(
        callee,
        CallableCandidateKind::MetaFunction,
        Some(primitive),
        body_entry_policy,
        return_object_policy,
        classified_shape,
        parameter_shape,
        CandidatePreparationContext {
            lookup_env,
            demanded_execution,
            build_identity,
            provenance: provenance.clone(),
        },
    ) {
        CandidatePrepResult::ApplicablePlaceholder(candidate) => *candidate,
        CandidatePrepResult::Deferred { reason, .. } => {
            let message = match reason {
                CandidatePrepDeferredReason::BodyEntryPolicyMismatch => {
                    "candidate preparation deferred because body-entry policy is not meta-executable"
                }
                CandidatePrepDeferredReason::ParameterShapeCompatibilityDeferred => {
                    "candidate preparation deferred because parameter shape compatibility is incomplete"
                }
            };
            return Err(BuildError::single(Diagnostic::hard_error(
                message,
                Some(provenance),
            )));
        }
        CandidatePrepResult::Diagnostic(diagnostic) => {
            if !unresolved_type_names.is_empty() {
                let names = unresolved_type_names.join(", ");
                return Err(BuildError::single(Diagnostic::hard_error(
                    format!(
                        "meta hard error: {primitive_name} argument `{names}` could not be resolved as a type object"
                    ),
                    Some(provenance),
                )));
            }
            return Err(BuildError::single(diagnostic));
        }
    };
    let mut invocation_input = MetaInvocationInput::new(candidate, provenance);
    invocation_input.struct_decoded_pattern = struct_decoded_pattern;
    Ok(invocation_input)
}

pub fn compile_time_assert(
    condition: bool,
    provenance: Provenance,
    message: impl Into<String>,
) -> Result<(), Diagnostic> {
    if condition {
        Ok(())
    } else {
        Err(Diagnostic::hard_error(
            format!("meta hard error: {}", message.into()),
            Some(provenance),
        ))
    }
}

/// Classify the actual structural leaves produced by the struct decoder.
///
/// A named Pattern such as `((uint8 inner) t)` has one field leaf (`inner`)
/// under the top Pattern name (`t`).  Candidate preparation must therefore
/// consume that decoded leaf, not the invocation Product atom containing the
/// whole named Pattern expression.
fn classify_decoded_struct_field_arguments(
    type_env: &dyn TypeResolutionEnv,
    pattern: &TypePatternExprShape,
    context: &ResolverContext,
    provenance: Provenance,
) -> Result<ArgProductShape, BuildError> {
    let mut leaves = Vec::new();
    collect_decoded_struct_leaves(pattern, &mut leaves);

    let mut atoms = Vec::with_capacity(leaves.len());
    let mut resolved = Vec::with_capacity(leaves.len());
    let mut diagnostics = Vec::new();
    for (external_type_expr, field_name, field_provenance) in leaves {
        atoms.push(ProductAtom::Unsupported {
            summary: format!("decoded struct field `{field_name}`"),
            provenance: field_provenance.clone(),
        });
        let StructLeafTypeExprShape::Path(path) = external_type_expr else {
            diagnostics.push(Diagnostic::hard_error(
                format!(
                    "invalid struct syntax: unsupported type expression for struct field `{field_name}`"
                ),
                Some(field_provenance.clone()),
            ));
            continue;
        };
        match type_env.resolve_field_type_path(&path.segments, context, &field_provenance) {
            Ok(identity) => resolved.push(identity),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    if !diagnostics.is_empty() {
        return Err(BuildError { diagnostics });
    }

    let flattened = FlattenedProductObject {
        atoms,
        provenance: provenance.clone(),
        invariant: FlattenedProductInvariant {
            no_direct_product_atom_remains: true,
        },
    };
    let mut shape = ArgProductShape::from_flattened(flattened);
    debug_assert_eq!(shape.raw_args.len(), resolved.len());
    for (raw_arg, (carrier_symbol, represented_type)) in shape.raw_args.iter_mut().zip(resolved) {
        *raw_arg = raw_arg
            .clone()
            .as_type_object_with_identity(carrier_symbol, represented_type);
    }
    Ok(shape)
}

fn collect_decoded_struct_leaves<'a>(
    pattern: &'a TypePatternExprShape,
    output: &mut Vec<(&'a StructLeafTypeExprShape, &'a str, &'a Provenance)>,
) {
    match pattern {
        TypePatternExprShape::Leaf {
            external_type_expr,
            local_pattern_name,
            provenance,
            ..
        } => output.push((external_type_expr, local_pattern_name.as_str(), provenance)),
        TypePatternExprShape::Product { elements, .. } => {
            for element in elements {
                collect_decoded_struct_leaves(element, output);
            }
        }
        TypePatternExprShape::Sum { alternatives, .. } => {
            for alternative in alternatives {
                collect_decoded_struct_leaves(alternative, output);
            }
        }
        TypePatternExprShape::Named { child, .. } => {
            collect_decoded_struct_leaves(child, output);
        }
    }
}

fn validate_struct_source_product(product: &NormProduct) -> Result<(), BuildError> {
    let mut diagnostics = Vec::new();
    for element in &product.elements {
        match element {
            NormProductElem::Expr(NormExpr::Product(nested)) => {
                diagnostics.push(Diagnostic::hard_error(
                    "invalid struct syntax: nested product fields are not supported in v0.8",
                    Some(Provenance::from_norm_origin(
                        "nested struct field product",
                        &nested.origin,
                    )),
                ));
            }
            NormProductElem::Unit { origin } => {
                diagnostics.push(Diagnostic::hard_error(
                    "invalid struct syntax: unit field or trailing unit is not supported",
                    Some(Provenance::from_norm_origin("unit struct field", origin)),
                ));
            }
            NormProductElem::Expr(_) => {}
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(BuildError { diagnostics })
    }
}

fn insert_projection_namespace(
    delta: &mut SemanticNameDelta,
    parent: NamespaceNodeId,
    name: &str,
    owner_type_symbol_id: SymbolId,
    owner_pattern_head: Option<crate::pattern_head::PatternHeadId>,
    fields: &[GeneratedFieldDefinition],
    projection: FieldProjection,
    provenance: Provenance,
) {
    let node_id = delta.allocate_node_id();
    let symbol_id = delta.allocate_symbol_id();
    delta.insert_node(NamespaceNode::new(
        node_id,
        name,
        NamespaceNodeKind::Virtual,
        SourceCategory::MetaInstantiationVirtualLayer,
        Some(parent),
        provenance.clone(),
    ));
    let mut namespace_symbol = SymbolObject::namespace(
        symbol_id,
        name,
        node_id,
        NamespaceNodeKind::Virtual,
        SourceCategory::MetaInstantiationVirtualLayer,
        Some(parent),
        provenance,
    );
    namespace_symbol.policy_metadata.policy_set = policy_set_meta_runtime();
    delta.insert_symbol(parent, namespace_symbol);
    insert_field_projection_layer(
        delta,
        node_id,
        owner_type_symbol_id,
        owner_pattern_head,
        fields,
        projection,
        None,
    );
}

fn insert_field_projection_layer(
    delta: &mut SemanticNameDelta,
    parent: NamespaceNodeId,
    owner_type_symbol_id: SymbolId,
    owner_pattern_head: Option<crate::pattern_head::PatternHeadId>,
    fields: &[GeneratedFieldDefinition],
    projection: FieldProjection,
    forced_provenance: Option<Provenance>,
) {
    for field in fields {
        let symbol_id = delta.allocate_symbol_id();
        let provenance = forced_provenance
            .clone()
            .unwrap_or_else(|| field.provenance.clone());
        let mut symbol = SymbolObject::placeholder(
            symbol_id,
            &field.name,
            SymbolKind::FieldFunction,
            SourceCategory::GeneratedChild,
            Some(parent),
            provenance.clone(),
        );
        symbol.policy_metadata.policy_set = policy_set_meta_runtime();
        symbol.visibility_metadata.namespace_visibility = Some(match field.visibility {
            StructuralMemberVisibility::Default | StructuralMemberVisibility::Public => {
                NamespaceVisibility::Public
            }
            StructuralMemberVisibility::Private => NamespaceVisibility::Private,
        });
        symbol.generation_origin = Some("core::struct field projection".to_string());
        symbol.cache_key_fragment = Some(format!(
            "field:{}:{}:{projection:?}",
            owner_type_symbol_id.as_u64(),
            field.name
        ));
        symbol.payload = SymbolPayload::FieldFunction(FieldObject {
            owner_type_symbol_id,
            owner_pattern_head,
            field_name: field.name.clone(),
            field_type_value: field.type_value,
            field_type_symbol_id: field.type_carrier_symbol,
            field_pattern_head: field.pattern_head,
            projection,
            callable_policy: CallablePolicyMetadata {
                body_entry_policy: policy_metadata(policy_set_runtime()),
                return_object_policy: policy_metadata(policy_set_runtime()),
            },
            provenance,
        });
        delta.insert_symbol(parent, symbol);
    }
}

/// Bind a meta invocation value into a declaration expansion.
///
/// This is the formal binding entry point. It dispatches on the invocation
/// value type:
///
/// - `ForwardedValue` with `TypeValue`: materializes a fresh declaration
///   that binds the returned type value.
/// - `GeneratedConstructionValue`: materialized by `bind_generated_construction_value`.
/// - `GeneratedTypeDefinitionValue`: materialized by `bind_generated_type_definition_value`.
///
/// Compatibility helper only. This creates a temporary
/// `TypeMaterializationState`, so it is not suitable for registry-backed world
/// binding of generated type definitions. Callers that install generated type
/// definitions into a `CompilationWorld` must use
/// `bind_meta_invocation_value_result_with_materialization_state` so the
/// world-owned `PatternHeadRegistry` remains authoritative.
pub fn bind_meta_invocation_value_result(
    value: MetaInvocationValue,
    snapshot: &SemanticNameIndex,
    parent_namespace: NamespaceNodeId,
    binding_name: &str,
    provenance: Provenance,
) -> Result<MetaExpansionResult, BuildError> {
    let mut materialization_state = TypeMaterializationState::default();
    bind_meta_invocation_value_result_with_materialization_state(
        value,
        snapshot,
        parent_namespace,
        binding_name,
        provenance,
        &mut materialization_state,
    )
}

pub fn bind_meta_invocation_value_result_with_materialization_state(
    value: MetaInvocationValue,
    snapshot: &SemanticNameIndex,
    parent_namespace: NamespaceNodeId,
    binding_name: &str,
    provenance: Provenance,
    materialization_state: &mut TypeMaterializationState,
) -> Result<MetaExpansionResult, BuildError> {
    match value {
        MetaInvocationValue::ForwardedValue(fv) => {
            let represented_type = fv.type_value;
            let mut delta = snapshot.empty_delta();
            let declared_id = delta.allocate_symbol_id();
            let type_namespace_id = delta.allocate_node_id();
            delta.insert_node(NamespaceNode {
                id: type_namespace_id,
                name: format!("{binding_name}<type-associated>"),
                kind: NamespaceNodeKind::Virtual,
                source_category: SourceCategory::DeclaredSymbol,
                parent: Some(parent_namespace),
                children: std::collections::BTreeMap::new(),
                policy_metadata: crate::policy_metadata(crate::policy_set_meta_runtime()),
                visibility_metadata: crate::model::VisibilityMetadata {
                    slots: std::collections::BTreeMap::new(),
                    ..crate::model::VisibilityMetadata::default()
                },
                provenance: provenance.clone(),
                diagnostics: Vec::new(),
            });
            let declared_symbol = SymbolObject {
                id: declared_id,
                kind: SymbolKind::Type,
                name: binding_name.to_string(),
                source_category: SourceCategory::DeclaredSymbol,
                node_kind: Some(NamespaceNodeKind::Virtual),
                parent: Some(parent_namespace),
                policy_metadata: crate::policy_metadata(crate::policy_set_meta_runtime()),
                visibility_metadata: crate::model::VisibilityMetadata {
                    slots: std::collections::BTreeMap::new(),
                    ..crate::model::VisibilityMetadata::default()
                },
                diagnostics: Vec::new(),
                generation_origin: Some("ForwardedValue(TypeValue) binding".to_string()),
                cache_key_fragment: None,
                provenance: Provenance::new(format!("declared forwarding type `{binding_name}`")),
                payload: SymbolPayload::Type(TypeObject {
                    carrier_symbol_id: declared_id,
                    represented_type,
                    owner_pattern_head: None,
                    fields: Vec::new(),
                    field_names: Vec::new(),
                    field_type_values: Vec::new(),
                    field_type_symbol_ids: Vec::new(),
                    type_associated_namespace: Some(type_namespace_id),
                    extraction_interface: None,
                    provenance: Provenance::new(format!(
                        "type-value binding `{binding_name}` from TypeValue({})",
                        represented_type.0
                    )),
                    generation_origin: Some("ForwardedValue(TypeValue) adapter".to_string()),
                    layout_slot: None,
                    abi_slot: None,
                }),
            };
            delta.insert_symbol(parent_namespace, declared_symbol.clone());
            Ok(MetaExpansionResult {
                replacement_object: declared_symbol,
                namespace_delta: delta,
                diagnostics: Vec::new(),
                provenance,
            })
        }
        MetaInvocationValue::GeneratedConstructionValue(gcv) => bind_generated_construction_value(
            &gcv,
            snapshot,
            parent_namespace,
            binding_name,
            provenance,
        ),
        MetaInvocationValue::GeneratedTypeDefinitionValue(gtdv) => {
            bind_generated_type_definition_value(
                gtdv,
                snapshot,
                parent_namespace,
                binding_name,
                provenance,
                materialization_state,
            )
        }
    }
}

fn bind_generated_type_definition_value(
    value: GeneratedTypeDefinitionValue,
    snapshot: &SemanticNameIndex,
    parent_namespace: NamespaceNodeId,
    binding_name: &str,
    provenance: Provenance,
    materialization_state: &mut TypeMaterializationState,
) -> Result<MetaExpansionResult, BuildError> {
    let expected = compute_type_definition_instance_id(&value.identity_material);
    if expected != value.type_definition_id {
        return Err(BuildError::single(Diagnostic::hard_error(
            format!(
                "meta hard error: GeneratedTypeDefinitionValue has mismatched TypeDefinitionInstanceId (expected {}, got {})",
                expected.as_u64(),
                value.type_definition_id.as_u64()
            ),
            Some(value.provenance.clone()),
        )));
    }
    if value.identity_material.return_slot_semantics != ReturnSlotSemantics::Generate {
        return Err(BuildError::single(Diagnostic::hard_error(
            "meta hard error: GeneratedTypeDefinitionValue must have Generate return-slot semantics",
            Some(value.provenance.clone()),
        )));
    }

    let mut delta = snapshot.empty_delta();
    let type_symbol_id = delta.allocate_symbol_id();
    // The namespace projection represents the canonical meta-type root when
    // the invocation owner registered one (`TypeValue = (OuterMetaInstance
    // Root, NormalizedStructBody)`); the raw definition-id projection is a
    // standalone-binding fallback for unregistered expansion, never a root
    // shared across meta functions.
    // Migration-only lookup material for the legacy private meta result.
    // This is deliberately domain-separated from both TypeDefinition identity
    // and Symbol identity.  Whole semantic type identity is the complete tau
    // observation installed by SemanticWorld, never this lookup key.
    let represented_type = value.canonical_type.unwrap_or(crate::TypeValueId(
        value.type_definition_id.0 ^ 0x6f2d_79b9_a341_c8d5,
    ));
    let type_namespace_id = delta.allocate_node_id();
    let value = if value.pattern_heads.is_some() {
        value
    } else {
        attach_type_definition_pattern_heads(value, materialization_state, provenance.clone())
            .map_err(BuildError::single)?
    };
    delta.insert_node(NamespaceNode::new(
        type_namespace_id,
        format!("{binding_name}<type-associated>"),
        NamespaceNodeKind::Virtual,
        SourceCategory::TypeAssociatedNamespace,
        Some(parent_namespace),
        provenance.clone(),
    ));

    let type_definition_fragment = format!("type-definition:{}", value.type_definition_id.as_u64());
    let mut type_object = SymbolObject::placeholder(
        type_symbol_id,
        binding_name,
        SymbolKind::Type,
        SourceCategory::DeclaredSymbol,
        Some(parent_namespace),
        provenance.clone(),
    );
    type_object.policy_metadata.policy_set = policy_set_meta_runtime();
    type_object.node_kind = Some(NamespaceNodeKind::Virtual);
    type_object.generation_origin = Some("core::struct generated type definition".to_string());
    // cache_key_fragment is a temporary carrier;
    // TypeDefinitionInstanceId is the semantic identity.
    type_object.cache_key_fragment = Some(type_definition_fragment.clone());
    type_object.payload = SymbolPayload::Type(TypeObject {
        carrier_symbol_id: type_symbol_id,
        represented_type,
        owner_pattern_head: value.pattern_heads.as_ref().map(|heads| heads.owner_head),
        fields: value
            .fields
            .iter()
            .map(|field| TypeField {
                name: field.name.clone(),
                type_value: field.type_value,
                type_symbol_id: field.type_carrier_symbol,
                pattern_head: field.pattern_head,
                visibility: field.visibility,
                provenance: field.provenance.clone(),
            })
            .collect(),
        field_names: value
            .fields
            .iter()
            .map(|field| field.name.clone())
            .collect(),
        field_type_values: value.fields.iter().map(|field| field.type_value).collect(),
        field_type_symbol_ids: value
            .fields
            .iter()
            .map(|field| field.type_carrier_symbol)
            .collect(),
        type_associated_namespace: Some(type_namespace_id),
        extraction_interface: Some(generated_type_extraction_interface(
            type_symbol_id,
            represented_type,
            value.pattern_heads.as_ref().map(|heads| heads.owner_head),
            &value.fields,
            provenance.clone(),
        )),
        provenance: provenance.clone(),
        generation_origin: Some(format!(
            "core::struct generated type definition {}",
            value.type_definition_id.as_u64()
        )),
        layout_slot: None,
        abi_slot: None,
    });

    delta.insert_symbol(parent_namespace, type_object.clone());
    insert_field_projection_layer(
        &mut delta,
        type_namespace_id,
        type_symbol_id,
        value.pattern_heads.as_ref().map(|heads| heads.owner_head),
        &value.fields,
        FieldProjection::Value,
        None,
    );
    insert_projection_namespace(
        &mut delta,
        type_namespace_id,
        "ref",
        type_symbol_id,
        value.pattern_heads.as_ref().map(|heads| heads.owner_head),
        &value.fields,
        FieldProjection::Ref,
        provenance.clone(),
    );
    insert_projection_namespace(
        &mut delta,
        type_namespace_id,
        "share",
        type_symbol_id,
        value.pattern_heads.as_ref().map(|heads| heads.owner_head),
        &value.fields,
        FieldProjection::Share,
        provenance.clone(),
    );

    Ok(MetaExpansionResult {
        replacement_object: type_object,
        namespace_delta: delta,
        diagnostics: Vec::new(),
        provenance,
    })
}

fn generated_type_extraction_interface(
    owner_type_symbol_id: SymbolId,
    owner_type_value: crate::TypeValueId,
    owner_pattern_head: Option<crate::pattern_head::PatternHeadId>,
    fields: &[GeneratedFieldDefinition],
    provenance: Provenance,
) -> TypeExtractionInterface {
    TypeExtractionInterface {
        owner_type_value,
        owner_type_symbol_id,
        owner_pattern_head,
        exposed_view: NamedProductExtractionShape {
            owner_type_value,
            owner_type_symbol_id,
            owner_pattern_head,
            fields: fields
                .iter()
                .filter(|field| field.visibility != StructuralMemberVisibility::Private)
                .map(|field| NamedExtractionField {
                    label: field.name.clone(),
                    field_type_value: field.type_value,
                    field_type_observation: field.type_observation,
                    field_type_symbol_id: field.type_carrier_symbol,
                    field_pattern_head: field.pattern_head,
                    field_index: field.index,
                    projection: FieldProjection::Value,
                    visibility: field.visibility,
                    provenance: field.provenance.clone(),
                })
                .collect(),
            provenance: provenance.clone(),
        },
        provenance,
    }
}

/// Bind a `GeneratedConstructionValue` into the namespace graph.
///
/// Creates a declared type symbol under `binding_name`. The `SymbolObject`
/// carries the `construction_instance_id` as a `cache_key_fragment`
/// (temporary carrier — the identity model is `ConstructionInstanceId`,
/// not the cache key).
///
/// The `TypeObject` payload attached here is a binding projection of the
/// `GeneratedConstructionValue`, not a carrier-derived identity. The
/// construction result already supplies semantic construction identity;
/// binding materializes its TypeValue projection under a fresh carrier.
///
/// The declared TypeObject's `carrier_symbol_id` is a fresh `SymbolId`; neither
/// the construction identity nor its TypeValue is derived from that carrier.
///
/// This function installs a `NamespaceDelta`. It is the binding layer —
/// `invoke_meta_callable` remains pure.
fn bind_generated_construction_value(
    gcv: &crate::meta_invocation::GeneratedConstructionValue,
    snapshot: &SemanticNameIndex,
    parent_namespace: NamespaceNodeId,
    binding_name: &str,
    provenance: Provenance,
) -> Result<MetaExpansionResult, BuildError> {
    let expected = crate::meta_invocation::compute_construction_instance_id(&gcv.identity_material);
    if expected != gcv.construction_instance_id {
        return Err(BuildError::single(Diagnostic::hard_error(
            format!(
                "meta hard error: GeneratedConstructionValue has mismatched construction_instance_id (expected {}, got {})",
                expected.as_u64(),
                gcv.construction_instance_id.as_u64()
            ),
            Some(gcv.provenance.clone()),
        )));
    }
    if gcv.identity_material.return_slot_semantics
        != crate::meta_invocation::ReturnSlotSemantics::Generate
    {
        return Err(BuildError::single(Diagnostic::hard_error(
            "meta hard error: GeneratedConstructionValue must have Generate return-slot semantics",
            Some(gcv.provenance.clone()),
        )));
    }

    let mut delta = snapshot.empty_delta();
    let declared_id = delta.allocate_symbol_id();
    let declared_symbol = SymbolObject {
        id: declared_id,
        kind: SymbolKind::Type,
        name: binding_name.to_string(),
        source_category: SourceCategory::DeclaredSymbol,
        node_kind: None,
        parent: Some(parent_namespace),
        policy_metadata: crate::policy_metadata(policy_set_meta_runtime()),
        visibility_metadata: crate::model::VisibilityMetadata {
            slots: std::collections::BTreeMap::new(),
            ..crate::model::VisibilityMetadata::default()
        },
        diagnostics: Vec::new(),
        generation_origin: Some(
            "core::UnaryConstructionPrototype generated construction".to_string(),
        ),
        cache_key_fragment: Some(format!(
            "construction:{}",
            gcv.construction_instance_id.as_u64()
        )),
        provenance: Provenance::new(format!(
            "declared construction type `{binding_name}` via core::UnaryConstructionPrototype"
        )),
        payload: SymbolPayload::Type(TypeObject {
            carrier_symbol_id: declared_id,
            // Migration-only lookup material for this legacy private result.
            // Construction identity and type lookup identity must not collapse.
            represented_type: crate::TypeValueId(
                gcv.construction_instance_id.0 ^ 0x38c4_15ea_d792_b60f,
            ),
            owner_pattern_head: None,
            fields: Vec::new(),
            field_names: Vec::new(),
            field_type_values: Vec::new(),
            field_type_symbol_ids: Vec::new(),
            type_associated_namespace: None,
            extraction_interface: None,
            provenance: Provenance::new(format!(
                "generated construction type `{binding_name}` (construction instance {})",
                gcv.construction_instance_id.as_u64()
            )),
            generation_origin: Some(
                "core::UnaryConstructionPrototype generated construction type".to_string(),
            ),
            layout_slot: None,
            abi_slot: None,
        }),
    };
    delta.insert_symbol(parent_namespace, declared_symbol.clone());

    Ok(MetaExpansionResult {
        replacement_object: declared_symbol,
        namespace_delta: delta,
        diagnostics: Vec::new(),
        provenance,
    })
}
