//! Resolved pattern-head identity and bounded extraction lookup.
//!
//! `PatternHeadId` is a build-local semantic identity for a materialized
//! pattern head. It is not a display name, not a `TypeValueId`, and not a
//! replacement for namespace/value lookup.

use std::collections::BTreeMap;

use lang_syntax::{NormNavComponent, NormOrigin};

use crate::{
    meta_invocation::{ConstructionInstanceId, TypeDefinitionInstanceId},
    model::{Diagnostic, FieldProjection, Provenance, ResolverCode, SymbolId},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PatternHeadId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalPatternPlaceId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternHeadKind {
    Owner,
    Field,
    Generated,
    External,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PatternHeadOrigin {
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
        owner_head: PatternHeadId,
        field_name: String,
        field_type_symbol_id: SymbolId,
        projection: FieldProjection,
    },
    Generated {
        construction_instance_id: ConstructionInstanceId,
    },
    GeneratedTypeDefinition {
        type_definition_id: TypeDefinitionInstanceId,
    },
    ExternalForward {
        target_symbol_id: SymbolId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternHead {
    pub id: PatternHeadId,
    pub kind: PatternHeadKind,
    pub origin: PatternHeadOrigin,
    pub display_name: String,
    pub provenance: Provenance,
}

/// Transitional categorical registry contexts for explicit low-level
/// materialization and tests. These variants are not the final
/// `ResolvedPatternScope` owner model, and ordinary binding must not derive one
/// from its destination path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternMaterializationContext {
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
    Generated {
        construction_instance_id: ConstructionInstanceId,
    },
    GeneratedTypeDefinition {
        type_definition_id: TypeDefinitionInstanceId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternExpectation {
    PatternHead,
    Constructor,
    ExtractionChild,
    TypePattern,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternLookupInput {
    AutoName {
        name: String,
        current_scope: PatternHeadId,
        expectation: PatternExpectation,
        provenance: Provenance,
    },
    ExplicitNav {
        components: Vec<NormNavComponent>,
        explicit_terminated: bool,
        current_scope: Option<PatternHeadId>,
        expectation: PatternExpectation,
        provenance: Provenance,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternFieldMaterialization {
    pub field_name: String,
    pub field_type_symbol_id: SymbolId,
    pub projection: FieldProjection,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternHeadMaterialization {
    pub owner_head: PatternHeadId,
    pub field_heads: Vec<(String, PatternHeadId)>,
}

#[derive(Clone, Debug, Default)]
pub struct TypeMaterializationState {
    pub pattern_heads: PatternHeadRegistry,
}

#[derive(Clone, Debug, Default)]
pub struct PatternHeadRegistry {
    next_id: u64,
    heads: BTreeMap<PatternHeadId, PatternHead>,
    by_origin: BTreeMap<PatternHeadOrigin, PatternHeadId>,
    child_scopes: BTreeMap<(PatternHeadId, String), PatternHeadId>,
    explicit_paths: BTreeMap<Vec<String>, PatternHeadId>,
}

impl PatternHeadRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allocate_owner_head(
        &mut self,
        context: PatternMaterializationContext,
        display_name: impl Into<String>,
        provenance: Provenance,
    ) -> PatternHeadId {
        let display_name = display_name.into();
        let (kind, origin) = match context {
            PatternMaterializationContext::Global { symbol_id } => (
                PatternHeadKind::Owner,
                PatternHeadOrigin::GlobalBinding { symbol_id },
            ),
            PatternMaterializationContext::Namespace {
                namespace_symbol_id,
                symbol_id,
            } => (
                PatternHeadKind::Owner,
                PatternHeadOrigin::NamespaceBinding {
                    namespace_symbol_id,
                    symbol_id,
                },
            ),
            PatternMaterializationContext::Local { place_id } => (
                PatternHeadKind::Owner,
                PatternHeadOrigin::LocalMaterialization {
                    place_id,
                    display_name: display_name.clone(),
                },
            ),
            PatternMaterializationContext::Generated {
                construction_instance_id,
            } => (
                PatternHeadKind::Generated,
                PatternHeadOrigin::Generated {
                    construction_instance_id,
                },
            ),
            PatternMaterializationContext::GeneratedTypeDefinition { type_definition_id } => (
                PatternHeadKind::Generated,
                PatternHeadOrigin::GeneratedTypeDefinition { type_definition_id },
            ),
        };
        self.allocate_head(kind, origin, display_name, provenance)
    }

    pub fn allocate_field_head(
        &mut self,
        owner_head: PatternHeadId,
        field_name: impl Into<String>,
        field_type_symbol_id: SymbolId,
        projection: FieldProjection,
        provenance: Provenance,
    ) -> Result<PatternHeadId, Diagnostic> {
        let field_name = field_name.into();
        let origin = PatternHeadOrigin::Field {
            owner_head,
            field_name: field_name.clone(),
            field_type_symbol_id,
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
            .with_code(ResolverCode::PatternHeadConflict));
        }
        let field_head = self.allocate_head(
            PatternHeadKind::Field,
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
    ) -> PatternHeadId {
        self.allocate_head(
            PatternHeadKind::Generated,
            PatternHeadOrigin::Generated {
                construction_instance_id,
            },
            display_name.into(),
            provenance,
        )
    }

    pub fn allocate_external_forward_head(
        &mut self,
        target_symbol_id: SymbolId,
        display_name: impl Into<String>,
        provenance: Provenance,
    ) -> PatternHeadId {
        self.allocate_head(
            PatternHeadKind::External,
            PatternHeadOrigin::ExternalForward { target_symbol_id },
            display_name.into(),
            provenance,
        )
    }

    pub fn materialize_struct_pattern_heads(
        &mut self,
        context: PatternMaterializationContext,
        display_name: impl Into<String>,
        fields: impl IntoIterator<Item = PatternFieldMaterialization>,
        provenance: Provenance,
    ) -> Result<PatternHeadMaterialization, Diagnostic> {
        let owner_head = self.allocate_owner_head(context, display_name, provenance);
        let mut field_heads = Vec::new();
        for field in fields {
            let field_head = self.allocate_field_head(
                owner_head,
                field.field_name.clone(),
                field.field_type_symbol_id,
                field.projection,
                field.provenance,
            )?;
            field_heads.push((field.field_name, field_head));
        }
        Ok(PatternHeadMaterialization {
            owner_head,
            field_heads,
        })
    }

    pub fn lookup_child(
        &self,
        owner_head: PatternHeadId,
        child_name: &str,
    ) -> Option<PatternHeadId> {
        self.child_scopes
            .get(&(owner_head, child_name.to_string()))
            .copied()
    }

    pub fn register_explicit_path(
        &mut self,
        components: impl IntoIterator<Item = impl Into<String>>,
        head_id: PatternHeadId,
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
            .with_code(ResolverCode::PatternHeadConflict));
        }
        self.explicit_paths.insert(components, head_id);
        Ok(())
    }

    pub fn lookup_explicit_path(&self, components: &[String]) -> Option<PatternHeadId> {
        self.explicit_paths.get(components).copied()
    }

    pub fn get(&self, head_id: PatternHeadId) -> Option<&PatternHead> {
        self.heads.get(&head_id)
    }

    pub fn resolve_pattern_lookup(
        &self,
        input: PatternLookupInput,
    ) -> Result<PatternHeadId, Diagnostic> {
        match input {
            PatternLookupInput::AutoName {
                name,
                current_scope,
                expectation,
                provenance,
            } => {
                if expectation != PatternExpectation::ExtractionChild {
                    return Err(Diagnostic::hard_error(
                        "restricted v0.9 pattern lookup only supports AutoName as an extraction child",
                        Some(provenance),
                    )
                    .with_code(ResolverCode::UnsupportedPatternExpectation));
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
            PatternLookupInput::ExplicitNav {
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
        kind: PatternHeadKind,
        origin: PatternHeadOrigin,
        display_name: String,
        provenance: Provenance,
    ) -> PatternHeadId {
        if let Some(existing) = self.by_origin.get(&origin) {
            return *existing;
        }

        let id = PatternHeadId(self.next_id);
        self.next_id += 1;
        let head = PatternHead {
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
