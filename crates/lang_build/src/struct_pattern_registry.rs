//! Struct-construction pattern material and bounded lookup.
//!
//! `StructPatternMaterialId` is a build-local semantic identity for a materialized
//! struct pattern material. It is not a display name, not a `TypeValueId`, and not a
//! replacement for namespace/value lookup.

use std::collections::BTreeMap;

use lang_syntax::{NormNavComponent, NormOrigin};

use crate::{
    identity::TypeValueId,
    meta_invocation::{ConstructionInstanceId, TypeDefinitionInstanceId},
    model::{Diagnostic, FieldProjection, Provenance, ResolverCode, SymbolId},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructPatternMaterialId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalPatternPlaceId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructPatternMaterialKind {
    Owner,
    Field,
    Construction,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StructPatternMaterialOrigin {
    GlobalBinding {
        symbol_id: SymbolId,
    },
    NamespaceBinding {
        namespace_symbol_id: SymbolId,
        symbol_id: SymbolId,
    },
    LocalMaterialization {
        place_id: LocalPatternPlaceId,
        display_name: String,
    },
    Field {
        owner_head: StructPatternMaterialId,
        field_name: String,
        /// Evaluated field type. A carrier Symbol is deliberately excluded:
        /// `let T: type = uint8` must materialize the same field head as
        /// spelling `uint8` directly.
        field_type_value: TypeValueId,
        projection: FieldProjection,
    },
    Construction {
        construction_instance_id: ConstructionInstanceId,
    },
    StructDefinition {
        type_definition_id: TypeDefinitionInstanceId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructPatternMaterial {
    pub id: StructPatternMaterialId,
    pub kind: StructPatternMaterialKind,
    pub origin: StructPatternMaterialOrigin,
    pub display_name: String,
    pub provenance: Provenance,
}

/// Explicit construction context for struct pattern material. This records
/// materialization provenance and never defines Pattern applicability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructPatternMaterialContext {
    Global {
        symbol_id: SymbolId,
    },
    Namespace {
        namespace_symbol_id: SymbolId,
        symbol_id: SymbolId,
    },
    Local {
        place_id: LocalPatternPlaceId,
    },
    Construction {
        construction_instance_id: ConstructionInstanceId,
    },
    StructDefinition {
        type_definition_id: TypeDefinitionInstanceId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructPatternLookupExpectation {
    StructPatternMaterial,
    Constructor,
    ExtractionChild,
    TypePattern,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructPatternLookupInput {
    AutoName {
        name: String,
        current_scope: StructPatternMaterialId,
        expectation: StructPatternLookupExpectation,
        provenance: Provenance,
    },
    ExplicitNav {
        components: Vec<NormNavComponent>,
        explicit_terminated: bool,
        current_scope: Option<StructPatternMaterialId>,
        expectation: StructPatternLookupExpectation,
        provenance: Provenance,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructFieldPatternMaterial {
    pub field_name: String,
    pub field_type_value: TypeValueId,
    pub projection: FieldProjection,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructPatternMaterialization {
    pub owner_head: StructPatternMaterialId,
    pub field_heads: Vec<(String, StructPatternMaterialId)>,
}

#[derive(Clone, Debug, Default)]
pub struct StructMaterializationState {
    pub pattern_materials: StructPatternMaterialRegistry,
}

#[derive(Clone, Debug, Default)]
pub struct StructPatternMaterialRegistry {
    next_id: u64,
    heads: BTreeMap<StructPatternMaterialId, StructPatternMaterial>,
    by_origin: BTreeMap<StructPatternMaterialOrigin, StructPatternMaterialId>,
    child_scopes: BTreeMap<(StructPatternMaterialId, String), StructPatternMaterialId>,
    explicit_paths: BTreeMap<Vec<String>, StructPatternMaterialId>,
}

impl StructPatternMaterialRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allocate_owner_head(
        &mut self,
        context: StructPatternMaterialContext,
        display_name: impl Into<String>,
        provenance: Provenance,
    ) -> StructPatternMaterialId {
        let display_name = display_name.into();
        let (kind, origin) = match context {
            StructPatternMaterialContext::Global { symbol_id } => (
                StructPatternMaterialKind::Owner,
                StructPatternMaterialOrigin::GlobalBinding { symbol_id },
            ),
            StructPatternMaterialContext::Namespace {
                namespace_symbol_id,
                symbol_id,
            } => (
                StructPatternMaterialKind::Owner,
                StructPatternMaterialOrigin::NamespaceBinding {
                    namespace_symbol_id,
                    symbol_id,
                },
            ),
            StructPatternMaterialContext::Local { place_id } => (
                StructPatternMaterialKind::Owner,
                StructPatternMaterialOrigin::LocalMaterialization {
                    place_id,
                    display_name: display_name.clone(),
                },
            ),
            StructPatternMaterialContext::Construction {
                construction_instance_id,
            } => (
                StructPatternMaterialKind::Construction,
                StructPatternMaterialOrigin::Construction {
                    construction_instance_id,
                },
            ),
            StructPatternMaterialContext::StructDefinition { type_definition_id } => (
                StructPatternMaterialKind::Construction,
                StructPatternMaterialOrigin::StructDefinition { type_definition_id },
            ),
        };
        self.allocate_head(kind, origin, display_name, provenance)
    }

    pub fn allocate_field_head(
        &mut self,
        owner_head: StructPatternMaterialId,
        field_name: impl Into<String>,
        field_type_value: TypeValueId,
        projection: FieldProjection,
        provenance: Provenance,
    ) -> Result<StructPatternMaterialId, Diagnostic> {
        let field_name = field_name.into();
        let origin = StructPatternMaterialOrigin::Field {
            owner_head,
            field_name: field_name.clone(),
            field_type_value,
            projection,
        };
        let child_key = (owner_head, field_name.clone());
        if let Some(existing_head) = self.child_scopes.get(&child_key).copied() {
            let existing_origin = self.heads.get(&existing_head).map(|head| &head.origin);
            if existing_origin == Some(&origin) {
                return Ok(existing_head);
            }
            return Err(Diagnostic::hard_error(
                format!(
                    "explicit pattern extraction child conflict: `{field_name}` is already registered under {:?}",
                    owner_head
                ),
                Some(provenance),
            )
            .with_code(ResolverCode::StructPatternMaterialConflict));
        }
        let field_head = self.allocate_head(
            StructPatternMaterialKind::Field,
            origin,
            field_name.clone(),
            provenance,
        );
        self.child_scopes.insert(child_key, field_head);
        Ok(field_head)
    }

    pub fn allocate_generated_head(
        &mut self,
        construction_instance_id: ConstructionInstanceId,
        display_name: impl Into<String>,
        provenance: Provenance,
    ) -> StructPatternMaterialId {
        self.allocate_head(
            StructPatternMaterialKind::Construction,
            StructPatternMaterialOrigin::Construction {
                construction_instance_id,
            },
            display_name.into(),
            provenance,
        )
    }

    pub fn materialize_struct_pattern(
        &mut self,
        context: StructPatternMaterialContext,
        display_name: impl Into<String>,
        fields: impl IntoIterator<Item = StructFieldPatternMaterial>,
        provenance: Provenance,
    ) -> Result<StructPatternMaterialization, Diagnostic> {
        let owner_head = self.allocate_owner_head(context, display_name, provenance);
        let mut field_heads = Vec::new();
        for field in fields {
            let field_head = self.allocate_field_head(
                owner_head,
                field.field_name.clone(),
                field.field_type_value,
                field.projection,
                field.provenance,
            )?;
            field_heads.push((field.field_name, field_head));
        }
        Ok(StructPatternMaterialization {
            owner_head,
            field_heads,
        })
    }

    pub fn lookup_child(
        &self,
        owner_head: StructPatternMaterialId,
        child_name: &str,
    ) -> Option<StructPatternMaterialId> {
        self.child_scopes
            .get(&(owner_head, child_name.to_string()))
            .copied()
    }

    pub fn register_explicit_path(
        &mut self,
        components: impl IntoIterator<Item = impl Into<String>>,
        head_id: StructPatternMaterialId,
        provenance: Provenance,
    ) -> Result<(), Diagnostic> {
        let components = components.into_iter().map(Into::into).collect::<Vec<_>>();
        if let Some(existing) = self.explicit_paths.get(&components).copied() {
            if existing == head_id {
                return Ok(());
            }
            return Err(Diagnostic::hard_error(
                format!(
                    "explicit pattern navigation path conflict: `{}` is already registered",
                    components.join("::")
                ),
                Some(provenance),
            )
            .with_code(ResolverCode::StructPatternMaterialConflict));
        }
        self.explicit_paths.insert(components, head_id);
        Ok(())
    }

    pub fn lookup_explicit_path(&self, components: &[String]) -> Option<StructPatternMaterialId> {
        self.explicit_paths.get(components).copied()
    }

    pub fn get(&self, head_id: StructPatternMaterialId) -> Option<&StructPatternMaterial> {
        self.heads.get(&head_id)
    }

    pub fn resolve_pattern_lookup(
        &self,
        input: StructPatternLookupInput,
    ) -> Result<StructPatternMaterialId, Diagnostic> {
        match input {
            StructPatternLookupInput::AutoName {
                name,
                current_scope,
                expectation,
                provenance,
            } => {
                if expectation != StructPatternLookupExpectation::ExtractionChild {
                    return Err(Diagnostic::hard_error(
                        "restricted v0.9 pattern lookup only supports AutoName as an extraction child",
                        Some(provenance),
                    )
                    .with_code(ResolverCode::UnsupportedStructPatternLookupExpectation));
                }
                self.lookup_child(current_scope, &name).ok_or_else(|| {
                    Diagnostic::hard_error(
                        format!(
                            "bounded extraction lookup failed: `{name}` is not a child of {:?}",
                            current_scope
                        ),
                        Some(provenance),
                    )
                    .with_code(ResolverCode::Unresolved)
                })
            }
            StructPatternLookupInput::ExplicitNav {
                components,
                provenance,
                ..
            } => {
                let names = explicit_nav_names(&components).ok_or_else(|| {
                    Diagnostic::hard_error(
                        "unsupported explicit pattern navigation component in restricted v0.9 resolver",
                        Some(provenance.clone()),
                    )
                    .with_code(ResolverCode::UnsupportedOverloadTarget)
                })?;
                self.lookup_explicit_path(&names).ok_or_else(|| {
                    Diagnostic::hard_error(
                        format!(
                            "explicit pattern navigation `{}` is unresolved",
                            names.join("::")
                        ),
                        Some(provenance),
                    )
                    .with_code(ResolverCode::Unresolved)
                })
            }
        }
    }

    fn allocate_head(
        &mut self,
        kind: StructPatternMaterialKind,
        origin: StructPatternMaterialOrigin,
        display_name: String,
        provenance: Provenance,
    ) -> StructPatternMaterialId {
        if let Some(existing) = self.by_origin.get(&origin) {
            return *existing;
        }

        let id = StructPatternMaterialId(self.next_id);
        self.next_id += 1;
        let head = StructPatternMaterial {
            id,
            kind,
            origin: origin.clone(),
            display_name,
            provenance,
        };
        self.by_origin.insert(origin, id);
        self.heads.insert(id, head);
        id
    }
}

fn explicit_nav_names(components: &[NormNavComponent]) -> Option<Vec<String>> {
    let mut names = Vec::new();
    for component in components {
        match component {
            NormNavComponent::Name { name, .. } => names.push(name.clone()),
            NormNavComponent::Operator { spelling, .. } => names.push(spelling.clone()),
            NormNavComponent::Group { .. } | NormNavComponent::Error(_) => return None,
        }
    }
    Some(names)
}

pub fn nav_component_name(name: impl Into<String>, origin: NormOrigin) -> NormNavComponent {
    NormNavComponent::Name {
        name: name.into(),
        origin,
    }
}
