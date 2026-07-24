//! v0.4 Normalized AST prototype.
//!
//! Value-side `NormExpr` and pattern-side `NormPattern` remain distinct. Raw
//! expression-shaped syntax is normalized as pattern material only when the
//! surrounding syntactic context is pattern, annotation, or extraction.
//! This prototype records that boundary; explicit bridge syntax/lowering is
//! future work unless it is already present in Raw AST.

use std::collections::BTreeSet;

use crate::{
    AliasBinderAst, AnnotationTermAst, AtomAst, AtomKind, BinderNameAst, BindingAnnotationAst,
    BindingPatternAst, BindingSlotAst, BodyBlockAst, CanonicalNameRole, CanonicalProductElementAst,
    CanonicalSkeletonAst, CaptureItemAst, ClosureAst, ClosureBodyAst, ClosurePlacementAst,
    DeduceListAst, EntityRefAst, ErrorAst, ExprAst, ExprKind, FnHeadPrefixAst, FormAst,
    HeadClauseAst, LetAliasAst, LetAst, NavComponentAst, OperatorExprAst, OperatorExprKind,
    OperatorFixity, OperatorNameAst, ParamClauseAst, PipeExprAst, PolicyAtomAst, PolicyChoiceAst,
    PolicyConjunctionAst, PolicySpecAst, ProductElementAst, ProductExprAst, ProductExtractAst,
    ProductExtractElementAst, ProgramAst, ReturnClauseAst, SegmentAst, SegmentElementAst,
    SelectorAst, Span, ValuePolicyPatternAst, WithClauseAst, WithClauseKind,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormProgram {
    pub forms: Vec<NormForm>,
    pub origin: NormOrigin,
}

/// A normalized program whose global Pattern-layer invariants have been
/// checked.
///
/// This certificate does not assert that parser recovery produced no
/// `NormExpr::Error` nodes. Consumers that need a recovery-free program must
/// require a separate proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternValidatedNormProgram {
    program: NormProgram,
}

impl PatternValidatedNormProgram {
    pub fn as_program(&self) -> &NormProgram {
        &self.program
    }

