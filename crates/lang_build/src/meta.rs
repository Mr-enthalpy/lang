use std::collections::BTreeSet;

use lang_syntax::{norm::NormNavComponent, NormExpr, NormOrigin, NormProduct, NormProductElem};

use crate::{
    call_target::{resolve_call_target, ResolvedCallTarget},
    graph::{BuildError, NamespaceGraphSnapshot, ResolveExpectation, ResolverContext},
    meta_candidate::{
        prepare_meta_callable_candidate_from_input, CandidateBuildIdentityPlaceholder,
        CandidatePreparationContext, CandidatePreparationInput, ParameterShape,
    },
    meta_invocation::{
        invoke_meta_callable, MetaInvocationInput, MetaInvocationResult, MetaInvocationValue,
        MetaValueTarget,
    },
    model::{
        CallablePolicyMetadata, CoreMetaFunction, Diagnostic, ExecutionEnv, FieldObject,
        FieldProjection, MetaFunctionObject, NamespaceDelta, NamespaceNode, NamespaceNodeId,
        NamespaceNodeKind, PolicyEnv, Provenance, SourceCategory, SymbolKind, SymbolObject,
        SymbolPayload, SyntaxObject, SyntaxObjectKind, TypeField, TypeObject,
    },
    normalized_call::{extract_single_call_site, NormalizedCallSite},
    policy_metadata, policy_set_meta_runtime, policy_set_runtime,
    product_shape::ProductMaterialRole,
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

    let SymbolPayload::MetaFunction(meta_function) = &resolved.callee.payload else {
        if resolved.callee.kind == SymbolKind::MetaFunction {
            return Err(BuildError::single(Diagnostic::hard_error(
                format!(
                    "meta hard error: `{}` has no meta-function payload",
                    resolved.callee.name
                ),
                Some(resolved.callee.provenance),
            )));
        }
        return Ok(None);
    };

    let syntax = SyntaxObject {
        kind: SyntaxObjectKind::Product(site.source_product.clone()),
        provenance: Provenance::from_norm_origin(
            "closed early-meta source syntax",
            product_origin(&site.source_product),
        ),
    };

    match meta_function.primitive {
        CoreMetaFunction::Struct => expand_struct_meta(
            snapshot,
            parent_namespace,
            binding_name,
            &syntax,
            context,
            provenance,
            meta_function,
        )
        .map(Some),
        CoreMetaFunction::Assert => Err(BuildError::single(Diagnostic::hard_error(
            "meta hard error: direct source-level `assert` expansion is not implemented in v0.6",
            Some(provenance),
        ))),
        CoreMetaFunction::Verify(_) => Err(BuildError::single(Diagnostic::hard_error(
            "meta hard error: source verification operations cannot be used as initializers",
            Some(provenance),
        ))),
        CoreMetaFunction::IdentityType => expand_identity_type_meta(
            &site,
            &resolved,
            snapshot,
            parent_namespace,
            binding_name,
            context,
            provenance,
            meta_function,
        )
        .map(Some),
        CoreMetaFunction::UnaryConstructionPrototype => Err(BuildError::single(
            Diagnostic::hard_error(
                "meta hard error: UnaryConstructionPrototype has no source-level expansion; use formal invocation",
                Some(provenance),
            ),
        )),
    }
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

fn expand_struct_meta(
    snapshot: &NamespaceGraphSnapshot,
    parent_namespace: NamespaceNodeId,
    binding_name: &str,
    syntax: &SyntaxObject,
    context: &ResolverContext,
    declaration_provenance: Provenance,
    _meta_function: &MetaFunctionObject,
) -> Result<MetaExpansionResult, BuildError> {
    let fields = parse_struct_fields(snapshot, syntax, context)?;
    let mut delta = snapshot.empty_delta();

    let type_symbol_id = delta.allocate_symbol_id();
    let type_namespace_id = delta.allocate_node_id();
    delta.insert_node(NamespaceNode::new(
        type_namespace_id,
        format!("{binding_name}<type-associated>"),
        NamespaceNodeKind::Virtual,
        SourceCategory::TypeAssociatedNamespace,
        Some(parent_namespace),
        declaration_provenance.clone(),
    ));

    let mut type_object = SymbolObject::placeholder(
        type_symbol_id,
        binding_name,
        SymbolKind::Type,
        SourceCategory::DeclaredSymbol,
        Some(parent_namespace),
        declaration_provenance.clone(),
    );
    type_object.policy_metadata.policy_set = policy_set_meta_runtime();
    type_object.node_kind = Some(NamespaceNodeKind::Virtual);
    type_object.generation_origin = Some("core::struct early meta expansion".to_string());
    type_object.cache_key_fragment = Some(format!("struct:{binding_name}:{}", fields.len()));
    type_object.payload = SymbolPayload::Type(TypeObject {
        type_symbol_id,
        fields: fields
            .iter()
            .map(|field| TypeField {
                name: field.name.clone(),
                type_symbol_id: field.type_symbol_id,
                provenance: field.provenance.clone(),
            })
            .collect(),
        field_names: fields.iter().map(|field| field.name.clone()).collect(),
        field_type_symbol_ids: fields.iter().map(|field| field.type_symbol_id).collect(),
        type_associated_namespace: Some(type_namespace_id),
        provenance: declaration_provenance.clone(),
        generation_origin: Some("core::struct early meta expansion".to_string()),
        layout_slot: None,
        abi_slot: None,
    });

    delta.insert_symbol(parent_namespace, type_object.clone());
    insert_field_projection_layer(
        &mut delta,
        type_namespace_id,
        type_symbol_id,
        &fields,
        FieldProjection::Value,
        None,
    );
    insert_projection_namespace(
        &mut delta,
        type_namespace_id,
        "ref",
        type_symbol_id,
        &fields,
        FieldProjection::Ref,
        declaration_provenance.clone(),
    );
    insert_projection_namespace(
        &mut delta,
        type_namespace_id,
        "share",
        type_symbol_id,
        &fields,
        FieldProjection::Share,
        declaration_provenance.clone(),
    );

    Ok(MetaExpansionResult {
        replacement_object: type_object,
        namespace_delta: delta,
        diagnostics: Vec::new(),
        provenance: declaration_provenance,
    })
}

fn insert_projection_namespace(
    delta: &mut NamespaceDelta,
    parent: NamespaceNodeId,
    name: &str,
    owner_type_symbol_id: crate::model::SymbolId,
    fields: &[StructFieldSpec],
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
    owner_type_symbol_id: crate::model::SymbolId,
    fields: &[StructFieldSpec],
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

#[derive(Clone, Debug)]
struct StructFieldSpec {
    name: String,
    type_symbol_id: crate::model::SymbolId,
    provenance: Provenance,
}

fn parse_struct_fields(
    snapshot: &NamespaceGraphSnapshot,
    syntax: &SyntaxObject,
    context: &ResolverContext,
) -> Result<Vec<StructFieldSpec>, BuildError> {
    let SyntaxObjectKind::Product(product) = &syntax.kind;
    let mut fields = Vec::new();
    let mut seen_names = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for element in &product.elements {
        match element {
            NormProductElem::Expr(expr) => match parse_field_expr(snapshot, expr, context) {
                Ok(field) => {
                    if !seen_names.insert(field.name.clone()) {
                        diagnostics.push(Diagnostic::hard_error(
                            format!("duplicate struct field `{}`", field.name),
                            Some(field.provenance),
                        ));
                    } else {
                        fields.push(field);
                    }
                }
                Err(diagnostic) => diagnostics.push(diagnostic),
            },
            NormProductElem::Unit { origin } => diagnostics.push(Diagnostic::hard_error(
                "invalid struct syntax: unit field or trailing unit is not supported",
                Some(Provenance::from_norm_origin(
                    "struct unit product element",
                    origin,
                )),
            )),
        }
    }

    if fields.is_empty() && diagnostics.is_empty() {
        diagnostics.push(Diagnostic::hard_error(
            "invalid struct syntax: struct requires at least one field",
            Some(syntax.provenance.clone()),
        ));
    }

    match compile_time_assert(
        diagnostics.is_empty(),
        syntax.provenance.clone(),
        "struct private checker failed",
    ) {
        Ok(()) => Ok(fields),
        Err(assert_diagnostic) => {
            diagnostics.push(assert_diagnostic);
            Err(BuildError { diagnostics })
        }
    }
}

fn parse_field_expr(
    snapshot: &NamespaceGraphSnapshot,
    expr: &NormExpr,
    context: &ResolverContext,
) -> Result<StructFieldSpec, Diagnostic> {
    let NormExpr::Call {
        source,
        target,
        origin,
    } = expr
    else {
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
            "invalid struct syntax: nested product fields are not supported in v0.6",
            Some(Provenance::from_norm_origin(
                "struct field source",
                &source.origin,
            )),
        ));
    }

    let type_expr = match &source.elements[0] {
        NormProductElem::Expr(NormExpr::Product(product)) => {
            return Err(Diagnostic::hard_error(
                "invalid struct syntax: nested product fields are not supported in v0.6",
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
                format!("unknown struct field type `{}`", type_path_str),
                Some(Provenance::from_norm_origin(
                    "struct field type",
                    expr_origin(type_expr),
                )),
            )
        })?;

    if type_symbol.kind != SymbolKind::Type {
        return Err(Diagnostic::hard_error(
            format!(
                "unknown struct field type `{}`: resolved symbol is not a type",
                type_path.join("::")
            ),
            Some(type_symbol.provenance),
        ));
    }

    let field_name = match target.as_ref() {
        NormExpr::Name { text, .. } => text.clone(),
        _ => {
            return Err(Diagnostic::hard_error(
                "invalid struct syntax: expected a field binder name",
                Some(Provenance::from_norm_origin(
                    "struct field binder",
                    expr_origin(target),
                )),
            ));
        }
    };

    Ok(StructFieldSpec {
        name: field_name,
        type_symbol_id: type_symbol.id,
        provenance: Provenance::from_norm_origin("struct field", origin),
    })
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

fn product_origin(product: &NormProduct) -> &NormOrigin {
    &product.origin
}

/// IdentityType:
///   r === arg (forwarding proof)
///   → ForwardedValue(TypeSymbol)
///   → binding declares a type symbol forwarding the target's symbol identity
///
/// **Pipeline (v0.8 substrate)**:
///
/// ```text
/// NormalizedCallSite.source_product
///   → ProductObject
///   → ArgProductShape
///   → classify_type_arguments (resolves names as type objects)
///   → CandidatePreparationInput
///   → prepare_meta_callable_candidate_from_input
///   → invoke_meta_callable → ForwardedValue(TypeSymbol)
///   → bind_meta_invocation_value_result
/// ```
///
/// **Declaration binding** (where `NamespaceDelta` is installed):
/// A declared forwarding type symbol is installed under `binding_name`.
/// The forwarding target is the `TypeSymbol` carried by the
/// `ForwardedValue`, not a clone of the original `TypeObject`.
fn expand_identity_type_meta(
    site: &NormalizedCallSite,
    resolved: &ResolvedCallTarget,
    snapshot: &NamespaceGraphSnapshot,
    parent_namespace: NamespaceNodeId,
    binding_name: &str,
    context: &ResolverContext,
    provenance: Provenance,
    _meta_function: &MetaFunctionObject,
) -> Result<MetaExpansionResult, BuildError> {
    // --- Substrate stage 1: source product → ArgProductShape ---
    let product_obj =
        site.source_product_object(ProductMaterialRole::MetaConstructionArgumentProduct);
    let shape = product_obj.to_arg_product_shape();

    // --- Substrate stage 2: classify type arguments (with report) ---
    let report = classify_type_arguments_with_report(&shape, &snapshot.capability(), context);

    // --- Substrate stage 3: candidate preparation ---
    let input = CandidatePreparationInput::new(
        resolved.callee.clone(),
        report.classified_shape.clone(),
        ParameterShape::type_parameter_signature(Provenance::new(
            "IdentityType : type -> type signature",
        )),
        CandidatePreparationContext {
            lookup_env: PolicyEnv::Meta,
            demanded_execution: ExecutionEnv::Meta,
            build_identity: CandidateBuildIdentityPlaceholder::default(),
            provenance: provenance.clone(),
        },
    );

    let candidate = match prepare_meta_callable_candidate_from_input(input) {
        crate::meta_candidate::CandidatePrepResult::ApplicablePlaceholder(c) => c,
        crate::meta_candidate::CandidatePrepResult::Deferred { reason, .. } => {
            return Err(BuildError::single(Diagnostic::hard_error(
                format!(
                    "meta hard error: IdentityType candidate preparation deferred ({reason:?})"
                ),
                Some(provenance.clone()),
            )));
        }
        crate::meta_candidate::CandidatePrepResult::Diagnostic(d) => {
            // Enrich diagnostic if unresolved type names are available
            if !report.unresolved_names.is_empty() {
                let names = report.unresolved_names.join(", ");
                return Err(BuildError::single(Diagnostic::hard_error(
                    format!(
                        "meta hard error: IdentityType argument `{names}` could not be resolved as a type object"
                    ),
                    Some(provenance.clone()),
                )));
            }
            return Err(BuildError::single(d));
        }
    };

    // --- Stage 4: formal meta invocation ---
    let invocation_input = MetaInvocationInput::new(*candidate, provenance.clone());

    let invocation_value = match invoke_meta_callable(invocation_input) {
        MetaInvocationResult::Value(v) => v,
        MetaInvocationResult::Diagnostic(d) => return Err(BuildError::single(d)),
    };

    // --- Stage 5: declaration binding (where NamespaceDelta is installed) ---
    bind_meta_invocation_value_result(
        invocation_value,
        snapshot,
        parent_namespace,
        binding_name,
        provenance,
    )
}

/// Bind a meta invocation value into a declaration expansion.
///
/// This is the formal binding entry point. It dispatches on the invocation
/// value type:
///
/// - `ForwardedValue` with `TypeSymbol`: materializes a declaration
///   that forwards the target type's symbol identity.
/// - `ForwardedValue` with other targets: not yet supported.
/// - `GeneratedConstructionValue`: materialized by `bind_generated_construction_value`.
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
                let declared_symbol = SymbolObject {
                    id: declared_id,
                    kind: SymbolKind::Type,
                    name: binding_name.to_string(),
                    source_category: SourceCategory::DeclaredSymbol,
                    node_kind: None,
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
                        type_associated_namespace: None,
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
                    replacement_object: declared_symbol.clone(),
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
    }
}

/// Bind a `GeneratedConstructionValue` into the namespace graph.
///
/// Creates a declared type symbol under `binding_name`. The `SymbolObject`
/// carries the `construction_instance_id` as a `cache_key_fragment`
/// (temporary carrier — the identity model is `ConstructionInstanceId`,
/// not the cache key).
///
/// The `TypeObject` payload attached here is a **binding / materialization
/// projection** of the `GeneratedConstructionValue`, not the invocation
/// result itself. A `TypeValueId` can be derived from the declared symbol
/// only after binding.
///
/// The declared symbol's `type_symbol_id` is a fresh `SymbolId` — the
/// construction identity is the `construction_instance_id`, not the
/// declared symbol ID.
///
/// This function installs a `NamespaceDelta`. It is the **binding**
/// layer — `invoke_meta_callable` remains pure.
fn bind_generated_construction_value(
    gcv: &crate::meta_invocation::GeneratedConstructionValue,
    snapshot: &NamespaceGraphSnapshot,
    parent_namespace: NamespaceNodeId,
    binding_name: &str,
    provenance: Provenance,
) -> Result<MetaExpansionResult, BuildError> {
    // Validate that the construction_instance_id matches the identity material.
    let expected = crate::meta_invocation::compute_construction_instance_id(&gcv.identity_material);
    if expected != gcv.construction_instance_id {
        return Err(BuildError::single(Diagnostic::hard_error(
            format!(
                "meta hard error: GeneratedConstructionValue has mismatched construction_instance_id (expected {}, got {})",
                expected.as_u64(), gcv.construction_instance_id.as_u64()
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

    let replacement = SymbolObject {
        id: declared_id,
        ..declared_symbol.clone()
    };

    Ok(MetaExpansionResult {
        replacement_object: replacement,
        namespace_delta: delta,
        diagnostics: Vec::new(),
        provenance,
    })
}
