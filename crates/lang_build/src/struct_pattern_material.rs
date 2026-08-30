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
    content_observation::{
        ContentObservationInterface, NamedObservedProduct, ObservedAtomContent, ObservedAtomKind,
        ObservedProductContent, ObservedProductElement, ObservedProductKind,
    },
    model::{Diagnostic, DiagnosticSeverity, Provenance},
};

// ---------------------------------------------------------------------------
// Symbol path shape
// ---------------------------------------------------------------------------

/// Lightweight path for external type-symbol lookups inside type-pattern
/// expressions (e.g. `uint8` in `uint8 a`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructSymbolPathMaterial {
    pub segments: Vec<String>,
}

impl StructSymbolPathMaterial {
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
pub enum StructLeafSyntaxMaterial {
    /// A simple type path (e.g. `uint8`, `Vec::std`).
    Path(StructSymbolPathMaterial),

    /// A type expression that the decoder cannot fully reduce to a simple
    /// path at this stage. Carries a debug description and provenance for
    /// diagnostics.
    NormalizedAst {
        description: String,
        provenance: Provenance,
    },
}

impl From<StructSymbolPathMaterial> for StructLeafSyntaxMaterial {
    fn from(p: StructSymbolPathMaterial) -> Self {
        Self::Path(p)
    }
}

impl StructLeafSyntaxMaterial {
    pub fn path(path: StructSymbolPathMaterial) -> Self {
        Self::Path(path)
    }

    /// Semantic equality: compares structural identity without provenance.
    pub fn materially_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Path(a), Self::Path(b)) => a == b,
            (
                Self::NormalizedAst { description: a, .. },
                Self::NormalizedAst { description: b, .. },
            ) => a == b,
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Type-pattern expression shape
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StructuralMemberVisibility {
    Default,
    Public,
    Private,
}

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
pub enum StructPatternSyntaxMaterial {
    /// Leaf field: `external_type_expr local_pattern_name`.
    /// Example: `uint8 a` (lookup `uint8` externally, bind `a` locally).
    /// Example: `int Vec a` (type expression `int Vec`, local name `a`).
    Leaf {
        external_type_expr: StructLeafSyntaxMaterial,
        local_pattern_name: String,
        visibility: StructuralMemberVisibility,
        provenance: Provenance,
    },

    /// Product of elements: `(elem1, elem2, ...)`.
    /// Example: `(uint8 a, uint8 b)`.
    Product {
        elements: Vec<StructPatternSyntaxMaterial>,
        provenance: Provenance,
    },

    /// Sum of alternatives: `alt1 | alt2 | ...`.
    /// Example: `(uint8 a, uint8 b) Some | None`.
    Sum {
        alternatives: Vec<StructPatternSyntaxMaterial>,
        provenance: Provenance,
    },

    /// Named construction: `(child_structure pattern_name)`.
    /// Example: `((uint8 a, uint8 b) mytype)` — `mytype` is the
    /// pattern/constructor name, not the externally bound symbol.
    Named {
        child: Box<StructPatternSyntaxMaterial>,
        pattern_name: String,
        visibility: StructuralMemberVisibility,
        provenance: Provenance,
    },
}

impl StructPatternSyntaxMaterial {
    /// Ignore transparent singleton Product wrappers introduced by the
    /// invocation argument parentheses.  This does not erase a Product with
    /// zero or multiple children.
    pub fn transparent_singleton(&self) -> &Self {
        let mut current = self;
        loop {
            match current {
                Self::Product { elements, .. } if elements.len() == 1 => {
                    current = &elements[0];
                }
                _ => return current,
            }
        }
    }

    pub fn top_pattern_name(&self) -> Option<&str> {
        match self.transparent_singleton() {
            Self::Named { pattern_name, .. } => Some(pattern_name),
            _ => None,
        }
    }

    pub fn is_named_empty_pattern(&self) -> bool {
        matches!(
            self.transparent_singleton(),
            Self::Named { child, .. }
                if matches!(
                    child.transparent_singleton(),
                    Self::Product { elements, .. } if elements.is_empty()
                )
        )
    }

