//! core::struct normalized-AST decoder.
//!
//! This module interprets generic normalized AST expressions in the context
//! of `core::struct` type-pattern expressions. It produces
//! `TypePatternExprShape` from `NormExpr` without depending on
//! struct-specific parser or normalizer nodes.
//!
//! The parser and normalizer remain generic. All struct-specific
//! interpretation (product/sum recognition, name attachment, leaf decoding)
//! is local to this decoder.
//!
//! Key rules:
//! - `,` is product / `*` in struct type-pattern context.
//! - `|` is the canonical sum-pattern result form.
//! - `+` is a pattern-combination / reduction action, not a canonical sum
//!   form. It is not directly decoded as Sum.
//! - `(child parent_name)` attaches a local pattern/constructor name
//!   (right side) to child structure (left side).
//! - In leaf position, the final bare Name in an application chain is the
//!   local field/payload pattern name; the prefix is the type expression.
//!   This rule is struct-decoding-local only.

use lang_syntax::{NormExpr, NormProduct, NormProductElem};

use crate::{
    model::{Diagnostic, DiagnosticSeverity, Provenance},
    pattern_space::{StructLeafTypeExprShape, SymbolPathShape, TypePatternExprShape},
};

pub type StructDecodeResult = Result<TypePatternExprShape, Diagnostic>;

/// Wrapper for a successfully decoded struct type-pattern expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedStructPattern {
    pub type_pattern_expr: TypePatternExprShape,
    pub provenance: Provenance,
}

impl DecodedStructPattern {
    pub fn new(type_pattern_expr: TypePatternExprShape, provenance: Provenance) -> Self {
        Self {
            type_pattern_expr,
            provenance,
        }
    }
}

/// Decode a single normalized expression as a struct type-pattern expression.
///
/// The argument is the source expression inside `( ... ) |> struct`.
/// Products, calls, and operator expressions are recursively interpreted.
pub fn decode_struct_type_pattern_expr(
    expr: &NormExpr,
    provenance: Provenance,
) -> StructDecodeResult {
    match expr {
        NormExpr::Product(product) => decode_product(product, provenance),
        NormExpr::Call { source, target, .. } => decode_call(source, target, provenance),
        NormExpr::Name { text, .. } => {
            // A bare name at the top level of a struct argument is only valid
            // as a nullary constructor alternative inside a Sum context.
            // At the top level it is not a valid type-pattern expression.
            Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                format!(
                    "bare name `{}` is only valid as a nullary constructor alternative inside a sum pattern; \
                     wrap it in a Named constructor or place it inside a product",
                    text
                ),
                Some(provenance),
            ))
        }
        _ => Err(Diagnostic::new(
            DiagnosticSeverity::Error,
            format!(
                "unsupported normalized AST shape for core::struct: {:?}",
                expr_variant_name(expr)
            ),
            Some(provenance),
        )),
    }
}

/// Decode a product as a bare product type-pattern expression.
fn decode_product(product: &NormProduct, provenance: Provenance) -> StructDecodeResult {
    let mut elements: Vec<TypePatternExprShape> = Vec::new();
    let mut field_names: BTreeMap<String, usize> = BTreeMap::new();
    for elem in &product.elements {
        match elem {
            NormProductElem::Expr(inner) => {
                let decoded = decode_struct_type_pattern_expr(inner, provenance.clone())?;
                // Check for duplicate field names inside product
                if let TypePatternExprShape::Leaf {
                    local_pattern_name, ..
                } = &decoded
                {
                    if let Some(prev_idx) = field_names.get(local_pattern_name) {
                        return Err(Diagnostic::new(
                            DiagnosticSeverity::Error,
                            format!(
                                "duplicate field name `{}` in struct product (first seen at index {})",
                                local_pattern_name, prev_idx
                            ),
                            Some(provenance),
                        ));
                    }
                    field_names.insert(local_pattern_name.clone(), elements.len());
                }
                elements.push(decoded);
            }
            NormProductElem::Unit { .. } => {
                // Unit elements in struct argument are ignored (not fields)
            }
        }
    }
    Ok(TypePatternExprShape::product(elements, provenance))
}

