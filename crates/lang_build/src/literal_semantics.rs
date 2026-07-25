//! Builtin atomic-type and literal-value substrate.
//!
//! Literals enter semantic evaluation as values.  In particular, a string
//! literal is a compile-stage `str` value; it is neither storage nor `str ref`.
//! Runtime storage/reference construction belongs to ordinary callable
//! transition implementations.

use lang_syntax::{NormExpr, NormLiteralKind};

use crate::{
    identity::{SemanticValueId, TypeValueId},
    policy_pair::{
        PatternComponentPolicy, PolicyPair, PolicyStage, StageSet, ValueComponentPolicy,
        ValuePresence,
    },
    Provenance,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AtomicBuiltinType {
    Uint,
    Int,
    Float,
    Buffer,
    Str,
}

/// Type-value projections for the five first atomic builtin types.
///
/// The registry is explicit so this module never invents numeric type
/// identities.  Callers obtain the ids from the world/namespace graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtomicBuiltinTypeIds {
    pub uint: TypeValueId,
    pub int: TypeValueId,
    pub float: TypeValueId,
    pub buffer: TypeValueId,
    pub str_: TypeValueId,
}

impl AtomicBuiltinTypeIds {
    pub fn get(&self, builtin: AtomicBuiltinType) -> TypeValueId {
        match builtin {
            AtomicBuiltinType::Uint => self.uint,
            AtomicBuiltinType::Int => self.int,
            AtomicBuiltinType::Float => self.float,
            AtomicBuiltinType::Buffer => self.buffer,
            AtomicBuiltinType::Str => self.str_,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralValue {
    pub id: SemanticValueId,
    pub kind: NormLiteralKind,
    pub text: String,
    pub builtin_type: AtomicBuiltinType,
    pub type_value: TypeValueId,
    pub policy: PolicyPair,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiteralMaterializationFailure {
    NotLiteral,
}

/// Materialize one normalized literal as a compile-stage value.
///
/// The syntax crate currently exposes integer, float, and string literal
/// kinds. `uint` and `buffer` remain atomic builtin identities available to
/// later literal/form constructors; this function does not guess either from
/// an existing syntax kind.
pub fn materialize_literal_value(
    expr: &NormExpr,
    builtin_types: &AtomicBuiltinTypeIds,
    id: SemanticValueId,
    provenance: Provenance,
) -> Result<LiteralValue, LiteralMaterializationFailure> {
    let NormExpr::Literal { kind, text, .. } = expr else {
        return Err(LiteralMaterializationFailure::NotLiteral);
    };
    let builtin_type = match kind {
        NormLiteralKind::Int => AtomicBuiltinType::Int,
        NormLiteralKind::Float => AtomicBuiltinType::Float,
        NormLiteralKind::String => AtomicBuiltinType::Str,
    };
    Ok(LiteralValue {
        id,
        kind: *kind,
        text: text.clone(),
        builtin_type,
        type_value: builtin_types.get(builtin_type),
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
