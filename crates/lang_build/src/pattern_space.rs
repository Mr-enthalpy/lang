//! Sum-pattern space and type-pattern expression shape substrate.
//!
//! This module provides shape-level representations for:
//! - product/sum type-pattern expressions (Leaf, Product, Sum, Named)
//! - closed sum pattern spaces (if | else, Some | None, etc.)
//! - selected sum patterns (one chosen branch)
//!
//! It does not parse surface syntax, execute `struct` meta-functions, or
//! install symbols into the namespace graph. It is a pure shape substrate.

use crate::{
    extraction_view::{
        ExposedExtractionInterface, NamedProductExtractionShape, ProductNormalFormElem,
        ProductNormalFormKind, ProductNormalFormShape, ValuePointKind, ValuePointShape,
    },
    model::{Diagnostic, DiagnosticSeverity, Provenance},
};

// ---------------------------------------------------------------------------
// Symbol path shape
// ---------------------------------------------------------------------------

/// Lightweight path for external type-symbol lookups inside type-pattern
/// expressions (e.g. `uint8` in `uint8 a`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolPathShape {
    pub segments: Vec<String>,
}

impl SymbolPathShape {
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    pub fn single(segment: impl Into<String>) -> Self {
        Self {
            segments: vec![segment.into()],
        }
    }
}

// ---------------------------------------------------------------------------
// Struct leaf type expression shape
// ---------------------------------------------------------------------------

/// The type-side expression of a struct leaf field.
///
/// A leaf's left side is not restricted to a simple type path. It may be a
/// type expression such as `int Vec` in `int Vec a`. The simplest case is a
/// path like `uint8`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructLeafTypeExprShape {
    /// A simple type path (e.g. `uint8`, `Vec::std`).
    Path(SymbolPathShape),

    /// A type expression that the decoder cannot fully reduce to a simple
    /// path at this stage. Carries a debug description and provenance for
    /// diagnostics.
    NormalizedAst {
        description: String,
        provenance: Provenance,
    },
}

impl From<SymbolPathShape> for StructLeafTypeExprShape {
    fn from(p: SymbolPathShape) -> Self {
        Self::Path(p)
    }
}

impl StructLeafTypeExprShape {
    pub fn path(path: SymbolPathShape) -> Self {
        Self::Path(path)
    }
}

// ---------------------------------------------------------------------------
// Type-pattern expression shape
// ---------------------------------------------------------------------------

/// Shape-level representation of a product/sum type-pattern expression.
///
/// Naming convention:
/// - Leaf `external_type_expr` — type expression needing external resolution
///   (e.g. `uint8` in `uint8 a`, or `int Vec` in `int Vec a`)
/// - Leaf `local_pattern_name` — local field/payload name within this
///   type-pattern expression (e.g. `a`)
/// - Named `pattern_name` — pattern/constructor name at the current
///   construction layer, not looked up externally
/// - Outer `let` binding name — the type symbol installed into the symbol
///   graph; distinct from any inner pattern/construction name
///
/// `,` is product / `*`, `|` is the canonical sum-pattern result form.
/// `+` is a pattern-combination / reduction action, not a canonical sum
/// form. Parenthesised sub-expressions are same-level children. The parent
/// name appears on the right, child structure on the left:
/// `(child_structure parent_pattern_name)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypePatternExprShape {
    /// Leaf field: `external_type_expr local_pattern_name`.
    /// Example: `uint8 a` (lookup `uint8` externally, bind `a` locally).
    /// Example: `int Vec a` (type expression `int Vec`, local name `a`).
    Leaf {
        external_type_expr: StructLeafTypeExprShape,
        local_pattern_name: String,
        provenance: Provenance,
    },

    /// Product of elements: `(elem1, elem2, ...)`.
    /// Example: `(uint8 a, uint8 b)`.
    Product {
        elements: Vec<TypePatternExprShape>,
        provenance: Provenance,
    },

    /// Sum of alternatives: `alt1 | alt2 | ...`.
    /// Example: `(uint8 a, uint8 b) Some | None`.
    Sum {
        alternatives: Vec<TypePatternExprShape>,
        provenance: Provenance,
    },

    /// Named construction: `(child_structure pattern_name)`.
    /// Example: `((uint8 a, uint8 b) mytype)` — `mytype` is the
    /// pattern/constructor name, not the externally bound symbol.
    Named {
        child: Box<TypePatternExprShape>,
        pattern_name: String,
        provenance: Provenance,
    },
}

impl TypePatternExprShape {
    pub fn leaf(
        external_type_expr: StructLeafTypeExprShape,
        local_pattern_name: impl Into<String>,
        provenance: Provenance,
    ) -> Self {
        Self::Leaf {
            external_type_expr,
            local_pattern_name: local_pattern_name.into(),
            provenance,
        }
    }

    pub fn product(elements: Vec<TypePatternExprShape>, provenance: Provenance) -> Self {
        Self::Product {
            elements,
            provenance,
        }
    }

