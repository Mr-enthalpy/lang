use crate::{
    meta_invocation::{ConstructionInstanceId, MetaInvocationValue},
    model::{FieldProjection, Provenance, SymbolId},
};

/// Semantic construction substrate for constructed values.
///
/// Represents a value produced by a constructor (owner constructor,
/// field-pattern constructor, or generated construction). Each layer
/// wraps a payload that can be peeled by `?` or reconstructed by
/// applying the constructor to a payload.
///
/// This is a pattern semantic substrate, not a pattern-space calculus.
/// D/Done, packs, borrow/access-tree, and layout are not implemented.
///
/// # Equality
///
/// `PartialEq` (the `==` operator) compares the full structural value
/// including `Provenance`. Two values reconstructed from the same
/// constructor and payload but with different provenance strings will
/// compare unequal under `==`.
///
/// For semantic roundtrip verification (e.g., construct → peel →
/// reconstruct), use [`ConstructedValue::semantic_eq`] which compares
/// only constructor identity and payload, ignoring provenance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstructedValue {
    /// Owner wrapper: `(payload TB)` where TB is an owner type.
    /// Question view peels to the payload.
    Owner {
        /// Placeholder type identity. v0.9 uses SymbolId; future must
        /// distinguish TypeValueId, PlaceId, and SymbolId.
        owner_type_symbol_id: SymbolId,
        payload: Box<ConstructedValue>,
        provenance: Provenance,
    },

    /// Field-pattern wrapper: `(payload field::TB)`.
    /// Question view peels to the payload.
    Field {
        /// Placeholder type identity. v0.9 uses SymbolId; future must
        /// distinguish TypeValueId, PlaceId, and SymbolId.
        owner_type_symbol_id: SymbolId,
        field_name: String,
        /// Placeholder field type identity. Future: TypeValueId.
        field_type_symbol_id: SymbolId,
        projection: FieldProjection,
        payload: Box<ConstructedValue>,
        provenance: Provenance,
    },

    /// Leaf value wrapping a MetaInvocationValue.
    /// Question view is idempotent.
    Leaf {
        value: MetaInvocationValue,
        provenance: Provenance,
    },
}

/// Constructor head — identifies how to reconstruct a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstructorHead {
    /// Reconstructs an owner value from a field-pattern payload.
    Owner { owner_type_symbol_id: SymbolId },

    /// Reconstructs a field-pattern value from a leaf payload.
    Field {
        owner_type_symbol_id: SymbolId,
        field_name: String,
        field_type_symbol_id: SymbolId,
        projection: FieldProjection,
    },

    /// Generated construction identity (unary construction prototype).
    Generated {
        construction_instance_id: ConstructionInstanceId,
    },
}

impl ConstructedValue {
    /// Extract the constructor head for this constructed value.
    pub fn constructor_head(&self) -> Option<ConstructorHead> {
        match self {
            ConstructedValue::Owner {
                owner_type_symbol_id,
                ..
            } => Some(ConstructorHead::Owner {
                owner_type_symbol_id: *owner_type_symbol_id,
            }),
            ConstructedValue::Field {
                owner_type_symbol_id,
                field_name,
                field_type_symbol_id,
                projection,
                ..
            } => Some(ConstructorHead::Field {
                owner_type_symbol_id: *owner_type_symbol_id,
                field_name: field_name.clone(),
                field_type_symbol_id: *field_type_symbol_id,
                projection: *projection,
            }),
            ConstructedValue::Leaf { .. } => None,
        }
    }

    /// The provenance of this constructed value.
    pub fn provenance(&self) -> &Provenance {
        match self {
            ConstructedValue::Owner { provenance, .. }
            | ConstructedValue::Field { provenance, .. }
            | ConstructedValue::Leaf { provenance, .. } => provenance,
        }
    }

