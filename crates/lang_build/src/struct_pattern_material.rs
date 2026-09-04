//! Struct Pattern syntax material.
//!
//! The decoder preserves Leaf, Product, Sum, and Named input for conversion to
//! `CanonicalPatternValue`. This material does not decide Pattern applicability
//! or extraction.

use crate::model::Provenance;

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
}
