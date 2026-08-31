use lang_syntax::{NormExpr, NormProduct, NormProductElem};

use crate::{
    content_observation::{NamedObservedField, NamedObservedProduct, TypeContentObservation},
    meta_candidate::{
        prepare_meta_callable_candidate_with_declared_planes, CallableCandidateKind,
        CandidatePrepDeferredReason, CandidatePrepResult, CandidatePreparationContext,
        ParameterShape,
    },
    meta_invocation::{
        attach_struct_pattern_materials, compute_struct_construction_material_id,
        GeneratedFieldDefinition, MetaInvocationInput, StructConstructionMaterial,
    },
    model::{
        CallablePolicyViews, CoreMetaFunction, CoreTypeProjection, Diagnostic, ExecutionEnv,
        FieldObject, FieldProjection, NamespaceNode, NamespaceNodeId, NamespaceNodeKind, PolicyEnv,
        Provenance, SemanticNameDelta, SourceCategory, SymbolId, SymbolKind, SymbolObject,
        SymbolPayload, TypeField,
    },
    normalized_call::NormalizedCallSite,
    policy_pair::{declared_policy_view, NamespaceVisibility, PolicyMode, PolicyStage},
    product_shape::{
        ArgProductShape, FlattenedProductInvariant, FlattenedProductObject, ProductAtom,
        ProductMaterialRole,
    },
    semantic_name_index::{BuildError, ResolverContext, SemanticNameIndex},
    struct_pattern_material::{
        StructLeafSyntaxMaterial, StructPatternSyntaxMaterial, StructuralMemberVisibility,
    },
    struct_pattern_registry::StructMaterializationState,
    type_argument::{classify_type_arguments_env_with_report, TypeResolutionEnv},
};

