//! Normalized call-site extraction boundary.
//!
//! This module extracts call sites from normalized source expressions and
//! produces the argument product shape consumed by candidate preparation.
//!
//! It preserves Expression barriers, does **not** perform symbol lookup, does
//! **not** decide callable validity, does **not** erase Unit, and does **not**
//! infer type identities.
//!
//! The current implementation boundary lives in `lang_build::normalized_call`,
//! `lang_build::product_shape`, and `lang_build::meta_candidate`. These are
//! substrate boundaries, not full implementations of the future systems.

use lang_syntax::{NormExpr, NormProduct};

use crate::{
    model::{Diagnostic, Provenance},
    product_shape::{ArgProductShape, ProductMaterialRole, ProductObject},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedCallSite {
    pub source_product: NormProduct,
    pub target: NormExpr,
    pub provenance: Provenance,
}

impl NormalizedCallSite {
    /// Expose the `ProductObject` boundary before jumping to `ArgProductShape`.
    /// Makes the pipeline chain `call-site → ProductObject → ArgProductShape`
    /// explicitly visible in caller code.
    pub fn source_product_object(&self, role: ProductMaterialRole) -> ProductObject {
        ProductObject::from_norm_product(self.source_product.clone(), role)
    }

    pub fn to_arg_product_shape(&self, role: ProductMaterialRole) -> ArgProductShape {
        self.source_product_object(role).to_arg_product_shape()
    }
}

pub fn extract_single_call_site(expr: &NormExpr) -> Result<NormalizedCallSite, Diagnostic> {
    match expr {
        NormExpr::Call {
            source,
            target,
            origin,
        } => Ok(NormalizedCallSite {
            source_product: source.clone(),
            target: *target.clone(),
            provenance: Provenance::from_norm_origin("NormalizedCallSite", origin),
        }),
        other => Err(Diagnostic::hard_error(
            format!(
                "expected a normalized Call expression for call-site extraction, got {}",
                expr_kind_name(other)
            ),
            None,
        )),
    }
}

fn expr_kind_name(expr: &NormExpr) -> &'static str {
    match expr {
        NormExpr::Call { .. } => "Call",
        NormExpr::Name { .. } => "Name",
        NormExpr::Literal { .. } => "Literal",
        NormExpr::Nav { .. } => "Nav",
        NormExpr::OperatorTarget { .. } => "OperatorTarget",
        NormExpr::Product(_) => "Product",
        NormExpr::Closure(_) => "Closure",
        NormExpr::Unsupported { .. } => "Unsupported",
        NormExpr::Error(_) => "Error",
    }
}
