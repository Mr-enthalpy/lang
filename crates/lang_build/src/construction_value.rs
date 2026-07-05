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
/// `==` (`PartialEq`) compares constructor identity and payload only;
/// provenance is excluded from semantic equality. Two values
/// reconstructed from the same constructor and payload but with
/// different provenance strings compare equal.
///
/// For exact object-identity comparison including provenance, use
/// [`ConstructedValue::exact_eq_with_provenance`].
///
/// # Integration test gap
///
/// The `(t inner) |> struct` group-lift path through the struct
/// decoder is not yet tested end-to-end through this substrate.
/// Future work must verify that the decoder reads `(t inner)` as a
/// group-lifted source-product leaf entry, not as a raw unary Product.
#[derive(Clone, Debug, Eq)]
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

impl PartialEq for ConstructedValue {
    fn eq(&self, other: &Self) -> bool {
        self.semantic_eq(other)
    }
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
    /// This is also what `==` (`PartialEq`) delegates to.
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

    /// Exact object-identity comparison including provenance.
    ///
    /// Returns true only if constructor identity, payload, AND
    /// provenance all match. Use for debugging or exact identity
    /// checks; use `==` (or [`semantic_eq`]) for semantic equality.
    pub fn exact_eq_with_provenance(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ConstructedValue::Owner {
                    owner_type_symbol_id: o1,
                    payload: p1,
                    provenance: prov1,
                },
                ConstructedValue::Owner {
                    owner_type_symbol_id: o2,
                    payload: p2,
                    provenance: prov2,
                },
            ) => o1 == o2 && p1.exact_eq_with_provenance(p2) && prov1 == prov2,
            (
                ConstructedValue::Field {
                    owner_type_symbol_id: o1,
                    field_name: f1,
                    field_type_symbol_id: t1,
                    projection: pr1,
                    payload: p1,
                    provenance: prov1,
                },
                ConstructedValue::Field {
                    owner_type_symbol_id: o2,
                    field_name: f2,
                    field_type_symbol_id: t2,
                    projection: pr2,
                    payload: p2,
                    provenance: prov2,
                },
            ) => {
                o1 == o2
                    && f1 == f2
                    && t1 == t2
                    && pr1 == pr2
                    && p1.exact_eq_with_provenance(p2)
                    && prov1 == prov2
            }
            (
                ConstructedValue::Leaf {
                    value: v1,
                    provenance: prov1,
                },
                ConstructedValue::Leaf {
                    value: v2,
                    provenance: prov2,
                },
            ) => v1 == v2 && prov1 == prov2,
            _ => false,
        }
    }

    /// Unwrap to the innermost leaf MetaInvocationValue for lowering.
    ///
    /// **Not question-view semantics.** This is an internal inspection
    /// / lowering helper. It recursively peels all Owner and Field
    /// layers. It must NOT be used by equality, pattern matching, or
    /// ordinary extraction — those must use [`constructed_question_view`]
    /// which peels exactly one layer.
    pub fn into_leaf_value_for_lowering(self) -> MetaInvocationValue {
        match self {
            ConstructedValue::Owner { payload, .. } | ConstructedValue::Field { payload, .. } => {
                payload.into_leaf_value_for_lowering()
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

/// Report whether `?` would peel one layer of this ConstructedValue.
///
/// Returns true for Owner and Field (non-leaf extraction interface).
/// Returns false for Leaf (`?` is idempotent — the leaf value itself
/// is the result).
pub fn question_view_peels(cv: &ConstructedValue) -> bool {
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