/// Decode a call expression `(source_elements target_expr)`.
///
/// In struct type-pattern context:
/// - If target is a Name, it's a Named or Leaf construction.
/// - If target is an operator spelling `|`, it's a Sum.
/// - If target is an operator spelling `+`, it's rejected (requires prior
///   pattern-combination reduction).
fn decode_call(
    source: &NormProduct,
    target: &NormExpr,
    provenance: Provenance,
) -> StructDecodeResult {
    match target {
        NormExpr::Name { text: name, .. } => decode_call_with_name_target(source, name, provenance),
        NormExpr::OperatorTarget { spelling, .. } => {
            match spelling.as_str() {
                "|" => {
                    // Canonical sum: | directly decodes as Sum
                    decode_sum_from_source(source, provenance)
                }
                "+" => {
                    // + requires prior pattern-combination reduction
                    Err(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        "`+` is a pattern-combination / reduction action, not a canonical sum form. \
                         core::struct requires a canonical `|` sum-pattern. Apply the visible \
                         pattern-combination operation first to reduce `+` to `|`."
                            .to_string(),
                        Some(provenance),
                    ))
                }
                _ => Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    format!(
                        "unsupported operator `{}` in struct type-pattern expression",
                        spelling
                    ),
                    Some(provenance),
                )),
            }
        }
        _ => Err(Diagnostic::new(
            DiagnosticSeverity::Error,
            format!(
                "unsupported target in struct call expression: {:?}",
                expr_variant_name(target)
            ),
            Some(provenance),
        )),
    }
}

/// Decode a call with a Name target.
///
/// Two cases:
/// 1. Single type-name source + Name target → Leaf (field with type expr + local name)
/// 2. Multi-element source + Name target → Named (type-pattern construction)
fn decode_call_with_name_target(
    source: &NormProduct,
    name: &str,
    provenance: Provenance,
) -> StructDecodeResult {
    let elems: Vec<&NormExpr> = source
        .elements
        .iter()
        .filter_map(|e| match e {
            NormProductElem::Expr(expr) => Some(expr),
            NormProductElem::Unit { .. } => None,
        })
        .collect();

    match elems.len() {
        0 => {
            // Nullary: Named(Product[], name) — e.g. bare `None` or `if`
            Ok(TypePatternExprShape::named(
                TypePatternExprShape::product(vec![], provenance.clone()),
                name,
                provenance,
            ))
        }
        1 => {
            // Single source element: could be a Leaf or Named leaf
            let source_expr = elems[0];
            match source_expr {
                // Simple type name → Leaf
                NormExpr::Name {
                    text: type_name, ..
                } => Ok(TypePatternExprShape::leaf(
                    StructLeafTypeExprShape::Path(SymbolPathShape::single(type_name.clone())),
                    name,
                    provenance,
                )),
                // Navigation path (e.g. Vec::std) → Leaf with path
                NormExpr::Nav { components, .. } => {
                    let segments: Vec<String> = components
                        .iter()
                        .filter_map(|c| match c {
                            lang_syntax::norm::NormNavComponent::Name { name, .. } => {
                                Some(name.clone())
                            }
                            _ => None,
                        })
                        .collect();
                    if segments.is_empty() {
                        return Err(Diagnostic::new(
                            DiagnosticSeverity::Error,
                            "navigation expression in struct leaf has no name components"
                                .to_string(),
                            Some(provenance),
                        ));
                    }
                    Ok(TypePatternExprShape::leaf(
                        StructLeafTypeExprShape::Path(SymbolPathShape::new(segments)),
                        name,
                        provenance,
                    ))
                }
                // Any other expression (including inner Call) → Leaf with the
                // expression as type_expr, and `name` as the local pattern name.
                //
                // This implements the mechanical leaf rule:
                //   E Name → Leaf(external_type_expr = E, local_pattern_name = Name)
                //
                // Example: `int Vec a` → Leaf(type_expr = Call(int, Vec), name = a)
                // The rightmost Name in the application chain is the local
                // field/payload pattern name; the prefix is the type expression.
                // This rule is struct-decoding-local only.
                other => {
                    let type_desc = format!("{:?}", other);
                    Ok(TypePatternExprShape::leaf(
                        StructLeafTypeExprShape::NormalizedAst {
                            description: type_desc,
                            provenance: provenance.clone(),
                        },
                        name,
                        provenance,
                    ))
                }
            }
        }
        _ => {
            // Multiple source elements: decode each and wrap as Named
            let mut children: Vec<TypePatternExprShape> = Vec::new();
            for expr in elems {
                children.push(decode_struct_type_pattern_expr(expr, provenance.clone())?);
            }
            let child = if children.len() == 1 {
                children.into_iter().next().unwrap()
            } else {
                TypePatternExprShape::product(children, provenance.clone())
            };
            Ok(TypePatternExprShape::named(child, name, provenance))
        }
    }
}

