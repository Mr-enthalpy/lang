use lang_syntax::{NormExpr, NormProduct, NormProductElem};

use crate::{
    callable_body::{
        compute_struct_construction_material_id, BuiltinBodyInput, StructConstructionMaterial,
        StructFieldConstructionMaterial,
    },
    candidate_preparation::{
        prepare_callable_candidate_with_declared_planes, CandidatePrepIncompleteReason,
        CandidatePrepResult, CandidatePreparationContext, ParameterShape,
    },
    model::{
        BuiltinCallableImpl, CallablePolicyViews, CoreTypeProjection, Diagnostic,
        FieldFunctionProjection, FieldProjection, NamespaceGraphSymbol, NamespaceNode,
        NamespaceNodeId, NamespaceNodeKind, Provenance, SemanticNameDelta, SourceCategory,
        SymbolId, SymbolKind, SymbolPayload, TypeField,
    },
    normalized_call::NormalizedCallSite,
    policy_pair::{
        declared_policy_view, NamespaceVisibility, ObservationHorizon, PolicyMode, Stage,
    },
    product_shape::{
        ArgProductShape, FlattenedProductInvariant, FlattenedProductMaterial, ProductAtom,
    },
    semantic_name_index::{BuildError, ResolverContext, SemanticNameIndex},
    semantic_world::{OrdinaryCallEntry, OrdinaryCallableImplementation},
    struct_pattern_material::{
        StructLeafSyntaxMaterial, StructPatternSyntaxMaterial, StructuralMemberVisibility,
    },
    type_argument::{classify_type_arguments_env_with_report, TypeResolutionEnv},
};

/// Namespace installation material for a completed struct construction.
#[derive(Clone, Debug)]
pub(crate) struct StructProjectionInstall {
    pub replacement_symbol: NamespaceGraphSymbol,
    pub namespace_delta: SemanticNameDelta,
    pub diagnostics: Vec<Diagnostic>,
}

/// Distinguish decided argument diagnostics from an incomplete applicability relation.
#[derive(Debug)]
pub(crate) enum BuiltinPreparationFailure {
    Diagnostic(BuildError),
    /// Applicability is unknown; the candidate family cannot reach maxima.
    Incomplete(Diagnostic),
}

impl From<BuildError> for BuiltinPreparationFailure {
    fn from(error: BuildError) -> Self {
        Self::Diagnostic(error)
    }
}

