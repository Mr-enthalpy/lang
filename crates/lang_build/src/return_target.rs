//! Return target binding over normalized return terminal material.
//!
//! This pass answers only "which active return frame receives this return
//! event?" It does not perform later control-flow completion, return value
//! typing, propagation, lowering, lifetime checks, or resource scheduling.

use lang_syntax::{
    NormBindingSlot, NormClosure, NormExpr, NormForm, NormOrigin, NormPattern, NormPatternElem,
    NormProgram, NormReturnEvent, NormReturnTargetSyntax,
};

use crate::model::{Diagnostic, Provenance, ResolverCode, SymbolId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReturnFrameId(pub usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnSlotRef {
    /// The complete normalized return binding slot. Keeping the Pattern and
    /// its annotations/policy is required for later pattern-directed result
    /// delivery; this pass still performs target binding only.
    pub binding_slot: Option<NormBindingSlot>,
    /// Convenience spelling for the restricted current self-target
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
    /// Temporary v0.9 identity placeholder.
    ///
    /// This is the normalized spelling bound by the first written formal
    /// position, whatever that spelling is. That position denotes callable
    /// self; the word `self` is conventional, not semantic.
    ///
    /// Future explicit-target resolution must use the lexical self slot /
    /// function-object self identity, not text equality.
    ///
    /// TODO(return-capability): do not reuse this field as the semantic self
    /// identity for return completion. It is only a diagnostic/validation
    /// placeholder until lexical self-slot identity is available.
    pub name: String,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnTargetFrame {
    pub frame_id: ReturnFrameId,
    pub return_slot: ReturnSlotRef,
    pub owner: ReturnFrameOwner,
    pub self_identity: Option<ReturnSelfIdentity>,
    /// Anonymous callable type that gives source `Self` its semantic identity.
    /// This is present for every alpha-normalized callable, including in-place
    /// closures, independently of the temporary written-self binder path.
    pub anonymous_self_owner: Option<lang_syntax::NormSemanticOwnerId>,
    pub origin: NormOrigin,
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

    pub fn push_frame(
        &mut self,
        owner: ReturnFrameOwner,
        return_slot: ReturnSlotRef,
        self_identity: Option<ReturnSelfIdentity>,
        anonymous_self_owner: Option<lang_syntax::NormSemanticOwnerId>,
        origin: NormOrigin,
    ) -> ReturnTargetFrame {
        let frame = ReturnTargetFrame {
            frame_id: ReturnFrameId(self.next_id),
            return_slot,
            owner,
            self_identity,
            anonymous_self_owner,
            origin,
        };
        self.next_id += 1;
        self.frames.push(frame.clone());
        frame
    }

    pub fn pop_frame(&mut self) -> Option<ReturnTargetFrame> {
        self.frames.pop()
    }

    pub fn nearest(&self) -> Option<&ReturnTargetFrame> {
        self.frames.last()
    }

    pub fn find_self_identity(&self, name: &str) -> Vec<&ReturnTargetFrame> {
        self.frames
            .iter()
            .rev()
            .filter(|frame| {
                frame
                    .self_identity
                    .as_ref()
                    .is_some_and(|identity| identity.name == name)
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnresolvedReturnTargetForm {
    ImplicitNearest,
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

pub struct ReturnTargetBinder {
    stack: ReturnTargetStack,
    report: ReturnTargetBindingReport,
}

impl ReturnTargetBinder {
    pub fn new() -> Self {
        Self {
            stack: ReturnTargetStack::new(),
            report: ReturnTargetBindingReport::default(),
        }
    }

    pub fn finish(self) -> ReturnTargetBindingReport {
        self.report
    }

    pub fn enter_returnable_closure(&mut self, closure: &NormClosure, owner: ReturnFrameOwner) {
        let return_slot = return_slot_ref(closure);
        let self_identity = self_identity_from_closure(closure);
        let anonymous_self_owner = closure.semantic_owner.map(|owner| owner.id);
        let frame = self.stack.push_frame(
            owner,
            return_slot,
            self_identity,
            anonymous_self_owner,
            closure.origin.clone(),
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
            NormForm::Expr(expr) | NormForm::TailValue(expr) => self.visit_expr(expr),
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
            NormExpr::Name { .. }
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
        match resolve_return_target(&self.stack, return_event) {
            Ok(resolved_target) => self.report.bound_events.push(BoundReturnEvent {
                value: return_event.value.clone(),
                unresolved_target: unbound.target,
                resolved_target,
                origin: return_event.origin.clone(),
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

impl Default for ReturnTargetBinder {
    fn default() -> Self {
        Self::new()
    }
}

fn resolve_return_target(
    stack: &ReturnTargetStack,
    return_event: &NormReturnEvent,
) -> Result<ResolvedReturnTarget, Diagnostic> {
    match &return_event.target {
        NormReturnTargetSyntax::ImplicitNearest => stack
            .nearest()
            .map(|frame| ResolvedReturnTarget::ActiveFrame(frame.frame_id))
            .ok_or_else(|| {
                return_diagnostic(
                    ResolverCode::ReturnOutsideReturnableContext,
                    "ReturnOutsideReturnableContext: return event has no active return target",
                    &return_event.origin,
                )
            }),
        NormReturnTargetSyntax::Explicit(target) => resolve_explicit_return_target(stack, target),
    }
}

fn resolve_explicit_return_target(
    stack: &ReturnTargetStack,
    target: &NormExpr,
) -> Result<ResolvedReturnTarget, Diagnostic> {
    match target {
        NormExpr::Name { text, origin } => {
            let matches = stack.find_self_identity(text);
            match matches.as_slice() {
                [frame] => Ok(ResolvedReturnTarget::ActiveFrame(frame.frame_id)),
                [] => Err(return_diagnostic(
                    ResolverCode::ReturnTargetNotActive,
                    "ReturnTargetNotActive: explicit return target is not active",
                    origin,
                )),
                _ => Err(return_diagnostic(
                    ResolverCode::AmbiguousReturnTarget,
                    "AmbiguousReturnTarget: explicit return target matched multiple active frames",
                    origin,
                )),
            }
        }
        NormExpr::Call { origin, .. }
        | NormExpr::Product(lang_syntax::NormProduct { origin, .. })
        | NormExpr::Literal { origin, .. }
        | NormExpr::Nav { origin, .. }
        | NormExpr::Closure(NormClosure { origin, .. })
        | NormExpr::OperatorTarget { origin, .. }
        | NormExpr::Error(lang_syntax::NormError { origin, .. })
        | NormExpr::Unsupported { origin, .. } => Err(return_diagnostic(
            ResolverCode::UnsupportedReturnTargetForm,
            "UnsupportedReturnTargetForm: explicit return target form is outside the restricted v0.9 return target binder",
            origin,
        )),
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
    let Some(head) = &closure.head else {
        return ReturnSlotRef {
            binding_slot: None,
            name: None,
            origin: closure.origin.clone(),
        };
    };
    let Some(returns) = &head.returns else {
        return ReturnSlotRef {
            binding_slot: None,
            name: None,
            origin: head.origin.clone(),
        };
    };
    ReturnSlotRef {
        binding_slot: Some(returns.clone()),
        name: binding_slot_name(returns),
        origin: returns.origin.clone(),
    }
}

fn self_identity_from_closure(closure: &NormClosure) -> Option<ReturnSelfIdentity> {
    let head = closure.head.as_ref()?;
    match head.formal_frame().written_self? {
        NormPatternElem::BindingSlot(slot) => match &slot.value_pattern {
            NormPattern::Binder { name, origin } => Some(ReturnSelfIdentity {
                name: name.clone(),
                origin: origin.clone(),
            }),
            NormPattern::OperatorBinder { spelling, origin } => Some(ReturnSelfIdentity {
                name: spelling.clone(),
                origin: origin.clone(),
            }),
            _ => None,
        },
        NormPatternElem::Pattern(pattern) => match pattern {
            NormPattern::Binder { name, origin } => Some(ReturnSelfIdentity {
                name: name.clone(),
                origin: origin.clone(),
            }),
            NormPattern::OperatorBinder { spelling, origin } => Some(ReturnSelfIdentity {
                name: spelling.clone(),
                origin: origin.clone(),
            }),
            _ => None,
        },
        _ => None,
    }
}

fn binding_slot_name(slot: &NormBindingSlot) -> Option<String> {
    match &slot.value_pattern {
        NormPattern::Binder { name, .. } => Some(name.clone()),
        NormPattern::OperatorBinder { spelling, .. } => Some(spelling.clone()),
        _ => None,
    }
}

fn unbound_event(return_event: &NormReturnEvent) -> UnboundReturnEvent {
    UnboundReturnEvent {
        value: return_event.value.clone(),
        target: match &return_event.target {
            NormReturnTargetSyntax::ImplicitNearest => UnresolvedReturnTargetForm::ImplicitNearest,
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
            NormForm::Expr(expr) | NormForm::TailValue(expr) => {
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
        NormExpr::Name { .. }
        | NormExpr::Literal { .. }
        | NormExpr::Nav { .. }
        | NormExpr::OperatorTarget { .. }
        | NormExpr::Error(_)
        | NormExpr::Unsupported { .. } => {}
    }
}