    pub fn into_program(self) -> NormProgram {
        self.program
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternInvalidNormProgram {
    pub program: NormProgram,
    pub pattern_errors: Vec<PatternValidationError>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormForm {
    Let(NormDecl),
    Alias(NormDecl),
    Expr(NormExpr),
    TailValue(NormExpr),
    ReturnEvent(NormReturnEvent),
    Error(NormError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormReturnEvent {
    pub value: NormExpr,
    pub target: NormReturnTargetSyntax,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormReturnTargetSyntax {
    ImplicitNearest,
    Explicit(NormExpr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormDecl {
    Let {
        slot: NormBindingSlot,
        origin: NormOrigin,
    },
    Alias {
        policy: Option<NormPolicySpec>,
        binder: NormAliasBinder,
        target: NormEntityRef,
        origin: NormOrigin,
    },
    Error(NormError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormExpr {
    Call {
        source: NormProduct,
        target: Box<NormExpr>,
        origin: NormOrigin,
    },
    Product(NormProduct),
    Name {
        text: String,
        origin: NormOrigin,
    },
    Literal {
        kind: NormLiteralKind,
        text: String,
        origin: NormOrigin,
    },
    Nav {
        components: Vec<NormNavComponent>,
        explicit_terminated: bool,
        origin: NormOrigin,
    },
    Closure(NormClosure),
    OperatorTarget {
        spelling: String,
        fixity: NormOperatorFixity,
        arity: usize,
        origin: NormOrigin,
    },
    Error(NormError),
    Unsupported {
        raw_kind_summary: String,
        origin: NormOrigin,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormProduct {
    pub elements: Vec<NormProductElem>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormProductElem {
    Expr(NormExpr),
    Unit { origin: NormOrigin },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormPattern {
    Binder {
        name: String,
        origin: NormOrigin,
    },
    OperatorBinder {
        spelling: String,
        origin: NormOrigin,
    },
    Product {
        elements: Vec<NormPatternElem>,
        origin: NormOrigin,
    },
    /// Match the remaining normalized nodes at this structural level.
    Pack {
        inner: Box<NormPattern>,
        origin: NormOrigin,
    },
    Unit {
        origin: NormOrigin,
    },
    HoleRef {
        target: HoleBinderId,
        name: String,
        origin: NormOrigin,
    },
    AnonymousHole {
        origin: NormOrigin,
    },
    Name {
        name: String,
        origin: NormOrigin,
    },
    Literal {
        text: String,
        origin: NormOrigin,
    },
    Nav {
        components: Vec<NormNavComponent>,
        explicit_terminated: bool,
        origin: NormOrigin,
    },
    Sequence {
        elements: Vec<NormPattern>,
        origin: NormOrigin,
    },
    Skeleton {
        skeleton: NormSkeleton,
        origin: NormOrigin,
    },
    BindingSlot {
        slot: Box<NormBindingSlot>,
        origin: NormOrigin,
    },
    Error(NormError),
    Unsupported {
        raw_kind_summary: String,
        origin: NormOrigin,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormPatternElem {
    Pattern(NormPattern),
    BindingSlot(NormBindingSlot),
    Unit { origin: NormOrigin },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackPatternLayerError {
    pub pack_count: usize,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternValidationError {
    MultiplePacks(PackPatternLayerError),
    NonCanonicalPackOperand {
        origin: NormOrigin,
    },
    DuplicateHole {
        name: String,
        first: HoleBinderId,
        duplicate: HoleBinderId,
        origin: NormOrigin,
    },
}

impl PatternValidationError {
    pub fn origin(&self) -> &NormOrigin {
        match self {
            Self::MultiplePacks(error) => &error.origin,
            Self::NonCanonicalPackOperand { origin } | Self::DuplicateHole { origin, .. } => origin,
        }
    }
}

/// Validate the semantic, post-normalization rule that one structural level
/// may contain at most one pack. Nested Product and Sequence containers are
/// independent levels; Pack and BindingSlot are transparent.
pub fn validate_pack_pattern_layers(pattern: &NormPattern) -> Result<(), PackPatternLayerError> {
    match pattern {
        NormPattern::Product { elements, .. } => {
            validate_pack_pattern_element_level(elements)?;
            for element in elements {
                match element {
                    NormPatternElem::Pattern(pattern) => validate_pack_pattern_layers(pattern)?,
                    NormPatternElem::BindingSlot(slot) => {
                        validate_pack_pattern_layers(&slot.value_pattern)?;
                        if let Some(annotation) = &slot.annotation {
                            validate_pack_pattern_layers(&annotation.pattern)?;
                        }
                    }
                    NormPatternElem::Unit { .. } => {}
                }
            }
            Ok(())
        }
        NormPattern::Pack { inner, origin } => {
            if matches!(inner.as_ref(), NormPattern::Pack { .. }) {
                return Err(PackPatternLayerError {
                    pack_count: 2,
                    origin: origin.clone(),
                });
            }
            validate_pack_pattern_layers(inner)
        }
        NormPattern::Sequence { elements, .. } => {
            let packs = elements
                .iter()
                .filter_map(direct_pack_pattern_origin)
                .collect::<Vec<_>>();
            if packs.len() > 1 {
                return Err(PackPatternLayerError {
                    pack_count: packs.len(),
                    origin: (*packs[1]).clone(),
                });
            }
            for element in elements {
                validate_pack_pattern_layers(element)?;
            }
            Ok(())
        }
        NormPattern::BindingSlot { slot, .. } => {
            validate_pack_pattern_layers(&slot.value_pattern)?;
            if let Some(annotation) = &slot.annotation {
                validate_pack_pattern_layers(&annotation.pattern)?;
            }
            Ok(())
        }
        NormPattern::Binder { .. }
        | NormPattern::OperatorBinder { .. }
        | NormPattern::Unit { .. }
        | NormPattern::HoleRef { .. }
        | NormPattern::AnonymousHole { .. }
        | NormPattern::Name { .. }
        | NormPattern::Literal { .. }
        | NormPattern::Nav { .. }
        | NormPattern::Skeleton { .. }
        | NormPattern::Error(_)
        | NormPattern::Unsupported { .. } => Ok(()),
    }
}

pub fn validate_pack_pattern_element_level(
    elements: &[NormPatternElem],
) -> Result<(), PackPatternLayerError> {
    let packs = elements
        .iter()
        .filter_map(|element| match element {
            NormPatternElem::Pattern(NormPattern::Pack { origin, .. }) => Some(origin),
            NormPatternElem::BindingSlot(slot) => match &slot.value_pattern {
                NormPattern::Pack { origin, .. } => Some(origin),
                _ => None,
            },
            NormPatternElem::Pattern(_) | NormPatternElem::Unit { .. } => None,
        })
        .collect::<Vec<_>>();
    if packs.len() > 1 {
        return Err(PackPatternLayerError {
            pack_count: packs.len(),
            origin: (*packs[1]).clone(),
        });
    }
    Ok(())
}

/// Run the current global normalized Pattern invariants over every
/// Pattern-bearing location in a normalized program.
///
/// This pass owns pack cardinality, rejection of a bare Product Pack operand,
/// and uniqueness of DeduceList hole names across the active telescope. It
/// does not prove resolved ordered/unordered Pack applicability, stable
/// Pattern-head identity, general matching support, or recovery freedom.
pub fn validate_normalized_patterns(
    program: &NormProgram,
) -> Result<(), Vec<PatternValidationError>> {
    let mut errors = Vec::new();
    collect_program_pack_errors(program, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn collect_program_pack_errors(program: &NormProgram, errors: &mut Vec<PatternValidationError>) {
    for form in &program.forms {
        match form {
            NormForm::Let(decl) | NormForm::Alias(decl) => {
                collect_decl_pack_errors(decl, errors);
            }
            NormForm::Expr(expr) | NormForm::TailValue(expr) => {
                collect_expr_pack_errors(expr, errors);
            }
            NormForm::ReturnEvent(event) => {
                collect_expr_pack_errors(&event.value, errors);
                if let NormReturnTargetSyntax::Explicit(target) = &event.target {
                    collect_expr_pack_errors(target, errors);
                }
            }
            NormForm::Error(_) => {}
        }
    }
}

fn collect_decl_pack_errors(decl: &NormDecl, errors: &mut Vec<PatternValidationError>) {
    match decl {
        NormDecl::Let { slot, .. } => collect_slot_pack_errors(slot, errors),
        NormDecl::Alias { target, .. } => {
            collect_nav_component_pack_errors(&target.components, errors);
        }
        NormDecl::Error(_) => {}
    }
}

fn collect_slot_pack_errors(slot: &NormBindingSlot, errors: &mut Vec<PatternValidationError>) {
    for hole in &slot.deduce {
        if let Some(first) = hole.duplicate_of {
            errors.push(PatternValidationError::DuplicateHole {
                name: hole.name.clone(),
                first,
                duplicate: hole.id,
                origin: hole.origin.clone(),
            });
        }
        if let Some(annotation) = &hole.annotation {
            collect_pattern_pack_errors(&annotation.pattern, errors);
        }
    }
    collect_pattern_pack_errors(&slot.value_pattern, errors);
    if let Some(annotation) = &slot.annotation {
        collect_pattern_pack_errors(&annotation.pattern, errors);
    }
    if let Some(initializer) = &slot.initializer {
        collect_expr_pack_errors(initializer, errors);
    }
}

fn collect_pattern_pack_errors(pattern: &NormPattern, errors: &mut Vec<PatternValidationError>) {
    match pattern {
        NormPattern::Product { elements, .. } => {
            collect_pattern_element_level_error(elements, errors);
            for element in elements {
                match element {
                    NormPatternElem::Pattern(pattern) => {
                        collect_pattern_pack_errors(pattern, errors);
                    }
                    NormPatternElem::BindingSlot(slot) => {
                        collect_slot_pack_errors(slot, errors);
                    }
                    NormPatternElem::Unit { .. } => {}
                }
            }
        }
        NormPattern::Sequence { elements, .. } => {
            collect_pattern_level_error(elements, errors);
            for element in elements {
                collect_pattern_pack_errors(element, errors);
            }
        }
        NormPattern::Pack { inner, origin } => {
            if direct_pack_pattern_origin(inner).is_some() {
                errors.push(PatternValidationError::MultiplePacks(
                    PackPatternLayerError {
                        pack_count: 2,
                        origin: origin.clone(),
                    },
                ));
            }
            if matches!(inner.as_ref(), NormPattern::Product { .. }) {
                errors.push(PatternValidationError::NonCanonicalPackOperand {
                    origin: origin.clone(),
                });
            }
            collect_pattern_pack_errors(inner, errors);
        }
        NormPattern::BindingSlot { slot, .. } => collect_slot_pack_errors(slot, errors),
        NormPattern::Nav { components, .. } => {
            collect_nav_component_pack_errors(components, errors);
        }
        NormPattern::Skeleton { skeleton, .. } => {
            collect_skeleton_pack_errors(skeleton, errors);
        }
        NormPattern::Binder { .. }
        | NormPattern::OperatorBinder { .. }
        | NormPattern::Unit { .. }
        | NormPattern::HoleRef { .. }
        | NormPattern::AnonymousHole { .. }
        | NormPattern::Name { .. }
        | NormPattern::Literal { .. }
        | NormPattern::Error(_)
        | NormPattern::Unsupported { .. } => {}
    }
}

fn collect_pattern_element_level_error(
    elements: &[NormPatternElem],
    errors: &mut Vec<PatternValidationError>,
) {
    let packs = elements
        .iter()
        .filter_map(direct_pack_element_origin)
        .collect::<Vec<_>>();
    if packs.len() > 1 {
        errors.push(PatternValidationError::MultiplePacks(
            PackPatternLayerError {
                pack_count: packs.len(),
                origin: (*packs[1]).clone(),
            },
        ));
    }
}

fn collect_pattern_level_error(elements: &[NormPattern], errors: &mut Vec<PatternValidationError>) {
    let packs = elements
        .iter()
        .filter_map(direct_pack_pattern_origin)
        .collect::<Vec<_>>();
    if packs.len() > 1 {
        errors.push(PatternValidationError::MultiplePacks(
            PackPatternLayerError {
                pack_count: packs.len(),
                origin: (*packs[1]).clone(),
            },
        ));
    }
}

fn direct_pack_element_origin(element: &NormPatternElem) -> Option<&NormOrigin> {
    match element {
        NormPatternElem::Pattern(pattern) => direct_pack_pattern_origin(pattern),
        NormPatternElem::BindingSlot(slot) => direct_pack_pattern_origin(&slot.value_pattern),
        NormPatternElem::Unit { .. } => None,
    }
}

fn direct_pack_pattern_origin(pattern: &NormPattern) -> Option<&NormOrigin> {
    match pattern {
        NormPattern::Pack { origin, .. } => Some(origin),
        NormPattern::BindingSlot { slot, .. } => direct_pack_pattern_origin(&slot.value_pattern),
        _ => None,
    }
}

fn collect_expr_pack_errors(expr: &NormExpr, errors: &mut Vec<PatternValidationError>) {
    match expr {
        NormExpr::Call { source, target, .. } => {
            for element in &source.elements {
                if let NormProductElem::Expr(expr) = element {
                    collect_expr_pack_errors(expr, errors);
                }
            }
            collect_expr_pack_errors(target, errors);
        }
        NormExpr::Product(product) => {
            for element in &product.elements {
                if let NormProductElem::Expr(expr) = element {
                    collect_expr_pack_errors(expr, errors);
                }
            }
        }
        NormExpr::Nav { components, .. } => {
            collect_nav_component_pack_errors(components, errors);
        }
        NormExpr::Closure(closure) => collect_closure_pack_errors(closure, errors),
        NormExpr::Name { .. }
        | NormExpr::Literal { .. }
        | NormExpr::OperatorTarget { .. }
        | NormExpr::Error(_)
        | NormExpr::Unsupported { .. } => {}
    }
}

fn collect_closure_pack_errors(closure: &NormClosure, errors: &mut Vec<PatternValidationError>) {
    if let Some(head) = &closure.head {
        for hole in &head.deduce {
            if let Some(first) = hole.duplicate_of {
                errors.push(PatternValidationError::DuplicateHole {
                    name: hole.name.clone(),
                    first,
                    duplicate: hole.id,
                    origin: hole.origin.clone(),
                });
            }
        }
    }
    if let Some(head) = &closure.head {
        for hole in &head.deduce {
            if let Some(annotation) = &hole.annotation {
                collect_pattern_pack_errors(&annotation.pattern, errors);
            }
        }
        collect_pattern_element_level_error(&head.params, errors);
        for param in &head.params {
            match param {
                NormPatternElem::Pattern(pattern) => collect_pattern_pack_errors(pattern, errors),
                NormPatternElem::BindingSlot(slot) => collect_slot_pack_errors(slot, errors),
                NormPatternElem::Unit { .. } => {}
            }
        }
        if let Some(returns) = &head.returns {
            collect_slot_pack_errors(returns, errors);
        }
        for capture in &head.captures {
            collect_slot_pack_errors(&capture.slot, errors);
            collect_expr_pack_errors(&capture.initializer, errors);
        }
        for clause in &head.clauses {
            match clause {
                NormHeadClause::Require { expr, .. }
                | NormHeadClause::Pre { expr, .. }
                | NormHeadClause::Post { expr, .. }
                | NormHeadClause::LifetimePre { expr, .. }
                | NormHeadClause::LifetimePost { expr, .. } => {
                    collect_expr_pack_errors(expr, errors);
                }
                NormHeadClause::Error(_) => {}
            }
        }
    }
    match &closure.body {
        NormClosureBody::Block(body) | NormClosureBody::NamedBlock { body, .. } => {
            collect_program_pack_errors(body, errors);
        }
        NormClosureBody::Defaulted { .. } | NormClosureBody::Delete(_) => {}
    }
}

fn collect_nav_component_pack_errors(
    components: &[NormNavComponent],
    errors: &mut Vec<PatternValidationError>,
) {
    for component in components {
        if let NormNavComponent::Group { expr, .. } = component {
            collect_expr_pack_errors(expr, errors);
        }
    }
}

fn collect_skeleton_pack_errors(skeleton: &NormSkeleton, errors: &mut Vec<PatternValidationError>) {
    match skeleton {
        NormSkeleton::Segment { elements, .. } => {
            for element in elements {
                collect_skeleton_pack_errors(element, errors);
            }
        }
        NormSkeleton::Product { elements, .. } => {
            for element in elements {
                if let NormSkeletonElem::Skeleton(skeleton) = element {
                    collect_skeleton_pack_errors(skeleton, errors);
                }
            }
        }
        NormSkeleton::Nav { components, .. } => {
            collect_nav_component_pack_errors(components, errors);
        }
        NormSkeleton::Wildcard { .. }
        | NormSkeleton::Name { .. }
        | NormSkeleton::HoleRef { .. }
        | NormSkeleton::Literal { .. }
        | NormSkeleton::Error(_) => {}
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormAnnotation {
    pub pattern: NormPattern,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormBindingSlot {
    pub policy: Option<NormPolicySpec>,
    pub has_let: bool,
    pub deduce: Vec<NormHoleDecl>,
    pub value_pattern: NormPattern,
    pub annotation: Option<NormAnnotation>,
    pub with_clause: Option<NormWithClause>,
    pub initializer: Option<Box<NormExpr>>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormPolicySpec {
    pub value_policy: NormValuePolicyPattern,
    pub pattern_policy: Option<NormPolicyConjunction>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormValuePolicyPattern {
    Conjunction(NormPolicyConjunction),
    Absent { origin: NormOrigin },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormPolicyConjunction {
    pub choices: Vec<NormPolicyChoice>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormPolicyChoice {
    pub atoms: Vec<NormPolicyAtom>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormPolicyAtom {
    Name {
        text: String,
        origin: NormOrigin,
    },
    HoleRef {
        target: HoleBinderId,
        text: String,
        origin: NormOrigin,
    },
    Group {
        conjunction: Box<NormPolicyConjunction>,
        origin: NormOrigin,
    },
    AbsentValuePattern {
        origin: NormOrigin,
    },
    Error(NormError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormHoleDecl {
    pub id: HoleBinderId,
    pub name: String,
    pub annotation: Option<NormAnnotation>,
    /// Present only on an invalid redeclaration. DeduceLists are telescopes and
    /// monotonically extend the active hole environment; they never shadow.
    pub duplicate_of: Option<HoleBinderId>,
    pub origin: NormOrigin,
}

/// Alpha-normalized lexical identity of a DeduceList binder within one
/// normalized-program owner.
///
/// The ordinal is allocated by lexical traversal and is intentionally
/// independent of source spans and source spelling. Spans remain provenance,
/// not semantic binder identity. Equality is meaningful only inside the
/// owning `NormProgram`; build-world identity must pair this local identity
/// with an owner/source-unit identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HoleBinderId {
    repr: HoleBinderIdRepr,
}

impl HoleBinderId {
    fn provisional_source() -> Self {
        Self {
            repr: HoleBinderIdRepr::ProvisionalSource,
        }
    }

    fn provisional_generated(key: GeneratedHoleKey) -> Self {
        Self {
            repr: HoleBinderIdRepr::ProvisionalGenerated(key),
        }
    }

    fn alpha(ordinal: u32) -> Self {
        Self {
            repr: HoleBinderIdRepr::LocalOrdinal(ordinal),
        }
    }

    fn generated_key(self) -> Option<GeneratedHoleKey> {
        match self.repr {
            HoleBinderIdRepr::ProvisionalGenerated(key) => Some(key),
            HoleBinderIdRepr::LocalOrdinal(_) | HoleBinderIdRepr::ProvisionalSource => None,
        }
    }

    /// Return the owner-scoped ordinal assigned by alpha normalization.
    ///
    /// This is not a cross-program or cross-source-unit identity.
    pub fn local_ordinal(self) -> u32 {
        match self.repr {
            HoleBinderIdRepr::LocalOrdinal(ordinal) => ordinal,
            HoleBinderIdRepr::ProvisionalSource | HoleBinderIdRepr::ProvisionalGenerated(_) => {
                panic!("provisional hole identity has no local alpha ordinal")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum HoleBinderIdRepr {
    LocalOrdinal(u32),
    ProvisionalSource,
    ProvisionalGenerated(GeneratedHoleKey),
}

/// Hygienic, generated-syntax-local key used before alpha normalization.
///
/// The key is interpreted inside the generated callable scope. It is never
/// looked up through source spelling, so a generated display name such as `T`
/// cannot collide with a user-written hole of the same spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeneratedHoleKey {
    pub rule: NormRule,
    pub local_ordinal: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct VisibleHole {
    id: HoleBinderId,
    key: VisibleHoleKey,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum VisibleHoleKey {
    SourceName(String),
    Generated(GeneratedHoleKey),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormWithClause {
    pub names: Vec<String>,
    pub explicit_empty: bool,
    pub error: Option<NormError>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormClosure {
    pub placement: NormClosurePlacement,
    pub head: Option<NormClosureHead>,
    pub body: NormClosureBody,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormClosureBody {
    Block(NormProgram),
    NamedBlock {
        strategy: String,
        body: NormProgram,
        origin: NormOrigin,
    },
    Defaulted {
        origin: NormOrigin,
    },
    Delete(NormDeleteBody),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NormOverloadStrategy {
    Ordinary,
    Named(String),
}

impl NormClosureBody {
    pub fn overload_strategy(&self) -> NormOverloadStrategy {
        match self {
            Self::NamedBlock { strategy, .. } => NormOverloadStrategy::Named(strategy.clone()),
            Self::Block(_) | Self::Defaulted { .. } | Self::Delete(_) => {
                NormOverloadStrategy::Ordinary
            }
        }
    }

    pub fn user_body(&self) -> Option<&NormProgram> {
        match self {
            Self::Block(body) | Self::NamedBlock { body, .. } => Some(body),
            Self::Defaulted { .. } | Self::Delete(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormDeleteBody {
    /// Normalized source spelling of the optional string literal, including
    /// quotes. Callable-tail parsing rejects non-string message expressions.
    pub message: Option<String>,
    pub origin: NormOrigin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormClosurePlacement {
    InPlace,
    Ordinary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormClosureHead {
    pub deduce: Vec<NormHoleDecl>,
    pub captures: Vec<NormCapture>,
    pub params: Vec<NormPatternElem>,
    pub call_policy: Option<NormPolicySpec>,
    pub returns: Option<NormBindingSlot>,
    pub clauses: Vec<NormHeadClause>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormCapture {
    pub slot: NormBindingSlot,
    pub initializer: NormExpr,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormHeadClause {
    Require { expr: NormExpr, origin: NormOrigin },
    Pre { expr: NormExpr, origin: NormOrigin },
    Post { expr: NormExpr, origin: NormOrigin },
    LifetimePre { expr: NormExpr, origin: NormOrigin },
    LifetimePost { expr: NormExpr, origin: NormOrigin },
    Error(NormError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormEntityRef {
    pub components: Vec<NormNavComponent>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormAliasBinder {
    Name {
        name: String,
        origin: NormOrigin,
    },
    Operator {
        spelling: String,
        origin: NormOrigin,
    },
    Error(NormError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormNavComponent {
    Name {
        name: String,
        origin: NormOrigin,
    },
    Operator {
        spelling: String,
        origin: NormOrigin,
    },
    Group {
        expr: Box<NormExpr>,
        origin: NormOrigin,
    },
    Error(NormError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormSkeleton {
    Segment {
        elements: Vec<NormSkeleton>,
        origin: NormOrigin,
    },
    Product {
        elements: Vec<NormSkeletonElem>,
        origin: NormOrigin,
    },
    Wildcard {
        origin: NormOrigin,
    },
    Name {
        name: String,
        role: NormCanonicalNameRole,
        origin: NormOrigin,
    },
    HoleRef {
        target: HoleBinderId,
        name: String,
        origin: NormOrigin,
    },
    Nav {
        components: Vec<NormNavComponent>,
        explicit_terminated: bool,
        origin: NormOrigin,
    },
    Literal {
        text: String,
        origin: NormOrigin,
    },
    Error(NormError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormSkeletonElem {
    Skeleton(NormSkeleton),
    Unit { origin: NormOrigin },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormCanonicalNameRole {
    Hole,
    NodeName,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormLiteralKind {
    Int,
    Float,
    String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormOperatorFixity {
    Prefix,
    Postfix,
    Binary,
    BracketCall,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormError {
    pub message: String,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormOrigin {
    Source(Span),
    Generated {
        rule: NormRule,
        span: Span,
    },
    Derived {
        rule: NormRule,
        span: Span,
        summary: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NormRule {
    ProductLift,
    ProductMerge,
    PipeFallback,
    SecondLegalityRepair,
    OperatorLowering,
    PrefixNegativeLowering,
    DotClosureLowering,
    MemberLowering,
    DoubleDotLowering,
    BracketCallLowering,
    BranchNameExpansion,
    AliasPreserve,
    ClosureNormalize,
    CaptureNameInference,
    PatternNormalize,
    Unsupported,
}

/// Scope construction and alpha-normalization for DeduceList binders.
///
/// Raw parsing preserves lexical spelling and provisional hole roles. This
/// pass is the sole producer of semantic `HoleBinderId` values: it walks the
/// normalized tree in lexical order, assigns fresh ordinals, rewrites every
/// scoped Pattern/policy occurrence to the exact binder, and diagnoses active
/// name redeclarations through `duplicate_of`.
#[derive(Default)]
struct HoleAlphaNormalizer {
    next_ordinal: u32,
}

impl HoleAlphaNormalizer {
    fn fresh_id(&mut self) -> HoleBinderId {
        let id = HoleBinderId::alpha(self.next_ordinal);
        self.next_ordinal = self
            .next_ordinal
            .checked_add(1)
            .expect("normalized program contains more than u32::MAX hole binders");
        id
    }

    fn normalize_program(&mut self, program: &mut NormProgram, holes: &[VisibleHole]) {
        for form in &mut program.forms {
            self.normalize_form(form, holes);
        }
    }

    fn normalize_form(&mut self, form: &mut NormForm, holes: &[VisibleHole]) {
        match form {
            NormForm::Let(decl) | NormForm::Alias(decl) => self.normalize_decl(decl, holes),
            NormForm::Expr(expr) | NormForm::TailValue(expr) => {
                self.normalize_expr(expr, holes);
            }
            NormForm::ReturnEvent(event) => {
                self.normalize_expr(&mut event.value, holes);
                if let NormReturnTargetSyntax::Explicit(target) = &mut event.target {
                    self.normalize_expr(target, holes);
                }
            }
            NormForm::Error(_) => {}
        }
    }

    fn normalize_decl(&mut self, decl: &mut NormDecl, holes: &[VisibleHole]) {
        match decl {
            NormDecl::Let { slot, .. } => {
                self.normalize_slot(slot, holes);
            }
            NormDecl::Alias { policy, target, .. } => {
                if let Some(policy) = policy {
                    self.normalize_policy_spec(policy, holes);
                }
                self.normalize_nav_components(&mut target.components, holes);
            }
            NormDecl::Error(_) => {}
        }
    }

    /// Normalize a let-shaped slot and return the hole environment visible to
    /// its following Pattern, annotation, and initializer. The returned
    /// environment never escapes the slot itself.
    fn normalize_slot(
        &mut self,
        slot: &mut NormBindingSlot,
        inherited: &[VisibleHole],
    ) -> Vec<VisibleHole> {
        // BindingSlot source order is policy, let, DeduceList, Pattern,
        // annotation, initializer. The leading policy may see inherited holes
        // but never a hole declared later by this slot.
        if let Some(policy) = &mut slot.policy {
            self.normalize_policy_spec(policy, inherited);
        }
        let visible = self.normalize_deduce_list(&mut slot.deduce, inherited);
        self.normalize_pattern(&mut slot.value_pattern, &visible);
        if let Some(annotation) = &mut slot.annotation {
            self.normalize_pattern(&mut annotation.pattern, &visible);
        }
        if let Some(initializer) = &mut slot.initializer {
            self.normalize_expr(initializer, &visible);
        }
        visible
    }

    fn normalize_deduce_list(
        &mut self,
        deduce: &mut [NormHoleDecl],
        inherited: &[VisibleHole],
    ) -> Vec<VisibleHole> {
        let mut visible = inherited.to_vec();
        for hole in deduce {
            // A telescope annotation sees ancestors and earlier binders, but
            // never the binder being declared or a later binder.
            if let Some(annotation) = &mut hole.annotation {
                self.normalize_pattern(&mut annotation.pattern, &visible);
            }

            let provisional_key = hole.id.generated_key();
            let id = self.fresh_id();
            let duplicate_of = provisional_key
                .is_none()
                .then(|| find_visible_source_hole(&visible, &hole.name).map(|first| first.id))
                .flatten();
            hole.id = id;
            hole.duplicate_of = duplicate_of;
            if duplicate_of.is_none() {
                visible.push(VisibleHole {
                    id,
                    key: provisional_key.map_or_else(
                        || VisibleHoleKey::SourceName(hole.name.clone()),
                        VisibleHoleKey::Generated,
                    ),
                });
            }
        }
        visible
    }

    fn normalize_expr(&mut self, expr: &mut NormExpr, holes: &[VisibleHole]) {
        match expr {
            NormExpr::Call { source, target, .. } => {
                self.normalize_product(source, holes);
                self.normalize_expr(target, holes);
            }
            NormExpr::Product(product) => self.normalize_product(product, holes),
            NormExpr::Nav { components, .. } => {
                self.normalize_nav_components(components, holes);
            }
            NormExpr::Closure(closure) => self.normalize_closure(closure, holes),
            NormExpr::Name { .. }
            | NormExpr::Literal { .. }
            | NormExpr::OperatorTarget { .. }
            | NormExpr::Error(_)
            | NormExpr::Unsupported { .. } => {}
        }
    }

    fn normalize_product(&mut self, product: &mut NormProduct, holes: &[VisibleHole]) {
        for element in &mut product.elements {
            if let NormProductElem::Expr(expr) = element {
                self.normalize_expr(expr, holes);
            }
        }
    }

    fn normalize_closure(&mut self, closure: &mut NormClosure, inherited: &[VisibleHole]) {
        let visible = if let Some(head) = &mut closure.head {
            let visible = self.normalize_deduce_list(&mut head.deduce, inherited);

            // Capture initializers are simultaneous with respect to value
            // binders. Each capture-local DeduceList still scopes that
            // capture's own initializer.
            for capture in &mut head.captures {
                let capture_holes = self.normalize_slot(&mut capture.slot, &visible);
                self.normalize_expr(&mut capture.initializer, &capture_holes);
            }
            for param in &mut head.params {
                self.normalize_pattern_element(param, &visible);
            }
            if let Some(policy) = &mut head.call_policy {
                self.normalize_policy_spec(policy, &visible);
            }
            if let Some(returns) = &mut head.returns {
                self.normalize_slot(returns, &visible);
            }
            for clause in &mut head.clauses {
                match clause {
                    NormHeadClause::Require { expr, .. }
                    | NormHeadClause::Pre { expr, .. }
                    | NormHeadClause::Post { expr, .. }
                    | NormHeadClause::LifetimePre { expr, .. }
                    | NormHeadClause::LifetimePost { expr, .. } => {
                        self.normalize_expr(expr, &visible);
                    }
                    NormHeadClause::Error(_) => {}
                }
            }
            visible
        } else {
            inherited.to_vec()
        };

        match &mut closure.body {
            NormClosureBody::Block(body) | NormClosureBody::NamedBlock { body, .. } => {
                self.normalize_program(body, &visible);
            }
            NormClosureBody::Defaulted { .. } | NormClosureBody::Delete(_) => {}
        }
    }

    fn normalize_pattern_element(&mut self, element: &mut NormPatternElem, holes: &[VisibleHole]) {
        match element {
            NormPatternElem::Pattern(pattern) => self.normalize_pattern(pattern, holes),
            NormPatternElem::BindingSlot(slot) => {
                self.normalize_slot(slot, holes);
            }
            NormPatternElem::Unit { .. } => {}
        }
    }

    fn normalize_pattern(&mut self, pattern: &mut NormPattern, holes: &[VisibleHole]) {
        match pattern {
            NormPattern::Product { elements, .. } => {
                for element in elements {
                    self.normalize_pattern_element(element, holes);
                }
            }
            NormPattern::Pack { inner, .. } => self.normalize_pattern(inner, holes),
            NormPattern::Name { name, origin } => {
                if let Some(hole) = find_visible_source_hole(holes, name) {
                    *pattern = NormPattern::HoleRef {
                        target: hole.id,
                        name: name.clone(),
                        origin: origin.clone(),
                    };
                }
            }
            NormPattern::HoleRef {
                target,
                name,
                origin,
            } => {
                if let Some(hole) = find_visible_hole_ref(holes, *target, name) {
                    *target = hole.id;
                } else if (*target).generated_key().is_some() {
                    panic!("generated hole reference has no hygienic binder in scope");
                } else {
                    *pattern = NormPattern::Name {
                        name: name.clone(),
                        origin: origin.clone(),
                    };
                }
            }
            NormPattern::Nav { components, .. } => {
                self.normalize_nav_components(components, holes);
            }
            NormPattern::Sequence { elements, .. } => {
                for element in elements {
                    self.normalize_pattern(element, holes);
                }
            }
            NormPattern::Skeleton { skeleton, .. } => {
                self.normalize_skeleton(skeleton, holes);
            }
            NormPattern::BindingSlot { slot, .. } => {
                self.normalize_slot(slot, holes);
            }
            NormPattern::Binder { .. }
            | NormPattern::OperatorBinder { .. }
            | NormPattern::Unit { .. }
            | NormPattern::AnonymousHole { .. }
            | NormPattern::Literal { .. }
            | NormPattern::Error(_)
            | NormPattern::Unsupported { .. } => {}
        }
    }

    fn normalize_skeleton(&mut self, skeleton: &mut NormSkeleton, holes: &[VisibleHole]) {
        match skeleton {
            NormSkeleton::Segment { elements, .. } => {
                for element in elements {
                    self.normalize_skeleton(element, holes);
                }
            }
            NormSkeleton::Product { elements, .. } => {
                for element in elements {
                    if let NormSkeletonElem::Skeleton(skeleton) = element {
                        self.normalize_skeleton(skeleton, holes);
                    }
                }
            }
            NormSkeleton::Name { name, role, origin } => {
                if let Some(hole) = find_visible_source_hole(holes, name) {
                    *skeleton = NormSkeleton::HoleRef {
                        target: hole.id,
                        name: name.clone(),
                        origin: origin.clone(),
                    };
                } else if *role == NormCanonicalNameRole::Hole {
                    // Raw roles are provisional lexical hints only.
                    *role = NormCanonicalNameRole::Unknown;
                }
            }
            NormSkeleton::HoleRef {
                target,
                name,
                origin,
            } => {
                if let Some(hole) = find_visible_hole_ref(holes, *target, name) {
                    *target = hole.id;
                } else if (*target).generated_key().is_some() {
                    panic!("generated skeleton hole reference has no hygienic binder in scope");
                } else {
                    *skeleton = NormSkeleton::Name {
                        name: name.clone(),
                        role: NormCanonicalNameRole::Unknown,
                        origin: origin.clone(),
                    };
                }
            }
            NormSkeleton::Nav { components, .. } => {
                self.normalize_nav_components(components, holes);
            }
            NormSkeleton::Wildcard { .. }
            | NormSkeleton::Literal { .. }
            | NormSkeleton::Error(_) => {}
        }
    }

    fn normalize_nav_components(
        &mut self,
        components: &mut [NormNavComponent],
        holes: &[VisibleHole],
    ) {
        for component in components {
            if let NormNavComponent::Group { expr, .. } = component {
                self.normalize_expr(expr, holes);
            }
        }
    }

    fn normalize_policy_spec(&mut self, policy: &mut NormPolicySpec, holes: &[VisibleHole]) {
        if let NormValuePolicyPattern::Conjunction(conjunction) = &mut policy.value_policy {
            self.normalize_policy_conjunction(conjunction, holes);
        }
        if let Some(conjunction) = &mut policy.pattern_policy {
            self.normalize_policy_conjunction(conjunction, holes);
        }
    }

    fn normalize_policy_conjunction(
        &mut self,
        conjunction: &mut NormPolicyConjunction,
        holes: &[VisibleHole],
    ) {
        for choice in &mut conjunction.choices {
            for atom in &mut choice.atoms {
                match atom {
                    NormPolicyAtom::Name { text, origin } => {
                        if let Some(hole) = find_visible_source_hole(holes, text) {
                            *atom = NormPolicyAtom::HoleRef {
                                target: hole.id,
                                text: text.clone(),
                                origin: origin.clone(),
                            };
                        }
                    }
                    NormPolicyAtom::HoleRef {
                        target,
                        text,
                        origin,
                    } => {
                        if let Some(hole) = find_visible_hole_ref(holes, *target, text) {
                            *target = hole.id;
                        } else if (*target).generated_key().is_some() {
                            panic!(
                                "generated policy hole reference has no hygienic binder in scope"
                            );
                        } else {
                            *atom = NormPolicyAtom::Name {
                                text: text.clone(),
                                origin: origin.clone(),
                            };
                        }
                    }
                    NormPolicyAtom::Group { conjunction, .. } => {
                        self.normalize_policy_conjunction(conjunction, holes);
                    }
                    NormPolicyAtom::AbsentValuePattern { .. } | NormPolicyAtom::Error(_) => {}
                }
            }
        }
    }
}

pub fn normalize_program(raw: &ProgramAst) -> NormProgram {
    let mut program = NormProgram {
        forms: raw.forms.iter().map(normalize_form).collect(),
        origin: NormOrigin::Source(raw.span),
    };
    HoleAlphaNormalizer::default().normalize_program(&mut program, &[]);
    program
}

/// The Pattern-layer-validated handoff for downstream build/semantic
/// consumers.
///
/// `normalize_program` remains available for dump/recovery tooling that must
/// inspect invalid structure. Passing this function proves only the global
/// normalized Pattern invariants; it does not prove the absence of recovered
/// syntax errors.
pub fn normalize_and_validate_patterns(
    raw: &ProgramAst,
) -> Result<PatternValidatedNormProgram, PatternInvalidNormProgram> {
    validate_normalized_pattern_layers(normalize_program(raw))
}

pub fn validate_normalized_pattern_layers(
    program: NormProgram,
) -> Result<PatternValidatedNormProgram, PatternInvalidNormProgram> {
    match validate_normalized_patterns(&program) {
        Ok(()) => Ok(PatternValidatedNormProgram { program }),
        Err(pattern_errors) => Err(PatternInvalidNormProgram {
            program,
            pattern_errors,
        }),
    }
}

pub fn dump_norm_program(program: &NormProgram) -> String {
    let mut output = String::new();
    line(
        &mut output,
        0,
        &format!("NormProgram {}", origin_inline(&program.origin)),
    );
    line(&mut output, 1, "forms:");
    for form in &program.forms {
        dump_norm_form(&mut output, form, 2);
    }
    output
}

fn normalize_form(form: &FormAst) -> NormForm {
    match form {
        FormAst::Let(let_ast) => NormForm::Let(normalize_let_decl(let_ast)),
        FormAst::AliasLet(alias) => NormForm::Alias(normalize_alias_decl(alias)),
        FormAst::Expr(expr) => NormForm::Expr(normalize_expr(expr)),
        FormAst::ReturnEvent(return_ev) => {
            let value = normalize_expr(&return_ev.value);
            let target = match &return_ev.target {
                crate::ReturnTargetAst::ImplicitNearest { .. } => {
                    NormReturnTargetSyntax::ImplicitNearest
                }
                crate::ReturnTargetAst::Explicit { target, .. } => {
                    NormReturnTargetSyntax::Explicit(normalize_expr(target))
                }
            };
            NormForm::ReturnEvent(NormReturnEvent {
                value,
                target,
                origin: NormOrigin::Source(return_ev.span),
            })
        }
        FormAst::Error(error) => NormForm::Error(normalize_error(error)),
    }
}

fn normalize_let_decl(let_ast: &LetAst) -> NormDecl {
    NormDecl::Let {
        slot: normalize_binding_slot(&let_ast.slot, &[]),
        origin: NormOrigin::Source(let_ast.span),
    }
}

fn normalize_alias_decl(alias: &LetAliasAst) -> NormDecl {
    let policy = alias.policy.as_ref().map(normalize_policy_spec);
    let binder = match &alias.binder {
        AliasBinderAst::Name(name) => NormAliasBinder::Name {
            name: name.text.clone(),
            origin: NormOrigin::Source(name.span),
        },
        AliasBinderAst::Operator(operator) => NormAliasBinder::Operator {
            spelling: operator.spelling.clone(),
            origin: NormOrigin::Source(operator.span),
        },
        AliasBinderAst::Error(error) => NormAliasBinder::Error(normalize_error(error)),
    };

    NormDecl::Alias {
        policy,
        binder,
        target: normalize_entity_ref(&alias.target),
        origin: NormOrigin::Generated {
            rule: NormRule::AliasPreserve,
            span: alias.span,
        },
    }
}

fn normalize_policy_spec(policy: &PolicySpecAst) -> NormPolicySpec {
    let value_policy = match &policy.value_policy {
        ValuePolicyPatternAst::Conjunction(conjunction) => {
            NormValuePolicyPattern::Conjunction(normalize_policy_conjunction(conjunction))
        }
        ValuePolicyPatternAst::Absent { span } => NormValuePolicyPattern::Absent {
            origin: NormOrigin::Source(*span),
        },
    };
    NormPolicySpec {
        value_policy,
        pattern_policy: policy
            .pattern_policy
            .as_ref()
            .map(normalize_policy_conjunction),
        origin: NormOrigin::Source(policy.span),
    }
}

fn normalize_policy_conjunction(conjunction: &PolicyConjunctionAst) -> NormPolicyConjunction {
    NormPolicyConjunction {
        choices: conjunction
            .choices
            .iter()
            .map(normalize_policy_choice)
            .collect(),
        origin: NormOrigin::Source(conjunction.span),
    }
}

fn normalize_policy_choice(choice: &PolicyChoiceAst) -> NormPolicyChoice {
    NormPolicyChoice {
        atoms: choice.atoms.iter().map(normalize_policy_atom).collect(),
        origin: NormOrigin::Source(choice.span),
    }
}

fn normalize_policy_atom(atom: &PolicyAtomAst) -> NormPolicyAtom {
    match atom {
        PolicyAtomAst::Name(name) => NormPolicyAtom::Name {
            text: name.text.clone(),
            origin: NormOrigin::Source(name.span),
        },
        PolicyAtomAst::Group { conjunction, span } => NormPolicyAtom::Group {
            conjunction: Box::new(normalize_policy_conjunction(conjunction)),
            origin: NormOrigin::Source(*span),
        },
        PolicyAtomAst::AbsentValuePattern { span } => NormPolicyAtom::AbsentValuePattern {
            origin: NormOrigin::Source(*span),
        },
        PolicyAtomAst::Error(error) => NormPolicyAtom::Error(normalize_error(error)),
    }
}

fn normalize_expr(expr: &ExprAst) -> NormExpr {
    // Value-side entry point. This must never reinterpret expression material as
    // extraction/pattern material; pattern contexts use the dedicated
    // normalize_*_as_pattern path below.
    match &expr.kind {
        ExprKind::Pipe(pipe) => normalize_pipe(pipe),
        ExprKind::Product(product) => NormExpr::Product(normalize_product_expr(product, true)),
        ExprKind::Error(error) => NormExpr::Error(normalize_error(error)),
    }
}

fn normalize_pipe(pipe: &PipeExprAst) -> NormExpr {
    let mut segments = pipe.segments.iter();
    let Some(first) = segments.next() else {
        return NormExpr::Error(NormError {
            message: "empty pipe expression".to_string(),
            origin: NormOrigin::Source(pipe.span),
        });
    };

    let mut current = normalize_segment_without_incoming(first);
    for segment in segments {
        current = normalize_segment_with_incoming(current, segment);
    }
    current
}

fn normalize_segment_without_incoming(segment: &SegmentAst) -> NormExpr {
    let items = normalize_segment_items(segment);
    lower_item_chain(None, &items, segment.span)
}

fn normalize_segment_with_incoming(incoming: NormExpr, segment: &SegmentAst) -> NormExpr {
    let items = normalize_segment_items(segment);
    if items.is_empty() {
        return NormExpr::Error(NormError {
            message: "empty incoming pipe segment".to_string(),
            origin: NormOrigin::Source(segment.span),
        });
    }

    let product_index = (1..items.len()).find(|index| items[*index].source_product().is_some());

    if let Some(product_index) = product_index {
        let target = lower_item_chain(None, &items[..product_index], segment.span);
        let incoming_product = source_product_from_expr(incoming, segment.span);
        let continuation_product = items[product_index]
            .source_product()
            .expect("product_index selected a source product");
        let merged = merge_products(incoming_product, continuation_product, segment.span);
        let mut current = make_call(
            merged,
            target,
            NormOrigin::Derived {
                rule: NormRule::ProductMerge,
                span: segment.span,
                summary: "source-product continuation".to_string(),
            },
        );
        current = lower_item_chain(Some(current), &items[product_index + 1..], segment.span);
        current
    } else {
        let target = lower_item_chain(None, &items, segment.span);
        let source = source_product_from_expr(incoming, segment.span);
        make_call(
            source,
            target,
            NormOrigin::Derived {
                rule: NormRule::PipeFallback,
                span: segment.span,
                summary: "no following source product".to_string(),
            },
        )
    }
}

#[derive(Clone, Debug)]
enum SegmentItem {
    Expr {
        expr: NormExpr,
        source_product: Option<NormProduct>,
    },
    Product(NormProduct),
}

impl SegmentItem {
    fn expr(&self) -> Option<NormExpr> {
        match self {
            SegmentItem::Expr { expr, .. } => Some(expr.clone()),
            SegmentItem::Product(_) => None,
        }
    }

    fn source_product(&self) -> Option<NormProduct> {
        match self {
            SegmentItem::Expr { source_product, .. } => source_product.clone(),
            SegmentItem::Product(product) => Some(product.clone()),
        }
    }
}

fn normalize_segment_items(segment: &SegmentAst) -> Vec<SegmentItem> {
    segment
        .elements
        .iter()
        .map(|element| match element {
            SegmentElementAst::OperatorExpr(expr) => normalize_operator_expr_item(expr),
            SegmentElementAst::Product(product) => {
                SegmentItem::Product(normalize_product_expr(product, true))
            }
        })
        .collect()
}

fn normalize_operator_expr_item(expr: &OperatorExprAst) -> SegmentItem {
    match &expr.kind {
        OperatorExprKind::Product(product) => {
            SegmentItem::Product(normalize_product_expr(product, true))
        }
        OperatorExprKind::Atom(atom) => match &atom.kind {
            AtomKind::Group(inner) => {
                let lowered = normalize_expr(inner);
                let source_product = NormProduct {
                    elements: vec![NormProductElem::Expr(lowered.clone())],
                    origin: NormOrigin::Generated {
                        rule: NormRule::ProductLift,
                        span: atom.span,
                    },
                };
                SegmentItem::Expr {
                    expr: lowered,
                    source_product: Some(source_product),
                }
            }
            _ => SegmentItem::Expr {
                expr: normalize_operator_expr(expr),
                source_product: None,
            },
        },
        _ => SegmentItem::Expr {
            expr: normalize_operator_expr(expr),
            source_product: None,
        },
    }
}

fn lower_item_chain(
    initial: Option<NormExpr>,
    items: &[SegmentItem],
    fallback_span: Span,
) -> NormExpr {
    let mut current = initial;
    let mut index = 0;

    while index < items.len() {
        let expr = items[index].expr();
        let product = items[index].source_product();
        let next_expr = items.get(index + 1).and_then(SegmentItem::expr);
        let should_use_product =
            product.is_some() && (expr.is_none() || (current.is_some() && next_expr.is_some()));

        if should_use_product {
            let product = product.expect("should_use_product requires a product");
            if let Some(target) = next_expr {
                let repaired = make_call(
                    product,
                    target,
                    NormOrigin::Derived {
                        rule: NormRule::SecondLegalityRepair,
                        span: fallback_span,
                        summary: "repaired product-before-target".to_string(),
                    },
                );
                current = Some(match current {
                    Some(previous) => make_call(
                        source_product_from_expr(previous, fallback_span),
                        repaired,
                        NormOrigin::Derived {
                            rule: NormRule::SecondLegalityRepair,
                            span: fallback_span,
                            summary: "repaired product target in expression chain".to_string(),
                        },
                    ),
                    None => repaired,
                });
                index += 2;
                continue;
            }

            current = Some(match current {
                Some(previous) => make_call(
                    source_product_from_expr(previous, fallback_span),
                    NormExpr::Unsupported {
                        raw_kind_summary: "dangling product cannot be a call target".to_string(),
                        origin: NormOrigin::Generated {
                            rule: NormRule::Unsupported,
                            span: fallback_span,
                        },
                    },
                    NormOrigin::Generated {
                        rule: NormRule::Unsupported,
                        span: fallback_span,
                    },
                ),
                None => NormExpr::Product(product),
            });
            index += 1;
            continue;
        }

        if let Some(expr) = items[index].expr() {
            current = Some(match current {
                Some(previous) => make_call(
                    source_product_from_expr(previous, fallback_span),
                    expr,
                    NormOrigin::Derived {
                        rule: NormRule::PipeFallback,
                        span: fallback_span,
                        summary: "ordinary expression-chain growth".to_string(),
                    },
                ),
                None => expr,
            });
            index += 1;
            continue;
        }

        let product = items[index]
            .source_product()
            .expect("non-expression item must have a source product");
        if index + 1 < items.len() {
            if let Some(target) = items[index + 1].expr() {
                let repaired = make_call(
                    product,
                    target,
                    NormOrigin::Derived {
                        rule: NormRule::SecondLegalityRepair,
                        span: fallback_span,
                        summary: "repaired product-before-target".to_string(),
                    },
                );
                current = Some(match current {
                    Some(previous) => make_call(
                        source_product_from_expr(previous, fallback_span),
                        repaired,
                        NormOrigin::Derived {
                            rule: NormRule::SecondLegalityRepair,
                            span: fallback_span,
                            summary: "repaired product target in expression chain".to_string(),
                        },
                    ),
                    None => repaired,
                });
                index += 2;
                continue;
            }
        }

        current = Some(match current {
            Some(previous) => make_call(
                source_product_from_expr(previous, fallback_span),
                NormExpr::Unsupported {
                    raw_kind_summary: "dangling product cannot be a call target".to_string(),
                    origin: NormOrigin::Generated {
                        rule: NormRule::Unsupported,
                        span: fallback_span,
                    },
                },
                NormOrigin::Generated {
                    rule: NormRule::Unsupported,
                    span: fallback_span,
                },
            ),
            None => NormExpr::Product(product),
        });
        index += 1;
    }

    current.unwrap_or_else(|| {
        NormExpr::Error(NormError {
            message: "empty expression segment".to_string(),
            origin: NormOrigin::Source(fallback_span),
        })
    })
}

fn normalize_operator_expr(expr: &OperatorExprAst) -> NormExpr {
    match &expr.kind {
        OperatorExprKind::Atom(atom) => normalize_atom(atom),
        OperatorExprKind::Product(product) => {
            NormExpr::Product(normalize_product_expr(product, true))
        }
        OperatorExprKind::OperatorSugar {
            operator,
            fixity,
            args,
            span,
        } => normalize_operator_sugar(operator, *fixity, args, *span),
        OperatorExprKind::NavPath {
            components,
            span,
            explicit_terminated,
        } => NormExpr::Nav {
            components: components.iter().map(normalize_nav_component).collect(),
            explicit_terminated: *explicit_terminated,
            origin: NormOrigin::Source(*span),
        },
        OperatorExprKind::MemberSugar {
            object,
            selector,
            span,
        } => normalize_member_sugar(normalize_operator_expr(object), selector, *span),
        OperatorExprKind::DoubleDotSugar {
            object,
            selector,
            args,
            span,
        } => normalize_double_dot_sugar(normalize_operator_expr(object), selector, args, *span),
        OperatorExprKind::BracketCallSugar {
            object,
            operator,
            args,
            span,
        } => normalize_bracket_call_sugar(normalize_operator_expr(object), operator, args, *span),
        OperatorExprKind::Error(error) => NormExpr::Error(normalize_error(error)),
    }
}

fn normalize_atom(atom: &AtomAst) -> NormExpr {
    match &atom.kind {
        AtomKind::Name(name) => NormExpr::Name {
            text: name.text.clone(),
            origin: NormOrigin::Source(name.span),
        },
        AtomKind::IntLiteral(text) => NormExpr::Literal {
            kind: NormLiteralKind::Int,
            text: text.clone(),
            origin: NormOrigin::Source(atom.span),
        },
        AtomKind::FloatLiteral(text) => NormExpr::Literal {
            kind: NormLiteralKind::Float,
            text: text.clone(),
            origin: NormOrigin::Source(atom.span),
        },
        AtomKind::StringLiteral(text) => NormExpr::Literal {
            kind: NormLiteralKind::String,
            text: text.clone(),
            origin: NormOrigin::Source(atom.span),
        },
        AtomKind::Group(expr) => normalize_expr(expr),
        AtomKind::NavPath {
            components,
            explicit_terminated,
        } => NormExpr::Nav {
            components: components.iter().map(normalize_nav_component).collect(),
            explicit_terminated: *explicit_terminated,
            origin: NormOrigin::Source(atom.span),
        },
        AtomKind::DotClosure { selector } => normalize_dot_closure(selector, atom.span),
        AtomKind::MemberSugar { object, selector } => {
            normalize_member_sugar(normalize_atom(object), selector, atom.span)
        }
        AtomKind::DoubleDotSugar {
            object,
            selector,
            args,
        } => normalize_double_dot_sugar(normalize_atom(object), selector, args, atom.span),
        AtomKind::BracketCallSugar {
            object,
            operator,
            args,
        } => normalize_bracket_call_sugar(normalize_atom(object), operator, args, atom.span),
        AtomKind::Closure(closure) => NormExpr::Closure(normalize_closure(closure)),
        AtomKind::Error(error) => NormExpr::Error(normalize_error(error)),
    }
}

fn normalize_operator_sugar(
    operator: &OperatorNameAst,
    fixity: OperatorFixity,
    args: &[OperatorExprAst],
    span: Span,
) -> NormExpr {
    match fixity {
        OperatorFixity::Prefix if operator.spelling == "-" && args.len() == 1 => {
            let operand = normalize_operator_expr(&args[0]);
            let closure = generated_prefix_negative_closure(span);
            make_call(
                source_product_from_expr(operand, span),
                NormExpr::Closure(closure),
                NormOrigin::Generated {
                    rule: NormRule::PrefixNegativeLowering,
                    span,
                },
            )
        }
        OperatorFixity::Postfix if args.len() == 1 => {
            let source = source_product_from_expr(normalize_operator_expr(&args[0]), span);
            make_call(
                source,
                operator_target(operator, NormOperatorFixity::Postfix, 1),
                NormOrigin::Generated {
                    rule: NormRule::OperatorLowering,
                    span,
                },
            )
        }
        OperatorFixity::Binary if args.len() == 2 => {
            let source = NormProduct {
                elements: vec![
                    NormProductElem::Expr(normalize_operator_expr(&args[0])),
                    NormProductElem::Expr(normalize_operator_expr(&args[1])),
                ],
                origin: NormOrigin::Generated {
                    rule: NormRule::OperatorLowering,
                    span,
                },
            };
            make_call(
                source,
                operator_target(operator, NormOperatorFixity::Binary, 2),
                NormOrigin::Generated {
                    rule: NormRule::OperatorLowering,
                    span,
                },
            )
        }
        _ => NormExpr::Unsupported {
            raw_kind_summary: format!(
                "operator sugar fixity={} arity={}",
                raw_fixity_label(fixity),
                args.len()
            ),
            origin: NormOrigin::Generated {
                rule: NormRule::Unsupported,
                span,
            },
        },
    }
}

fn normalize_member_sugar(object: NormExpr, selector: &SelectorAst, span: Span) -> NormExpr {
    make_call(
        source_product_from_expr(object, span),
        normalize_dot_closure(selector, span),
        NormOrigin::Generated {
            rule: NormRule::MemberLowering,
            span,
        },
    )
}

fn normalize_dot_closure(selector: &SelectorAst, span: Span) -> NormExpr {
    let rule = NormRule::DotClosureLowering;
    let selector_name = selector_name(selector);
    let body = make_call(
        NormProduct {
            elements: vec![
                NormProductElem::Expr(generated_name("val", span, rule)),
                NormProductElem::Expr(generated_name("args", span, rule)),
            ],
            origin: NormOrigin::Generated { rule, span },
        },
        generated_nav(&[selector_name.as_str(), "T"], span, rule),
        NormOrigin::Generated { rule, span },
    );
    NormExpr::Closure(generated_field_function_closure(span, body))
}

fn normalize_double_dot_sugar(
    object: NormExpr,
    selector: &SelectorAst,
    args: &ProductExprAst,
    span: Span,
) -> NormExpr {
    let selector_name = selector_name(selector);
    let mut elements = vec![NormProductElem::Expr(generated_name(
        "val",
        span,
        NormRule::DoubleDotLowering,
    ))];
    elements.extend(normalize_product_elements(args, false));
    let body = make_call(
        NormProduct {
            elements,
            origin: NormOrigin::Generated {
                rule: NormRule::DoubleDotLowering,
                span,
            },
        },
        generated_nav(
            &[selector_name.as_str(), "T"],
            span,
            NormRule::DoubleDotLowering,
        ),
        NormOrigin::Generated {
            rule: NormRule::DoubleDotLowering,
            span,
        },
    );
    let closure = generated_receiver_closure(NormRule::DoubleDotLowering, span, body);

    make_call(
        source_product_from_expr(object, span),
        NormExpr::Closure(closure),
        NormOrigin::Generated {
            rule: NormRule::DoubleDotLowering,
            span,
        },
    )
}

fn normalize_bracket_call_sugar(
    object: NormExpr,
    operator: &OperatorNameAst,
    args: &ProductExprAst,
    span: Span,
) -> NormExpr {
    let mut elements = vec![NormProductElem::Expr(object)];
    elements.extend(normalize_product_elements(args, false));
    let source = NormProduct {
        elements,
        origin: NormOrigin::Generated {
            rule: NormRule::BracketCallLowering,
            span,
        },
    };

    make_call(
        source,
        operator_target(
            operator,
            NormOperatorFixity::BracketCall,
            args.elements.len() + 1,
        ),
        NormOrigin::Generated {
            rule: NormRule::BracketCallLowering,
            span,
        },
    )
}

fn normalize_product_expr(product: &ProductExprAst, empty_is_unit: bool) -> NormProduct {
    NormProduct {
        elements: normalize_product_elements(product, empty_is_unit),
        origin: NormOrigin::Source(product.span),
    }
}

fn normalize_product_elements(
    product: &ProductExprAst,
    empty_is_unit: bool,
) -> Vec<NormProductElem> {
    if product.elements.is_empty() && empty_is_unit {
        return vec![NormProductElem::Unit {
            origin: NormOrigin::Source(product.span),
        }];
    }

    product
        .elements
        .iter()
        .map(|element| match element {
            ProductElementAst::Expr(expr) => NormProductElem::Expr(normalize_expr(expr)),
            ProductElementAst::Unit { span } => NormProductElem::Unit {
                origin: NormOrigin::Source(*span),
            },
        })
        .collect()
}

fn source_product_from_expr(expr: NormExpr, span: Span) -> NormProduct {
    let expr_span = expr_span(&expr).unwrap_or(span);
    match expr {
        NormExpr::Product(product) => product,
        expr => NormProduct {
            elements: vec![NormProductElem::Expr(expr)],
            origin: NormOrigin::Generated {
                rule: NormRule::ProductLift,
                span: expr_span,
            },
        },
    }
}

fn merge_products(left: NormProduct, right: NormProduct, span: Span) -> NormProduct {
    let mut elements = left.elements;
    elements.extend(right.elements);
    NormProduct {
        elements,
        origin: NormOrigin::Derived {
            rule: NormRule::ProductMerge,
            span,
            summary: "merged source product with continuation product".to_string(),
        },
    }
}

fn make_call(source: NormProduct, target: NormExpr, origin: NormOrigin) -> NormExpr {
    NormExpr::Call {
        source,
        target: Box::new(target),
        origin,
    }
}

fn operator_target(
    operator: &OperatorNameAst,
    fixity: NormOperatorFixity,
    arity: usize,
) -> NormExpr {
    NormExpr::OperatorTarget {
        spelling: operator.spelling.clone(),
        fixity,
        arity,
        origin: NormOrigin::Source(operator.span),
    }
}

fn normalize_closure(closure: &ClosureAst) -> NormClosure {
    let body = match &closure.body {
        ClosureBodyAst::Block(block) => NormClosureBody::Block(normalize_body_block(block)),
        ClosureBodyAst::NamedBlock {
            strategy,
            block,
            span,
        } => NormClosureBody::NamedBlock {
            strategy: strategy.text.clone(),
            body: normalize_body_block(block),
            origin: NormOrigin::Source(*span),
        },
        ClosureBodyAst::Defaulted { span, .. } => NormClosureBody::Defaulted {
            origin: NormOrigin::Source(*span),
        },
        ClosureBodyAst::Delete(del) => NormClosureBody::Delete(NormDeleteBody {
            message: del.message.clone(),
            origin: NormOrigin::Source(del.span),
        }),
    };
    NormClosure {
        placement: match closure.placement {
            ClosurePlacementAst::InPlace => NormClosurePlacement::InPlace,
            ClosurePlacementAst::Ordinary => NormClosurePlacement::Ordinary,
        },
        head: closure.head.as_ref().map(normalize_closure_head),
        body,
        origin: NormOrigin::Source(closure.span),
    }
}

fn normalize_body_block(body: &BodyBlockAst) -> NormProgram {
    let len = body.forms.len();
    let forms: Vec<NormForm> = body
        .forms
        .iter()
        .enumerate()
        .map(|(i, form)| {
            if i == len - 1 {
                match form {
                    FormAst::Expr(expr) => NormForm::TailValue(normalize_expr(expr)),
                    _ => normalize_form(form),
                }
            } else {
                normalize_form(form)
            }
        })
        .collect();
    NormProgram {
        forms,
        origin: NormOrigin::Source(body.span),
    }
}

fn normalize_closure_head(head: &FnHeadPrefixAst) -> NormClosureHead {
    let (deduce, holes) = normalize_deduce_list(head.deduce.as_ref(), &[]);
    let params = head
        .params
        .as_ref()
        .map(|params| normalize_param_clause(params, &holes))
        .unwrap_or_default();
    let captures = head
        .captures
        .as_ref()
        .map(|captures| {
            captures
                .items
                .iter()
                .map(|item| normalize_capture_item(item, &holes))
                .collect()
        })
        .unwrap_or_default();
    let call_policy = head.call_policy.as_ref().map(normalize_policy_spec);
    let returns = head
        .returns
        .as_ref()
        .map(|returns| normalize_return_clause(returns, &holes));
    let clauses = head.clauses.iter().map(normalize_head_clause).collect();

    NormClosureHead {
        deduce,
        captures,
        params,
        call_policy,
        returns,
        clauses,
        origin: NormOrigin::Generated {
            rule: NormRule::ClosureNormalize,
            span: head.span,
        },
    }
}

fn normalize_capture_item(item: &CaptureItemAst, holes: &[VisibleHole]) -> NormCapture {
    match item {
        CaptureItemAst::Explicit {
            slot,
            initializer,
            span,
        } => {
            let mut slot = normalize_binding_slot(slot, holes);
            slot.has_let = true;
            slot.initializer = None;
            NormCapture {
                slot,
                initializer: normalize_expr(initializer),
                origin: NormOrigin::Source(*span),
            }
        }
        CaptureItemAst::Inferred { initializer, span } => {
            let initializer = normalize_expr(initializer);
            let names = infer_capture_binding_names(&initializer);
            let value_pattern = if names.len() == 1 {
                NormPattern::Binder {
                    name: names.iter().next().expect("one inferred name").clone(),
                    origin: NormOrigin::Derived {
                        rule: NormRule::CaptureNameInference,
                        span: *span,
                        summary: "unique free non-call bare name".to_string(),
                    },
                }
            } else {
                let detail = if names.is_empty() {
                    "found no candidate".to_string()
                } else {
                    format!("found {}", names.into_iter().collect::<Vec<_>>().join(", "))
                };
                NormPattern::Error(NormError {
                    message: format!(
                        "capture shorthand requires exactly one free non-call bare name; {detail}"
                    ),
                    origin: NormOrigin::Derived {
                        rule: NormRule::CaptureNameInference,
                        span: *span,
                        summary: "ambiguous capture shorthand".to_string(),
                    },
                })
            };
            let origin = NormOrigin::Derived {
                rule: NormRule::CaptureNameInference,
                span: *span,
                summary: "inferred capture elaborates to a let-shaped binding".to_string(),
            };
            NormCapture {
                slot: NormBindingSlot {
                    policy: None,
                    has_let: true,
                    deduce: Vec::new(),
                    value_pattern,
                    annotation: None,
                    with_clause: None,
                    initializer: None,
                    origin: origin.clone(),
                },
                initializer,
                origin,
            }
        }
    }
}

fn infer_capture_binding_names(initializer: &NormExpr) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_free_non_call_names_expr(initializer, &BTreeSet::new(), false, &mut names);
    names
}

fn collect_free_non_call_names_expr(
    expr: &NormExpr,
    bound: &BTreeSet<String>,
    direct_call_target: bool,
    names: &mut BTreeSet<String>,
) {
    match expr {
        NormExpr::Name { text, .. } => {
            if !direct_call_target && !bound.contains(text) {
                names.insert(text.clone());
            }
        }
        NormExpr::Call { source, target, .. } => {
            for element in &source.elements {
                if let NormProductElem::Expr(expr) = element {
                    collect_free_non_call_names_expr(expr, bound, false, names);
                }
            }
            collect_free_non_call_names_expr(target, bound, true, names);
        }
        NormExpr::Product(product) => {
            for element in &product.elements {
                if let NormProductElem::Expr(expr) = element {
                    collect_free_non_call_names_expr(expr, bound, false, names);
                }
            }
        }
        NormExpr::Nav { components, .. } => {
            for component in components {
                if let NormNavComponent::Group { expr, .. } = component {
                    collect_free_non_call_names_expr(expr, bound, false, names);
                }
            }
        }
        NormExpr::Closure(closure) => {
            collect_free_non_call_names_closure(closure, bound, names);
        }
        NormExpr::Literal { .. }
        | NormExpr::OperatorTarget { .. }
        | NormExpr::Error(_)
        | NormExpr::Unsupported { .. } => {}
    }
}

fn collect_free_non_call_names_closure(
    closure: &NormClosure,
    outer_bound: &BTreeSet<String>,
    names: &mut BTreeSet<String>,
) {
    let mut body_bound = outer_bound.clone();
    if let Some(head) = &closure.head {
        // Capture initializers are simultaneous and therefore all see only the
        // enclosing environment.
        for capture in &head.captures {
            collect_free_non_call_names_expr(&capture.initializer, outer_bound, false, names);
        }
        for capture in &head.captures {
            collect_pattern_binder_names(&capture.slot.value_pattern, &mut body_bound);
        }
        for param in &head.params {
            collect_pattern_element_binder_names(param, &mut body_bound);
        }
        if let Some(returns) = &head.returns {
            collect_pattern_binder_names(&returns.value_pattern, &mut body_bound);
        }
        for clause in &head.clauses {
            let expr = match clause {
                NormHeadClause::Require { expr, .. }
                | NormHeadClause::Pre { expr, .. }
                | NormHeadClause::Post { expr, .. }
                | NormHeadClause::LifetimePre { expr, .. }
                | NormHeadClause::LifetimePost { expr, .. } => Some(expr),
                NormHeadClause::Error(_) => None,
            };
            if let Some(expr) = expr {
                collect_free_non_call_names_expr(expr, &body_bound, false, names);
            }
        }
    }

    if let Some(body) = closure.body.user_body() {
        collect_free_non_call_names_program(body, &mut body_bound, names);
    }
}

fn collect_free_non_call_names_program(
    program: &NormProgram,
    bound: &mut BTreeSet<String>,
    names: &mut BTreeSet<String>,
) {
    for form in &program.forms {
        match form {
            NormForm::Let(NormDecl::Let { slot, .. }) => {
                if let Some(initializer) = &slot.initializer {
                    collect_free_non_call_names_expr(initializer, bound, false, names);
                }
                collect_pattern_binder_names(&slot.value_pattern, bound);
            }
            NormForm::Alias(NormDecl::Alias { binder, target, .. }) => {
                for component in &target.components {
                    if let NormNavComponent::Group { expr, .. } = component {
                        collect_free_non_call_names_expr(expr, bound, false, names);
                    }
                }
                if let NormAliasBinder::Name { name, .. } = binder {
                    bound.insert(name.clone());
                }
            }
            NormForm::Expr(expr) | NormForm::TailValue(expr) => {
                collect_free_non_call_names_expr(expr, bound, false, names);
            }
            NormForm::ReturnEvent(event) => {
                collect_free_non_call_names_expr(&event.value, bound, false, names);
                if let NormReturnTargetSyntax::Explicit(target) = &event.target {
                    collect_free_non_call_names_expr(target, bound, false, names);
                }
            }
            NormForm::Let(NormDecl::Alias { .. } | NormDecl::Error(_))
            | NormForm::Alias(NormDecl::Let { .. } | NormDecl::Error(_))
            | NormForm::Error(_) => {}
        }
    }
}

fn collect_pattern_element_binder_names(element: &NormPatternElem, bound: &mut BTreeSet<String>) {
    match element {
        NormPatternElem::Pattern(pattern) => collect_pattern_binder_names(pattern, bound),
        NormPatternElem::BindingSlot(slot) => {
            collect_pattern_binder_names(&slot.value_pattern, bound)
        }
        NormPatternElem::Unit { .. } => {}
    }
}

fn collect_pattern_binder_names(pattern: &NormPattern, bound: &mut BTreeSet<String>) {
    match pattern {
        NormPattern::Binder { name, .. } => {
            bound.insert(name.clone());
        }
        NormPattern::Product { elements, .. } => {
            for element in elements {
                collect_pattern_element_binder_names(element, bound);
            }
        }
        NormPattern::Pack { inner, .. } => collect_pattern_binder_names(inner, bound),
        NormPattern::BindingSlot { slot, .. } => {
            collect_pattern_binder_names(&slot.value_pattern, bound)
        }
        NormPattern::Sequence { elements, .. } => {
            for element in elements {
                collect_pattern_binder_names(element, bound);
            }
        }
        NormPattern::OperatorBinder { .. }
        | NormPattern::Unit { .. }
        | NormPattern::HoleRef { .. }
        | NormPattern::AnonymousHole { .. }
        | NormPattern::Name { .. }
        | NormPattern::Literal { .. }
        | NormPattern::Nav { .. }
        | NormPattern::Skeleton { .. }
        | NormPattern::Error(_)
        | NormPattern::Unsupported { .. } => {}
    }
}

fn normalize_param_clause(params: &ParamClauseAst, holes: &[VisibleHole]) -> Vec<NormPatternElem> {
    params
        .extract
        .elements
        .iter()
        .map(|element| match element {
            ProductExtractElementAst::Slot(slot) => {
                NormPatternElem::BindingSlot(normalize_binding_slot(slot, holes))
            }
            ProductExtractElementAst::Unit { span } => NormPatternElem::Unit {
                origin: NormOrigin::Source(*span),
            },
        })
        .collect()
}

fn normalize_return_clause(returns: &ReturnClauseAst, holes: &[VisibleHole]) -> NormBindingSlot {
    normalize_binding_slot(&returns.slot, holes)
}

fn normalize_head_clause(clause: &HeadClauseAst) -> NormHeadClause {
    match clause {
        HeadClauseAst::Require { expr, span } => NormHeadClause::Require {
            expr: normalize_expr(expr),
            origin: NormOrigin::Source(*span),
        },
        HeadClauseAst::Pre { expr, span } => NormHeadClause::Pre {
            expr: normalize_expr(expr),
            origin: NormOrigin::Source(*span),
        },
        HeadClauseAst::Post { expr, span } => NormHeadClause::Post {
            expr: normalize_expr(expr),
            origin: NormOrigin::Source(*span),
        },
        HeadClauseAst::LifetimePre { expr, span } => NormHeadClause::LifetimePre {
            expr: normalize_expr(expr),
            origin: NormOrigin::Source(*span),
        },
        HeadClauseAst::LifetimePost { expr, span } => NormHeadClause::LifetimePost {
            expr: normalize_expr(expr),
            origin: NormOrigin::Source(*span),
        },
        HeadClauseAst::Error(error) => NormHeadClause::Error(normalize_error(error)),
    }
}

fn normalize_binding_slot(
    slot: &BindingSlotAst,
    inherited_holes: &[VisibleHole],
) -> NormBindingSlot {
    let (deduce, holes) = normalize_deduce_list(slot.deduce.as_ref(), inherited_holes);

    NormBindingSlot {
        policy: slot.policy.as_ref().map(normalize_policy_spec),
        has_let: slot.has_let,
        deduce,
        value_pattern: normalize_binding_pattern(&slot.pattern, &holes),
        annotation: slot
            .annotation
            .as_ref()
            .map(|annotation| normalize_binding_annotation(annotation, &holes)),
        with_clause: slot.with_clause.as_ref().map(normalize_with_clause),
        initializer: slot
            .initializer
            .as_ref()
            .map(|initializer| Box::new(normalize_expr(initializer))),
        origin: NormOrigin::Generated {
            rule: NormRule::PatternNormalize,
            span: slot.span,
        },
    }
}

fn normalize_deduce_list(
    deduce: Option<&DeduceListAst>,
    inherited_holes: &[VisibleHole],
) -> (Vec<NormHoleDecl>, Vec<VisibleHole>) {
    let mut visible = inherited_holes.to_vec();
    let mut normalized = Vec::new();

    if let Some(deduce) = deduce {
        for binder in &deduce.binders {
            let id = HoleBinderId::provisional_source();
            let duplicate_of =
                find_visible_source_hole(&visible, &binder.name.text).map(|hole| hole.id);
            let annotation = binder
                .annotation
                .as_ref()
                .map(|annotation| normalize_annotation_term(annotation, &visible));
            normalized.push(NormHoleDecl {
                id,
                name: binder.name.text.clone(),
                annotation,
                duplicate_of,
                origin: NormOrigin::Generated {
                    rule: NormRule::PatternNormalize,
                    span: binder.span,
                },
            });
            if duplicate_of.is_none() {
                visible.push(VisibleHole {
                    id,
                    key: VisibleHoleKey::SourceName(binder.name.text.clone()),
                });
            }
        }
    }

    (normalized, visible)
}

fn find_visible_source_hole<'a>(holes: &'a [VisibleHole], name: &str) -> Option<&'a VisibleHole> {
    holes.iter().rev().find(|hole| {
        matches!(
            &hole.key,
            VisibleHoleKey::SourceName(visible_name) if visible_name == name
        )
    })
}

fn find_visible_generated_hole(
    holes: &[VisibleHole],
    key: GeneratedHoleKey,
) -> Option<&VisibleHole> {
    holes.iter().rev().find(|hole| {
        matches!(
            &hole.key,
            VisibleHoleKey::Generated(visible_key) if *visible_key == key
        )
    })
}

fn find_visible_hole_ref<'a>(
    holes: &'a [VisibleHole],
    provisional_target: HoleBinderId,
    display_name: &str,
) -> Option<&'a VisibleHole> {
    provisional_target.generated_key().map_or_else(
        || find_visible_source_hole(holes, display_name),
        |key| find_visible_generated_hole(holes, key),
    )
}

fn normalize_binding_pattern(pattern: &BindingPatternAst, holes: &[VisibleHole]) -> NormPattern {
    // Extraction-side entry point. Binder/name/skeleton material stays in the
    // NormPattern family and is not treated as value-side call target material.
    match pattern {
        BindingPatternAst::Binder(BinderNameAst::Text(name)) => NormPattern::Binder {
            name: name.text.clone(),
            origin: NormOrigin::Source(name.span),
        },
        BindingPatternAst::Binder(BinderNameAst::Operator(operator)) => {
            NormPattern::OperatorBinder {
                spelling: operator.spelling.clone(),
                origin: NormOrigin::Source(operator.span),
            }
        }
        BindingPatternAst::Product(product) => normalize_product_extract_pattern(product, holes),
        BindingPatternAst::Pack { inner, span } => NormPattern::Pack {
            inner: Box::new(normalize_pack_operand_pattern(inner, holes)),
            origin: NormOrigin::Source(*span),
        },
        BindingPatternAst::Skeleton(skeleton) => normalize_canonical_pattern(skeleton, holes),
        BindingPatternAst::Error(error) => NormPattern::Error(normalize_error(error)),
    }
}

fn normalize_pack_operand_pattern(
    pattern: &BindingPatternAst,
    holes: &[VisibleHole],
) -> NormPattern {
    match pattern {
        BindingPatternAst::Skeleton(skeleton) => normalize_canonical_pack_operand(skeleton, holes),
        other => normalize_binding_pattern(other, holes),
    }
}

fn normalize_canonical_pattern(
    skeleton: &CanonicalSkeletonAst,
    holes: &[VisibleHole],
) -> NormPattern {
    match skeleton {
        CanonicalSkeletonAst::Segment { elements, span } => NormPattern::Sequence {
            elements: elements
                .iter()
                .map(|element| normalize_canonical_pattern(element, holes))
                .collect(),
            origin: NormOrigin::Generated {
                rule: NormRule::PatternNormalize,
                span: *span,
            },
        },
        CanonicalSkeletonAst::Pack { inner, span } => NormPattern::Pack {
            inner: Box::new(normalize_canonical_pack_operand(inner, holes)),
            origin: NormOrigin::Source(*span),
        },
        CanonicalSkeletonAst::ProductExtract { elements, span } => NormPattern::Product {
            elements: elements
                .iter()
                .map(|element| match element {
                    CanonicalProductElementAst::Skeleton(skeleton) => {
                        NormPatternElem::Pattern(normalize_canonical_pattern(skeleton, holes))
                    }
                    CanonicalProductElementAst::Unit { span } => NormPatternElem::Unit {
                        origin: NormOrigin::Source(*span),
                    },
                })
                .collect(),
            origin: NormOrigin::Generated {
                rule: NormRule::PatternNormalize,
                span: *span,
            },
        },
        CanonicalSkeletonAst::Name { name, .. }
            if find_visible_source_hole(holes, &name.text).is_some() =>
        {
            let target = find_visible_source_hole(holes, &name.text)
                .expect("guard proved visible hole")
                .id;
            NormPattern::HoleRef {
                target,
                name: name.text.clone(),
                origin: NormOrigin::Source(name.span),
            }
        }
        CanonicalSkeletonAst::Error(error) => NormPattern::Error(normalize_error(error)),
        atom => NormPattern::Skeleton {
            skeleton: normalize_canonical_skeleton(atom),
            origin: NormOrigin::Generated {
                rule: NormRule::PatternNormalize,
                span: skeleton_span(atom),
            },
        },
    }
}

fn normalize_canonical_pack_operand(
    skeleton: &CanonicalSkeletonAst,
    holes: &[VisibleHole],
) -> NormPattern {
    match skeleton {
        CanonicalSkeletonAst::Name { name, .. }
            if find_visible_source_hole(holes, &name.text).is_none() =>
        {
            NormPattern::Binder {
                name: name.text.clone(),
                origin: NormOrigin::Source(name.span),
            }
        }
        CanonicalSkeletonAst::ProductExtract { .. } => {
            let normalized = normalize_canonical_pattern(skeleton, holes);
            match normalized {
                NormPattern::Product {
                    mut elements,
                    origin,
                } if elements.len() == 1 => match elements.pop().expect("one element") {
                    NormPatternElem::Pattern(pattern) => pattern,
                    element => NormPattern::Product {
                        elements: vec![element],
                        origin,
                    },
                },
                other => other,
            }
        }
        _ => normalize_canonical_pattern(skeleton, holes),
    }
}

fn normalize_product_extract_pattern(
    product: &ProductExtractAst,
    holes: &[VisibleHole],
) -> NormPattern {
    let elements = product
        .elements
        .iter()
        .map(|element| match element {
            ProductExtractElementAst::Slot(slot) => {
                NormPatternElem::BindingSlot(normalize_binding_slot(slot, holes))
            }
            ProductExtractElementAst::Unit { span } => NormPatternElem::Unit {
                origin: NormOrigin::Source(*span),
            },
        })
        .collect();
    NormPattern::Product {
        elements,
        origin: NormOrigin::Generated {
            rule: NormRule::PatternNormalize,
            span: product.span,
        },
    }
}

fn normalize_binding_annotation(
    annotation: &BindingAnnotationAst,
    holes: &[VisibleHole],
) -> NormAnnotation {
    // Annotation syntax is classifier/pattern material. It deliberately lowers
    // to NormAnnotation { pattern: NormPattern } rather than NormExpr.
    match annotation {
        BindingAnnotationAst::Expr(expr) => normalize_annotation_expr(expr, holes),
        BindingAnnotationAst::Compound { left, right, span } => {
            let left_pattern = normalize_annotation_term(left, holes).pattern;
            let right_pattern = normalize_annotation_expr(right, holes).pattern;
            NormAnnotation {
                pattern: NormPattern::Sequence {
                    elements: vec![left_pattern, right_pattern],
                    origin: NormOrigin::Generated {
                        rule: NormRule::PatternNormalize,
                        span: *span,
                    },
                },
                origin: NormOrigin::Generated {
                    rule: NormRule::PatternNormalize,
                    span: *span,
                },
            }
        }
        BindingAnnotationAst::Error(error) => NormAnnotation {
            pattern: NormPattern::Error(normalize_error(error)),
            origin: NormOrigin::Source(error.span),
        },
    }
}

fn normalize_annotation_term(term: &AnnotationTermAst, holes: &[VisibleHole]) -> NormAnnotation {
    match term {
        AnnotationTermAst::Expr(expr) => normalize_annotation_expr(expr, holes),
        AnnotationTermAst::Hole { span } => NormAnnotation {
            pattern: NormPattern::AnonymousHole {
                origin: NormOrigin::Source(*span),
            },
            origin: NormOrigin::Generated {
                rule: NormRule::PatternNormalize,
                span: *span,
            },
        },
    }
}

fn normalize_annotation_expr(expr: &ExprAst, holes: &[VisibleHole]) -> NormAnnotation {
    // Bridge from raw expression-shaped parser surface into pattern context.
    // This is not value-to-pattern conversion for runtime values.
    NormAnnotation {
        pattern: normalize_expr_as_pattern(expr, holes),
        origin: NormOrigin::Generated {
            rule: NormRule::PatternNormalize,
            span: expr.span,
        },
    }
}

fn normalize_expr_as_pattern(expr: &ExprAst, holes: &[VisibleHole]) -> NormPattern {
    // Pattern-side lowering for raw expression-shaped syntax in annotation or
    // extraction contexts. Names become PatternName/HoleRef, not NormExpr::Name.
    match &expr.kind {
        ExprKind::Pipe(pipe) => normalize_pipe_as_pattern(pipe, holes),
        ExprKind::Product(product) => {
            let elements = product
                .elements
                .iter()
                .map(|element| match element {
                    ProductElementAst::Expr(expr) => {
                        NormPatternElem::Pattern(normalize_expr_as_pattern(expr, holes))
                    }
                    ProductElementAst::Unit { span } => NormPatternElem::Unit {
                        origin: NormOrigin::Source(*span),
                    },
                })
                .collect();
            NormPattern::Product {
                elements,
                origin: NormOrigin::Generated {
                    rule: NormRule::PatternNormalize,
                    span: product.span,
                },
            }
        }
        ExprKind::Error(error) => NormPattern::Error(normalize_error(error)),
    }
}

fn normalize_pipe_as_pattern(pipe: &PipeExprAst, holes: &[VisibleHole]) -> NormPattern {
    // Pipe-shaped raw syntax in pattern context is preserved as pattern
    // sequence material. It does not participate in value-side call lowering.
    let mut elements = Vec::new();
    for segment in &pipe.segments {
        for element in &segment.elements {
            match element {
                SegmentElementAst::OperatorExpr(expr) => {
                    elements.push(normalize_operator_expr_as_pattern(expr, holes));
                }
                SegmentElementAst::Product(product) => {
                    let product_pattern = normalize_expr_as_pattern(
                        &ExprAst {
                            kind: ExprKind::Product(product.clone()),
                            span: product.span,
                        },
                        holes,
                    );
                    elements.push(product_pattern);
                }
            }
        }
    }

    if elements.len() == 1 {
        elements.remove(0)
    } else {
        NormPattern::Sequence {
            elements,
            origin: NormOrigin::Generated {
                rule: NormRule::PatternNormalize,
                span: pipe.span,
            },
        }
    }
}

fn normalize_operator_expr_as_pattern(
    expr: &OperatorExprAst,
    holes: &[VisibleHole],
) -> NormPattern {
    // Operator-expression raw syntax in pattern context stays in NormPattern.
    // Unsupported sugar is surfaced explicitly instead of silently becoming a
    // value-side expression.
    match &expr.kind {
        OperatorExprKind::Atom(atom) => normalize_atom_as_pattern(atom, holes),
        OperatorExprKind::Product(product) => normalize_expr_as_pattern(
            &ExprAst {
                kind: ExprKind::Product(product.clone()),
                span: product.span,
            },
            holes,
        ),
        OperatorExprKind::NavPath {
            components,
            span,
            explicit_terminated,
        } => NormPattern::Nav {
            components: components.iter().map(normalize_nav_component).collect(),
            explicit_terminated: *explicit_terminated,
            origin: NormOrigin::Source(*span),
        },
        OperatorExprKind::Error(error) => NormPattern::Error(normalize_error(error)),
        other => NormPattern::Unsupported {
            raw_kind_summary: annotation_operator_pattern_summary(other),
            origin: NormOrigin::Generated {
                rule: NormRule::Unsupported,
                span: expr.span,
            },
        },
    }
}

fn normalize_atom_as_pattern(atom: &AtomAst, holes: &[VisibleHole]) -> NormPattern {
    // Atom raw syntax in pattern context remains bounded extraction material:
    // PatternName/PatternNav/HoleRef labels are intentionally distinct from
    // value-side Name/Nav dumps.
    match &atom.kind {
        AtomKind::Name(name) if find_visible_source_hole(holes, &name.text).is_some() => {
            let target = find_visible_source_hole(holes, &name.text)
                .expect("guard proved visible hole")
                .id;
            NormPattern::HoleRef {
                target,
                name: name.text.clone(),
                origin: NormOrigin::Source(name.span),
            }
        }
        AtomKind::Name(name) => NormPattern::Name {
            name: name.text.clone(),
            origin: NormOrigin::Source(name.span),
        },
        AtomKind::IntLiteral(text)
        | AtomKind::FloatLiteral(text)
        | AtomKind::StringLiteral(text) => NormPattern::Literal {
            text: text.clone(),
            origin: NormOrigin::Source(atom.span),
        },
        AtomKind::Group(expr) => normalize_expr_as_pattern(expr, holes),
        AtomKind::NavPath {
            components,
            explicit_terminated,
        } => NormPattern::Nav {
            components: components.iter().map(normalize_nav_component).collect(),
            explicit_terminated: *explicit_terminated,
            origin: NormOrigin::Source(atom.span),
        },
        AtomKind::Error(error) => NormPattern::Error(normalize_error(error)),
        other => NormPattern::Unsupported {
            raw_kind_summary: annotation_atom_pattern_summary(other),
            origin: NormOrigin::Generated {
                rule: NormRule::Unsupported,
                span: atom.span,
            },
        },
    }
}

fn normalize_with_clause(with_clause: &WithClauseAst) -> NormWithClause {
    match &with_clause.kind {
        WithClauseKind::Empty => NormWithClause {
            names: Vec::new(),
            explicit_empty: true,
            error: None,
            origin: NormOrigin::Source(with_clause.span),
        },
        WithClauseKind::Items { items } => NormWithClause {
            names: items.iter().map(|item| item.text.clone()).collect(),
            explicit_empty: false,
            error: None,
            origin: NormOrigin::Source(with_clause.span),
        },
        WithClauseKind::Error(error) => NormWithClause {
            names: Vec::new(),
            explicit_empty: false,
            error: Some(normalize_error(error)),
            origin: NormOrigin::Source(with_clause.span),
        },
    }
}

fn normalize_canonical_skeleton(skeleton: &CanonicalSkeletonAst) -> NormSkeleton {
    match skeleton {
        CanonicalSkeletonAst::Segment { elements, span } => NormSkeleton::Segment {
            elements: elements.iter().map(normalize_canonical_skeleton).collect(),
            origin: NormOrigin::Source(*span),
        },
        CanonicalSkeletonAst::Pack { span, .. } => NormSkeleton::Error(NormError {
            message: "canonical Pack must normalize through NormPattern".to_string(),
            origin: NormOrigin::Generated {
                rule: NormRule::Unsupported,
                span: *span,
            },
        }),
        CanonicalSkeletonAst::ProductExtract { elements, span } => NormSkeleton::Product {
            elements: elements
                .iter()
                .map(|element| match element {
                    CanonicalProductElementAst::Skeleton(skeleton) => {
                        NormSkeletonElem::Skeleton(normalize_canonical_skeleton(skeleton))
                    }
                    CanonicalProductElementAst::Unit { span } => NormSkeletonElem::Unit {
                        origin: NormOrigin::Source(*span),
                    },
                })
                .collect(),
            origin: NormOrigin::Source(*span),
        },
        CanonicalSkeletonAst::Wildcard { span } => NormSkeleton::Wildcard {
            origin: NormOrigin::Source(*span),
        },
        CanonicalSkeletonAst::Name { name, role, span } => NormSkeleton::Name {
            name: name.text.clone(),
            role: normalize_canonical_role(*role),
            origin: NormOrigin::Source(*span),
        },
        CanonicalSkeletonAst::NavPath { names, span } => NormSkeleton::Nav {
            components: names
                .iter()
                .map(|name| NormNavComponent::Name {
                    name: name.text.clone(),
                    origin: NormOrigin::Source(name.span),
                })
                .collect(),
            explicit_terminated: false,
            origin: NormOrigin::Source(*span),
        },
        CanonicalSkeletonAst::Literal { text, span } => NormSkeleton::Literal {
            text: text.clone(),
            origin: NormOrigin::Source(*span),
        },
        CanonicalSkeletonAst::Error(error) => NormSkeleton::Error(normalize_error(error)),
    }
}

fn normalize_canonical_role(role: CanonicalNameRole) -> NormCanonicalNameRole {
    match role {
        CanonicalNameRole::Hole => NormCanonicalNameRole::Hole,
        CanonicalNameRole::NodeName => NormCanonicalNameRole::NodeName,
        CanonicalNameRole::Unknown => NormCanonicalNameRole::Unknown,
    }
}

fn normalize_entity_ref(entity_ref: &EntityRefAst) -> NormEntityRef {
    NormEntityRef {
        components: entity_ref
            .components
            .iter()
            .map(normalize_nav_component)
            .collect(),
        origin: NormOrigin::Generated {
            rule: NormRule::AliasPreserve,
            span: entity_ref.span,
        },
    }
}

fn normalize_nav_component(component: &NavComponentAst) -> NormNavComponent {
    match component {
        NavComponentAst::Text(name) => NormNavComponent::Name {
            name: name.text.clone(),
            origin: NormOrigin::Source(name.span),
        },
        NavComponentAst::Operator(operator) => NormNavComponent::Operator {
            spelling: operator.spelling.clone(),
            origin: NormOrigin::Source(operator.span),
        },
        NavComponentAst::Group(expr) => NormNavComponent::Group {
            expr: Box::new(normalize_expr(expr)),
            origin: NormOrigin::Source(expr.span),
        },
        NavComponentAst::Error(error) => NormNavComponent::Error(normalize_error(error)),
    }
}

fn generated_prefix_negative_closure(span: Span) -> NormClosure {
    let body_expr = make_call(
        NormProduct {
            elements: vec![
                NormProductElem::Expr(generated_nav(
                    &["zero", "T"],
                    span,
                    NormRule::PrefixNegativeLowering,
                )),
                NormProductElem::Expr(generated_name(
                    "val",
                    span,
                    NormRule::PrefixNegativeLowering,
                )),
            ],
            origin: NormOrigin::Generated {
                rule: NormRule::PrefixNegativeLowering,
                span,
            },
        },
        NormExpr::OperatorTarget {
            spelling: "-".to_string(),
            fixity: NormOperatorFixity::Binary,
            arity: 2,
            origin: NormOrigin::Generated {
                rule: NormRule::PrefixNegativeLowering,
                span,
            },
        },
        NormOrigin::Generated {
            rule: NormRule::PrefixNegativeLowering,
            span,
        },
    );
    generated_receiver_closure(NormRule::PrefixNegativeLowering, span, body_expr)
}

fn generated_receiver_closure(rule: NormRule, span: Span, body_expr: NormExpr) -> NormClosure {
    let type_hole_id = HoleBinderId::provisional_generated(GeneratedHoleKey {
        rule,
        local_ordinal: 0,
    });
    NormClosure {
        placement: NormClosurePlacement::InPlace,
        head: Some(NormClosureHead {
            deduce: vec![NormHoleDecl {
                id: type_hole_id,
                name: "T".to_string(),
                annotation: Some(NormAnnotation {
                    pattern: NormPattern::Name {
                        name: "type".to_string(),
                        origin: NormOrigin::Generated { rule, span },
                    },
                    origin: NormOrigin::Generated { rule, span },
                }),
                duplicate_of: None,
                origin: NormOrigin::Generated { rule, span },
            }],
            captures: Vec::new(),
            params: vec![NormPatternElem::BindingSlot(NormBindingSlot {
                policy: None,
                has_let: false,
                deduce: Vec::new(),
                value_pattern: NormPattern::Binder {
                    name: "val".to_string(),
                    origin: NormOrigin::Generated { rule, span },
                },
                annotation: Some(NormAnnotation {
                    pattern: NormPattern::HoleRef {
                        target: type_hole_id,
                        name: "T".to_string(),
                        origin: NormOrigin::Generated { rule, span },
                    },
                    origin: NormOrigin::Generated { rule, span },
                }),
                with_clause: None,
                initializer: None,
                origin: NormOrigin::Generated { rule, span },
            })],
            call_policy: None,
            returns: None,
            clauses: Vec::new(),
            origin: NormOrigin::Generated { rule, span },
        }),
        body: NormClosureBody::Block(NormProgram {
            forms: vec![NormForm::TailValue(body_expr)],
            origin: NormOrigin::Generated { rule, span },
        }),
        origin: NormOrigin::Generated { rule, span },
    }
}

fn generated_field_function_closure(span: Span, body_expr: NormExpr) -> NormClosure {
    let rule = NormRule::DotClosureLowering;
    let mut closure = generated_receiver_closure(rule, span, body_expr);
    let head = closure
        .head
        .as_mut()
        .expect("generated receiver closure always has a head");
    head.params
        .push(NormPatternElem::BindingSlot(NormBindingSlot {
            policy: None,
            has_let: false,
            deduce: Vec::new(),
            value_pattern: NormPattern::Pack {
                inner: Box::new(NormPattern::Binder {
                    name: "args".to_string(),
                    origin: NormOrigin::Generated { rule, span },
                }),
                origin: NormOrigin::Generated { rule, span },
            },
            annotation: None,
            with_clause: None,
            initializer: None,
            origin: NormOrigin::Generated { rule, span },
        }));
    closure
}

fn generated_name(name: &str, span: Span, rule: NormRule) -> NormExpr {
    NormExpr::Name {
        text: name.to_string(),
        origin: NormOrigin::Generated { rule, span },
    }
}

fn generated_nav(names: &[&str], span: Span, rule: NormRule) -> NormExpr {
    NormExpr::Nav {
        components: names
            .iter()
            .map(|name| NormNavComponent::Name {
                name: (*name).to_string(),
                origin: NormOrigin::Generated { rule, span },
            })
            .collect(),
        explicit_terminated: false,
        origin: NormOrigin::Generated { rule, span },
    }
}

fn selector_name(selector: &SelectorAst) -> String {
    match selector {
        SelectorAst::Text(name) => name.text.clone(),
    }
}

fn normalize_error(error: &ErrorAst) -> NormError {
    NormError {
        message: error.message.clone(),
        origin: NormOrigin::Source(error.span),
    }
}

fn expr_span(expr: &NormExpr) -> Option<Span> {
    match expr {
        NormExpr::Call { origin, .. }
        | NormExpr::Name { origin, .. }
        | NormExpr::Literal { origin, .. }
        | NormExpr::Nav { origin, .. }
        | NormExpr::OperatorTarget { origin, .. }
        | NormExpr::Unsupported { origin, .. } => Some(origin_span(origin)),
        NormExpr::Product(product) => Some(origin_span(&product.origin)),
        NormExpr::Closure(closure) => Some(origin_span(&closure.origin)),
        NormExpr::Error(error) => Some(origin_span(&error.origin)),
    }
}

fn origin_span(origin: &NormOrigin) -> Span {
    match origin {
        NormOrigin::Source(span)
        | NormOrigin::Generated { span, .. }
        | NormOrigin::Derived { span, .. } => *span,
    }
}

fn skeleton_span(skeleton: &CanonicalSkeletonAst) -> Span {
    match skeleton {
        CanonicalSkeletonAst::Segment { span, .. }
        | CanonicalSkeletonAst::Pack { span, .. }
        | CanonicalSkeletonAst::ProductExtract { span, .. }
        | CanonicalSkeletonAst::Wildcard { span }
        | CanonicalSkeletonAst::Name { span, .. }
        | CanonicalSkeletonAst::NavPath { span, .. }
        | CanonicalSkeletonAst::Literal { span, .. } => *span,
        CanonicalSkeletonAst::Error(error) => error.span,
    }
}

fn annotation_operator_pattern_summary(kind: &OperatorExprKind) -> String {
    match kind {
        OperatorExprKind::OperatorSugar { .. } => {
            "operator sugar in annotation pattern".to_string()
        }
        OperatorExprKind::MemberSugar { .. } => "member sugar in annotation pattern".to_string(),
        OperatorExprKind::DoubleDotSugar { .. } => {
            "double-dot sugar in annotation pattern".to_string()
        }
        OperatorExprKind::BracketCallSugar { .. } => {
            "bracket-call sugar in annotation pattern".to_string()
        }
        _ => "unsupported annotation operator pattern".to_string(),
    }
}

fn annotation_atom_pattern_summary(kind: &AtomKind) -> String {
    match kind {
        AtomKind::MemberSugar { .. } => "member sugar in annotation pattern".to_string(),
        AtomKind::DoubleDotSugar { .. } => "double-dot sugar in annotation pattern".to_string(),
        AtomKind::BracketCallSugar { .. } => "bracket-call sugar in annotation pattern".to_string(),
        AtomKind::Closure(_) => "closure in annotation pattern".to_string(),
        _ => "unsupported annotation atom pattern".to_string(),
    }
}

fn dump_norm_form(output: &mut String, form: &NormForm, indent: usize) {
    match form {
        NormForm::Let(decl) => {
            line(output, indent, "Form Let");
            dump_norm_decl(output, decl, indent + 1);
        }
        NormForm::Alias(decl) => {
            line(output, indent, "Form Alias");
            dump_norm_decl(output, decl, indent + 1);
        }
        NormForm::Expr(expr) => {
            line(output, indent, "Form Expr");
            dump_norm_expr(output, expr, indent + 1);
        }
        NormForm::TailValue(expr) => {
            line(output, indent, "Form TailValue");
            dump_norm_expr(output, expr, indent + 1);
        }
        NormForm::ReturnEvent(return_ev) => {
            line(output, indent, "Form ReturnEvent");
            line(output, indent + 1, "value");
            dump_norm_expr(output, &return_ev.value, indent + 2);
            line(output, indent + 1, "target");
            match &return_ev.target {
                NormReturnTargetSyntax::ImplicitNearest => {
                    line(output, indent + 2, "ImplicitNearest");
                }
                NormReturnTargetSyntax::Explicit(target) => {
                    line(output, indent + 2, "Explicit");
                    dump_norm_expr(output, target, indent + 3);
                }
            }
        }
        NormForm::Error(error) => {
            line(output, indent, "Form Error");
            dump_norm_error(output, error, indent + 1);
        }
    }
}

fn dump_norm_decl(output: &mut String, decl: &NormDecl, indent: usize) {
    match decl {
        NormDecl::Let { slot, origin } => {
            line(
                output,
                indent,
                &format!("Decl Let {}", origin_inline(origin)),
            );
            dump_binding_slot(output, slot, indent + 1);
        }
        NormDecl::Alias {
            policy,
            binder,
            target,
            origin,
        } => {
            line(
                output,
                indent,
                &format!("Decl Alias {}", origin_inline(origin)),
            );
            if let Some(policy) = policy {
                line(output, indent + 1, "policy:");
                dump_norm_policy_spec(output, policy, indent + 2);
            }
            line(output, indent + 1, "binder:");
            dump_alias_binder(output, binder, indent + 2);
            line(output, indent + 1, "target:");
            dump_entity_ref(output, target, indent + 2);
        }
        NormDecl::Error(error) => {
            line(output, indent, "Decl Error");
            dump_norm_error(output, error, indent + 1);
        }
    }
}

fn dump_norm_expr(output: &mut String, expr: &NormExpr, indent: usize) {
    match expr {
        NormExpr::Call {
            source,
            target,
            origin,
        } => {
            line(output, indent, &format!("Call {}", origin_inline(origin)));
            line(output, indent + 1, "source:");
            dump_product(output, source, indent + 2);
            line(output, indent + 1, "target:");
            dump_norm_expr(output, target, indent + 2);
        }
        NormExpr::Product(product) => dump_product(output, product, indent),
        NormExpr::Name { text, origin } => line(
            output,
            indent,
            &format!("Name \"{}\" {}", escape_text(text), origin_inline(origin)),
        ),
        NormExpr::Literal { kind, text, origin } => line(
            output,
            indent,
            &format!(
                "Literal {} \"{}\" {}",
                literal_kind_label(*kind),
                escape_text(text),
                origin_inline(origin)
            ),
        ),
        NormExpr::Nav {
            components,
            origin,
            explicit_terminated,
        } => {
            if *explicit_terminated {
                line(
                    output,
                    indent,
                    &format!("Nav terminated {}", origin_inline(origin)),
                );
            } else {
                line(output, indent, &format!("Nav {}", origin_inline(origin)));
            }
            line(output, indent + 1, "components:");
            for component in components {
                dump_nav_component(output, component, indent + 2);
            }
        }
        NormExpr::Closure(closure) => dump_closure(output, closure, indent),
        NormExpr::OperatorTarget {
            spelling,
            fixity,
            arity,
            origin,
        } => line(
            output,
            indent,
            &format!(
                "OperatorTarget spelling=\"{}\" fixity={} arity={} {}",
                escape_text(spelling),
                norm_fixity_label(*fixity),
                arity,
                origin_inline(origin)
            ),
        ),
        NormExpr::Error(error) => {
            line(output, indent, "Expr Error");
            dump_norm_error(output, error, indent + 1);
        }
        NormExpr::Unsupported {
            raw_kind_summary,
            origin,
        } => line(
            output,
            indent,
            &format!(
                "Unsupported \"{}\" {}",
                escape_text(raw_kind_summary),
                origin_inline(origin)
            ),
        ),
    }
}

fn dump_product(output: &mut String, product: &NormProduct, indent: usize) {
    line(
        output,
        indent,
        &format!("Product {}", origin_inline(&product.origin)),
    );
    line(output, indent + 1, "elements:");
    if product.elements.is_empty() {
        line(output, indent + 2, "(empty)");
    }
    for element in &product.elements {
        match element {
            NormProductElem::Expr(expr) => {
                line(output, indent + 2, "ExprElem");
                dump_norm_expr(output, expr, indent + 3);
            }
            NormProductElem::Unit { origin } => {
                line(
                    output,
                    indent + 2,
                    &format!("Unit {}", origin_inline(origin)),
                );
            }
        }
    }
}

fn dump_norm_policy_spec(output: &mut String, policy: &NormPolicySpec, indent: usize) {
    line(output, indent, "PolicySpec");
    line(output, indent + 1, "value_policy:");
    match &policy.value_policy {
        NormValuePolicyPattern::Conjunction(conjunction) => {
            dump_norm_policy_conjunction(output, conjunction, indent + 2)
        }
        NormValuePolicyPattern::Absent { .. } => line(output, indent + 2, "Absent"),
    }
    line(output, indent + 1, "pattern_policy:");
    match &policy.pattern_policy {
        Some(pattern_policy) => dump_norm_policy_conjunction(output, pattern_policy, indent + 2),
        None => line(output, indent + 2, "None"),
    }
}

fn dump_norm_policy_conjunction(
    output: &mut String,
    conjunction: &NormPolicyConjunction,
    indent: usize,
) {
    line(output, indent, "PolicyConjunction");
    for choice in &conjunction.choices {
        line(output, indent + 1, "PolicyChoice");
        for atom in &choice.atoms {
            match atom {
                NormPolicyAtom::Name { text, .. } => {
                    line(output, indent + 2, &format!("PolicyAtom Name \"{text}\""));
                }
                NormPolicyAtom::HoleRef { text, .. } => {
                    line(
                        output,
                        indent + 2,
                        &format!("PolicyAtom HoleRef \"{text}\""),
                    );
                }
                NormPolicyAtom::Group { conjunction, .. } => {
                    line(output, indent + 2, "PolicyAtom Group");
                    dump_norm_policy_conjunction(output, conjunction, indent + 3);
                }
                NormPolicyAtom::AbsentValuePattern { .. } => {
                    line(output, indent + 2, "AbsentValuePattern");
                }
                NormPolicyAtom::Error(error) => dump_norm_error(output, error, indent + 2),
            }
        }
    }
}

fn dump_binding_slot(output: &mut String, slot: &NormBindingSlot, indent: usize) {
    line(
        output,
        indent,
        &format!(
            "BindingSlot let={} {}",
            slot.has_let,
            origin_inline(&slot.origin)
        ),
    );
    if let Some(policy) = &slot.policy {
        line(output, indent + 1, "policy:");
        dump_norm_policy_spec(output, policy, indent + 2);
    }
    line(output, indent + 1, "deduce:");
    if slot.deduce.is_empty() {
        line(output, indent + 2, "None");
    } else {
        for hole in &slot.deduce {
            dump_hole_decl(output, hole, indent + 2);
        }
    }
    line(output, indent + 1, "value_pattern:");
    dump_pattern(output, &slot.value_pattern, indent + 2);
    line(output, indent + 1, "annotation:");
    match &slot.annotation {
        Some(annotation) => dump_annotation(output, annotation, indent + 2),
        None => line(output, indent + 2, "None"),
    }
    line(output, indent + 1, "with_clause:");
    match &slot.with_clause {
        Some(with_clause) => dump_with_clause(output, with_clause, indent + 2),
        None => line(output, indent + 2, "None"),
    }
    line(output, indent + 1, "initializer:");
    match &slot.initializer {
        Some(initializer) => dump_norm_expr(output, initializer, indent + 2),
        None => line(output, indent + 2, "None"),
    }
}

fn dump_hole_decl(output: &mut String, hole: &NormHoleDecl, indent: usize) {
    line(
        output,
        indent,
        &format!(
            "HoleDecl \"{}\" {}",
            escape_text(&hole.name),
            origin_inline(&hole.origin)
        ),
    );
    line(output, indent + 1, "annotation:");
    match &hole.annotation {
        Some(annotation) => dump_annotation(output, annotation, indent + 2),
        None => line(output, indent + 2, "None"),
    }
}

fn dump_annotation(output: &mut String, annotation: &NormAnnotation, indent: usize) {
    line(
        output,
        indent,
        &format!("AnnotationPattern {}", origin_inline(&annotation.origin)),
    );
    dump_pattern(output, &annotation.pattern, indent + 1);
}

fn dump_pattern(output: &mut String, pattern: &NormPattern, indent: usize) {
    match pattern {
        NormPattern::Binder { name, origin } => line(
            output,
            indent,
            &format!("Binder \"{}\" {}", escape_text(name), origin_inline(origin)),
        ),
        NormPattern::OperatorBinder { spelling, origin } => line(
            output,
            indent,
            &format!(
                "OperatorBinder \"{}\" {}",
                escape_text(spelling),
                origin_inline(origin)
            ),
        ),
        NormPattern::Product { elements, origin } => {
            line(
                output,
                indent,
                &format!("PatternProduct {}", origin_inline(origin)),
            );
            line(output, indent + 1, "elements:");
            if elements.is_empty() {
                line(output, indent + 2, "(empty)");
            }
            for element in elements {
                dump_pattern_elem(output, element, indent + 2);
            }
        }
        NormPattern::Pack { inner, origin } => {
            line(output, indent, &format!("Pack {}", origin_inline(origin)));
            dump_pattern(output, inner, indent + 1);
        }
        NormPattern::Unit { origin } => {
            line(output, indent, &format!("Unit {}", origin_inline(origin)))
        }
        NormPattern::HoleRef { name, origin, .. } => line(
            output,
            indent,
            &format!(
                "HoleRef \"{}\" {}",
                escape_text(name),
                origin_inline(origin)
            ),
        ),
        NormPattern::AnonymousHole { origin } => line(
            output,
            indent,
            &format!("AnnotationHole {}", origin_inline(origin)),
        ),
        NormPattern::Name { name, origin } => line(
            output,
            indent,
            &format!(
                "PatternName \"{}\" {}",
                escape_text(name),
                origin_inline(origin)
            ),
        ),
        NormPattern::Literal { text, origin } => line(
            output,
            indent,
            &format!(
                "PatternLiteral \"{}\" {}",
                escape_text(text),
                origin_inline(origin)
            ),
        ),
        NormPattern::Nav {
            components,
            explicit_terminated,
            origin,
        } => {
            line(
                output,
                indent,
                &format!(
                    "PatternNav terminated={} {}",
                    explicit_terminated,
                    origin_inline(origin)
                ),
            );
            line(output, indent + 1, "components:");
            for component in components {
                dump_nav_component(output, component, indent + 2);
            }
        }
        NormPattern::Sequence { elements, origin } => {
            line(
                output,
                indent,
                &format!("PatternSequence {}", origin_inline(origin)),
            );
            line(output, indent + 1, "elements:");
            for element in elements {
                dump_pattern(output, element, indent + 2);
            }
        }
        NormPattern::Skeleton { skeleton, origin } => {
            line(
                output,
                indent,
                &format!("PatternSkeleton {}", origin_inline(origin)),
            );
            dump_skeleton(output, skeleton, indent + 1);
        }
        NormPattern::BindingSlot { slot, origin } => {
            line(
                output,
                indent,
                &format!("PatternBindingSlot {}", origin_inline(origin)),
            );
            dump_binding_slot(output, slot, indent + 1);
        }
        NormPattern::Error(error) => {
            line(output, indent, "Pattern Error");
            dump_norm_error(output, error, indent + 1);
        }
        NormPattern::Unsupported {
            raw_kind_summary,
            origin,
        } => line(
            output,
            indent,
            &format!(
                "PatternUnsupported \"{}\" {}",
                escape_text(raw_kind_summary),
                origin_inline(origin)
            ),
        ),
    }
}

fn dump_pattern_elem(output: &mut String, element: &NormPatternElem, indent: usize) {
    match element {
        NormPatternElem::Pattern(pattern) => dump_pattern(output, pattern, indent),
        NormPatternElem::BindingSlot(slot) => {
            line(output, indent, "BindingSlotElem");
            dump_binding_slot(output, slot, indent + 1);
        }
        NormPatternElem::Unit { origin } => {
            line(output, indent, &format!("Unit {}", origin_inline(origin)));
        }
    }
}

fn dump_skeleton(output: &mut String, skeleton: &NormSkeleton, indent: usize) {
    match skeleton {
        NormSkeleton::Segment { elements, origin } => {
            line(
                output,
                indent,
                &format!("SkeletonSegment {}", origin_inline(origin)),
            );
            for element in elements {
                dump_skeleton(output, element, indent + 1);
            }
        }
        NormSkeleton::Product { elements, origin } => {
            line(
                output,
                indent,
                &format!("SkeletonProduct {}", origin_inline(origin)),
            );
            for element in elements {
                match element {
                    NormSkeletonElem::Skeleton(skeleton) => {
                        dump_skeleton(output, skeleton, indent + 1)
                    }
                    NormSkeletonElem::Unit { origin } => {
                        line(
                            output,
                            indent + 1,
                            &format!("Unit {}", origin_inline(origin)),
                        );
                    }
                }
            }
        }
        NormSkeleton::Wildcard { origin } => {
            line(
                output,
                indent,
                &format!("SkeletonWildcard {}", origin_inline(origin)),
            );
        }
        NormSkeleton::Name { name, role, origin } => line(
            output,
            indent,
            &format!(
                "SkeletonName \"{}\" role={} {}",
                escape_text(name),
                canonical_role_label(*role),
                origin_inline(origin)
            ),
        ),
        NormSkeleton::HoleRef { name, origin, .. } => line(
            output,
            indent,
            &format!(
                "SkeletonHoleRef \"{}\" {}",
                escape_text(name),
                origin_inline(origin)
            ),
        ),
        NormSkeleton::Nav {
            components,
            explicit_terminated: _et,
            origin,
        } => line(
            output,
            indent,
            &format!(
                "SkeletonNav [{}] {}",
                components
                    .iter()
                    .map(|c| match c {
                        NormNavComponent::Name { name, .. } => format!("\"{}\"", escape_text(name)),
                        NormNavComponent::Operator { spelling, .. } => spelling.clone(),
                        NormNavComponent::Group { .. } => "(...)".to_string(),
                        NormNavComponent::Error(_) => "<?>".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                origin_inline(origin)
            ),
        ),
        NormSkeleton::Literal { text, origin } => line(
            output,
            indent,
            &format!(
                "SkeletonLiteral \"{}\" {}",
                escape_text(text),
                origin_inline(origin)
            ),
        ),
        NormSkeleton::Error(error) => {
            line(output, indent, "Skeleton Error");
            dump_norm_error(output, error, indent + 1);
        }
    }
}

fn dump_with_clause(output: &mut String, with_clause: &NormWithClause, indent: usize) {
    line(
        output,
        indent,
        &format!(
            "WithClause explicit_empty={} {}",
            with_clause.explicit_empty,
            origin_inline(&with_clause.origin)
        ),
    );
    if !with_clause.names.is_empty() {
        line(output, indent + 1, "names:");
        for name in &with_clause.names {
            line(output, indent + 2, &format!("\"{}\"", escape_text(name)));
        }
    }
    if let Some(error) = &with_clause.error {
        line(output, indent + 1, "error:");
        dump_norm_error(output, error, indent + 2);
    }
}

fn dump_closure(output: &mut String, closure: &NormClosure, indent: usize) {
    line(
        output,
        indent,
        &format!(
            "Closure placement={} {}",
            closure_placement_label(closure.placement),
            origin_inline(&closure.origin)
        ),
    );
    line(output, indent + 1, "head:");
    match &closure.head {
        Some(head) => dump_closure_head(output, head, indent + 2),
        None => line(output, indent + 2, "None"),
    }
    line(output, indent + 1, "body:");
    match &closure.body {
        NormClosureBody::Block(program) => dump_norm_program_body(output, program, indent + 2),
        NormClosureBody::NamedBlock { strategy, body, .. } => {
            line(
                output,
                indent + 2,
                &format!("UserBody strategy=Named({strategy})"),
            );
            dump_norm_program_body(output, body, indent + 3);
        }
        NormClosureBody::Defaulted { .. } => line(output, indent + 2, "Defaulted"),
        NormClosureBody::Delete(del) => {
            line(output, indent + 2, "Delete");
            if let Some(message) = &del.message {
                line(output, indent + 3, &format!("String {message}"));
            } else {
                line(output, indent + 3, "None");
            }
        }
    }
}

fn dump_closure_head(output: &mut String, head: &NormClosureHead, indent: usize) {
    line(
        output,
        indent,
        &format!("ClosureHead {}", origin_inline(&head.origin)),
    );
    line(output, indent + 1, "deduce:");
    if head.deduce.is_empty() {
        line(output, indent + 2, "None");
    } else {
        for hole in &head.deduce {
            dump_hole_decl(output, hole, indent + 2);
        }
    }
    if !head.captures.is_empty() {
        line(output, indent + 1, "captures:");
        for capture in &head.captures {
            line(
                output,
                indent + 2,
                &format!("Capture {}", origin_inline(&capture.origin)),
            );
            line(output, indent + 3, "slot:");
            dump_binding_slot(output, &capture.slot, indent + 4);
            line(output, indent + 3, "initializer:");
            dump_norm_expr(output, &capture.initializer, indent + 4);
        }
    }
    line(output, indent + 1, "params:");
    if head.params.is_empty() {
        line(output, indent + 2, "None");
    } else {
        for param in &head.params {
            dump_pattern_elem(output, param, indent + 2);
        }
    }
    if let Some(policy) = &head.call_policy {
        line(output, indent + 1, "call_policy:");
        dump_norm_policy_spec(output, policy, indent + 2);
    }
    if let Some(returns) = &head.returns {
        line(output, indent + 1, "returns:");
        dump_binding_slot(output, returns, indent + 2);
    }
    if !head.clauses.is_empty() {
        line(output, indent + 1, "clauses:");
        for clause in &head.clauses {
            dump_head_clause(output, clause, indent + 2);
        }
    }
}

fn dump_head_clause(output: &mut String, clause: &NormHeadClause, indent: usize) {
    match clause {
        NormHeadClause::Require { expr, origin } => {
            dump_named_clause(output, "Require", expr, origin, indent)
        }
        NormHeadClause::Pre { expr, origin } => {
            dump_named_clause(output, "Pre", expr, origin, indent)
        }
        NormHeadClause::Post { expr, origin } => {
            dump_named_clause(output, "Post", expr, origin, indent)
        }
        NormHeadClause::LifetimePre { expr, origin } => {
            dump_named_clause(output, "LifetimePre", expr, origin, indent)
        }
        NormHeadClause::LifetimePost { expr, origin } => {
            dump_named_clause(output, "LifetimePost", expr, origin, indent)
        }
        NormHeadClause::Error(error) => {
            line(output, indent, "HeadClause Error");
            dump_norm_error(output, error, indent + 1);
        }
    }
}

fn dump_named_clause(
    output: &mut String,
    name: &str,
    expr: &NormExpr,
    origin: &NormOrigin,
    indent: usize,
) {
    line(
        output,
        indent,
        &format!("{} {}", name, origin_inline(origin)),
    );
    dump_norm_expr(output, expr, indent + 1);
}

fn dump_norm_program_body(output: &mut String, program: &NormProgram, indent: usize) {
    line(
        output,
        indent,
        &format!("NormBody {}", origin_inline(&program.origin)),
    );
    line(output, indent + 1, "forms:");
    if program.forms.is_empty() {
        line(output, indent + 2, "(empty)");
    }
    for form in &program.forms {
        dump_norm_form(output, form, indent + 2);
    }
}

fn dump_alias_binder(output: &mut String, binder: &NormAliasBinder, indent: usize) {
    match binder {
        NormAliasBinder::Name { name, origin } => line(
            output,
            indent,
            &format!("Name \"{}\" {}", escape_text(name), origin_inline(origin)),
        ),
        NormAliasBinder::Operator { spelling, origin } => line(
            output,
            indent,
            &format!(
                "Operator \"{}\" {}",
                escape_text(spelling),
                origin_inline(origin)
            ),
        ),
        NormAliasBinder::Error(error) => {
            line(output, indent, "AliasBinder Error");
            dump_norm_error(output, error, indent + 1);
        }
    }
}

fn dump_entity_ref(output: &mut String, entity_ref: &NormEntityRef, indent: usize) {
    line(
        output,
        indent,
        &format!("EntityRef {}", origin_inline(&entity_ref.origin)),
    );
    line(output, indent + 1, "components:");
    for component in &entity_ref.components {
        dump_nav_component(output, component, indent + 2);
    }
}

fn dump_nav_component(output: &mut String, component: &NormNavComponent, indent: usize) {
    match component {
        NormNavComponent::Name { name, origin } => line(
            output,
            indent,
            &format!("Name \"{}\" {}", escape_text(name), origin_inline(origin)),
        ),
        NormNavComponent::Operator { spelling, origin } => line(
            output,
            indent,
            &format!(
                "Operator \"{}\" {}",
                escape_text(spelling),
                origin_inline(origin)
            ),
        ),
        NormNavComponent::Group { expr, origin } => {
            line(output, indent, &format!("Group {}", origin_inline(origin)));
            dump_norm_expr(output, expr, indent + 1);
        }
        NormNavComponent::Error(error) => {
            line(output, indent, "NavComponent Error");
            dump_norm_error(output, error, indent + 1);
        }
    }
}

fn dump_norm_error(output: &mut String, error: &NormError, indent: usize) {
    line(
        output,
        indent,
        &format!(
            "Error \"{}\" {}",
            escape_text(&error.message),
            origin_inline(&error.origin)
        ),
    );
}

fn origin_inline(origin: &NormOrigin) -> String {
    match origin {
        NormOrigin::Source(span) => format!("origin=Source{}", span_inline(*span)),
        NormOrigin::Generated { rule, span } => {
            format!(
                "origin=Generated({}){}",
                rule_label(*rule),
                span_inline(*span)
            )
        }
        NormOrigin::Derived {
            rule,
            span,
            summary,
        } => format!(
            "origin=Derived({}; {}){}",
            rule_label(*rule),
            escape_text(summary),
            span_inline(*span)
        ),
    }
}

fn span_inline(span: Span) -> String {
    format!(
        "@{}:{}[{}..{}]",
        span.line, span.column, span.byte_start, span.byte_end
    )
}

fn line(output: &mut String, indent: usize, text: &str) {
    for _ in 0..indent {
        output.push_str("  ");
    }
    output.push_str(text);
    output.push('\n');
}

fn rule_label(rule: NormRule) -> &'static str {
    match rule {
        NormRule::ProductLift => "ProductLift",
        NormRule::ProductMerge => "ProductMerge",
        NormRule::PipeFallback => "PipeFallback",
        NormRule::SecondLegalityRepair => "SecondLegalityRepair",
        NormRule::OperatorLowering => "OperatorLowering",
        NormRule::PrefixNegativeLowering => "PrefixNegativeLowering",
        NormRule::DotClosureLowering => "DotClosureLowering",
        NormRule::MemberLowering => "MemberLowering",
        NormRule::DoubleDotLowering => "DoubleDotLowering",
        NormRule::BracketCallLowering => "BracketCallLowering",
        NormRule::BranchNameExpansion => "BranchNameExpansion",
        NormRule::AliasPreserve => "AliasPreserve",
        NormRule::ClosureNormalize => "ClosureNormalize",
        NormRule::CaptureNameInference => "CaptureNameInference",
        NormRule::PatternNormalize => "PatternNormalize",
        NormRule::Unsupported => "Unsupported",
    }
}

fn raw_fixity_label(fixity: OperatorFixity) -> &'static str {
    match fixity {
        OperatorFixity::Prefix => "Prefix",
        OperatorFixity::Postfix => "Postfix",
        OperatorFixity::Binary => "Binary",
    }
}

fn norm_fixity_label(fixity: NormOperatorFixity) -> &'static str {
    match fixity {
        NormOperatorFixity::Prefix => "Prefix",
        NormOperatorFixity::Postfix => "Postfix",
        NormOperatorFixity::Binary => "Binary",
        NormOperatorFixity::BracketCall => "BracketCall",
    }
}

fn literal_kind_label(kind: NormLiteralKind) -> &'static str {
    match kind {
        NormLiteralKind::Int => "Int",
        NormLiteralKind::Float => "Float",
        NormLiteralKind::String => "String",
    }
}

fn closure_placement_label(placement: NormClosurePlacement) -> &'static str {
    match placement {
        NormClosurePlacement::InPlace => "InPlace",
        NormClosurePlacement::Ordinary => "Ordinary",
    }
}

fn canonical_role_label(role: NormCanonicalNameRole) -> &'static str {
    match role {
        NormCanonicalNameRole::Hole => "Hole",
        NormCanonicalNameRole::NodeName => "NodeName",
        NormCanonicalNameRole::Unknown => "Unknown",
    }
}

fn escape_text(text: &str) -> String {
    let mut escaped = String::new();

    for ch in text.chars() {
        match ch {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            _ => escaped.push(ch),
        }
    }

    escaped
}
