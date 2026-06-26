use std::collections::BTreeSet;

use lang_syntax::{norm::NormNavComponent, NormExpr, NormOrigin, NormProduct, NormProductElem};

use crate::{
    graph::{BuildError, NamespaceGraphSnapshot, ResolveExpectation, ResolverContext},
    model::{
        CoreMetaFunction, Diagnostic, FieldObject, FieldProjection, MetaFunctionObject,
        NamespaceDelta, NamespaceNode, NamespaceNodeId, NamespaceNodeKind, PolicyEnv, Provenance,
        ResolverCode, SourceCategory, SymbolKind, SymbolObject, SymbolPayload, SyntaxObject,
        SyntaxObjectKind, TypeField, TypeObject,
    },
    policy_set_meta_runtime,
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
    let NormExpr::Call { source, target, .. } = initializer else {
        return Ok(None);
    };

    let target_path = match expr_to_source_path(target) {
        Some(path) => path,
        None => return Ok(None),
    };

    let target_symbol = match snapshot.capability().resolve_meta_function_with_policy(
        &target_path.join("::"),
        context,
        PolicyEnv::Meta,
    ) {
        Ok(symbol) => symbol,
        Err(diagnostic) => match diagnostic.code {
            Some(ResolverCode::Unresolved) | None => return Ok(None),
            Some(ResolverCode::Ambiguous) | Some(ResolverCode::Conflict) => {
                return Err(BuildError::single(diagnostic))
            }
        },
    };

    let SymbolPayload::MetaFunction(meta_function) = &target_symbol.payload else {
        if target_symbol.kind == SymbolKind::MetaFunction {
            return Err(BuildError::single(Diagnostic::hard_error(
                format!(
                    "meta hard error: `{}` has no meta-function payload",
                    target_symbol.name
                ),
                Some(target_symbol.provenance),
            )));
        }
        return Ok(None);
    };

    let syntax = SyntaxObject {
        kind: SyntaxObjectKind::Product(source.clone()),
        provenance: Provenance::from_norm_origin(
            "closed early-meta source syntax",
            product_origin(source),
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
