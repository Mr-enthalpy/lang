//! Product / argument-shape boundary from the v0.8 construction contract.
//!
//! This module is the product/argument-shape boundary between normalized surface
//! syntax and later candidate preparation.
//!
//! It is **not** the full pattern engine, **not** overload resolution, **not**
//! runtime ABI lowering, and **not** mechanical argument passing.
//!
//! Its job is to preserve and normalize product structure:
//! - exposed Product nodes flatten in order;
//! - Expression nodes are opaque barriers;
//! - Unit is preserved;
//! - provenance is preserved.
//!
//! `RawArgShape` refinement API is placeholder classification support. It is
//! **not** type checking.
//!
//! The current implementation boundary lives in `lang_build::product_shape`,
//! `lang_build::identity`, and `lang_build::meta_candidate`. These are substrate
//! boundaries, not full implementations of the future systems.

use lang_syntax::{NormError, NormExpr, NormOrigin, NormProduct, NormProductElem};

use crate::{identity::TypeValueId, model::Provenance, model::SymbolId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductObject {
    pub original: NormProduct,
    pub provenance: Provenance,
    pub material_role: ProductMaterialRole,
}

impl ProductObject {
    pub fn from_norm_product(product: NormProduct, material_role: ProductMaterialRole) -> Self {
        let provenance = Provenance::from_norm_origin("ProductObject", &product.origin);
        Self {
            original: product,
            provenance,
            material_role,
        }
    }

    pub fn flatten(&self) -> FlattenedProductObject {
        let mut atoms = Vec::new();
        flatten_product(&self.original, &mut atoms);
        FlattenedProductObject {
            atoms,
            provenance: self.provenance.clone(),
            invariant: FlattenedProductInvariant {
                no_direct_product_atom_remains: true,
            },
        }
    }

    pub fn to_arg_product_shape(&self) -> ArgProductShape {
        ArgProductShape::from_flattened(self.flatten())
    }
}

/// Future policy/candidate-prep role marker.
///
/// This enum distinguishes the context in which a product object is constructed.
/// It does **not** encode type-check results or runtime ABI decisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductMaterialRole {
    /// Source side of a normalized call.
    SourceProduct,
    /// Candidate-preparation input (argument product).
    CallableArgumentProduct,
    /// Meta-construction argument product.
    MetaConstructionArgumentProduct,
    /// Temporary boundary only.
    Placeholder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlattenedProductObject {
    pub atoms: Vec<ProductAtom>,
    pub provenance: Provenance,
    pub invariant: FlattenedProductInvariant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlattenedProductInvariant {
    /// Contract marker for product semantic normalization.
    ///
    /// `ProductAtom` intentionally has no Product variant, so this is not a
    /// separate runtime proof. It records the no-direct-Product-atom invariant
    /// at the object boundary consumed by `ArgProductShape`.
    pub no_direct_product_atom_remains: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductAtom {
    Expression {
        expr: NormExpr,
        provenance: Provenance,
    },
    Unit {
        provenance: Provenance,
    },
    Unsupported {
        summary: String,
        provenance: Provenance,
    },
}

impl ProductAtom {
    pub fn provenance(&self) -> &Provenance {
        match self {
            Self::Expression { provenance, .. }
            | Self::Unit { provenance }
            | Self::Unsupported { provenance, .. } => provenance,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgProductShape {
    pub flattened: FlattenedProductObject,
    pub arity: usize,
    pub raw_args: Vec<RawArgShape>,
    pub provenance: Provenance,
}

impl ArgProductShape {
    pub fn from_product_object(product: &ProductObject) -> Self {
        product.to_arg_product_shape()
    }

    pub fn from_flattened(flattened: FlattenedProductObject) -> Self {
        let raw_args = flattened
            .atoms
            .iter()
            .enumerate()
            .map(|(index, atom)| RawArgShape::from_product_atom(index, atom))
            .collect::<Vec<_>>();
        Self {
            arity: raw_args.len(),
            provenance: flattened.provenance.clone(),
            flattened,
            raw_args,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawArgShape {
    pub index: usize,
    pub value_class: RawArgValueClass,
    pub explicit_pass_mode: Option<ExplicitPassMode>,
    pub known_type_symbol_id: Option<SymbolId>,
    pub known_first_order_type_value: Option<TypeValueId>,
    pub provenance: Provenance,
}

impl RawArgShape {
    pub fn from_product_atom(index: usize, atom: &ProductAtom) -> Self {
        let value_class = match atom {
            ProductAtom::Expression { .. } => RawArgValueClass::UnknownExpression,
            ProductAtom::Unit { .. } => RawArgValueClass::NonValue(NonValueArgKind::ProductUnit),
            ProductAtom::Unsupported { summary, .. } => RawArgValueClass::Unsupported {
                summary: summary.clone(),
            },
        };
        Self {
            index,
            value_class,
            explicit_pass_mode: None,
            known_type_symbol_id: None,
            known_first_order_type_value: None,
            provenance: atom.provenance().clone(),
        }
    }

    pub fn is_value(&self) -> Option<bool> {
        match self.value_class {
            RawArgValueClass::Value => Some(true),
            RawArgValueClass::NonValue(_) => Some(false),
            RawArgValueClass::UnknownExpression | RawArgValueClass::Unsupported { .. } => None,
        }
    }

    /// Returns true only after this argument has been positively classified as
    /// a value argument.
    ///
    /// `UnknownExpression` returns false at the candidate-prep placeholder
    /// boundary because mechanical pass insertion is not allowed before
    /// value/type/rank/meta/pattern classification. This is not a final
    /// semantic claim that ordinary expressions never receive automatic pass
    /// actions after later classification.
    pub fn receives_automatic_pass_action(&self) -> bool {
        matches!(self.value_class, RawArgValueClass::Value)
    }

    /// Controlled refinement: replace the value class while preserving index,
    /// provenance, and existing type-value / pass-mode fields.
    ///
    /// This is placeholder classification support, **not** type checking.
    pub fn with_value_class(self, value_class: RawArgValueClass) -> Self {
        Self {
            value_class,
            ..self
        }
    }

    /// Controlled refinement: set a known first-order type-value projection.
    pub fn with_known_first_order_type_value(self, type_value: TypeValueId) -> Self {
        Self {
            known_first_order_type_value: Some(type_value),
            ..self
        }
    }

    /// Refine an `UnknownExpression` into a positively classified value.
    ///
    /// After this call, `receives_automatic_pass_action()` returns `true`.
    /// This is an object-boundary placeholder operation — it does **not**
    /// represent completed semantic value typing.
    pub fn as_resolved_value(self) -> Self {
        self.with_value_class(RawArgValueClass::Value)
    }

    /// Refine an `UnknownExpression` into a non-value with the given kind.
    ///
    /// After this call, `is_value()` returns `Some(false)` and
    /// `receives_automatic_pass_action()` remains `false`.
    /// This is an object-boundary placeholder operation — it does **not**
    /// represent completed semantic non-value classification.
    pub fn as_non_value(self, kind: NonValueArgKind) -> Self {
        self.with_value_class(RawArgValueClass::NonValue(kind))
    }

    /// Refine into `NonValue(TypeObject)` and record the type-object's
    /// `SymbolId` as the primary identity. Derives `TypeValueId` as a
    /// secondary projection.
    ///
    /// This is the primary classification entry point — `TypeSymbol` is the
    /// identity source; `TypeValueId` is derived projection material, never
    /// a binding lookup key.
    pub fn as_type_object_with_type_symbol(self, symbol_id: SymbolId) -> Self {
        let type_value = crate::identity::type_value_id_from_type_symbol_placeholder(symbol_id);
        self.with_value_class(RawArgValueClass::NonValue(NonValueArgKind::TypeObject))
            .with_known_type_symbol_id(symbol_id)
            .with_known_first_order_type_value(type_value)
    }

    fn with_known_type_symbol_id(self, symbol_id: SymbolId) -> Self {
        Self {
            known_type_symbol_id: Some(symbol_id),
            ..self
        }
    }

    /// Refine into `Value` and record the value's first-order type-value
    /// projection. The argument material is a value; the type-value
    /// projection identifies the value's type.
    ///
    /// This is an object-boundary placeholder operation. It does **not**
    /// perform type checking.
    pub fn as_resolved_value_with_value_type(self, type_value: TypeValueId) -> Self {
        self.with_value_class(RawArgValueClass::Value)
            .with_known_first_order_type_value(type_value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RawArgValueClass {
    Value,
    NonValue(NonValueArgKind),
    UnknownExpression,
    Unsupported { summary: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonValueArgKind {
    TypeObject,
    RankObject,
    NamespaceObject,
    MetaObject,
    PatternObject,
    ProductUnit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplicitPassMode {
    Move,
    Ref,
    Share,
    Copy,
    In,
}

fn flatten_product(product: &NormProduct, atoms: &mut Vec<ProductAtom>) {
    for element in &product.elements {
        match element {
            NormProductElem::Expr(NormExpr::Product(product)) => flatten_product(product, atoms),
            NormProductElem::Expr(expr) => atoms.push(product_atom_from_expr(expr)),
            NormProductElem::Unit { origin } => atoms.push(ProductAtom::Unit {
                provenance: Provenance::from_norm_origin("product Unit", origin),
            }),
        }
    }
}

fn product_atom_from_expr(expr: &NormExpr) -> ProductAtom {
    match expr {
        NormExpr::Unsupported {
            raw_kind_summary,
            origin,
        } => ProductAtom::Unsupported {
            summary: raw_kind_summary.clone(),
            provenance: Provenance::from_norm_origin("unsupported product atom", origin),
        },
        NormExpr::Error(NormError { message, origin }) => ProductAtom::Unsupported {
            summary: message.clone(),
            provenance: Provenance::from_norm_origin("error product atom", origin),
        },
        _ => ProductAtom::Expression {
            expr: expr.clone(),
            provenance: Provenance::from_norm_origin("product expression", expr_origin(expr)),
        },
    }
}

fn expr_origin(expr: &NormExpr) -> &NormOrigin {
    match expr {
        NormExpr::Call { origin, .. }
        | NormExpr::Name { origin, .. }
        | NormExpr::Literal { origin, .. }
        | NormExpr::Nav { origin, .. }
        | NormExpr::OperatorTarget { origin, .. }
        | NormExpr::Unsupported { origin, .. } => origin,
        NormExpr::Product(product) => &product.origin,
        NormExpr::Closure(closure) => &closure.origin,
        NormExpr::Error(error) => &error.origin,
    }
}
