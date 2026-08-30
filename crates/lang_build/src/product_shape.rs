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
//! `RawArgShape` refinement records observed argument content. It is **not**
//! type checking.

use lang_syntax::{NormError, NormExpr, NormOrigin, NormProduct, NormProductElem};

use crate::{
    canonical_value::{CanonicalTypeObservation, CanonicalValueAddr},
    identity::{SemanticValueId, TypeValueId},
    model::Provenance,
    model::SymbolId,
    policy_pair::{PolicyMode, PolicyResultEntry},
    semantic_world::{ObjectPlaceId, PatternValueId},
};

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
    /// Already-resolved semantic argument inserted by an authorized
    /// mechanical/compiler operation. This is not a fabricated source path.
    SemanticValue {
        value: SemanticValueId,
        type_value: TypeValueId,
        mode: PolicyMode,
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
            | Self::SemanticValue { provenance, .. }
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
    pub fn empty(provenance: Provenance) -> Self {
        Self {
            flattened: FlattenedProductObject {
                atoms: Vec::new(),
                provenance: provenance.clone(),
                invariant: FlattenedProductInvariant {
                    no_direct_product_atom_remains: true,
                },
            },
            arity: 0,
            raw_args: Vec::new(),
            provenance,
        }
    }

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
    /// Source-level pattern name recorded when a semantic type resolution
    /// classified this argument without a graph carrier Symbol. This is
    /// naming/navigation material for binder substitution only; identity
    /// still flows through `known_first_order_type_value`.
    pub known_type_pattern_name: Option<String>,
    pub known_first_order_type_value: Option<TypeValueId>,
    /// The resolved carrier's own binding-level pure-P member view, when this
    /// argument was classified from a named type carrier.
    ///
    /// A represented TypeValue is shared by every carrier binding it, so it
    /// cannot answer "what Policy did *this* binding expose?".
    /// The binding view therefore rides along with the argument instead of
    /// being reconstructed downstream.
    pub known_type_member_view: Option<PolicyResultEntry<SemanticValueId, PatternValueId>>,
    /// The resolved carrier's own object place, when this argument was
    /// classified from a named type carrier.
    ///
    /// A pure P is a real object, so two carriers of one Pattern can hold
    /// different Val2.  The place is the observation coordinate that decides
    /// *which* Val2 the canonicalizer reads — it is deliberately NOT identity
    /// material and never enters a normal form:
    ///
    /// ```text
    /// Norm_type(x) = ⟨Norm_P(P_x), Norm_Val2(Val2_x)⟩
    /// place(x)     ↦ Val2_x                (observation only)
    /// ```
    pub known_type_carrier_place: Option<ObjectPlaceId>,
    /// Complete immutable `tau` snapshot carried by the resolved binding.
    /// This is distinct from `known_type_observation`, which is the core-only
    /// coordinate used by Pattern structural identity.
    pub known_complete_type_observation: Option<CanonicalValueAddr>,
    /// The interned `Addr(Norm_type)` of this type argument's observation —
    /// the recursive P + Val2 normal form read at the carrier place — attached
    /// at a world-connected invocation boundary.
    ///
    /// When present, structural type-identity positions (struct pattern
    /// leaves, field signatures, extraction fields) consume this address
    /// instead of the bare `TypeValueId` projection, so two observations of
    /// one TypeValue with different Val2 never over-merge.  When absent, no
    /// canonical type observation is available to the consumer.
    pub known_type_observation: Option<CanonicalValueAddr>,
    /// Resolved Val1 identity when this source atom names an already-evaluated
    /// semantic value.  Policy slicing remains on the Symbol/value-view edge;
    /// this field never becomes a substitute for Symbol or Pattern identity.
    pub known_semantic_value: Option<SemanticValueId>,
    pub known_value_mode: Option<PolicyMode>,
    pub provenance: Provenance,
}