/// Namespace installation material for a completed struct construction.
#[derive(Clone, Debug)]
pub(crate) struct StructProjectionInstall {
    pub replacement_object: SymbolObject,
    pub namespace_delta: SemanticNameDelta,
    pub diagnostics: Vec<Diagnostic>,
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
    provenance: Provenance,
) -> Result<MetaInvocationInput, BuildError> {
    let primitive_name = match primitive {
        CoreMetaFunction::Struct => "struct",
        CoreMetaFunction::Assert => "assert",
        CoreMetaFunction::Verify(_) => "verify",
        CoreMetaFunction::IdentityType => "IdentityType",
    };

    let arg_product_shape =
        site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let mut unresolved_type_names = Vec::new();
    let mut struct_decoded_pattern: Option<crate::struct_decoder::DecodedStructPattern> = None;
    let (classified_shape, parameter_shape) = match primitive {
        CoreMetaFunction::IdentityType => {
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
            provenance: provenance.clone(),
        },
    ) {
        CandidatePrepResult::Applicable(candidate) => *candidate,
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
                        "meta hard error: {primitive_name} argument `{names}` could not be resolved as a pure type Object"
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

/// Classify the actual structural leaves produced by the struct decoder.
///
/// A named Pattern such as `((uint8 inner) t)` has one field leaf (`inner`)
/// under the top Pattern name (`t`).  Candidate preparation must therefore
/// consume that decoded leaf, not the invocation Product atom containing the
/// whole named Pattern expression.
fn classify_decoded_struct_field_arguments(
    type_env: &dyn TypeResolutionEnv,
    pattern: &StructPatternSyntaxMaterial,
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
        let StructLeafSyntaxMaterial::Path(path) = external_type_expr else {
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
            .as_complete_type_projection_with_identity(carrier_symbol, represented_type);
    }
    Ok(shape)
}

fn collect_decoded_struct_leaves<'a>(
    pattern: &'a StructPatternSyntaxMaterial,
    output: &mut Vec<(&'a StructLeafSyntaxMaterial, &'a str, &'a Provenance)>,
) {
    match pattern {
        StructPatternSyntaxMaterial::Leaf {
            external_type_expr,
            local_pattern_name,
            provenance,
            ..
        } => output.push((external_type_expr, local_pattern_name.as_str(), provenance)),
        StructPatternSyntaxMaterial::Product { elements, .. } => {
            for element in elements {
                collect_decoded_struct_leaves(element, output);
            }
        }
        StructPatternSyntaxMaterial::Sum { alternatives, .. } => {
            for alternative in alternatives {
                collect_decoded_struct_leaves(alternative, output);
            }
        }
        StructPatternSyntaxMaterial::Named { child, .. } => {
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
                    "invalid struct syntax: nested product fields are not supported by the struct decoder",
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
    owner_struct_pattern_registry: Option<crate::struct_pattern_registry::StructPatternMaterialId>,
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
    namespace_symbol.policy_view = Some(declared_policy_view(
        &[PolicyStage::Meta, PolicyStage::Runtime],
        PolicyMode::Plain,
    ));
    delta.insert_symbol(parent, namespace_symbol);
    insert_field_projection_layer(
        delta,
        node_id,
        owner_type_symbol_id,
        owner_struct_pattern_registry,
        fields,
        projection,
        None,
    );
}

fn insert_field_projection_layer(
    delta: &mut SemanticNameDelta,
    parent: NamespaceNodeId,
    owner_type_symbol_id: SymbolId,
    owner_struct_pattern_registry: Option<crate::struct_pattern_registry::StructPatternMaterialId>,
    fields: &[GeneratedFieldDefinition],
    projection: FieldProjection,
    forced_provenance: Option<Provenance>,
) {
    for field in fields {
        let symbol_id = delta.allocate_symbol_id();
        let provenance = forced_provenance
            .clone()
            .unwrap_or_else(|| field.provenance.clone());
        let mut symbol = SymbolObject::new(
            symbol_id,
            &field.name,
            SymbolKind::FieldFunction,
            SourceCategory::GeneratedChild,
            Some(parent),
            provenance.clone(),
        );
        symbol.policy_view = Some(declared_policy_view(
            &[PolicyStage::Meta, PolicyStage::Runtime],
            PolicyMode::Plain,
        ));
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
            owner_struct_pattern_registry,
            field_name: field.name.clone(),
            field_type_value: field.type_value,
            field_type_symbol_id: field.type_carrier_symbol,
            field_struct_pattern_registry: field.struct_pattern_registry,
            projection,
            callable_policy: CallablePolicyViews {
                body_entry_policy: declared_policy_view(&[PolicyStage::Runtime], PolicyMode::Plain),
                return_object_policy: declared_policy_view(
                    &[PolicyStage::Runtime],
                    PolicyMode::Plain,
                ),
            },
            provenance,
        });
        delta.insert_symbol(parent, symbol);
    }
}

/// Expand replayable `struct` execution material as a namespace projection of
/// an already formed complete type value.
///
/// The complete type is an input to this rendering boundary. Construction
/// material cannot manufacture a type lookup key or whole-type identity.
pub(crate) fn expand_struct_construction_material(
    value: StructConstructionMaterial,
    complete_type: &crate::CompleteTypeValue,
    snapshot: &SemanticNameIndex,
    parent_namespace: NamespaceNodeId,
    binding_name: &str,
    provenance: Provenance,
    materialization_state: &mut StructMaterializationState,
) -> Result<StructProjectionInstall, BuildError> {
    let expected = compute_struct_construction_material_id(&value.identity_material);
    if expected != value.material_id {
        return Err(BuildError::single(Diagnostic::hard_error(
            format!(
                "meta hard error: StructConstructionMaterial has mismatched material identity (expected {}, got {})",
                expected.as_u64(),
                value.material_id.as_u64()
            ),
            Some(value.provenance.clone()),
        )));
    }
    let mut delta = snapshot.empty_delta();
    let type_symbol_id = delta.allocate_symbol_id();
    if value
        .canonical_type
        .is_some_and(|lookup| lookup != complete_type.lookup_key())
    {
        return Err(BuildError::single(Diagnostic::hard_error(
            "struct construction material does not belong to the supplied complete type",
            Some(value.provenance.clone()),
        )));
    }
    let represented_type = complete_type.lookup_key();
    let type_namespace_id = delta.allocate_node_id();
    let value = if value.pattern_materials.is_some() {
        value
    } else {
        attach_struct_pattern_materials(value, materialization_state, provenance.clone())
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

    let mut type_projection = SymbolObject::new(
        type_symbol_id,
        binding_name,
        SymbolKind::CompleteTypeProjection,
        SourceCategory::DeclaredSymbol,
        Some(parent_namespace),
        provenance.clone(),
    );
    type_projection.policy_view = Some(declared_policy_view(
        &[PolicyStage::Meta, PolicyStage::Runtime],
        PolicyMode::Plain,
    ));
    type_projection.node_kind = Some(NamespaceNodeKind::Virtual);
    type_projection.generation_origin = Some("core::struct construction".to_string());
    type_projection.cache_key_fragment = None;
    type_projection.payload = SymbolPayload::CompleteTypeProjection(CoreTypeProjection {
        carrier_symbol_id: type_symbol_id,
        represented_type,
        owner_struct_pattern_registry: value
            .pattern_materials
            .as_ref()
            .map(|heads| heads.owner_head),
        fields: value
            .fields
            .iter()
            .map(|field| TypeField {
                name: field.name.clone(),
                type_value: field.type_value,
                type_symbol_id: field.type_carrier_symbol,
                struct_pattern_registry: field.struct_pattern_registry,
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
            value
                .pattern_materials
                .as_ref()
                .map(|heads| heads.owner_head),
            &value.fields,
            provenance.clone(),
        )),
        provenance: provenance.clone(),
        generation_origin: Some(format!(
            "core::struct construction material {}",
            value.material_id.as_u64()
        )),
        layout_slot: None,
        abi_slot: None,
    });

    delta.insert_symbol(parent_namespace, type_projection.clone());
    insert_field_projection_layer(
        &mut delta,
        type_namespace_id,
        type_symbol_id,
        value
            .pattern_materials
            .as_ref()
            .map(|heads| heads.owner_head),
        &value.fields,
        FieldProjection::Value,
        None,
    );
    insert_projection_namespace(
        &mut delta,
        type_namespace_id,
        "ref",
        type_symbol_id,
        value
            .pattern_materials
            .as_ref()
            .map(|heads| heads.owner_head),
        &value.fields,
        FieldProjection::Ref,
        provenance.clone(),
    );
    insert_projection_namespace(
        &mut delta,
        type_namespace_id,
        "share",
        type_symbol_id,
        value
            .pattern_materials
            .as_ref()
            .map(|heads| heads.owner_head),
        &value.fields,
        FieldProjection::Share,
        provenance.clone(),
    );

    Ok(StructProjectionInstall {
        replacement_object: type_projection,
        namespace_delta: delta,
        diagnostics: Vec::new(),
    })
}

fn generated_type_extraction_interface(
    owner_type_symbol_id: SymbolId,
    owner_type_value: crate::TypeValueId,
    owner_struct_pattern_registry: Option<crate::struct_pattern_registry::StructPatternMaterialId>,
    fields: &[GeneratedFieldDefinition],
    provenance: Provenance,
) -> TypeContentObservation {
    TypeContentObservation {
        owner_type_value,
        owner_type_symbol_id,
        owner_struct_pattern_registry,
        exposed_view: NamedObservedProduct {
            owner_type_value,
            owner_type_symbol_id,
            owner_struct_pattern_registry,
            fields: fields
                .iter()
                .filter(|field| field.visibility != StructuralMemberVisibility::Private)
                .map(|field| NamedObservedField {
                    label: field.name.clone(),
                    field_type_value: field.type_value,
                    field_type_observation: field.type_observation,
                    field_type_symbol_id: field.type_carrier_symbol,
                    field_struct_pattern_registry: field.struct_pattern_registry,
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
