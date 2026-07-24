//! Invocation-frame substrate.
//!
//! `InvocationFrame` is the semantic layer between candidate selection and
//! callable body entry.
//!
//! `ProductObject` / `ArgProductShape` describe only the explicit
//! user-supplied call product. They do not contain function-object `self`.
//!
//! `self` is injected by the invocation frame and occupies callable formal slot
//! 0. Zero-user-argument callables still have a self slot.
//! When a closure writes any formal position, the first written formal is the
//! explicit Pattern for this slot regardless of binder spelling. Only later
//! formals consume the explicit user product.
//!
//! Declaration-context `()` call-entry definitions, such as
//! `let ()::ref::T = (self: T ref) => { ... }`, use the same frame model:
//! formal slot 0 is self-position and the explicit user product remains
//! separate.
//!
//! This module does not implement callable target resolution, overload
//! selection, declaration-context call-entry injection, associated-space symbol
//! injection, overload synthesis, body evaluation, return execution, D/Done,
//! lifetime checking, or implicit `?`.

use crate::{
    model::{Diagnostic, ExecutionEnv, PolicyEnv, Provenance, SymbolId},
    product_shape::ArgProductShape,
};

pub const SELF_SLOT_INDEX: usize = 0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallableFrameShape {
    pub self_slot: SelfSlotShape,
    pub explicit_parameter_shape: ExplicitParameterShape,
    pub return_target_shape: ReturnTargetShape,
    pub provenance: Provenance,
}

impl CallableFrameShape {
    pub fn new_with_self_slot(
        explicit_parameter_count: usize,
        return_target_shape: ReturnTargetShape,
        provenance: Provenance,
    ) -> Self {
        Self::new_with_self_slot_kind(
            explicit_parameter_count,
            SelfSlotKind::FunctionObject,
            return_target_shape,
            provenance,
        )
    }

    pub fn new_with_self_slot_kind(
        explicit_parameter_count: usize,
        self_slot_kind: SelfSlotKind,
        return_target_shape: ReturnTargetShape,
        provenance: Provenance,
    ) -> Self {
        Self {
            self_slot: SelfSlotShape {
                slot_index: SELF_SLOT_INDEX,
                kind: self_slot_kind,
                provenance: provenance.clone(),
            },
            explicit_parameter_shape: ExplicitParameterShape {
                user_parameter_count: explicit_parameter_count,
                provenance: provenance.clone(),
            },
            return_target_shape,
            provenance,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelfSlotShape {
    pub slot_index: usize,
    pub kind: SelfSlotKind,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelfSlotKind {
    FunctionObject,
    CallableObject,
    PrimitiveCoreObject,
    Placeholder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplicitParameterShape {
    pub user_parameter_count: usize,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnTargetShape {
    ImplicitNearest,
    ExplicitTargetSyntax,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvocationFrame {
    pub callable: InvocationCallableRef,
    pub self_position: SelfPosition,
    pub explicit_arg_product: ArgProductShape,
    pub lookup_env: InvocationLookupEnv,
    pub execution_env: InvocationExecutionEnv,
    pub provenance: Provenance,
}

impl InvocationFrame {
    pub fn new(
        callable: InvocationCallableRef,
        self_position: SelfPosition,
        explicit_arg_product: ArgProductShape,
        lookup_env: InvocationLookupEnv,
        execution_env: InvocationExecutionEnv,
        provenance: Provenance,
    ) -> Result<Self, Diagnostic> {
        if self_position.slot_index != SELF_SLOT_INDEX {
            return Err(Diagnostic::hard_error(
                format!(
                    "invocation frame self-position must occupy slot 0, got slot {}",
                    self_position.slot_index
                ),
                Some(self_position.provenance.clone()),
            ));
        }

        if explicit_arg_product.arity != explicit_arg_product.raw_args.len() {
            return Err(Diagnostic::hard_error(
                format!(
                    "ArgProductShape arity/raw-arg mismatch at invocation-frame boundary: arity {}, raw args {}",
                    explicit_arg_product.arity,
                    explicit_arg_product.raw_args.len()
                ),
                Some(explicit_arg_product.provenance.clone()),
            ));
        }
        if explicit_arg_product.arity != explicit_arg_product.flattened.atoms.len() {
            return Err(Diagnostic::hard_error(
                format!(
                    "ArgProductShape arity/atom mismatch at invocation-frame boundary: arity {}, atoms {}",
                    explicit_arg_product.arity,
                    explicit_arg_product.flattened.atoms.len()
                ),
                Some(explicit_arg_product.provenance.clone()),
            ));
        }

        // `ArgProductShape` has no representation for self material. Its arity
        // and raw args are the explicit user-supplied product only; self is the
        // separate `self_position` injected by this frame.
        Ok(Self {
            callable,
            self_position,
            explicit_arg_product,
            lookup_env,
            execution_env,
            provenance,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvocationCallableRef {
    Symbol(SymbolId),
    Placeholder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelfPosition {
    pub slot_index: usize,
    pub source: SelfPositionSource,
    pub provenance: Provenance,
}

impl SelfPosition {
    pub fn placeholder_from_callable_symbol(symbol_id: SymbolId, provenance: Provenance) -> Self {
        Self {
            slot_index: SELF_SLOT_INDEX,
            source: SelfPositionSource::PlaceholderFromCallableSymbol(symbol_id),
            provenance,
        }
    }

    pub fn placeholder_from_call_entry(provenance: Provenance) -> Self {
        Self {
            slot_index: SELF_SLOT_INDEX,
            source: SelfPositionSource::PlaceholderFromCallEntry,
            provenance,
        }
    }

    pub fn primitive_core_object(provenance: Provenance) -> Self {
        Self {
            slot_index: SELF_SLOT_INDEX,
            source: SelfPositionSource::PrimitiveCoreObject,
            provenance,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelfPositionSource {
    PlaceholderFromCallableSymbol(SymbolId),
    PlaceholderFromCallEntry,
    PrimitiveCoreObject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvocationLookupEnv {
    pub policy_env: PolicyEnv,
}

impl InvocationLookupEnv {
    pub fn new(policy_env: PolicyEnv) -> Self {
        Self { policy_env }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvocationExecutionEnv {
    pub execution_env: ExecutionEnv,
}

impl InvocationExecutionEnv {
    pub fn new(execution_env: ExecutionEnv) -> Self {
        Self { execution_env }
    }
}
