use lang_syntax::{norm::NormNavComponent, NormExpr, NormOrigin, NormProduct, NormProductElem};

use crate::{
    call_target::resolve_call_target,
    extraction_view::{NamedExtractionField, NamedProductExtractionShape, TypeExtractionInterface},
    graph::{BuildError, NamespaceGraphSnapshot, ResolveExpectation, ResolverContext},
    meta_cache::MetaInstanceCache,
    meta_candidate::{
        prepare_meta_callable_candidate_from_input, CandidateBuildIdentityPlaceholder,
        CandidatePrepDeferredReason, CandidatePrepResult, CandidatePreparationContext,
        CandidatePreparationInput, ParameterShape,
    },
    meta_invocation::{
        compute_type_definition_instance_id, invoke_meta_callable, invoke_meta_callable_cached,
        GeneratedFieldDefinition, GeneratedTypeDefinitionValue, MetaInvocationInput,
        MetaInvocationResult, MetaInvocationValue, MetaValueTarget, ReturnSlotSemantics,
    },
    model::{
        CallablePolicyMetadata, CoreMetaFunction, Diagnostic, ExecutionEnv, FieldObject,
        FieldProjection, NamespaceDelta, NamespaceNode, NamespaceNodeId, NamespaceNodeKind,
        PolicyEnv, Provenance, SourceCategory, SymbolId, SymbolKind, SymbolObject, SymbolPayload,
        TypeField, TypeObject,
    },
    normalized_call::extract_single_call_site,
    policy_metadata, policy_set_meta_runtime, policy_set_runtime,
    product_shape::{ArgProductShape, ProductAtom, ProductMaterialRole},
    type_argument::classify_type_arguments_with_report,
};

/// Result of a successful early meta expansion.
#[derive(Clone, Debug)]
pub struct MetaExpansionResult {
    pub replacement_object: SymbolObject,
    pub namespace_delta: NamespaceDelta,
    pub diagnostics: Vec<Diagnostic>,
    pub provenance: Provenance,
}

pub fn try_expand_early_meta_initializer(
    snapshot: &NamespaceGraphSnapshot,
    parent_namespace: NamespaceNodeId,
    binding_name: &str,
    initializer: &NormExpr,
    context: &ResolverContext,
    provenance: Provenance,
) -> Result<Option<MetaExpansionResult>, BuildError> {
    let site = match extract_single_call_site(initializer) {
        Ok(site) => site,
        Err(_) => return Ok(None),
    };

    let Some(resolved) = resolve_call_target(
        &site.target,
        &snapshot.capability(),
        context,
        PolicyEnv::Meta,
    )
    .map_err(BuildError::single)?
    else {
        return Ok(None);
    };

    match &resolved.callee.payload {
        SymbolPayload::MetaFunction(meta_function) if meta_function.primitive.is_some() => {
            expand_meta_initializer_via_invocation(
                initializer,
                snapshot,
                parent_namespace,
                binding_name,
                context,
                PolicyEnv::Meta,
                ExecutionEnv::Meta,
                CandidateBuildIdentityPlaceholder::default(),
                provenance,
                None,
            )
            .map(Some)
        }
        SymbolPayload::MetaFunction(_) => Ok(None),
        _ => {
            if resolved.callee.kind == SymbolKind::MetaFunction {
                Err(BuildError::single(Diagnostic::hard_error(
                    format!(
                        "meta hard error: `{}` has no meta-function payload",
                        resolved.callee.name
                    ),
                    Some(resolved.callee.provenance),
                )))
            } else {
                Ok(None)
            }
        }
    }
}