impl RawArgShape {
    pub fn from_product_atom(index: usize, atom: &ProductAtom) -> Self {
        let value_class = match atom {
            ProductAtom::Expression { .. } => RawArgValueClass::UnknownExpression,
            ProductAtom::Unit { .. } => RawArgValueClass::NonValue(NonValueArgKind::ProductUnit),
            ProductAtom::SemanticValue { .. } => RawArgValueClass::Value,
            ProductAtom::Unsupported { summary, .. } => RawArgValueClass::Unsupported {
                summary: summary.clone(),
            },
        };
        let (known_first_order_type_value, known_semantic_value, known_value_mode) = match atom {
            ProductAtom::SemanticValue {
                value,
                type_value,
                mode,
                ..
            } => (Some(*type_value), Some(*value), Some(*mode)),
            _ => (None, None, None),
        };
        Self {
            index,
            value_class,
            explicit_pass_mode: None,
            known_type_symbol_id: None,
            known_type_pattern_name: None,
            known_first_order_type_value,
            known_type_member_view: None,
            known_type_carrier_place: None,
            known_complete_type_observation: None,
            known_type_observation: None,
            known_semantic_value,
            known_value_mode,
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

    /// The type observation carried by this argument for structural
    /// type-identity positions.
    ///
    /// `Observed(addr)` is authoritative `Addr(Norm_type)` material. A bare
    /// `TypeValueId` never produces an observation.
    pub fn type_observation(&self) -> Option<CanonicalTypeObservation> {
        self.known_type_observation
            .map(CanonicalTypeObservation::Observed)
    }

    /// Returns true only after this argument has been positively classified as
    /// a value argument.
    ///
    /// `UnknownExpression` returns false because mechanical pass insertion is
    /// not allowed before
    /// value/type/rank/meta/pattern classification. This is not a final
    /// semantic claim that ordinary expressions never receive automatic pass
    /// actions after later classification.
    pub fn receives_automatic_pass_action(&self) -> bool {
        matches!(self.value_class, RawArgValueClass::Value)
    }

    /// Controlled refinement: replace the value class while preserving index,
    /// provenance, and existing type-value / pass-mode fields.
    ///
    /// This records a completed classification step; it is **not** type checking.
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
    /// This is an object-boundary classification operation — it does **not**
    /// represent completed semantic value typing.
    pub fn as_resolved_value(self) -> Self {
        self.with_value_class(RawArgValueClass::Value)
    }

    /// Refine an `UnknownExpression` into a non-value with the given kind.
    ///
    /// After this call, `is_value()` returns `Some(false)` and
    /// `receives_automatic_pass_action()` remains `false`.
    /// This is an object-boundary classification operation — it does **not**
    /// represent completed semantic non-value classification.
    pub fn as_non_value(self, kind: NonValueArgKind) -> Self {
        self.with_value_class(RawArgValueClass::NonValue(kind))
    }

    /// Refine into `NonValue(CoreTypeProjection)` while keeping carrier Symbol and
    /// represented type value independent.
    ///
    /// Ordinary `let T: type = uint8` passes the fresh carrier `T` together
    /// with the already-existing `uint8` TypeValue.  Candidate identity and
    /// type equality consume the TypeValue; the carrier remains graph/place
    /// material only.
    pub fn as_complete_type_projection_with_identity(
        self,
        carrier_symbol: SymbolId,
        represented_type: TypeValueId,
    ) -> Self {
        self.with_value_class(RawArgValueClass::NonValue(
            NonValueArgKind::CoreTypeProjection,
        ))
        .with_known_type_symbol_id(carrier_symbol)
        .with_known_first_order_type_value(represented_type)
    }

    fn with_known_type_symbol_id(self, symbol_id: SymbolId) -> Self {
        Self {
            known_type_symbol_id: Some(symbol_id),
            ..self
        }
    }

    /// Refine into `NonValue(CoreTypeProjection)` from a semantic-world resolution
    /// that carries a pattern name and represented type value, with an
    /// optional graph projection Symbol.
    ///
    /// The pattern name and represented `TypeValueId` are
    /// substitution/navigation and Core-lookup material. Canonical identity
    /// is carried separately by the Core and whole-type observation addresses;
    /// the lookup projection is never the complete type identity.
    pub fn as_complete_type_projection_named(
        self,
        top_pattern_name: String,
        represented_type: TypeValueId,
        carrier_symbol: Option<SymbolId>,
        member_view: Option<PolicyResultEntry<SemanticValueId, PatternValueId>>,
        carrier_place: Option<ObjectPlaceId>,
        complete_type_observation: Option<CanonicalValueAddr>,
    ) -> Self {
        let refined = self
            .with_value_class(RawArgValueClass::NonValue(
                NonValueArgKind::CoreTypeProjection,
            ))
            .with_known_first_order_type_value(represented_type);
        Self {
            known_type_pattern_name: Some(top_pattern_name),
            known_type_symbol_id: carrier_symbol,
            known_type_member_view: member_view,
            known_type_carrier_place: carrier_place,
            known_complete_type_observation: complete_type_observation,
            ..refined
        }
    }

    /// Refine into `Value` and record the value's first-order type-value
    /// projection. The argument material is a value; the type-value
    /// projection identifies the value's type.
    ///
    /// This is an object-boundary classification operation. It does **not**
    /// perform type checking.
    pub fn as_resolved_value_with_value_type(self, type_value: TypeValueId) -> Self {
        self.with_value_class(RawArgValueClass::Value)
            .with_known_first_order_type_value(type_value)
    }

    pub fn as_resolved_semantic_value(
        self,
        value: SemanticValueId,
        type_value: TypeValueId,
        mode: PolicyMode,
    ) -> Self {
        Self {
            known_semantic_value: Some(value),
            known_value_mode: Some(mode),
            ..self.as_resolved_value_with_value_type(type_value)
        }
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
    CoreTypeProjection,
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
        NormExpr::PolicyLet { origin, .. }
        | NormExpr::Call { origin, .. }
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