/// Decode a sum expression from a call source product.
///
/// Each element in the source product is decoded as a sum alternative.
fn decode_sum_from_source(source: &NormProduct, provenance: Provenance) -> StructDecodeResult {
    let mut alternatives: Vec<TypePatternExprShape> = Vec::new();
    let mut alt_names: BTreeMap<String, usize> = BTreeMap::new();

    for elem in &source.elements {
        match elem {
            NormProductElem::Expr(inner) => {
                let decoded = decode_struct_type_pattern_expr(inner, provenance.clone())?;
                // Extract the alternative label for duplicate checking
                let label = match &decoded {
                    TypePatternExprShape::Named { pattern_name, .. } => pattern_name.clone(),
                    TypePatternExprShape::Leaf {
                        local_pattern_name, ..
                    } => local_pattern_name.clone(),
                    TypePatternExprShape::Product { .. } => {
                        return Err(Diagnostic::new(
                            DiagnosticSeverity::Error,
                            "bare Product is not a valid sum alternative — wrap it in a Named constructor"
                                .to_string(),
                            Some(provenance),
                        ));
                    }
                    TypePatternExprShape::Sum { .. } => {
                        return Err(Diagnostic::new(
                            DiagnosticSeverity::Error,
                            "bare Sum is not a valid sum alternative — wrap it in a Named constructor"
                                .to_string(),
                            Some(provenance),
                        ));
                    }
                };
                if let Some(prev_idx) = alt_names.get(&label) {
                    return Err(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        format!(
                            "duplicate alternative name `{}` in sum pattern (first seen at index {})",
                            label, prev_idx
                        ),
                        Some(provenance),
                    ));
                }
                alt_names.insert(label, alternatives.len());
                alternatives.push(decoded);
            }
            NormProductElem::Unit { .. } => {}
        }
    }

    if alternatives.is_empty() {
        return Err(Diagnostic::new(
            DiagnosticSeverity::Error,
            "empty sum pattern space is not allowed in struct type-pattern expression".to_string(),
            Some(provenance),
        ));
    }

    Ok(TypePatternExprShape::sum(alternatives, provenance))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

use std::collections::BTreeMap;

fn expr_variant_name(expr: &NormExpr) -> &'static str {
    match expr {
        NormExpr::Call { .. } => "Call",
        NormExpr::Product(_) => "Product",
        NormExpr::Name { .. } => "Name",
        NormExpr::Literal { .. } => "Literal",
        NormExpr::Nav { .. } => "Nav",
        NormExpr::Closure(_) => "Closure",
        NormExpr::OperatorTarget { .. } => "OperatorTarget",
        NormExpr::Error(_) => "Error",
        NormExpr::Unsupported { .. } => "Unsupported",
    }
}