    /// Whether this decoded shape contains Pattern identity but no value
    /// leaves.
    ///
    /// A bare name is decoded as `Named(Product[], name)`.  Sums such as
    /// `if | else` are therefore pure no-value Pattern structures rather than
    /// zero-arity field products.  An anonymous empty Product alone has no
    /// Pattern identity and does not satisfy this predicate.
    pub fn is_pure_pattern_without_value(&self) -> bool {
        fn facts(pattern: &StructPatternSyntaxMaterial) -> (bool, bool) {
            match pattern {
                StructPatternSyntaxMaterial::Leaf { .. } => (true, false),
                StructPatternSyntaxMaterial::Product { elements, .. } => {
                    elements.iter().fold((false, false), |acc, element| {
                        let next = facts(element);
                        (acc.0 || next.0, acc.1 || next.1)
                    })
                }
                StructPatternSyntaxMaterial::Sum { alternatives, .. } => {
                    alternatives
                        .iter()
                        .fold((false, false), |acc, alternative| {
                            let next = facts(alternative);
                            (acc.0 || next.0, acc.1 || next.1)
                        })
                }
                StructPatternSyntaxMaterial::Named { child, .. } => {
                    let child = facts(child);
                    (child.0, true)
                }
            }
        }

        let (has_value_leaf, has_pattern_name) = facts(self);
        !has_value_leaf && has_pattern_name
    }

    pub fn leaf(
        external_type_expr: StructLeafSyntaxMaterial,
        local_pattern_name: impl Into<String>,
        provenance: Provenance,
    ) -> Self {
        Self::Leaf {
            external_type_expr,
            local_pattern_name: local_pattern_name.into(),
            visibility: StructuralMemberVisibility::Default,
            provenance,
        }
    }

    pub fn product(elements: Vec<StructPatternSyntaxMaterial>, provenance: Provenance) -> Self {
        Self::Product {
            elements,
            provenance,
        }
    }

    pub fn sum(alternatives: Vec<StructPatternSyntaxMaterial>, provenance: Provenance) -> Self {
        Self::Sum {
            alternatives,
            provenance,
        }
    }

    pub fn named(
        child: StructPatternSyntaxMaterial,
        pattern_name: impl Into<String>,
        provenance: Provenance,
    ) -> Self {
        Self::Named {
            child: Box::new(child),
            pattern_name: pattern_name.into(),
            visibility: StructuralMemberVisibility::Default,
            provenance,
        }
    }

    pub fn with_structural_visibility(mut self, visibility: StructuralMemberVisibility) -> Self {
        match &mut self {
            Self::Leaf {
                visibility: member_visibility,
                ..
            }
            | Self::Named {
                visibility: member_visibility,
                ..
            } => *member_visibility = visibility,
            Self::Product { .. } | Self::Sum { .. } => {}
        }
        self
    }

    /// Semantic equality: compares structural identity without provenance.
    pub fn materially_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (
                StructPatternSyntaxMaterial::Leaf {
                    external_type_expr: e1,
                    local_pattern_name: n1,
                    visibility: v1,
                    ..
                },
                StructPatternSyntaxMaterial::Leaf {
                    external_type_expr: e2,
                    local_pattern_name: n2,
                    visibility: v2,
                    ..
                },
            ) => e1.materially_equal(e2) && n1 == n2 && v1 == v2,
            (
                StructPatternSyntaxMaterial::Product { elements: es1, .. },
                StructPatternSyntaxMaterial::Product { elements: es2, .. },
            ) => {
                es1.len() == es2.len()
                    && es1
                        .iter()
                        .zip(es2.iter())
                        .all(|(a, b)| a.materially_equal(b))
            }
            (
                StructPatternSyntaxMaterial::Sum {
                    alternatives: as1, ..
                },
                StructPatternSyntaxMaterial::Sum {
                    alternatives: as2, ..
                },
            ) => {
                as1.len() == as2.len()
                    && as1
                        .iter()
                        .zip(as2.iter())
                        .all(|(a, b)| a.materially_equal(b))
            }
            (
                StructPatternSyntaxMaterial::Named {
                    child: c1,
                    pattern_name: n1,
                    visibility: v1,
                    ..
                },
                StructPatternSyntaxMaterial::Named {
                    child: c2,
                    pattern_name: n2,
                    visibility: v2,
                    ..
                },
            ) => c1.materially_equal(c2) && n1 == n2 && v1 == v2,
            _ => false,
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
pub struct StructSumSyntaxMaterial {
    pub alternatives: Vec<StructSumAlternative>,
    pub provenance: Provenance,
}

impl StructSumSyntaxMaterial {
    /// Semantic equality: compares alternatives without provenance.
    pub fn materially_equal(&self, other: &Self) -> bool {
        self.alternatives.len() == other.alternatives.len()
            && self
                .alternatives
                .iter()
                .zip(other.alternatives.iter())
                .all(|(a, b)| a.materially_equal(b))
    }
}

impl StructSumAlternative {
    /// Semantic equality: compares label and payload without provenance.
    pub fn materially_equal(&self, other: &Self) -> bool {
        self.label == other.label
            && sum_payload_shape_materially_equal(&self.payload_shape, &other.payload_shape)
    }
}

fn sum_payload_shape_materially_equal(
    lhs: &Option<StructSumPayloadMaterial>,
    rhs: &Option<StructSumPayloadMaterial>,
) -> bool {
    match (lhs, rhs) {
        (Some(l), Some(r)) => match (l, r) {
            (StructSumPayloadMaterial::Unit, StructSumPayloadMaterial::Unit) => true,
            (StructSumPayloadMaterial::ValuePoint, StructSumPayloadMaterial::ValuePoint) => true,
            (StructSumPayloadMaterial::Product(p1), StructSumPayloadMaterial::Product(p2)) => {
                p1.observationally_equal(p2)
            }
            (
                StructSumPayloadMaterial::NamedProduct(n1),
                StructSumPayloadMaterial::NamedProduct(n2),
            ) => n1.observationally_equal(n2),
            _ => false,
        },
        (None, None) => true,
        _ => false,
    }
}

/// One alternative inside a closed sum pattern space.
///
/// Each alternative has a label (branch name), an optional payload shape,
/// and provenance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructSumAlternative {
    pub label: String,
    pub payload_shape: Option<StructSumPayloadMaterial>,
    pub provenance: Provenance,
}

/// The payload shape carried by a sum-pattern alternative.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructSumPayloadMaterial {
    Unit,
    ValuePoint,
    Product(ObservedProductContent),
    NamedProduct(NamedObservedProduct),
}

