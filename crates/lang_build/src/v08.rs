use lang_syntax::{NormError, NormExpr, NormOrigin, NormProduct, NormProductElem};

use crate::{
    callable_body_allows_execution,
    model::{
        Diagnostic, ExecutionEnv, FieldObject, MetaFunctionObject, PolicyEnv, PolicyMetadata,
        Provenance, SymbolId, SymbolKind, SymbolObject, SymbolPayload,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductObject {
    pub original: NormProduct,
    pub provenance: Provenance,
    pub material_role: ProductMaterialRole,
}

impl ProductObject {
    pub fn from_norm_product(product: NormProduct, material_role: ProductMaterialRole) -> Self {
        let provenance = Provenance::from_norm_origin("v0.8 ProductObject", &product.origin);
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductMaterialRole {
    SourceProduct,
    CallableArgumentProduct,
    MetaConstructionArgumentProduct,
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

    pub fn receives_automatic_pass_action(&self) -> bool {
        matches!(self.value_class, RawArgValueClass::Value)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeValueId(pub u64);

impl TypeValueId {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlaceId(pub u64);

impl PlaceId {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeValueBindingPlaceholder {
    pub symbol: SymbolId,
    pub place: PlaceId,
    pub type_value: TypeValueId,
    pub provenance: Provenance,
}

impl TypeValueBindingPlaceholder {
    pub fn new(
        symbol: SymbolId,
        place: PlaceId,
        type_value: TypeValueId,
        provenance: Provenance,
    ) -> Self {
        Self {
            symbol,
            place,
            type_value,
            provenance,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliasChain {
    pub source_symbol: SymbolId,
    pub forwarded_target: SymbolId,
    pub final_symbol: Option<SymbolId>,
    pub final_value: Option<TypeValueId>,
    pub final_place: Option<PlaceId>,
    pub provenance_chain: Vec<Provenance>,
    pub writable_boundary: AliasWritableBoundary,
    pub cycle_detection_state: AliasCycleDetectionState,
}

impl AliasChain {
    pub fn new(
        source_symbol: SymbolId,
        forwarded_target: SymbolId,
        provenance: Provenance,
    ) -> Self {
        Self {
            source_symbol,
            forwarded_target,
            final_symbol: Some(forwarded_target),
            final_value: None,
            final_place: None,
            provenance_chain: vec![provenance],
            writable_boundary: AliasWritableBoundary::Unknown,
            cycle_detection_state: AliasCycleDetectionState::NotChecked,
        }
    }

    pub fn query_disposition(&self, mode: AliasQueryMode) -> AliasQueryDisposition {
        match mode {
            AliasQueryMode::TypeValueEvaluation => AliasQueryDisposition::FollowValueChain,
            AliasQueryMode::CallableLookup => AliasQueryDisposition::PolicyAwareSymbolResolution,
            AliasQueryMode::InjectionPlaceTarget => AliasQueryDisposition::FollowPlaceWithBoundary,
        }
    }

    pub fn creates_fresh_writable_place(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AliasQueryMode {
    TypeValueEvaluation,
    CallableLookup,
    InjectionPlaceTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AliasQueryDisposition {
    FollowValueChain,
    PolicyAwareSymbolResolution,
    FollowPlaceWithBoundary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AliasWritableBoundary {
    Unknown,
    ForwardTargetBoundary,
    ReadOnlyBoundary,
    WritableTargetBoundary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AliasCycleDetectionState {
    NotChecked,
    Visiting,
    Acyclic,
    CycleDetected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterShape {
    pub expected_arity: Option<usize>,
    pub provenance: Provenance,
}

impl ParameterShape {
    pub fn deferred(provenance: Provenance) -> Self {
        Self {
            expected_arity: None,
            provenance,
        }
    }

    pub fn exact_arity(expected_arity: usize, provenance: Provenance) -> Self {
        Self {
            expected_arity: Some(expected_arity),
            provenance,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidatePreparationContext {
    pub symbol_visibility: PolicyEnv,
    pub demanded_execution: ExecutionEnv,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidatePolicyPlanes {
    pub symbol_visibility: PolicyEnv,
    pub demanded_execution: ExecutionEnv,
    pub body_entry_policy: PolicyMetadata,
    pub return_object_policy: PolicyMetadata,
}

impl CandidatePolicyPlanes {
    pub fn body_entry_allows_demanded_execution(&self) -> bool {
        callable_body_allows_execution(&self.body_entry_policy, self.demanded_execution)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedCallableCandidate {
    pub callee_symbol_id: SymbolId,
    pub callee_name: String,
    pub callable_kind: CallableCandidateKind,
    pub arg_product_shape: ArgProductShape,
    pub parameter_shape: ParameterShape,
    pub policy_planes: CandidatePolicyPlanes,
    pub canonical_key_seed: CanonicalMetaInstanceKeySeed,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallableCandidateKind {
    MetaFunction,
    FieldFunction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalMetaInstanceKeySeed {
    pub callee_function_symbol_id: SymbolId,
    pub argument_arity: usize,
    pub argument_type_values: Vec<Option<TypeValueId>>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidatePrepResult {
    Deferred {
        candidate: Box<PreparedCallableCandidate>,
        reason: CandidatePrepDeferredReason,
    },
    ApplicablePlaceholder(Box<PreparedCallableCandidate>),
    Diagnostic(Diagnostic),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidatePrepDeferredReason {
    ParameterShapeCompatibilityDeferred,
    BodyEntryPolicyMismatch,
}

pub fn prepare_meta_callable_candidate(
    callee: &SymbolObject,
    arg_product_shape: ArgProductShape,
    parameter_shape: ParameterShape,
    context: CandidatePreparationContext,
) -> CandidatePrepResult {
    let Some((callable_kind, body_entry_policy, return_object_policy)) =
        callable_policy_from_symbol(callee)
    else {
        return CandidatePrepResult::Diagnostic(
            Diagnostic::hard_error(
                "candidate preparation requires a graph-resolved callable SymbolObject",
                Some(callee.provenance.clone()),
            )
            .with_symbol_context(callee.id),
        );
    };

    let policy_planes = CandidatePolicyPlanes {
        symbol_visibility: context.symbol_visibility,
        demanded_execution: context.demanded_execution,
        body_entry_policy,
        return_object_policy,
    };
    let canonical_key_seed = CanonicalMetaInstanceKeySeed {
        callee_function_symbol_id: callee.id,
        argument_arity: arg_product_shape.arity,
        argument_type_values: arg_product_shape
            .raw_args
            .iter()
            .map(|raw_arg| raw_arg.known_first_order_type_value)
            .collect(),
        provenance: context.provenance.clone(),
    };
    let candidate = PreparedCallableCandidate {
        callee_symbol_id: callee.id,
        callee_name: callee.name.clone(),
        callable_kind,
        arg_product_shape,
        parameter_shape,
        policy_planes,
        canonical_key_seed,
        provenance: context.provenance,
    };

    let Some(expected_arity) = candidate.parameter_shape.expected_arity else {
        return CandidatePrepResult::Deferred {
            candidate: Box::new(candidate),
            reason: CandidatePrepDeferredReason::ParameterShapeCompatibilityDeferred,
        };
    };
    if expected_arity != candidate.arg_product_shape.arity {
        return CandidatePrepResult::Diagnostic(
            Diagnostic::hard_error(
                format!(
                    "candidate preparation arity mismatch: expected {expected_arity}, got {}",
                    candidate.arg_product_shape.arity
                ),
                Some(candidate.parameter_shape.provenance.clone()),
            )
            .with_symbol_context(candidate.callee_symbol_id),
        );
    }
    if !candidate
        .policy_planes
        .body_entry_allows_demanded_execution()
    {
        return CandidatePrepResult::Deferred {
            candidate: Box::new(candidate),
            reason: CandidatePrepDeferredReason::BodyEntryPolicyMismatch,
        };
    }

    CandidatePrepResult::ApplicablePlaceholder(Box::new(candidate))
}

fn callable_policy_from_symbol(
    callee: &SymbolObject,
) -> Option<(CallableCandidateKind, PolicyMetadata, PolicyMetadata)> {
    match &callee.payload {
        SymbolPayload::MetaFunction(MetaFunctionObject {
            body_entry_policy,
            return_object_policy,
            ..
        }) if callee.kind == SymbolKind::MetaFunction => Some((
            CallableCandidateKind::MetaFunction,
            body_entry_policy.clone(),
            return_object_policy.clone(),
        )),
        SymbolPayload::FieldFunction(FieldObject {
            callable_policy, ..
        }) if callee.kind == SymbolKind::FieldFunction => Some((
            CallableCandidateKind::FieldFunction,
            callable_policy.body_entry_policy.clone(),
            callable_policy.return_object_policy.clone(),
        )),
        _ => None,
    }
}

fn flatten_product(product: &NormProduct, atoms: &mut Vec<ProductAtom>) {
    for element in &product.elements {
        match element {
            NormProductElem::Expr(NormExpr::Product(product)) => flatten_product(product, atoms),
            NormProductElem::Expr(expr) => atoms.push(product_atom_from_expr(expr)),
            NormProductElem::Unit { origin } => atoms.push(ProductAtom::Unit {
                provenance: Provenance::from_norm_origin("v0.8 product Unit", origin),
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
            provenance: Provenance::from_norm_origin("v0.8 unsupported product atom", origin),
        },
        NormExpr::Error(NormError { message, origin }) => ProductAtom::Unsupported {
            summary: message.clone(),
            provenance: Provenance::from_norm_origin("v0.8 error product atom", origin),
        },
        _ => ProductAtom::Expression {
            expr: expr.clone(),
            provenance: Provenance::from_norm_origin("v0.8 product expression", expr_origin(expr)),
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
