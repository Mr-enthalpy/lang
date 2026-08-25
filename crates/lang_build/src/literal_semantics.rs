//! Abstract literal formation and later concrete construction substrate.
//!
//! Literal evaluation first creates an exact value of `integer`, `real`, or
//! `character` at compile Policy.  Concrete target Types are consulted only
//! by the later construction boundary, and same-Type Policy materialization
//! is a separate migration.  The older atomic/concrete registries remain
//! lookup catalogs for that construction boundary; they do not contextually
//! choose the literal's initial semantic Type.

use std::collections::BTreeMap;

use lang_syntax::{NormExpr, NormLiteralKind};

use crate::{
    canonical_value::canonical_literal_content,
    identity::{SemanticValueId, TypeValueId},
    policy_pair::{
        PatternComponentPolicy, PolicyPair, PolicyStage, StageSet, ValueComponentPolicy,
        ValuePresence,
    },
    CompilationWorld, Diagnostic, Provenance, SymbolKind, SymbolObject,
};

/// Canonical compile-time literal families.  These are ordinary semantic
/// Types (`integer`, `real`, `character`), not a parser-directed concrete
/// machine-type universe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AbstractLiteralFamily {
    Integer,
    Real,
    Character,
}

impl AbstractLiteralFamily {
    pub const fn type_name(self) -> &'static str {
        match self {
            Self::Integer => "integer",
            Self::Real => "real",
            Self::Character => "character",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AbstractLiteralExactValue {
    Integer(String),
    Real(String),
    Character(char),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbstractLiteralValue {
    pub id: SemanticValueId,
    pub family: AbstractLiteralFamily,
    pub exact: AbstractLiteralExactValue,
    pub type_value: TypeValueId,
    pub policy: PolicyPair,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AbstractLiteralFormationFailure {
    NotLiteral,
    AbstractTypeUnavailable(AbstractLiteralFamily),
    /// The frozen frontend exposes a string family but the canonical
    /// character spelling/storage representation remains Open.  Do not
    /// silently reinterpret arbitrary strings as characters.
    CharacterSpellingOpen,
}

/// Form the exact abstract semantic literal before any concrete target is
/// consulted.  The caller supplies only the canonical abstract Type lookup;
/// no expected machine Type enters this boundary.
pub fn form_abstract_literal_value(
    expr: &NormExpr,
    mut resolve_abstract_type: impl FnMut(AbstractLiteralFamily) -> Option<TypeValueId>,
    id: SemanticValueId,
    provenance: Provenance,
) -> Result<AbstractLiteralValue, AbstractLiteralFormationFailure> {
    let NormExpr::Literal { kind, text, .. } = expr else {
        return Err(AbstractLiteralFormationFailure::NotLiteral);
    };
    let (family, exact) = match kind {
        NormLiteralKind::Int => (
            AbstractLiteralFamily::Integer,
            AbstractLiteralExactValue::Integer(canonical_literal_content(*kind, text)),
        ),
        NormLiteralKind::Float => (
            AbstractLiteralFamily::Real,
            AbstractLiteralExactValue::Real(canonical_literal_content(*kind, text)),
        ),
        NormLiteralKind::String => {
            return Err(AbstractLiteralFormationFailure::CharacterSpellingOpen);
        }
    };
    let type_value = resolve_abstract_type(family).ok_or(
        AbstractLiteralFormationFailure::AbstractTypeUnavailable(family),
    )?;
    Ok(AbstractLiteralValue {
        id,
        family,
        exact,
        type_value,
        policy: compile_literal_policy(),
        provenance,
    })
}

pub fn abstract_character_value(
    value: char,
    type_value: TypeValueId,
    id: SemanticValueId,
    provenance: Provenance,
) -> AbstractLiteralValue {
    AbstractLiteralValue {
        id,
        family: AbstractLiteralFamily::Character,
        exact: AbstractLiteralExactValue::Character(value),
        type_value,
        policy: compile_literal_policy(),
        provenance,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AtomicBuiltinType {
    Uint,
    Int,
    Float,
    Buffer,
    Str,
}

impl AtomicBuiltinType {
    pub const fn symbol_name(self) -> &'static str {
        match self {
            Self::Uint => "uint",
            Self::Int => "int",
            Self::Float => "float",
            Self::Buffer => "buffer",
            Self::Str => "str",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AtomicBuiltinTypeRegistryFailure {
    NotTypeSymbol {
        key: AtomicBuiltinType,
        actual_kind: SymbolKind,
    },
    SymbolNameMismatch {
        key: AtomicBuiltinType,
        actual_name: String,
    },
}

/// Current first-order projections for installed atomic builtin Type symbols.
///
/// The key denotes the intended type identity. The stored `TypeValueId` is
/// transitional projection material, not final canonical type-value tracking.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AtomicBuiltinTypeRegistry {
    types: BTreeMap<AtomicBuiltinType, TypeValueId>,
}

impl AtomicBuiltinTypeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_resolved_type_symbol(
        &mut self,
        key: AtomicBuiltinType,
        symbol: &SymbolObject,
    ) -> Result<(), AtomicBuiltinTypeRegistryFailure> {
        if symbol.kind != SymbolKind::Type {
            return Err(AtomicBuiltinTypeRegistryFailure::NotTypeSymbol {
                key,
                actual_kind: symbol.kind,
            });
        }
        if symbol.name != key.symbol_name() {
            return Err(AtomicBuiltinTypeRegistryFailure::SymbolNameMismatch {
                key,
                actual_name: symbol.name.clone(),
            });
        }
        let crate::SymbolPayload::Type(type_object) = &symbol.payload else {
            return Err(AtomicBuiltinTypeRegistryFailure::NotTypeSymbol {
                key,
                actual_kind: symbol.kind,
            });
        };
        self.types.insert(key, type_object.represented_type);
        Ok(())
    }

    pub fn get(&self, key: AtomicBuiltinType) -> Option<TypeValueId> {
        self.types.get(&key).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (AtomicBuiltinType, TypeValueId)> + '_ {
        self.types.iter().map(|(key, value)| (*key, *value))
    }
}

/// Syntactic literal family retained from normalized input.
///
/// This is not an atomic builtin type `T`: an integer spelling may later
/// select either a signed or unsigned concrete numeric type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiteralFamily {
    Integer,
    Float,
    String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumericFamily {
    Uint,
    Int,
    Float,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NumericTypeKey {
    pub family: NumericFamily,
    pub width: u16,
}

impl NumericTypeKey {
    pub const fn new(family: NumericFamily, width: u16) -> Self {
        Self { family, width }
    }
}

/// Concrete numeric `Tnum` identities.
///
/// Keys classify the numeric family/width while values are current first-order
/// `TypeValueId` projections of resolved canonical Type symbols.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NumericTypeRegistry {
    types: BTreeMap<NumericTypeKey, TypeValueId>,
}

impl NumericTypeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: NumericTypeKey, type_value: TypeValueId) {
        self.types.insert(key, type_value);
    }

    pub fn get(&self, key: NumericTypeKey) -> Option<TypeValueId> {
        self.types.get(&key).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (NumericTypeKey, TypeValueId)> + '_ {
        self.types.iter().map(|(key, value)| (*key, *value))
    }

    /// Resolve the concrete numeric types already installed by core bootstrap.
    pub fn from_core_world(world: &CompilationWorld) -> Result<Self, Diagnostic> {
        let mut registry = Self::new();
        for (key, name) in [
            (NumericTypeKey::new(NumericFamily::Uint, 8), "uint8"),
            (NumericTypeKey::new(NumericFamily::Uint, 16), "uint16"),
            (NumericTypeKey::new(NumericFamily::Uint, 32), "uint32"),
            (NumericTypeKey::new(NumericFamily::Float, 32), "float32"),
        ] {
            registry.insert(key, world.resolve_type_value(name)?);
        }
        Ok(registry)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiteralTypeSelection {
    Numeric(NumericTypeKey),
    Atomic(AtomicBuiltinType),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralValue {
    pub id: SemanticValueId,
    pub kind: NormLiteralKind,
    pub text: String,
    pub literal_family: LiteralFamily,
    pub numeric_type: Option<NumericTypeKey>,
    pub type_value: TypeValueId,
    pub policy: PolicyPair,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiteralMaterializationFailure {
    NotLiteral,
    NumericLiteralRequiresConcreteNumericType {
        selected: AtomicBuiltinType,
    },
    NumericFamilySelectionMismatch {
        literal: LiteralFamily,
        selected: NumericFamily,
    },
    AtomicTypeSelectionMismatch {
        literal: LiteralFamily,
        selected: AtomicBuiltinType,
    },
    AtomicBuiltinTypeUnavailable {
        key: AtomicBuiltinType,
    },
    ConcreteNumericTypeUnavailable {
        key: NumericTypeKey,
    },
}

/// Materialize one normalized literal after context has selected its concrete
/// type.
///
/// This API intentionally has no implicit numeric default. Unsuffixed literal
/// defaulting/range selection remains a separate language decision.
pub fn materialize_literal_value(
    expr: &NormExpr,
    atomic_types: &AtomicBuiltinTypeRegistry,
    numeric_types: &NumericTypeRegistry,
    selection: LiteralTypeSelection,
    id: SemanticValueId,
    provenance: Provenance,
) -> Result<LiteralValue, LiteralMaterializationFailure> {
    let NormExpr::Literal { kind, text, .. } = expr else {
        return Err(LiteralMaterializationFailure::NotLiteral);
    };
    let literal_family = match kind {
        NormLiteralKind::Int => LiteralFamily::Integer,
        NormLiteralKind::Float => LiteralFamily::Float,
        NormLiteralKind::String => LiteralFamily::String,
    };

    let (numeric_type, type_value) = match selection {
        LiteralTypeSelection::Numeric(key) => {
            let compatible = match literal_family {
                LiteralFamily::Integer => {
                    matches!(key.family, NumericFamily::Uint | NumericFamily::Int)
                }
                LiteralFamily::Float => key.family == NumericFamily::Float,
                LiteralFamily::String => false,
            };
            if !compatible {
                return Err(
                    LiteralMaterializationFailure::NumericFamilySelectionMismatch {
                        literal: literal_family,
                        selected: key.family,
                    },
                );
            }
            let type_value = numeric_types
                .get(key)
                .ok_or(LiteralMaterializationFailure::ConcreteNumericTypeUnavailable { key })?;
            (Some(key), type_value)
        }
        LiteralTypeSelection::Atomic(selected) => {
            if matches!(
                selected,
                AtomicBuiltinType::Uint | AtomicBuiltinType::Int | AtomicBuiltinType::Float
            ) {
                return Err(
                    LiteralMaterializationFailure::NumericLiteralRequiresConcreteNumericType {
                        selected,
                    },
                );
            }
            let compatible = matches!(
                (literal_family, selected),
                (LiteralFamily::String, AtomicBuiltinType::Str)
            );
            if !compatible {
                return Err(LiteralMaterializationFailure::AtomicTypeSelectionMismatch {
                    literal: literal_family,
                    selected,
                });
            }
            let type_value = atomic_types.get(selected).ok_or(
                LiteralMaterializationFailure::AtomicBuiltinTypeUnavailable { key: selected },
            )?;
            (None, type_value)
        }
    };

    Ok(LiteralValue {
        id,
        kind: *kind,
        text: text.clone(),
        literal_family,
        numeric_type,
        type_value,
        policy: compile_literal_policy(),
        provenance,
    })
}

pub fn compile_literal_policy() -> PolicyPair {
    PolicyPair {
        value: ValueComponentPolicy {
            stages: StageSet::from([PolicyStage::Compile]),
            mutability: Default::default(),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: StageSet::from([PolicyStage::Compile]),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_string_spelling_does_not_close_character_semantics() {
        let expr = NormExpr::Literal {
            kind: NormLiteralKind::String,
            text: "\"x\"".into(),
            origin: lang_syntax::NormOrigin::Source(lang_syntax::Span::new(0, 0, 0, 3)),
        };
        assert_eq!(
            form_abstract_literal_value(
                &expr,
                |_| Some(TypeValueId(1)),
                SemanticValueId(1),
                Provenance::new("open character spelling"),
            ),
            Err(AbstractLiteralFormationFailure::CharacterSpellingOpen),
            "implementation convenience must not close the canonical Open character spelling"
        );
    }
}
