//! Type-argument classification boundary.
//!
//! Classifies `UnknownExpression` arguments in an `ArgProductShape` by
//! resolving their corresponding product-atom names through the namespace
//! graph as type objects. Classification sets `NonValue(TypeObject)` and
//! records the type-object's `SymbolId` as primary identity; `TypeValueId`
//! is derived as a secondary projection.
//!
//! This module does **not** resolve call targets, does **not** perform type
//! checking, does **not** insert mechanical pass actions, and does **not**
//! classify value/non-type arguments.

use lang_syntax::NormExpr;

use crate::{
    graph::{NamespaceGraphCapability, ResolverContext},
    model::PolicyEnv,
    product_shape::{ArgProductShape, ProductAtom, RawArgValueClass},
};

/// Classify type-object arguments within an `ArgProductShape`.
///
/// For each `UnknownExpression` argument whose corresponding atom is a
/// `NormExpr::Name`, resolves the name through the namespace graph as a type
/// object under the given policy. Successfully resolved arguments are refined
/// to `NonValue(TypeObject)` with the type-object's `SymbolId` as primary
/// identity; `TypeValueId` is derived as secondary projection material.
/// Unresolved names remain `UnknownExpression`.
///
/// Index, provenance, and pass-action boundaries are preserved. Unit and
/// Expression-barrier atoms are passed through unchanged.
pub fn classify_type_arguments(
    shape: &ArgProductShape,
    capability: &NamespaceGraphCapability<'_>,
    context: &ResolverContext,
) -> ArgProductShape {
    let mut args = shape.raw_args.clone();
    for raw_arg in &mut args {
        if !matches!(raw_arg.value_class, RawArgValueClass::UnknownExpression) {
            continue;
        }
        let atom = match shape.flattened.atoms.get(raw_arg.index) {
            Some(atom) => atom,
            None => continue,
        };
        let name = match atom {
            ProductAtom::Expression {
                expr: NormExpr::Name { text, .. },
                ..
            } => text.clone(),
            _ => continue,
        };
        let Ok(type_symbol) =
            capability.resolve_type_object_with_policy(&name, context, PolicyEnv::Meta)
        else {
            continue;
        };
        *raw_arg = raw_arg
            .clone()
            .as_type_object_with_type_symbol(type_symbol.id);
    }
    ArgProductShape {
        raw_args: args,
        ..shape.clone()
    }
}

/// Classification report: carries the classified shape alongside unresolved
/// type-name entries for near-cause diagnostics.
#[derive(Clone, Debug)]
pub struct TypeArgumentClassificationReport {
    pub classified_shape: ArgProductShape,
    pub unresolved_names: Vec<String>,
}

/// Classify type-object arguments and record unresolved names for diagnostics.
///
/// Same logic as `classify_type_arguments`, but also returns a list of
/// names that could not be resolved as type objects. Callers can surface
/// these as near-cause diagnostics.
pub fn classify_type_arguments_with_report(
    shape: &ArgProductShape,
    capability: &NamespaceGraphCapability<'_>,
    context: &ResolverContext,
) -> TypeArgumentClassificationReport {
    let mut args = shape.raw_args.clone();
    let mut unresolved = Vec::new();
    for raw_arg in &mut args {
        if !matches!(raw_arg.value_class, RawArgValueClass::UnknownExpression) {
            continue;
        }
        let atom = match shape.flattened.atoms.get(raw_arg.index) {
            Some(atom) => atom,
            None => continue,
        };
        let name = match atom {
            ProductAtom::Expression {
                expr: NormExpr::Name { text, .. },
                ..
            } => text.clone(),
            _ => continue,
        };
        match capability.resolve_type_object_with_policy(&name, context, PolicyEnv::Meta) {
            Ok(type_symbol) => {
                *raw_arg = raw_arg
                    .clone()
                    .as_type_object_with_type_symbol(type_symbol.id);
            }
            Err(_) => {
                unresolved.push(name);
            }
        }
    }
    TypeArgumentClassificationReport {
        classified_shape: ArgProductShape {
            raw_args: args,
            ..shape.clone()
        },
        unresolved_names: unresolved,
    }
}