pub fn expand_meta_initializer_via_invocation(
    initializer: &NormExpr,
    snapshot: &NamespaceGraphSnapshot,
    parent_namespace: NamespaceNodeId,
    binding_name: &str,
    resolver_context: &ResolverContext,
    lookup_env: PolicyEnv,
    demanded_execution: ExecutionEnv,
    build_identity: CandidateBuildIdentityPlaceholder,
    provenance: Provenance,
    cache: Option<&mut MetaInstanceCache>,
) -> Result<MetaExpansionResult, BuildError> {
    let site = extract_single_call_site(initializer).map_err(|_| {
        BuildError::single(Diagnostic::hard_error(
            "initializer is not a meta call initializer",
            Some(provenance.clone()),
        ))
    })?;

    let resolved = resolve_call_target(
        &site.target,
        &snapshot.capability(),
        resolver_context,
        lookup_env,
    )
    .map_err(BuildError::single)?
    .ok_or_else(|| {
        BuildError::single(Diagnostic::hard_error(
            "meta target did not resolve to a callable symbol",
            Some(site.provenance.clone()),
        ))
    })?;

    let SymbolPayload::MetaFunction(meta_function) = &resolved.callee.payload else {
        return Err(BuildError::single(Diagnostic::hard_error(
            format!(
                "meta hard error: `{}` has no meta-function payload",
                resolved.callee.name
            ),
            Some(resolved.callee.provenance),
        )));
    };
    let Some(primitive) = meta_function.primitive else {
        return Err(BuildError::single(Diagnostic::hard_error(
            format!(
                "meta hard error: `{}` is source-declared and must be invoked through overload selection",
                resolved.callee.name
            ),
            Some(resolved.callee.provenance),
        )));
    };
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
        CoreMetaFunction::IdentityType => {
            let report = classify_type_arguments_with_report(
                &arg_product_shape,
                &snapshot.capability(),
                resolver_context,
            );
            unresolved_type_names = report.unresolved_names;
            (
                report.classified_shape,
                ParameterShape::type_parameter_signature(Provenance::new(
                    "IdentityType : type -> type signature",
                )),
            )
        }
        CoreMetaFunction::UnaryConstructionPrototype => {
            let report = classify_type_arguments_with_report(
                &arg_product_shape,
                &snapshot.capability(),
                resolver_context,
            );
            unresolved_type_names = report.unresolved_names;
            (
                report.classified_shape,
                ParameterShape::type_parameter_signature(Provenance::new(
                    "UnaryConstructionPrototype : type -> type signature",
                )),
            )
        }
        CoreMetaFunction::Struct => {
            validate_struct_source_product(&site.source_product)?;
            let classified_shape =
                classify_struct_field_arguments(snapshot, &arg_product_shape, resolver_context)?;

            // Decode the struct argument as a type-pattern expression.
            // Decoder failure is fatal — the expression has entered the
            // core::struct type-pattern decoding path and must be valid.
            let source_arg = NormExpr::Product(site.source_product.clone());
            let decoded_shape = crate::struct_decoder::decode_struct_type_pattern_expr(
                &source_arg,
                provenance.clone(),
            )
            .map_err(|diag| BuildError::single(diag))?;
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

    let input = CandidatePreparationInput::new(
        resolved.callee.clone(),
        classified_shape,
        parameter_shape,
        CandidatePreparationContext {
            lookup_env,
            demanded_execution,
            build_identity,
            provenance: provenance.clone(),
        },
    );

    let candidate = match prepare_meta_callable_candidate_from_input(input) {
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

    let mut invocation_input = MetaInvocationInput::new(candidate, provenance.clone());
    invocation_input.struct_decoded_pattern = struct_decoded_pattern;
    let invocation_result = match cache {
        Some(cache) => invoke_meta_callable_cached(invocation_input, cache),
        None => invoke_meta_callable(invocation_input),
    };
    let invocation_value = match invocation_result {
        MetaInvocationResult::Value(value) => value,
        MetaInvocationResult::Diagnostic(diagnostic) => {
            return Err(BuildError::single(diagnostic));
        }
    };

    bind_meta_invocation_value_result(
        invocation_value,
        snapshot,
        parent_namespace,
        binding_name,
        provenance,
    )
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

fn classify_struct_field_arguments(
    snapshot: &NamespaceGraphSnapshot,
    shape: &ArgProductShape,
    context: &ResolverContext,
) -> Result<ArgProductShape, BuildError> {
    let mut args = shape.raw_args.clone();
    let mut diagnostics = Vec::new();

    for raw_arg in &mut args {
        let Some(atom) = shape.flattened.atoms.get(raw_arg.index) else {
            diagnostics.push(Diagnostic::hard_error(
                "struct argument product shape is missing field atom material",
                Some(raw_arg.provenance.clone()),
            ));
            continue;
        };
        match classify_struct_field_argument(snapshot, atom, context) {
            Ok(type_symbol_id) => {
                *raw_arg = raw_arg
                    .clone()
                    .as_type_object_with_type_symbol(type_symbol_id);
            }
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    if shape.arity == 0 && diagnostics.is_empty() {
        diagnostics.push(Diagnostic::hard_error(
            "invalid struct syntax: struct requires at least one field",
            Some(shape.provenance.clone()),
        ));
    }

    if diagnostics.is_empty() {
        Ok(ArgProductShape {
            raw_args: args,
            ..shape.clone()
        })
    } else {
        Err(BuildError { diagnostics })
    }
}

fn validate_struct_source_product(product: &NormProduct) -> Result<(), BuildError> {
    let mut diagnostics = Vec::new();
    for element in &product.elements {
        if let NormProductElem::Expr(NormExpr::Product(nested)) = element {
            diagnostics.push(Diagnostic::hard_error(
                "invalid struct syntax: nested product fields are not supported in v0.8",
                Some(Provenance::from_norm_origin(
                    "nested struct field product",
                    &nested.origin,
                )),
            ));
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(BuildError { diagnostics })
    }
}

fn classify_struct_field_argument(
    snapshot: &NamespaceGraphSnapshot,
    atom: &ProductAtom,
    context: &ResolverContext,
) -> Result<SymbolId, Diagnostic> {
    let ProductAtom::Expression { expr, .. } = atom else {
        return Err(Diagnostic::hard_error(
            "invalid struct syntax: unit field or trailing unit is not supported",
            Some(atom.provenance().clone()),
        ));
    };
    let NormExpr::Call { source, target, .. } = expr else {
        return Err(Diagnostic::hard_error(
            "invalid struct syntax: expected a field form like `uint8 a`",
            Some(Provenance::from_norm_origin(
                "struct field expression",
                expr_origin(expr),
            )),
        ));
    };

    if source.elements.len() != 1 {
        return Err(Diagnostic::hard_error(
            "invalid struct syntax: nested product fields are not supported in v0.8",
            Some(Provenance::from_norm_origin(
                "struct field source",
                &source.origin,
            )),
        ));
    }

    let type_expr = match &source.elements[0] {
        NormProductElem::Expr(NormExpr::Product(product)) => {
            return Err(Diagnostic::hard_error(
                "invalid struct syntax: nested product fields are not supported in v0.8",
                Some(Provenance::from_norm_origin(
                    "nested struct field product",
                    &product.origin,
                )),
            ));
        }
        NormProductElem::Expr(expr) => expr,
        NormProductElem::Unit { origin } => {
            return Err(Diagnostic::hard_error(
                "invalid struct syntax: unit field type is not supported",
                Some(Provenance::from_norm_origin(
                    "unit struct field type",
                    origin,
                )),
            ));
        }
    };

    if !matches!(target.as_ref(), NormExpr::Name { .. }) {
        return Err(Diagnostic::hard_error(
            "invalid struct syntax: expected a field binder name",
            Some(Provenance::from_norm_origin(
                "struct field binder",
                expr_origin(target),
            )),
        ));
    }

    let type_path = expr_to_source_path(type_expr).ok_or_else(|| {
        Diagnostic::hard_error(
            "invalid struct syntax: unsupported field type expression",
            Some(Provenance::from_norm_origin(
                "struct field type",
                expr_origin(type_expr),
            )),
        )
    })?;
    let type_path_str = type_path.join("::");
    let type_symbol = snapshot
        .capability()
        .resolve_type_object_with_policy(&type_path_str, context, PolicyEnv::Meta)
        .map_err(|_| {
            if let Ok(non_type_symbol) = snapshot.capability().resolve_with_policy(
                &type_path,
                context,
                ResolveExpectation::Object,
                PolicyEnv::Meta,
            ) {
                return Diagnostic::hard_error(
                    format!(
                        "unknown struct field type `{}`: resolved symbol is not a type",
                        type_path_str
                    ),
                    Some(non_type_symbol.provenance),
                );
            }
            Diagnostic::hard_error(
                format!("unknown struct field type `{type_path_str}`"),
                Some(Provenance::from_norm_origin(
                    "struct field type",
                    expr_origin(type_expr),
                )),
            )
        })?;

    Ok(type_symbol.id)
}

fn expr_to_source_path(expr: &NormExpr) -> Option<Vec<String>> {
    match expr {
        NormExpr::Name { text, .. } => Some(vec![text.clone()]),
        NormExpr::Nav { components, .. } => {
            let mut path = Vec::new();
            for component in components {
                match component {
                    NormNavComponent::Name { name, .. } => path.push(name.clone()),
                    _ => return None,
                }
            }
            Some(path)
        }
        _ => None,
    }
}

fn expr_origin(expr: &NormExpr) -> &NormOrigin {
    match expr {
        NormExpr::Call { origin, .. }
        | NormExpr::Name { origin, .. }
        | NormExpr::Literal { origin, .. }
        | NormExpr::Nav { origin, .. }
        | NormExpr::OperatorTarget { origin, .. }
        | NormExpr::Unsupported { origin, .. } => origin,
        NormExpr::Product(product) => &product.origin,
        NormExpr::Closure(closure) => &closure.origin,
        NormExpr::Error(error) => &error.origin,
    }
}

fn insert_projection_namespace(
    delta: &mut NamespaceDelta,
    parent: NamespaceNodeId,
    name: &str,
    owner_type_symbol_id: SymbolId,
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
        fields,
        projection,
        None,
    );
}

fn insert_field_projection_layer(
    delta: &mut NamespaceDelta,
    parent: NamespaceNodeId,
    owner_type_symbol_id: SymbolId,
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
        symbol.generation_origin = Some("core::struct field projection".to_string());
        symbol.cache_key_fragment = Some(format!(
            "field:{}:{}:{projection:?}",
            owner_type_symbol_id.as_u64(),
            field.name
        ));
        symbol.payload = SymbolPayload::FieldFunction(FieldObject {
            owner_type_symbol_id,
            field_name: field.name.clone(),
            field_type_symbol_id: field.type_symbol_id,
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
/// - `ForwardedValue` with `TypeSymbol`: materializes a declaration
///   that forwards the target type's symbol identity.
/// - `GeneratedConstructionValue`: materialized by `bind_generated_construction_value`.
/// - `GeneratedTypeDefinitionValue`: materialized by `bind_generated_type_definition_value`.
pub fn bind_meta_invocation_value_result(
    value: MetaInvocationValue,
    snapshot: &NamespaceGraphSnapshot,
    parent_namespace: NamespaceNodeId,
    binding_name: &str,
    provenance: Provenance,
) -> Result<MetaExpansionResult, BuildError> {
    match value {
        MetaInvocationValue::ForwardedValue(fv) => match fv.target {
            MetaValueTarget::TypeSymbol(type_symbol_id) => {
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
                    },
                    diagnostics: Vec::new(),
                    generation_origin: Some("ForwardedValue(TypeSymbol) binding".to_string()),
                    cache_key_fragment: None,
                    provenance: Provenance::new(format!(
                        "declared forwarding type `{binding_name}`"
                    )),
                    payload: SymbolPayload::Type(TypeObject {
                        type_symbol_id,
                        fields: Vec::new(),
                        field_names: Vec::new(),
                        field_type_symbol_ids: Vec::new(),
                        type_associated_namespace: Some(type_namespace_id),
                        extraction_interface: None,
                        provenance: Provenance::new(format!(
                            "forwarding type `{binding_name}` from TypeSymbol({})",
                            type_symbol_id.0
                        )),
                        generation_origin: Some("ForwardedValue(TypeSymbol) forwarder".to_string()),
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
        },
        MetaInvocationValue::GeneratedConstructionValue(gcv) => bind_generated_construction_value(
            &gcv,
            snapshot,
            parent_namespace,
            binding_name,
            provenance,
        ),
        MetaInvocationValue::GeneratedTypeDefinitionValue(gtdv) => {
            bind_generated_type_definition_value(
                &gtdv,
                snapshot,
                parent_namespace,
                binding_name,
                provenance,
            )
        }
    }
}

fn bind_generated_type_definition_value(
    value: &GeneratedTypeDefinitionValue,
    snapshot: &NamespaceGraphSnapshot,
    parent_namespace: NamespaceNodeId,
    binding_name: &str,
    provenance: Provenance,
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
    let type_namespace_id = delta.allocate_node_id();
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
        type_symbol_id,
        fields: value
            .fields
            .iter()
            .map(|field| TypeField {
                name: field.name.clone(),
                type_symbol_id: field.type_symbol_id,
                provenance: field.provenance.clone(),
            })
            .collect(),
        field_names: value
            .fields
            .iter()
            .map(|field| field.name.clone())
            .collect(),
        field_type_symbol_ids: value
            .fields
            .iter()
            .map(|field| field.type_symbol_id)
            .collect(),
        type_associated_namespace: Some(type_namespace_id),
        extraction_interface: Some(generated_type_extraction_interface(
            type_symbol_id,
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
        &value.fields,
        FieldProjection::Value,
        None,
    );
    insert_projection_namespace(
        &mut delta,
        type_namespace_id,
        "ref",
        type_symbol_id,
        &value.fields,
        FieldProjection::Ref,
        provenance.clone(),
    );
    insert_projection_namespace(
        &mut delta,
        type_namespace_id,
        "share",
        type_symbol_id,
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
    fields: &[GeneratedFieldDefinition],
    provenance: Provenance,
) -> TypeExtractionInterface {
    TypeExtractionInterface {
        owner_type_symbol_id,
        exposed_view: NamedProductExtractionShape {
            owner_type_symbol_id,
            fields: fields
                .iter()
                .map(|field| NamedExtractionField {
                    label: field.name.clone(),
                    field_type_symbol_id: field.type_symbol_id,
                    field_index: field.index,
                    projection: FieldProjection::Value,
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
/// `GeneratedConstructionValue`, not the invocation result itself. A type-value
/// projection can be derived from the declared symbol only after binding.
///
/// The declared symbol's `type_symbol_id` is a fresh `SymbolId` — the
/// construction identity is the `construction_instance_id`, not the
/// declared symbol ID.
///
/// This function installs a `NamespaceDelta`. It is the binding layer —
/// `invoke_meta_callable` remains pure.
fn bind_generated_construction_value(
    gcv: &crate::meta_invocation::GeneratedConstructionValue,
    snapshot: &NamespaceGraphSnapshot,
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
            type_symbol_id: declared_id,
            fields: Vec::new(),
            field_names: Vec::new(),
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
