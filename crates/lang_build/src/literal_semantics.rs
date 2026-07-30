//! Atomic builtin-type and concrete literal-type substrate.
//!
//! `AtomicBuiltinType` is a key for an actual builtin type identity (`T`), not
//! merely a literal classifier. `AtomicBuiltinTypeRegistry` resolves that key
//! to the current first-order `TypeValueId` projection of an installed Type
//! symbol. Numeric literals instead receive a concrete `Tnum` selected by
//! context.
//!
//! This helper is not wired into `evaluate_initializer_best_effort`; it does
//! not define an unsuffixed numeric default or claim initializer integration.
//! The current core bootstrap has no installed `str` Type symbol, so its
//! registry entry and literal materialization are not yet core-backed facts.

use std::collections::BTreeMap;

use lang_syntax::{NormExpr, NormLiteralKind};

use crate::{
    identity::{type_value_projection_from_type_symbol, SemanticValueId, TypeValueId},
    policy_pair::{
        PatternComponentPolicy, PolicyPair, PolicyStage, StageSet, ValueComponentPolicy,
        ValuePresence,
    },
    CompilationWorld, Diagnostic, Provenance, SymbolKind, SymbolObject,
};

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
        self.types
            .insert(key, type_value_projection_from_type_symbol(symbol.id));
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

fn compile_literal_policy() -> PolicyPair {
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
