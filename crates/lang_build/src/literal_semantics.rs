//! Builtin atomic-family and concrete literal-type substrate.
//!
//! `AtomicBuiltinFamily` is classification material (`T`), not a
//! `TypeValueId`. Numeric values receive a concrete `Tnum` selected by context
//! and resolved through canonical core Type symbols.
//!
//! This helper is not wired into `evaluate_initializer_best_effort`; it does
//! not define an unsuffixed numeric default or claim initializer integration.

use std::collections::BTreeMap;

use lang_syntax::{NormExpr, NormLiteralKind};

use crate::{
    identity::{type_value_projection_from_type_symbol, SemanticValueId, TypeValueId},
    policy_pair::{
        PatternComponentPolicy, PolicyPair, PolicyStage, StageSet, ValueComponentPolicy,
        ValuePresence,
    },
    CompilationWorld, Diagnostic, Provenance,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AtomicBuiltinFamily {
    Uint,
    Int,
    Float,
    Buffer,
    Str,
}

/// Syntactic literal family retained from normalized input.
///
/// This is not the atomic builtin family `T`: an integer spelling may later
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
/// Keys classify the numeric family/width while values are canonical
/// `TypeValueId` projections of resolved Type symbols. No family itself owns a
/// `TypeValueId`.
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
            let symbol = world
                .snapshot()
                .capability()
                .resolve_type_object(name, &world.package_context())?;
            registry.insert(key, type_value_projection_from_type_symbol(symbol.id));
        }
        Ok(registry)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiteralTypeSelection {
    Numeric(NumericTypeKey),
    Atomic {
        family: AtomicBuiltinFamily,
        type_value: TypeValueId,
    },
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
    AtomicNumericFamilyIsNotConcrete {
        family: AtomicBuiltinFamily,
    },
    NumericFamilySelectionMismatch {
        literal: LiteralFamily,
        selected: NumericFamily,
    },
    AtomicFamilySelectionMismatch {
        literal: LiteralFamily,
        selected: AtomicBuiltinFamily,
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
        LiteralTypeSelection::Atomic { family, type_value } => {
            if matches!(
                family,
                AtomicBuiltinFamily::Uint | AtomicBuiltinFamily::Int | AtomicBuiltinFamily::Float
            ) {
                return Err(
                    LiteralMaterializationFailure::AtomicNumericFamilyIsNotConcrete { family },
                );
            }
            let compatible = matches!(
                (literal_family, family),
                (LiteralFamily::String, AtomicBuiltinFamily::Str)
            );
            if !compatible {
                return Err(
                    LiteralMaterializationFailure::AtomicFamilySelectionMismatch {
                        literal: literal_family,
                        selected: family,
                    },
                );
            }
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
        namespace_visibility: None,
        export_root: false,
    }
}