/// Prepare builtin argument relations from the real call entry.
/// Its implementation selects argument handling; its declared Policy planes
/// remain the only Policy authority. No graph callable payload is read.
pub(crate) fn prepare_resolved_builtin_call(
    entry: &OrdinaryCallEntry,
    site: &NormalizedCallSite,
    type_env: &dyn TypeResolutionEnv,
    resolver_context: &ResolverContext,
    horizon: ObservationHorizon,
    provenance: Provenance,
) -> Result<BuiltinBodyInput, BuiltinPreparationFailure> {
    let OrdinaryCallableImplementation::Builtin(primitive) = entry.implementation else {
        unreachable!("builtin preparation requires a builtin call entry");
    };
    let primitive_name = match primitive {
        BuiltinCallableImpl::Struct => "struct",
        BuiltinCallableImpl::Assert => "assert",
        BuiltinCallableImpl::Verify(_) => "verify",
        BuiltinCallableImpl::IdentityType => "IdentityType",
    };

    let arg_product_shape = site.to_arg_product_shape();
    let mut unresolved_type_names = Vec::new();
    let mut struct_decoded_pattern: Option<crate::struct_decoder::DecodedStructPattern> = None;
    let (classified_shape, parameter_shape) = match primitive {
        BuiltinCallableImpl::IdentityType => {
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
        BuiltinCallableImpl::Struct => {
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
        BuiltinCallableImpl::Assert | BuiltinCallableImpl::Verify(_) => {
            return Err(BuiltinPreparationFailure::Incomplete(Diagnostic::hard_error(
                format!("builtin `{primitive_name}` applicability relation consumer is not connected"),
                Some(provenance),
            )));
        }
    };

    // All Policy planes are the real call entry's already declared facts.
    // Implementation identity supplies argument handling, never Policy.
    let candidate = match prepare_callable_candidate_with_declared_planes(
        entry.backing_declaration,
        &entry.declaration_name,
        entry.callable_view.clone(),
        entry.body_entry_view.clone(),
        entry.complete_result_view.clone(),
        classified_shape,
        parameter_shape,
        CandidatePreparationContext {
            horizon,
            provenance: provenance.clone(),
        },
    ) {
        CandidatePrepResult::Applicable(candidate) => *candidate,
        CandidatePrepResult::Incomplete { reason, .. } => {
            let message = match reason {
                CandidatePrepIncompleteReason::BodyEntryObservationHidden => {
                    "body-entry observation is not visible at the demanded horizon"
                }
                CandidatePrepIncompleteReason::ParameterShapeCompatibilityIncomplete => {
                    "candidate preparation is incomplete because parameter shape compatibility is not established"
                }
            };
            return Err(BuiltinPreparationFailure::Incomplete(
                Diagnostic::hard_error(message, Some(provenance)),
            ));
        }
        CandidatePrepResult::Diagnostic(diagnostic) => {
            if !unresolved_type_names.is_empty() {
                let names = unresolved_type_names.join(", ");
                return Err(BuildError::single(Diagnostic::hard_error(
                    format!(
                        "builtin argument error: {primitive_name} argument `{names}` could not be resolved as a pure type Object"
                    ),
                    Some(provenance),
                )).into());
            }
            return Err(BuildError::single(diagnostic).into());
        }
    };
    let mut invocation_input = BuiltinBodyInput::new(candidate, primitive, provenance);
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

    let flattened = FlattenedProductMaterial {
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
    fields: &[StructFieldConstructionMaterial],
    projection: FieldProjection,
    provenance: Provenance,
) {
    let node_id = delta.allocate_node_id();
    let symbol_id = delta.allocate_symbol_id();
    delta.insert_node(NamespaceNode::new(
        node_id,
        name,
        NamespaceNodeKind::Virtual,
        SourceCategory::GeneratedChild,
        Some(parent),
        provenance.clone(),
    ));
    let mut namespace_symbol = NamespaceGraphSymbol::namespace(
        symbol_id,
        name,
        node_id,
        NamespaceNodeKind::Virtual,
        SourceCategory::GeneratedChild,
        Some(parent),
        provenance,
    );
    namespace_symbol.policy_view = Some(declared_policy_view(Stage::Meta, PolicyMode::Plain));
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
    delta: &mut SemanticNameDelta,
    parent: NamespaceNodeId,
    owner_type_symbol_id: SymbolId,
    fields: &[StructFieldConstructionMaterial],
    projection: FieldProjection,
    forced_provenance: Option<Provenance>,
) {
    for field in fields {
        let symbol_id = delta.allocate_symbol_id();
        let provenance = forced_provenance
            .clone()
            .unwrap_or_else(|| field.provenance.clone());
        let mut symbol = NamespaceGraphSymbol::new(
            symbol_id,
            &field.name,
            SymbolKind::FieldFunction,
            SourceCategory::GeneratedChild,
            Some(parent),
            provenance.clone(),
        );
        symbol.policy_view = Some(declared_policy_view(Stage::Meta, PolicyMode::Plain));
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
        symbol.payload = SymbolPayload::FieldFunction(FieldFunctionProjection {
            owner_type_symbol_id,
            field_name: field.name.clone(),
            field_type_value: field.type_value,
            field_type_symbol_id: field.type_carrier_symbol,
            projection,
            callable_policy: CallablePolicyViews {
                body_entry_policy: declared_policy_view(Stage::Runtime, PolicyMode::Plain),
                return_object_policy: declared_policy_view(Stage::Runtime, PolicyMode::Plain),
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
) -> Result<StructProjectionInstall, BuildError> {
    let expected = compute_struct_construction_material_id(&value.identity_material);
    if expected != value.material_id {
        return Err(BuildError::single(Diagnostic::hard_error(
            format!(
                "struct projection: construction material has mismatched material identity (expected {}, got {})",
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
    delta.insert_node(NamespaceNode::new(
        type_namespace_id,
        format!("{binding_name}<type-associated>"),
        NamespaceNodeKind::Virtual,
        SourceCategory::TypeAssociatedNamespace,
        Some(parent_namespace),
        provenance.clone(),
    ));

    let mut type_projection = NamespaceGraphSymbol::new(
        type_symbol_id,
        binding_name,
        SymbolKind::CompleteTypeProjection,
        SourceCategory::DeclaredSymbol,
        Some(parent_namespace),
        provenance.clone(),
    );
    type_projection.policy_view = Some(declared_policy_view(Stage::Meta, PolicyMode::Plain));
    type_projection.node_kind = Some(NamespaceNodeKind::Virtual);
    type_projection.generation_origin = Some("core::struct construction".to_string());
    type_projection.cache_key_fragment = None;
    type_projection.payload = SymbolPayload::CompleteTypeProjection(CoreTypeProjection {
        carrier_symbol_id: type_symbol_id,
        represented_type,
        fields: value
            .fields
            .iter()
            .map(|field| TypeField {
                name: field.name.clone(),
                type_value: field.type_value,
                type_symbol_id: field.type_carrier_symbol,
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

    Ok(StructProjectionInstall {
        replacement_symbol: type_projection,
        namespace_delta: delta,
        diagnostics: Vec::new(),
    })
}

#[cfg(test)]
mod preparation_policy_tests {
    use super::*;

    #[test]
    fn changing_builtin_leaf_does_not_reconstruct_declared_policy_planes() {
        let base = crate::CompilationWorld::from_manifest(&crate::BuildManifest::new(
            "app",
            vec!["app".into()],
        ))
        .unwrap();
        let mut world = base.semantic_world().clone();
        let callable = declared_policy_view(Stage::Compile, PolicyMode::Const);
        let body = declared_policy_view(Stage::Seal, PolicyMode::Mut);
        let result = declared_policy_view(Stage::Runtime, PolicyMode::Plain);
        let installed = world
            .register_core_callable(
                base.package_root_node(),
                "declared",
                SymbolId(900_003),
                BuiltinCallableImpl::IdentityType,
                None,
                crate::DeclaredResultClass::CompleteType,
                callable.clone(),
                body.clone(),
                result.clone(),
                None,
                Provenance::new("declared planes"),
            )
            .unwrap();
        let crate::SemanticValuePayload::CallEntry(entry) =
            &world.value(installed.call_entry).unwrap().payload
        else {
            panic!("call entry");
        };
        for (implementation, source) in [
            (
                BuiltinCallableImpl::IdentityType,
                "let result = uint8 declared;",
            ),
            (
                BuiltinCallableImpl::Struct,
                "let result = (uint8 field) declared;",
            ),
        ] {
            let mut entry = entry.clone();
            entry.implementation = OrdinaryCallableImplementation::Builtin(implementation);
            let parsed = lang_syntax::parse(source);
            assert!(parsed.diagnostics.is_empty());
            let program = lang_syntax::normalize_program(&parsed.program);
            let lang_syntax::NormForm::Let(lang_syntax::NormDecl::Let { slot, .. }) =
                &program.forms[0]
            else {
                panic!("initializer");
            };
            let site =
                crate::extract_single_call_site(slot.initializer.as_deref().unwrap()).unwrap();
            let prepared = prepare_resolved_builtin_call(
                &entry,
                &site,
                &crate::SemanticTypeEnv::new(&world),
                &base.package_context(),
                ObservationHorizon::SealStatic,
                Provenance::new("entry observation"),
            )
            .unwrap();
            assert_eq!(
                prepared.candidate.callee_symbol_id,
                entry.backing_declaration
            );
            assert_eq!(prepared.candidate.callee_name, entry.declaration_name);
            assert_eq!(prepared.candidate.policy_planes.callable_view, callable);
            assert_eq!(prepared.candidate.policy_planes.body_entry_policy, body);
            assert_eq!(
                prepared.candidate.policy_planes.return_object_policy,
                result
            );
        }
    }
}
