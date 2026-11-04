//! Return target binding over normalized return terminal material.
//!
//! This pass answers only "which active return frame receives this return
//! event?" It does not perform later control-flow completion, return value
//! typing, propagation, lowering, lifetime checks, or resource scheduling.

use lang_syntax::{
    NormBindingSlot, NormClosure, NormClosurePlacement, NormExpr, NormForm, NormOrigin,
    NormPattern, NormPatternElem, NormProgram, NormReturnEvent, NormReturnTargetSyntax,
};

use crate::model::{Diagnostic, Provenance, ResolverCode, SymbolId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReturnFrameId(pub usize);

/// Alpha-stable identity of one callable's return slot.  The written return
/// binder spelling is diagnostic material only and never participates in
/// equality.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReturnSlotIdentity {
    pub callable_owner: lang_syntax::NormSemanticOwnerId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnSlotRef {
    pub identity: Option<ReturnSlotIdentity>,
    /// The complete normalized return binding slot. Keeping the Pattern and
    /// its annotations/policy is required for later pattern-directed result
    /// delivery; this pass still performs target binding only.
    pub binding_slot: Option<NormBindingSlot>,
    /// Convenience spelling for the currently supported self-target
    /// diagnostics. Product/extraction returns intentionally have no single
    /// name while their full slot remains available above.
    pub name: Option<String>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReturnFrameOwner {
    SourceCallable {
        symbol_id: Option<SymbolId>,
        name: Option<String>,
    },
    AnonymousClosure,
    Synthetic {
        description: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnSelfIdentity {
    /// Identity of the callable-local self position.  This exists even when
    /// the source writes no formal self binder.
    pub callable_owner: lang_syntax::NormSemanticOwnerId,
    /// Optional written spelling retained for diagnostics only.
    pub display_name: Option<String>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnTargetFrame {
    pub frame_id: ReturnFrameId,
    pub return_slot: ReturnSlotRef,
    pub owner: ReturnFrameOwner,
    pub self_identity: Option<ReturnSelfIdentity>,
    /// Lexical owner of the callable-local `Self` space and return frame.
    ///
    /// This is present for every alpha-normalized callable, including in-place
    /// closures, independently of any written self binder. It
    /// does not encode the callable's invocation receiver type.
    pub callable_self_owner: Option<lang_syntax::NormSemanticOwnerId>,
    pub boundary: ReturnBoundary,
    pub origin: NormOrigin,
}

/// Evidence supplied at actual callable entry, not inferred from target spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnBoundary {
    Ordinary,
    InPlace,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnTargetReachability {
    Reachable,
    Blocked,
    Unavailable,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReturnTargetStack {
    frames: Vec<ReturnTargetFrame>,
    next_id: usize,
}

impl ReturnTargetStack {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push without boundary evidence. Such a frame is Unknown, never transparent.
    pub fn push_frame(
        &mut self,
        owner: ReturnFrameOwner,
        return_slot: ReturnSlotRef,
        self_identity: Option<ReturnSelfIdentity>,
        callable_self_owner: Option<lang_syntax::NormSemanticOwnerId>,
        origin: NormOrigin,
    ) -> ReturnTargetFrame {
        self.push_frame_with_boundary(
            owner,
            return_slot,
            self_identity,
            callable_self_owner,
            origin,
            ReturnBoundary::Unknown,
        )
    }

    pub fn push_frame_with_boundary(
        &mut self,
        owner: ReturnFrameOwner,
        return_slot: ReturnSlotRef,
        self_identity: Option<ReturnSelfIdentity>,
        callable_self_owner: Option<lang_syntax::NormSemanticOwnerId>,
        origin: NormOrigin,
        boundary: ReturnBoundary,
    ) -> ReturnTargetFrame {
        let frame = ReturnTargetFrame {
            frame_id: ReturnFrameId(self.next_id),
            return_slot,
            owner,
            self_identity,
            callable_self_owner,
            boundary,
            origin,
        };
        self.next_id += 1;
        self.frames.push(frame.clone());
        frame
    }

    pub fn active_frame(&self, id: ReturnFrameId) -> Option<&ReturnTargetFrame> {
        self.frames.iter().find(|frame| frame.frame_id == id)
    }

    pub fn pop_frame(&mut self) -> Option<ReturnTargetFrame> {
        self.frames.pop()
    }

    /// Outermost frame of the reachable segment, not the globally first frame.
    /// None means either no active frame or unavailable boundary evidence.
    pub fn outermost_enclosing_function_target(&self) -> Option<&ReturnTargetFrame> {
        self.reachable_segment_start()
            .and_then(|index| self.frames.get(index))
    }

    fn reachable_segment_start(&self) -> Option<usize> {
        for (index, frame) in self.frames.iter().enumerate().rev() {
            match frame.boundary {
                ReturnBoundary::Ordinary => return Some(index),
                ReturnBoundary::InPlace => {}
                ReturnBoundary::Unknown => return None,
            }
        }
        Some(0)
    }

    /// Shared reachability guard for explicit targets and future return-capability calls.
    /// Active identity alone never authorizes crossing an ordinary closure barrier.
    pub fn target_reachability(&self, target: ReturnFrameId) -> ReturnTargetReachability {
        let Some(index) = self
            .frames
            .iter()
            .position(|frame| frame.frame_id == target)
        else {
            return ReturnTargetReachability::Blocked;
        };
        match self.reachable_segment_start() {
            Some(start) if index >= start => ReturnTargetReachability::Reachable,
            Some(_) => ReturnTargetReachability::Blocked,
            None => ReturnTargetReachability::Unavailable,
        }
    }
}

/// Result supplied by the semantic expression/type resolver for an explicit
/// return target and its actual active control-flow activation. The binder never reconstructs this identity
/// from a source spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplicitReturnTargetResolution {
    /// Resolved callable tag plus actual activation, supplied by the control-flow consumer.
    ActiveFrame(ReturnFrameId),
    NotActive,
    Unsupported,
}

pub trait ExplicitReturnTargetResolver {
    fn resolve_explicit_return_target(&self, target: &NormExpr) -> ExplicitReturnTargetResolution;
}

impl<F> ExplicitReturnTargetResolver for F
where
    F: Fn(&NormExpr) -> ExplicitReturnTargetResolution,
{
    fn resolve_explicit_return_target(&self, target: &NormExpr) -> ExplicitReturnTargetResolution {
        self(target)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnresolvedReturnTargetForm {
    Omitted,
    Explicit(NormExpr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnboundReturnEvent {
    pub value: NormExpr,
    pub target: UnresolvedReturnTargetForm,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedReturnTarget {
    ActiveFrame(ReturnFrameId),
    DiagnosticTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundReturnEvent {
    pub value: NormExpr,
    pub unresolved_target: UnresolvedReturnTargetForm,
    pub resolved_target: ResolvedReturnTarget,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreservedReturnReason {
    UnmaterializedClosureLiteral,
    /// The normalized expression has not yet been resolved to a callable
    /// object tag and reachable active frame, or required boundary evidence is
    /// unavailable. Spelling equality and assumed transparency are not fallbacks.
    SemanticTargetResolutionRequired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreservedUnboundReturnEvent {
    pub event: UnboundReturnEvent,
    pub reason: PreservedReturnReason,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReturnTargetBindingReport {
    pub frames: Vec<ReturnTargetFrame>,
    pub bound_events: Vec<BoundReturnEvent>,
    pub preserved_unbound_events: Vec<PreservedUnboundReturnEvent>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn elaborate_return_targets_in_program(program: &NormProgram) -> ReturnTargetBindingReport {
    let mut binder = ReturnTargetBinder::new();
    binder.visit_program(program);
    binder.finish()
}

pub fn elaborate_return_targets_in_returnable_closure(
    closure: &NormClosure,
    owner: ReturnFrameOwner,
) -> ReturnTargetBindingReport {
    let mut binder = ReturnTargetBinder::new();
    binder.enter_returnable_closure(closure, owner);
    binder.finish()
}

pub fn elaborate_return_targets_in_returnable_closure_with_resolver(
    closure: &NormClosure,
    owner: ReturnFrameOwner,
    resolver: &dyn ExplicitReturnTargetResolver,
) -> ReturnTargetBindingReport {
    let mut binder = ReturnTargetBinder::with_explicit_resolver(resolver);
    binder.enter_returnable_closure(closure, owner);
    binder.finish()
}

pub struct ReturnTargetBinder<'resolver> {
    stack: ReturnTargetStack,
    report: ReturnTargetBindingReport,
    explicit_resolver: Option<&'resolver dyn ExplicitReturnTargetResolver>,
}

impl ReturnTargetBinder<'static> {
    pub fn new() -> Self {
        Self {
            stack: ReturnTargetStack::new(),
            report: ReturnTargetBindingReport::default(),
            explicit_resolver: None,
        }
    }
}

impl<'resolver> ReturnTargetBinder<'resolver> {
    pub fn with_explicit_resolver(resolver: &'resolver dyn ExplicitReturnTargetResolver) -> Self {
        Self {
            stack: ReturnTargetStack::new(),
            report: ReturnTargetBindingReport::default(),
            explicit_resolver: Some(resolver),
        }
    }

    pub fn finish(self) -> ReturnTargetBindingReport {
        self.report
    }

    pub fn enter_returnable_closure(&mut self, closure: &NormClosure, owner: ReturnFrameOwner) {
        let return_slot = return_slot_ref(closure);
        let self_identity = self_identity_from_closure(closure);
        let callable_self_owner = closure.semantic_owner.map(|owner| owner.id);
        let frame = self.stack.push_frame_with_boundary(
            owner,
            return_slot,
            self_identity,
            callable_self_owner,
            closure.origin.clone(),
            match closure.placement {
                NormClosurePlacement::Ordinary => ReturnBoundary::Ordinary,
                NormClosurePlacement::InPlace => ReturnBoundary::InPlace,
            },
        );
        self.report.frames.push(frame);

        if let Some(program) = closure.body.user_body() {
            self.visit_program(program);
        }

        self.stack.pop_frame();
    }

    pub fn visit_program(&mut self, program: &NormProgram) {
        for form in &program.forms {
            self.visit_form(form);
        }
    }

    fn visit_form(&mut self, form: &NormForm) {
        match form {
            NormForm::ReturnEvent(return_event) => self.bind_return_event(return_event),
            NormForm::Let(decl) | NormForm::Alias(decl) => self.visit_decl(decl),
            NormForm::Expr(expr) => self.visit_expr(expr),
            NormForm::Error(_) => {}
        }
    }

    fn visit_decl(&mut self, decl: &lang_syntax::NormDecl) {
        match decl {
            lang_syntax::NormDecl::Let { slot, .. } => {
                if let Some(initializer) = &slot.initializer {
                    self.visit_expr(initializer);
                }
            }
            lang_syntax::NormDecl::Alias { .. } | lang_syntax::NormDecl::Error(_) => {}
        }
    }

    fn visit_expr(&mut self, expr: &NormExpr) {
        match expr {
            NormExpr::InterpretationFlip { operand, .. } => {
                lang_syntax::norm::visit_structural_value_occurrences(operand, &mut |expr| {
                    self.visit_expr(expr)
                })
            }
            NormExpr::PolicyLet { operand, .. } => self.visit_expr(operand),
            NormExpr::Call { source, target, .. } => {
                for elem in &source.elements {
                    if let lang_syntax::NormProductElem::Expr(expr) = elem {
                        self.visit_expr(expr);
                    }
                }
                self.visit_expr(target);
            }
            NormExpr::Product(product) => {
                for elem in &product.elements {
                    if let lang_syntax::NormProductElem::Expr(expr) = elem {
                        self.visit_expr(expr);
                    }
                }
            }
            NormExpr::Closure(closure) => {
                self.collect_preserved_returns_from_closure_literal(closure);
            }
            NormExpr::UniverseRank { .. }
            | NormExpr::Name { .. }
            | NormExpr::Literal { .. }
            | NormExpr::Nav { .. }
            | NormExpr::OperatorTarget { .. }
            | NormExpr::Error(_)
            | NormExpr::Unsupported { .. } => {}
        }
    }

    fn bind_return_event(&mut self, return_event: &NormReturnEvent) {
        self.visit_expr(&return_event.value);

        let unbound = unbound_event(return_event);
        match resolve_return_target(&self.stack, return_event, self.explicit_resolver) {
            Ok(Some(resolved_target)) => self.report.bound_events.push(BoundReturnEvent {
                value: return_event.value.clone(),
                unresolved_target: unbound.target,
                resolved_target,
                origin: return_event.origin.clone(),
            }),
            Ok(None) => self
                .report
                .preserved_unbound_events
                .push(PreservedUnboundReturnEvent {
                    event: unbound,
                    reason: PreservedReturnReason::SemanticTargetResolutionRequired,
                }),
            Err(diagnostic) => {
                self.report.diagnostics.push(diagnostic);
            }
        }
    }

    fn collect_preserved_returns_from_closure_literal(&mut self, closure: &NormClosure) {
        let mut events = Vec::new();
        collect_return_events_in_closure(closure, &mut events);
        self.report
            .preserved_unbound_events
            .extend(events.into_iter().map(|event| PreservedUnboundReturnEvent {
                event,
                reason: PreservedReturnReason::UnmaterializedClosureLiteral,
            }));
    }
}

impl Default for ReturnTargetBinder<'static> {
    fn default() -> Self {
        Self::new()
    }
}

fn resolve_return_target(
    stack: &ReturnTargetStack,
    return_event: &NormReturnEvent,
    explicit_resolver: Option<&dyn ExplicitReturnTargetResolver>,
) -> Result<Option<ResolvedReturnTarget>, Diagnostic> {
    match &return_event.target {
        NormReturnTargetSyntax::Omitted
            if !stack.frames.is_empty()
                && stack.outermost_enclosing_function_target().is_none() =>
        {
            Ok(None)
        }
        NormReturnTargetSyntax::Omitted => stack
            .outermost_enclosing_function_target()
            .map(|frame| Some(ResolvedReturnTarget::ActiveFrame(frame.frame_id)))
            .ok_or_else(|| {
                return_diagnostic(
                    ResolverCode::ReturnOutsideReturnableContext,
                    "ReturnOutsideReturnableContext: return event has no active return target",
                    &return_event.origin,
                )
            }),
        NormReturnTargetSyntax::Explicit(target) => {
            resolve_explicit_return_target(stack, target, explicit_resolver)
        }
    }
}

fn resolve_explicit_return_target(
    stack: &ReturnTargetStack,
    target: &NormExpr,
    resolver: Option<&dyn ExplicitReturnTargetResolver>,
) -> Result<Option<ResolvedReturnTarget>, Diagnostic> {
    let Some(resolver) = resolver else {
        return Ok(None);
    };
    match resolver.resolve_explicit_return_target(target) {
        ExplicitReturnTargetResolution::ActiveFrame(frame_id) => {
            if stack.active_frame(frame_id).is_some() {
                match stack.target_reachability(frame_id) {
                    ReturnTargetReachability::Reachable => Ok(Some(ResolvedReturnTarget::ActiveFrame(frame_id))),
                    ReturnTargetReachability::Unavailable => Ok(None),
                    ReturnTargetReachability::Blocked => Err(return_diagnostic(
                        ResolverCode::ReturnTargetOutsideReachableSegment,
                        "explicit return target crosses an ordinary closure barrier",
                        expr_origin(target),
                    )),
                }
            } else {
                Err(return_diagnostic(ResolverCode::ReturnTargetNotActive,
                    "resolved callable activation is not active", expr_origin(target)))
            }
        }
        ExplicitReturnTargetResolution::NotActive => Err(return_diagnostic(
            ResolverCode::ReturnTargetNotActive,
            "ReturnTargetNotActive: explicit return target is not active",
            expr_origin(target),
        )),
        ExplicitReturnTargetResolution::Unsupported => Err(return_diagnostic(
            ResolverCode::UnsupportedReturnTargetForm,
            "UnsupportedReturnTargetForm: explicit return target form is not supported by the return-target binder",
            expr_origin(target),
        )),
    }
}

fn expr_origin(expr: &NormExpr) -> &NormOrigin {
    match expr {
        NormExpr::InterpretationFlip { origin, .. }
        | NormExpr::PolicyLet { origin, .. }
        | NormExpr::Call { origin, .. }
        | NormExpr::Literal { origin, .. }
        | NormExpr::Nav { origin, .. }
        | NormExpr::OperatorTarget { origin, .. }
        | NormExpr::Unsupported { origin, .. } => origin,
        NormExpr::Product(lang_syntax::NormProduct { origin, .. })
        | NormExpr::Closure(NormClosure { origin, .. }) => origin,
        NormExpr::UniverseRank { origin } | NormExpr::Name { origin, .. } => origin,
        NormExpr::Error(lang_syntax::NormError { origin, .. }) => origin,
    }
}

fn return_diagnostic(
    code: ResolverCode,
    message: impl Into<String>,
    origin: &NormOrigin,
) -> Diagnostic {
    Diagnostic::hard_error(
        message,
        Some(Provenance::from_norm_origin(
            "return target binding",
            origin,
        )),
    )
    .with_code(code)
}

fn return_slot_ref(closure: &NormClosure) -> ReturnSlotRef {
    let identity = closure.semantic_owner.map(|owner| ReturnSlotIdentity {
        callable_owner: owner.id,
    });
    let Some(head) = &closure.head else {
        return ReturnSlotRef {
            identity,
            binding_slot: None,
            name: None,
            origin: closure.origin.clone(),
        };
    };
    let Some(returns) = &head.returns else {
        return ReturnSlotRef {
            identity,
            binding_slot: None,
            name: None,
            origin: head.origin.clone(),
        };
    };
    ReturnSlotRef {
        identity,
        binding_slot: Some(returns.clone()),
        name: binding_slot_name(returns),
        origin: returns.origin.clone(),
    }
}

fn self_identity_from_closure(closure: &NormClosure) -> Option<ReturnSelfIdentity> {
    let callable_owner = closure.semantic_owner?.id;
    let written = closure
        .head
        .as_ref()
        .and_then(|head| head.formal_frame().self_formal);
    let display_name = written.and_then(|written| match written {
        NormPatternElem::BindingSlot(slot) => binding_slot_name(slot),
        NormPatternElem::Pattern(pattern) => pattern_display_name(pattern),
        _ => None,
    });
    Some(ReturnSelfIdentity {
        callable_owner,
        display_name,
        origin: closure.origin.clone(),
    })
}

fn pattern_display_name(pattern: &NormPattern) -> Option<String> {
    match pattern {
        NormPattern::Binder { name, .. } => name.clone(),
        NormPattern::OperatorBinder { spelling, .. } => Some(spelling.clone()),
        _ => None,
    }
}

fn binding_slot_name(slot: &NormBindingSlot) -> Option<String> {
    match &slot.value_pattern {
        NormPattern::Binder { name, .. } => name.clone(),
        NormPattern::OperatorBinder { spelling, .. } => Some(spelling.clone()),
        _ => None,
    }
}

fn unbound_event(return_event: &NormReturnEvent) -> UnboundReturnEvent {
    UnboundReturnEvent {
        value: return_event.value.clone(),
        target: match &return_event.target {
            NormReturnTargetSyntax::Omitted => UnresolvedReturnTargetForm::Omitted,
            NormReturnTargetSyntax::Explicit(target) => {
                UnresolvedReturnTargetForm::Explicit(target.clone())
            }
        },
        origin: return_event.origin.clone(),
    }
}

fn collect_return_events_in_closure(closure: &NormClosure, events: &mut Vec<UnboundReturnEvent>) {
    if let Some(program) = closure.body.user_body() {
        collect_return_events_in_program(program, events);
    }
}

fn collect_return_events_in_program(program: &NormProgram, events: &mut Vec<UnboundReturnEvent>) {
    for form in &program.forms {
        match form {
            NormForm::ReturnEvent(return_event) => events.push(unbound_event(return_event)),
            NormForm::Let(decl) | NormForm::Alias(decl) => {
                collect_return_events_in_decl(decl, events);
            }
            NormForm::Expr(expr) => {
                collect_return_events_in_expr(expr, events);
            }
            NormForm::Error(_) => {}
        }
    }
}

fn collect_return_events_in_decl(
    decl: &lang_syntax::NormDecl,
    events: &mut Vec<UnboundReturnEvent>,
) {
    if let lang_syntax::NormDecl::Let { slot, .. } = decl {
        if let Some(initializer) = &slot.initializer {
            collect_return_events_in_expr(initializer, events);
        }
    }
}

fn collect_return_events_in_expr(expr: &NormExpr, events: &mut Vec<UnboundReturnEvent>) {
    match expr {
        NormExpr::InterpretationFlip { operand, .. } => {
            lang_syntax::norm::visit_structural_value_occurrences(operand, &mut |expr| {
                collect_return_events_in_expr(expr, events)
            })
        }
        NormExpr::PolicyLet { operand, .. } => collect_return_events_in_expr(operand, events),
        NormExpr::Call { source, target, .. } => {
            for elem in &source.elements {
                if let lang_syntax::NormProductElem::Expr(expr) = elem {
                    collect_return_events_in_expr(expr, events);
                }
            }
            collect_return_events_in_expr(target, events);
        }
        NormExpr::Product(product) => {
            for elem in &product.elements {
                if let lang_syntax::NormProductElem::Expr(expr) = elem {
                    collect_return_events_in_expr(expr, events);
                }
            }
        }
        NormExpr::Closure(closure) => collect_return_events_in_closure(closure, events),
        NormExpr::UniverseRank { .. }
        | NormExpr::Name { .. }
        | NormExpr::Literal { .. }
        | NormExpr::Nav { .. }
        | NormExpr::OperatorTarget { .. }
        | NormExpr::Error(_)
        | NormExpr::Unsupported { .. } => {}
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn event(explicit: bool) -> NormReturnEvent {
        let parsed = lang_syntax::parse("value return;");
        assert!(parsed.diagnostics.is_empty());
        let program = lang_syntax::normalize_program(&parsed.program);
        let NormForm::ReturnEvent(mut event) = program.forms[0].clone() else {
            panic!("return event")
        };
        if explicit {
            event.target = NormReturnTargetSyntax::Explicit(event.value.clone());
        }
        event
    }

    fn push(stack: &mut ReturnTargetStack, boundary: ReturnBoundary) -> ReturnTargetFrame {
        let origin = event(false).origin;
        stack.push_frame_with_boundary(
            ReturnFrameOwner::AnonymousClosure,
            ReturnSlotRef {
                identity: None,
                binding_slot: None,
                name: None,
                origin: origin.clone(),
            },
            None,
            None,
            origin,
            boundary,
        )
    }

    #[test]
    fn ordinary_barrier_limits_implicit_target_and_tracks_pops() {
        let mut stack = ReturnTargetStack::new();
        let outer = push(&mut stack, ReturnBoundary::Ordinary);
        let inner = push(&mut stack, ReturnBoundary::Ordinary);
        assert_ne!(outer.frame_id, inner.frame_id);
        assert_eq!(
            resolve_return_target(&stack, &event(false), None).unwrap(),
            Some(ResolvedReturnTarget::ActiveFrame(inner.frame_id))
        );
        assert_eq!(stack.pop_frame().unwrap().frame_id, inner.frame_id);
        assert_eq!(
            resolve_return_target(&stack, &event(false), None).unwrap(),
            Some(ResolvedReturnTarget::ActiveFrame(outer.frame_id))
        );
        stack.pop_frame();
        assert_eq!(
            resolve_return_target(&stack, &event(false), None)
                .unwrap_err()
                .code,
            Some(ResolverCode::ReturnOutsideReturnableContext)
        );
    }

    #[test]
    fn inplace_is_transparent_to_implicit_and_explicit_return() {
        let mut stack = ReturnTargetStack::new();
        let outer = push(&mut stack, ReturnBoundary::Ordinary);
        let inner = push(&mut stack, ReturnBoundary::InPlace);
        assert_eq!(
            resolve_return_target(&stack, &event(false), None).unwrap(),
            Some(ResolvedReturnTarget::ActiveFrame(outer.frame_id))
        );
        for frame in [outer, inner] {
            let resolver =
                |_: &NormExpr| ExplicitReturnTargetResolution::ActiveFrame(frame.frame_id);
            assert_eq!(
                resolve_return_target(&stack, &event(true), Some(&resolver)).unwrap(),
                Some(ResolvedReturnTarget::ActiveFrame(frame.frame_id))
            );
        }
    }

    #[test]
    fn active_outer_target_and_captured_capability_cannot_bypass_barrier() {
        let mut stack = ReturnTargetStack::new();
        let outer = push(&mut stack, ReturnBoundary::Ordinary);
        let inner = push(&mut stack, ReturnBoundary::Ordinary);
        push(&mut stack, ReturnBoundary::InPlace);
        assert!(stack.active_frame(outer.frame_id).is_some());
        assert_eq!(
            stack.target_reachability(outer.frame_id),
            ReturnTargetReachability::Blocked
        );
        assert_eq!(
            stack.target_reachability(inner.frame_id),
            ReturnTargetReachability::Reachable
        );
        let captured_capability =
            |_: &NormExpr| ExplicitReturnTargetResolution::ActiveFrame(outer.frame_id);
        assert_eq!(
            resolve_return_target(&stack, &event(true), Some(&captured_capability))
                .unwrap_err()
                .code,
            Some(ResolverCode::ReturnTargetOutsideReachableSegment)
        );
    }

    #[test]
    fn missing_boundary_evidence_preserves_both_targets_unbound() {
        let mut stack = ReturnTargetStack::new();
        let outer = push(&mut stack, ReturnBoundary::Ordinary);
        let unknown = stack.push_frame(
            outer.owner.clone(),
            outer.return_slot.clone(),
            outer.self_identity.clone(),
            outer.callable_self_owner,
            outer.origin.clone(),
        );
        assert_eq!(unknown.boundary, ReturnBoundary::Unknown);
        assert_eq!(
            stack.target_reachability(outer.frame_id),
            ReturnTargetReachability::Unavailable
        );
        let resolver = |_: &NormExpr| ExplicitReturnTargetResolution::ActiveFrame(outer.frame_id);
        assert_eq!(
            resolve_return_target(&stack, &event(false), None).unwrap(),
            None
        );
        assert_eq!(
            resolve_return_target(&stack, &event(true), Some(&resolver)).unwrap(),
            None
        );
        // A proved nearer barrier closes a segment independently of unknown outer boundaries.
        let inner = push(&mut stack, ReturnBoundary::Ordinary);
        assert_eq!(
            resolve_return_target(&stack, &event(false), None).unwrap(),
            Some(ResolvedReturnTarget::ActiveFrame(inner.frame_id))
        );
    }
}