// ---------------------------------------------------------------------------
// Selected sum pattern
// ---------------------------------------------------------------------------

/// One selected branch from a closed sum pattern space.
///
/// Used as the selector in guarded-branch evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedStructAlternative {
    pub space: StructSumSyntaxMaterial,
    pub selected_label: String,
    pub payload: Option<crate::content_observation::ObservedArgumentContent>,
    pub provenance: Provenance,
}

impl SelectedStructAlternative {
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

/// Derive a `StructSumSyntaxMaterial` from a `StructPatternSyntaxMaterial`.
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
pub fn derive_struct_sum_material(
    expr: &StructPatternSyntaxMaterial,
) -> Option<StructSumSyntaxMaterial> {
    let expr = expr.transparent_singleton();
    match expr {
        StructPatternSyntaxMaterial::Sum {
            alternatives,
            provenance,
        } => {
            let alts: Vec<StructSumAlternative> = alternatives
                .iter()
                .filter_map(alt_to_sum_alternative)
                .collect();
            if alts.is_empty() {
                None
            } else {
                Some(StructSumSyntaxMaterial {
                    alternatives: alts,
                    provenance: provenance.clone(),
                })
            }
        }
        StructPatternSyntaxMaterial::Named {
            child,
            pattern_name,
            provenance,
            ..
        } => match child.as_ref() {
            StructPatternSyntaxMaterial::Sum {
                alternatives,
                provenance: child_prov,
            } => {
                let alts: Vec<StructSumAlternative> = alternatives
                    .iter()
                    .filter_map(alt_to_sum_alternative)
                    .collect();
                if alts.is_empty() {
                    None
                } else {
                    Some(StructSumSyntaxMaterial {
                        alternatives: alts,
                        provenance: child_prov.clone(),
                    })
                }
            }
            StructPatternSyntaxMaterial::Product {
                elements,
                provenance: child_prov,
            } => {
                let payload =
                    StructSumPayloadMaterial::Product(product_payload_from_elements(elements));
                Some(StructSumSyntaxMaterial {
                    alternatives: vec![StructSumAlternative {
                        label: pattern_name.clone(),
                        payload_shape: Some(payload),
                        provenance: provenance.clone(),
                    }],
                    provenance: child_prov.clone(),
                })
            }
            StructPatternSyntaxMaterial::Leaf {
                local_pattern_name: _,
                provenance: child_prov,
                ..
            } => {
                // Named leaf is one constructor alternative with ValuePoint payload
                Some(StructSumSyntaxMaterial {
                    alternatives: vec![StructSumAlternative {
                        label: pattern_name.clone(),
                        payload_shape: Some(StructSumPayloadMaterial::ValuePoint),
                        provenance: provenance.clone(),
                    }],
                    provenance: child_prov.clone(),
                })
            }
            // Named(Named(...)) or nested Named — recurse
            _non_leaf => {
                let inner = derive_struct_sum_material(child)?;
                Some(StructSumSyntaxMaterial {
                    alternatives: inner.alternatives,
                    provenance: inner.provenance,
                })
            }
        },
        StructPatternSyntaxMaterial::Product {
            provenance: _,
            elements: _,
        } => None,
        StructPatternSyntaxMaterial::Leaf { .. } => None,
    }
}

/// Convert a `StructPatternSyntaxMaterial` alternative into a `StructSumAlternative`.
/// Returns `None` for variants that cannot be alternatives (bare Product, bare Sum).
fn alt_to_sum_alternative(alt: &StructPatternSyntaxMaterial) -> Option<StructSumAlternative> {
    match alt {
        StructPatternSyntaxMaterial::Named {
            child,
            pattern_name,
            provenance,
            ..
        } => {
            let payload = match child.as_ref() {
                StructPatternSyntaxMaterial::Product { elements, .. } => Some(
                    StructSumPayloadMaterial::Product(product_payload_from_elements(elements)),
                ),
                StructPatternSyntaxMaterial::Leaf { .. } => {
                    Some(StructSumPayloadMaterial::ValuePoint)
                }
                _ => None,
            };
            Some(StructSumAlternative {
                label: pattern_name.clone(),
                payload_shape: payload,
                provenance: provenance.clone(),
            })
        }
        StructPatternSyntaxMaterial::Leaf {
            local_pattern_name,
            provenance,
            ..
        } => {
            // A bare leaf alternative uses its local field/payload name as
            // the sum alternative label. Example: in `uint8 a | None`,
            // `a` is the leaf pattern name and becomes a valid alternative.
            Some(StructSumAlternative {
                label: local_pattern_name.clone(),
                payload_shape: Some(StructSumPayloadMaterial::ValuePoint),
                provenance: provenance.clone(),
            })
        }
        StructPatternSyntaxMaterial::Sum { .. } => None,
        StructPatternSyntaxMaterial::Product { .. } => None,
    }
}

/// Build a `ObservedProductContent` from type-pattern expression elements.
/// Each leaf becomes a labelled element; empty product → nullary product.
fn product_payload_from_elements(
    elements: &[StructPatternSyntaxMaterial],
) -> ObservedProductContent {
    let mut product_elems: Vec<ObservedProductElement> = Vec::new();
    let mut provenance = Provenance::new("product payload");

    for elem in elements {
        match elem {
            StructPatternSyntaxMaterial::Leaf {
                local_pattern_name,
                provenance: p,
                ..
            } => {
                provenance = p.clone();
                product_elems.push(ObservedProductElement {
                    label: Some(local_pattern_name.clone()),
                    value_shape: Box::new(
                        crate::content_observation::ObservedArgumentContent::ValuePoint(
                            ObservedAtomContent {
                                value_kind: ObservedAtomKind::Leaf,
                                extraction_interface: ContentObservationInterface::Leaf,
                                provenance: p.clone(),
                            },
                        ),
                    ),
                    type_value: None,
                    type_observation: None,
                    type_symbol_id: None,
                    provenance: p.clone(),
                });
            }
            _ => {
                // For non-leaf elements (nested Product/Named/Sum), create an
                // opaque product element with no label.
                product_elems.push(ObservedProductElement {
                    label: None,
                    value_shape: Box::new(
                        crate::content_observation::ObservedArgumentContent::ValuePoint(
                            ObservedAtomContent {
                                value_kind: ObservedAtomKind::Leaf,
                                extraction_interface: ContentObservationInterface::Leaf,
                                provenance: Provenance::new("nested pattern element"),
                            },
                        ),
                    ),
                    type_value: None,
                    type_observation: None,
                    type_symbol_id: None,
                    provenance: Provenance::new("nested pattern element"),
                });
            }
        }
    }

    ObservedProductContent {
        elements: product_elems,
        product_kind: ObservedProductKind::Bare,
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
pub fn bool_struct_sum_material_for_tests(provenance: Provenance) -> StructSumSyntaxMaterial {
    let if_alt = StructPatternSyntaxMaterial::named(
        StructPatternSyntaxMaterial::product(vec![], Provenance::new("if payload")),
        "if",
        Provenance::new("if branch"),
    );
    let else_alt = StructPatternSyntaxMaterial::named(
        StructPatternSyntaxMaterial::product(vec![], Provenance::new("else payload")),
        "else",
        Provenance::new("else branch"),
    );
    let sum =
        StructPatternSyntaxMaterial::sum(vec![if_alt, else_alt], Provenance::new("if | else sum"));
    let bool_expr = StructPatternSyntaxMaterial::named(sum, "bool", provenance);

    derive_struct_sum_material(&bool_expr)
        .expect("bool type-pattern expression must derive a valid sum pattern space")
}

/// Alias facts for the two ordinary value names associated with the bool
/// Pattern symbols. They do not add alternatives to the bool Pattern space.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructPatternAlias {
    pub alias: String,
    pub target: StructSymbolPathMaterial,
}

pub fn bool_struct_aliases_for_tests() -> Vec<StructPatternAlias> {
    vec![
        StructPatternAlias {
            alias: "true".to_string(),
            target: StructSymbolPathMaterial::new(vec!["if".to_string(), "bool".to_string()]),
        },
        StructPatternAlias {
            alias: "false".to_string(),
            target: StructSymbolPathMaterial::new(vec!["else".to_string(), "bool".to_string()]),
        },
    ]
}