    pub fn sum(alternatives: Vec<TypePatternExprShape>, provenance: Provenance) -> Self {
        Self::Sum {
            alternatives,
            provenance,
        }
    }

    pub fn named(
        child: TypePatternExprShape,
        pattern_name: impl Into<String>,
        provenance: Provenance,
    ) -> Self {
        Self::Named {
            child: Box::new(child),
            pattern_name: pattern_name.into(),
            provenance,
        }
    }
}

// ---------------------------------------------------------------------------
// Sum pattern space shape
// ---------------------------------------------------------------------------

/// A closed sum pattern space: a set of mutually exclusive branch
/// alternatives.
///
/// Examples:
/// - `if | else`
/// - `Some | None`
/// - `Ok | Err`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SumPatternSpaceShape {
    pub alternatives: Vec<SumPatternAlternative>,
    pub provenance: Provenance,
}

/// One alternative inside a closed sum pattern space.
///
/// Each alternative has a label (branch name), an optional payload shape,
/// and provenance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SumPatternAlternative {
    pub label: String,
    pub payload_shape: Option<SumPatternPayloadShape>,
    pub provenance: Provenance,
}

/// The payload shape carried by a sum-pattern alternative.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SumPatternPayloadShape {
    Unit,
    ValuePoint,
    Product(ProductNormalFormShape),
    NamedProduct(NamedProductExtractionShape),
}

// ---------------------------------------------------------------------------
// Selected sum pattern
// ---------------------------------------------------------------------------

/// One selected branch from a closed sum pattern space.
///
/// Used as the selector in guarded-branch evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedSumPattern {
    pub space: SumPatternSpaceShape,
    pub selected_label: String,
    pub payload: Option<crate::extraction_view::EvalResultNormalForm>,
    pub provenance: Provenance,
}