    /// Semantic equality: compares constructor identity and payload
    /// without considering provenance.
    ///
    /// Two values reconstructed from the same constructor and payload
    /// but with different provenance strings are semantically equal.
    /// This is the correct comparison for roundtrip verification.
    pub fn semantic_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ConstructedValue::Owner {
                    owner_type_symbol_id: o1,
                    payload: p1,
                    ..
                },
                ConstructedValue::Owner {
                    owner_type_symbol_id: o2,
                    payload: p2,
                    ..
                },
            ) => o1 == o2 && p1.semantic_eq(p2),
            (
                ConstructedValue::Field {
                    owner_type_symbol_id: o1,
                    field_name: f1,
                    field_type_symbol_id: t1,
                    projection: pr1,
                    payload: p1,
                    ..
                },
                ConstructedValue::Field {
                    owner_type_symbol_id: o2,
                    field_name: f2,
                    field_type_symbol_id: t2,
                    projection: pr2,
                    payload: p2,
                    ..
                },
            ) => o1 == o2 && f1 == f2 && t1 == t2 && pr1 == pr2 && p1.semantic_eq(p2),
            (
                ConstructedValue::Leaf { value: v1, .. },
                ConstructedValue::Leaf { value: v2, .. },
            ) => v1 == v2,
            _ => false,
        }
    }

    /// Unwrap to the innermost leaf MetaInvocationValue.
    ///
    /// **Not question-view semantics.** This is an internal inspection
    /// / lowering helper. It recursively peels all Owner and Field
    /// layers. It must NOT be used by equality, pattern matching, or
    /// ordinary extraction — those must use [`constructed_question_view`]
    /// which peels exactly one layer.
    pub fn into_leaf_value(self) -> MetaInvocationValue {
        match self {
            ConstructedValue::Owner { payload, .. } | ConstructedValue::Field { payload, .. } => {
                payload.into_leaf_value()
            }
            ConstructedValue::Leaf { value, .. } => value,
        }
    }
}

/// Construct an owner value from a field-pattern payload.
pub fn construct_owner_value(
    owner_type_symbol_id: SymbolId,
    payload: ConstructedValue,
    provenance: Provenance,
) -> ConstructedValue {
    ConstructedValue::Owner {
        owner_type_symbol_id,
        payload: Box::new(payload),
        provenance,
    }
}

/// Construct a field-pattern value from a leaf payload.
pub fn construct_field_value(
    owner_type_symbol_id: SymbolId,
    field_name: String,
    field_type_symbol_id: SymbolId,
    projection: FieldProjection,
    payload: ConstructedValue,
    provenance: Provenance,
) -> ConstructedValue {
    ConstructedValue::Field {
        owner_type_symbol_id,
        field_name,
        field_type_symbol_id,
        projection,
        payload: Box::new(payload),
        provenance,
    }
}

/// Wrap a MetaInvocationValue as a leaf constructed value.
pub fn leaf_value(value: MetaInvocationValue, provenance: Provenance) -> ConstructedValue {
    ConstructedValue::Leaf { value, provenance }
}

/// Apply one-step `?` view to a ConstructedValue.
///
/// Rules:
///   Owner(wrapping field-pattern F) → F
///   Field(payload P) → P
///   Leaf(V) → Leaf(V)  (idempotent)
pub fn constructed_question_view(cv: &ConstructedValue) -> ConstructedValue {
    match cv {
        ConstructedValue::Owner { payload, .. } => (**payload).clone(),
        ConstructedValue::Field { payload, .. } => (**payload).clone(),
        ConstructedValue::Leaf { .. } => cv.clone(),
    }
}

/// Report whether this ConstructedValue exposes a non-leaf extraction interface
/// (i.e., `?` would peel a layer).
pub fn has_question_view(cv: &ConstructedValue) -> bool {
    matches!(
        cv,
        ConstructedValue::Owner { .. } | ConstructedValue::Field { .. }
    )
}

/// Restricted v0.9 placeholder for field constructor head.
///
/// Does NOT query the namespace graph. Returns a `ConstructorHead::Field`
/// with the given metadata directly. Future implementation must use a
/// role-aware resolver under the owner type's companion namespace.
pub fn placeholder_field_constructor_head(
    owner_type_symbol_id: SymbolId,
    field_name: &str,
    field_type_symbol_id: SymbolId,
    projection: FieldProjection,
) -> ConstructorHead {
    ConstructorHead::Field {
        owner_type_symbol_id,
        field_name: field_name.to_string(),
        field_type_symbol_id,
        projection,
    }
}

/// Restricted v0.9 placeholder for owner constructor head.
///
/// Does NOT query the namespace graph. Future implementation must use
/// a role-aware resolver.
pub fn placeholder_owner_constructor_head(owner_type_symbol_id: SymbolId) -> ConstructorHead {
    ConstructorHead::Owner {
        owner_type_symbol_id,
    }
}