impl SelectedSumPattern {
    /// Validate that `selected_label` belongs to `self.space`.
    pub fn validate(&self) -> Result<(), Diagnostic> {
        let found = self
            .space
            .alternatives
            .iter()
            .any(|alt| alt.label == self.selected_label);
        if found {
            Ok(())
        } else {
            Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                format!(
                    "selected label `{}` is not an alternative in the sum pattern space",
                    self.selected_label
                ),
                Some(self.provenance.clone()),
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// Derive sum pattern space from type-pattern expression
// ---------------------------------------------------------------------------

/// Derive a `SumPatternSpaceShape` from a `TypePatternExprShape`.
///
/// Rules:
/// - `Sum([...])` → alternatives from each direct alternative.
/// - `Named { child: Sum([...]), pattern_name }` → recurse into child; outer
///   `pattern_name` is the enclosing type-pattern name.
/// - `Named { child: Product(elems), pattern_name }` → one constructor
///   alternative with product payload.
/// - `Named { child: Leaf { .. }, pattern_name }` → one constructor alternative
///   with ValuePoint payload.
/// - `Product(...)` → `None` (not a sum pattern space by itself).
/// - `Leaf { .. }` → `None` (not a sum pattern space by itself).
pub fn derive_sum_pattern_space(expr: &TypePatternExprShape) -> Option<SumPatternSpaceShape> {
    match expr {
        TypePatternExprShape::Sum {
            alternatives,
            provenance,
        } => {
            let alts: Vec<SumPatternAlternative> = alternatives
                .iter()
                .filter_map(alt_to_sum_alternative)
                .collect();
            if alts.is_empty() {
                None
            } else {
                Some(SumPatternSpaceShape {
                    alternatives: alts,
                    provenance: provenance.clone(),
                })
            }
        }
        TypePatternExprShape::Named {
            child,
            pattern_name,
            provenance,
        } => match child.as_ref() {
            TypePatternExprShape::Sum {
                alternatives,
                provenance: child_prov,
            } => {
                let alts: Vec<SumPatternAlternative> = alternatives
                    .iter()
                    .filter_map(alt_to_sum_alternative)
                    .collect();
                if alts.is_empty() {
                    None
                } else {
                    Some(SumPatternSpaceShape {
                        alternatives: alts,
                        provenance: child_prov.clone(),
                    })
                }
            }
            TypePatternExprShape::Product {
                elements,
                provenance: child_prov,
            } => {
                let payload =
                    SumPatternPayloadShape::Product(product_payload_from_elements(elements));
                Some(SumPatternSpaceShape {
                    alternatives: vec![SumPatternAlternative {
                        label: pattern_name.clone(),
                        payload_shape: Some(payload),
                        provenance: provenance.clone(),
                    }],
                    provenance: child_prov.clone(),
                })
            }
            TypePatternExprShape::Leaf {
                local_pattern_name: _,
                provenance: child_prov,
                ..
            } => {
                // Named leaf is one constructor alternative with ValuePoint payload
                Some(SumPatternSpaceShape {
                    alternatives: vec![SumPatternAlternative {
                        label: pattern_name.clone(),
                        payload_shape: Some(SumPatternPayloadShape::ValuePoint),
                        provenance: provenance.clone(),
                    }],
                    provenance: child_prov.clone(),
                })
            }
            // Named(Named(...)) or nested Named — recurse
            _non_leaf => {
                let inner = derive_sum_pattern_space(child)?;
                Some(SumPatternSpaceShape {
                    alternatives: inner.alternatives,
                    provenance: inner.provenance,
                })
            }
        },
        TypePatternExprShape::Product {
            provenance: _,
            elements: _,
        } => None,
        TypePatternExprShape::Leaf { .. } => None,
    }
}

/// Convert a `TypePatternExprShape` alternative into a `SumPatternAlternative`.
/// Returns `None` for variants that cannot be alternatives (bare Product, bare Leaf).
fn alt_to_sum_alternative(alt: &TypePatternExprShape) -> Option<SumPatternAlternative> {
    match alt {
        TypePatternExprShape::Named {
            child,
            pattern_name,
            provenance,
        } => {
            let payload = match child.as_ref() {
                TypePatternExprShape::Product { elements, .. } => Some(
                    SumPatternPayloadShape::Product(product_payload_from_elements(elements)),
                ),
                TypePatternExprShape::Leaf { .. } => Some(SumPatternPayloadShape::ValuePoint),
                _ => None,
            };
            Some(SumPatternAlternative {
                label: pattern_name.clone(),
                payload_shape: payload,
                provenance: provenance.clone(),
            })
        }
        TypePatternExprShape::Leaf { .. } => {
            // A bare leaf is a local field/payload name, not a sum alternative
            // label. Only Named { child: Leaf, pattern_name } can become a
            // sum alternative — the pattern_name serves as the alternative
            // label.
            None
        }
        TypePatternExprShape::Sum { .. } => None,
        TypePatternExprShape::Product { .. } => None,
    }
}

/// Build a `ProductNormalFormShape` from type-pattern expression elements.
/// Each leaf becomes a labelled element; empty product → nullary product.
fn product_payload_from_elements(elements: &[TypePatternExprShape]) -> ProductNormalFormShape {
    let mut product_elems: Vec<ProductNormalFormElem> = Vec::new();
    let mut provenance = Provenance::new("product payload");

    for elem in elements {
        match elem {
            TypePatternExprShape::Leaf {
                local_pattern_name,
                provenance: p,
                ..
            } => {
                provenance = p.clone();
                product_elems.push(ProductNormalFormElem {
                    label: Some(local_pattern_name.clone()),
                    value_shape: Box::new(
                        crate::extraction_view::EvalResultNormalForm::ValuePoint(ValuePointShape {
                            value_kind: ValuePointKind::Leaf,
                            extraction_interface: ExposedExtractionInterface::Leaf,
                            provenance: p.clone(),
                        }),
                    ),
                    type_symbol_id: None,
                    provenance: p.clone(),
                });
            }
            _ => {
                // For non-leaf elements (nested Product/Named/Sum), create an
                // opaque product element with no label.
                product_elems.push(ProductNormalFormElem {
                    label: None,
                    value_shape: Box::new(
                        crate::extraction_view::EvalResultNormalForm::ValuePoint(ValuePointShape {
                            value_kind: ValuePointKind::Leaf,
                            extraction_interface: ExposedExtractionInterface::Leaf,
                            provenance: Provenance::new("nested pattern element"),
                        }),
                    ),
                    type_symbol_id: None,
                    provenance: Provenance::new("nested pattern element"),
                });
            }
        }
    }

    ProductNormalFormShape {
        elements: product_elems,
        product_kind: ProductNormalFormKind::Bare,
        provenance,
    }
}

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

/// Build the `if | else` bool branch space from a type-pattern expression.
///
/// Semantics:
/// ```lang
/// let bool: type = ((if | else) bool) |> struct;
/// ```
///
/// The first `bool` is the external symbol being bound. The second `bool`
/// is the pattern/construction name attached to the sum pattern `if | else`.
///
/// This function constructs the inner type-pattern expression and derives
/// the sum pattern space from it (not hand-built).
pub fn bool_branch_space_for_tests(provenance: Provenance) -> SumPatternSpaceShape {
    let if_alt = TypePatternExprShape::named(
        TypePatternExprShape::product(vec![], Provenance::new("if payload")),
        "if",
        Provenance::new("if branch"),
    );
    let else_alt = TypePatternExprShape::named(
        TypePatternExprShape::product(vec![], Provenance::new("else payload")),
        "else",
        Provenance::new("else branch"),
    );
    let sum = TypePatternExprShape::sum(vec![if_alt, else_alt], Provenance::new("if | else sum"));
    let bool_expr = TypePatternExprShape::named(sum, "bool", provenance);

    derive_sum_pattern_space(&bool_expr)
        .expect("bool type-pattern expression must derive a valid sum pattern space")
}
