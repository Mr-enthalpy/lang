//! Canonical Pattern applicability and extraction boundary.
//!
//! The representation used by this first connected slice is deliberately
//! private.  Consumers receive proof-relevant solutions of `R_Gamma(P,c,rho)`;
//! they do not inspect a schema/shape AST and they never infer structural
//! incidence from ordinary Val2 membership.

use std::collections::BTreeMap;

use lang_syntax::{
    HoleBinderId, NormAnnotation, NormClosure, NormPattern, NormPatternElem, NormSkeleton,
    NormSkeletonElem,
};

use crate::{
    CanonicalFullNavigation, CanonicalPatternValue, CanonicalValueAddr, Diagnostic,
    OverloadArgShape, PatternValueId, Provenance, ResolvedHoleBinderId, ResolvedPatternRootId,
    SemanticOwnerId, SemanticValueId, SpecificityTuple,
};

/// Stable implicit candidate-family filter used only while Pattern itself
/// performs one registered real-field extraction.
///
/// This marker is not stored in Pattern normal form and ordinary `x.field`
/// lookup must never acquire it implicitly.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructuralDefault;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PatternSelector {
    Named(String),
    Positional(usize),
}

/// Proof that one child comes from the Pattern's registered structural value,
/// rather than from an ordinary navigable Val2 member.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectPatternChildEvidence {
    pub parent: PatternValueId,
    pub selector: PatternSelector,
    pub child: CanonicalPatternValue,
    pub extraction_family: StructuralDefault,
}

/// Query a recorded canonical structural Pattern value for direct incidence.
/// Ordinary Val2 state is intentionally absent from this interface.
pub fn direct_pattern_child_from_canonical_value(
    parent: PatternValueId,
    value: &CanonicalPatternValue,
    selector: &PatternSelector,
) -> Option<DirectPatternChildEvidence> {
    let body = match value {
        CanonicalPatternValue::NamedPattern { body, .. } => body.as_ref(),
        other => other,
    };
    let child = match (body, selector) {
        (CanonicalPatternValue::UnorderedLayer(entries), PatternSelector::Named(name)) => entries
            .iter()
            .find(|(navigation, _)| navigation.components().first() == Some(name))
            .map(|(_, child)| child.clone()),
        (CanonicalPatternValue::OrderedLayer(entries), PatternSelector::Positional(index)) => {
            entries.get(*index).map(|entry| entry.value.clone())
        }
        (CanonicalPatternValue::OrderedLayer(entries), PatternSelector::Named(name)) => entries
            .iter()
            .find(|entry| {
                entry
                    .navigation
                    .as_ref()
                    .and_then(|navigation| navigation.components().first())
                    == Some(name)
            })
            .map(|entry| entry.value.clone()),
        _ => None,
    }?;
    Some(DirectPatternChildEvidence {
        parent,
        selector: selector.clone(),
        child,
        extraction_family: StructuralDefault,
    })
}

/// Root-local identity of an ordinary value binder in a Pattern query.
/// Display spelling is retained separately for body transport only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedPatternBinderId {
    pub root: ResolvedPatternRootId,
    pub local_ordinal: u32,
}

/// Type observation extracted from a known argument.
///
/// Core observation is the preferred equality coordinate.  Pattern identity
/// is the explicit detached boundary for fixtures/world-free observations;
/// bare TypeValueId never participates in equality here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtractedTypeObservation {
    pub pattern: PatternValueId,
    pub core: Option<CanonicalValueAddr>,
}

impl ExtractedTypeObservation {
    fn semantic_eq(self, other: Self) -> bool {
        match (self.core, other.core) {
            (Some(left), Some(right)) => left == right,
            _ => self.pattern == other.pattern,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternLocalBinding {
    pub display_name: String,
    pub argument: OverloadArgShape,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternPackBinding {
    pub display_name: String,
    pub arguments: Vec<OverloadArgShape>,
}

/// One successful relational derivation `rho`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PatternRelationDerivation {
    pub holes: BTreeMap<ResolvedHoleBinderId, ExtractedTypeObservation>,
    pub local_bindings: BTreeMap<ResolvedPatternBinderId, PatternLocalBinding>,
    pub pack_bindings: BTreeMap<ResolvedPatternBinderId, PatternPackBinding>,
}

impl PatternRelationDerivation {
    fn bind_hole(
        &mut self,
        hole: ResolvedHoleBinderId,
        observation: ExtractedTypeObservation,
    ) -> bool {
        match self.holes.get(&hole).copied() {
            Some(existing) => existing.semantic_eq(observation),
            None => {
                self.holes.insert(hole, observation);
                true
            }
        }
    }

    fn merge(mut self, other: Self) -> Option<Self> {
        for (hole, observation) in other.holes {
            if !self.bind_hole(hole, observation) {
                return None;
            }
        }
        for (binder, binding) in other.local_bindings {
            if self.local_bindings.insert(binder, binding).is_some() {
                return None;
            }
        }
        for (binder, binding) in other.pack_bindings {
            if self.pack_bindings.insert(binder, binding).is_some() {
                return None;
            }
        }
        Some(self)
    }
}

/// Proof of applicability.  `solutions` is intentionally plural: uniqueness
/// is a derived consumer property, never an axiom of Pattern matching.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternApplicabilityProof {
    pub root: ResolvedPatternRootId,
    pub solutions: Vec<PatternRelationDerivation>,
    pub specificity: SpecificityTuple,
}

impl PatternApplicabilityProof {
    /// Body-evaluator transport for the currently restricted source-body
    /// evaluator.  It is derived from exact binder identities after the
    /// relation succeeds; spelling never participates in matching.
    pub fn named_bindings(&self) -> BTreeMap<String, OverloadArgShape> {
        self.solutions
            .first()
            .into_iter()
            .flat_map(|solution| solution.local_bindings.values())
            .map(|binding| (binding.display_name.clone(), binding.argument.clone()))
            .collect()
    }

    pub fn named_pack_bindings(&self) -> BTreeMap<String, Vec<OverloadArgShape>> {
        self.solutions
            .first()
            .into_iter()
            .flat_map(|solution| solution.pack_bindings.values())
            .map(|binding| (binding.display_name.clone(), binding.arguments.clone()))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NamedPatternObservation {
    pub pattern: PatternValueId,
    pub core: Option<CanonicalValueAddr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternRelationFailure {
    Inapplicable(Diagnostic),
    Unsupported(Diagnostic),
}

pub struct PatternRelationContext<'a> {
    pub root: ResolvedPatternRootId,
    frontend_owner: lang_syntax::NormSemanticOwnerId,
    resolve_named: Option<&'a dyn Fn(&str) -> Option<NamedPatternObservation>>,
}

impl<'a> PatternRelationContext<'a> {
    pub fn for_source_callable(
        closure: &NormClosure,
        callable_owner: SemanticOwnerId,
        resolve_named: Option<&'a dyn Fn(&str) -> Option<NamedPatternObservation>>,
    ) -> Result<Self, PatternRelationFailure> {
        let frontend_owner = closure
            .semantic_owner
            .map(|owner| owner.id)
            .ok_or_else(|| {
                PatternRelationFailure::Unsupported(Diagnostic::hard_error(
                    "source callable has no alpha-normalized semantic owner",
                    Some(Provenance::from_norm_origin(
                        "source callable Pattern owner",
                        &closure.origin,
                    )),
                ))
            })?;
        let local_root = closure
            .head
            .as_ref()
            .and_then(first_frontend_pattern_root)
            .map_or(0, |root| root.local_root);
        Ok(Self {
            root: ResolvedPatternRootId {
                owner: callable_owner,
                local_root,
            },
            frontend_owner,
            resolve_named,
        })
    }

    fn qualify_hole(
        &self,
        target: HoleBinderId,
        provenance: &Provenance,
    ) -> Result<ResolvedHoleBinderId, PatternRelationFailure> {
        let root = target.pattern_root();
        if root.owner != self.frontend_owner || root.local_root != self.root.local_root {
            return Err(PatternRelationFailure::Unsupported(Diagnostic::hard_error(
                "Pattern hole belongs to an inherited or unrelated PatternRoot that is not yet qualified in this callable",
                Some(provenance.clone()),
            )));
        }
        Ok(ResolvedHoleBinderId {
            root: self.root,
            local_binder: target.local_ordinal(),
        })
    }
}

/// Evaluate the currently supported parameter Product through the relational
/// Pattern authority.  The normalized AST is merely query input; callers only
/// consume the returned derivations.
pub fn solve_parameter_product_relation(
    params: &[NormPatternElem],
    args: &[OverloadArgShape],
    context: &PatternRelationContext<'_>,
) -> Result<PatternApplicabilityProof, PatternRelationFailure> {
    let pack_index = params.iter().position(param_is_pack);
    let fixed_suffix = pack_index.map_or(0, |index| params.len() - index - 1);
    let mut solutions = vec![PatternRelationDerivation::default()];
    let mut specificity = SpecificityTuple::default();

    for (index, param) in params.iter().enumerate() {
        let binder = ResolvedPatternBinderId {
            root: context.root,
            local_ordinal: index as u32,
        };
        let (derived, coordinate_specificity) = if Some(index) == pack_index {
            let remainder_end = args.len() - fixed_suffix;
            solve_pack(param, &args[index..remainder_end], binder)?
        } else {
            let arg_index = if let Some(pack_index) = pack_index {
                if index < pack_index {
                    index
                } else {
                    args.len() - (params.len() - index)
                }
            } else {
                index
            };
            solve_one(param, &args[arg_index], binder, context)?
        };
        let mut combined = Vec::new();
        for prefix in &solutions {
            for suffix in &derived {
                if let Some(merged) = prefix.clone().merge(suffix.clone()) {
                    if !combined.contains(&merged) {
                        combined.push(merged);
                    }
                }
            }
        }
        if combined.is_empty() {
            return Err(PatternRelationFailure::Inapplicable(
                Diagnostic::hard_error(
                    "parameter Pattern extractions disagree on a shared HoleBinderId",
                    param_provenance(param),
                ),
            ));
        }
        solutions = combined;
        specificity = specificity.add(coordinate_specificity);
    }

    Ok(PatternApplicabilityProof {
        root: context.root,
        solutions,
        specificity,
    })
}

fn solve_one(
    element: &NormPatternElem,
    arg: &OverloadArgShape,
    binder: ResolvedPatternBinderId,
    context: &PatternRelationContext<'_>,
) -> Result<(Vec<PatternRelationDerivation>, SpecificityTuple), PatternRelationFailure> {
    let NormPatternElem::BindingSlot(slot) = element else {
        return Err(unsupported(
            "parameter element is not a let-shaped binding slot",
            param_provenance(element),
        ));
    };
    if matches!(slot.value_pattern, NormPattern::Pack { .. }) {
        return Err(unsupported(
            "pack Pattern requires the remaining Product slice",
            Some(Provenance::from_norm_origin(
                "pack parameter Pattern",
                &slot.origin,
            )),
        ));
    }

    let mut derivations = solve_value_pattern(&slot.value_pattern, arg, binder, context)?;
    let mut specificity = specificity_for_pattern(&slot.value_pattern);
    if let Some(annotation) = &slot.annotation {
        let (constraints, annotation_specificity) = solve_annotation(annotation, arg, context)?;
        derivations = relational_product(derivations, constraints);
        if derivations.is_empty() {
            return Err(inapplicable(
                "parameter Pattern annotation has no successful extraction",
                arg,
            ));
        }
        specificity = specificity.add(annotation_specificity);
    }
    Ok((derivations, specificity))
}

fn solve_value_pattern(
    pattern: &NormPattern,
    arg: &OverloadArgShape,
    binder: ResolvedPatternBinderId,
    context: &PatternRelationContext<'_>,
) -> Result<Vec<PatternRelationDerivation>, PatternRelationFailure> {
    match pattern {
        NormPattern::Binder { name, .. } if name != "_" => {
            let mut derivation = PatternRelationDerivation::default();
            derivation.local_bindings.insert(
                binder,
                PatternLocalBinding {
                    display_name: name.clone(),
                    argument: arg.clone(),
                },
            );
            Ok(vec![derivation])
        }
        NormPattern::Binder { .. }
        | NormPattern::Skeleton {
            skeleton: NormSkeleton::Wildcard { .. },
            ..
        } => Ok(vec![PatternRelationDerivation::default()]),
        NormPattern::HoleRef { target, origin, .. } => {
            let observation = extracted_type(arg).ok_or_else(|| {
                inapplicable("Pattern hole could not observe an argument type", arg)
            })?;
            let provenance = Provenance::from_norm_origin("Pattern hole extraction", origin);
            let hole = context.qualify_hole(*target, &provenance)?;
            let mut derivation = PatternRelationDerivation::default();
            derivation.bind_hole(hole, observation);
            Ok(vec![derivation])
        }
        NormPattern::Name { name, .. } => solve_named(name, arg, context),
        NormPattern::Skeleton { skeleton, .. } => solve_skeleton(skeleton, arg, context),
        NormPattern::Sequence { elements, .. } => {
            let mut terms = Vec::new();
            for element in elements {
                collect_relational_terms(element, &mut terms);
            }
            solve_terms(&terms, arg, context)
        }
        other => Err(unsupported(
            "Pattern relation does not yet implement this canonical-space query",
            Some(Provenance::from_norm_origin(
                "parameter Pattern",
                pattern_origin(other),
            )),
        )),
    }
}

fn solve_annotation(
    annotation: &NormAnnotation,
    arg: &OverloadArgShape,
    context: &PatternRelationContext<'_>,
) -> Result<(Vec<PatternRelationDerivation>, SpecificityTuple), PatternRelationFailure> {
    match &annotation.pattern {
        NormPattern::Name { name, .. } if name == "type" => {
            if arg.is_value || arg.pattern_value.is_none() {
                return Err(inapplicable(
                    "type annotation expected a complete type Object",
                    arg,
                ));
            }
            Ok((
                vec![PatternRelationDerivation::default()],
                SpecificityTuple {
                    max_depth: 1,
                    sum_depth: 1,
                    non_discard_explicit_node_count: 1,
                    ..SpecificityTuple::default()
                },
            ))
        }
        pattern => {
            if !arg.is_value || extracted_type(arg).is_none() {
                return Err(inapplicable(
                    "value Pattern annotation expected a typed Val1 argument",
                    arg,
                ));
            }
            let derived = solve_value_pattern(
                pattern,
                arg,
                ResolvedPatternBinderId {
                    root: context.root,
                    local_ordinal: u32::MAX,
                },
                context,
            )?;
            Ok((derived, specificity_for_pattern(pattern)))
        }
    }
}

fn solve_pack(
    element: &NormPatternElem,
    args: &[OverloadArgShape],
    binder: ResolvedPatternBinderId,
) -> Result<(Vec<PatternRelationDerivation>, SpecificityTuple), PatternRelationFailure> {
    let NormPatternElem::BindingSlot(slot) = element else {
        return Err(unsupported(
            "pack element is not a binding slot",
            param_provenance(element),
        ));
    };
    let NormPattern::Pack { inner, origin } = &slot.value_pattern else {
        return Err(unsupported(
            "non-pack parameter cannot consume an argument slice",
            param_provenance(element),
        ));
    };
    let specificity = match inner.as_ref() {
        NormPattern::Binder { name, .. } if name != "_" => {
            let mut derivation = PatternRelationDerivation::default();
            derivation.pack_bindings.insert(
                binder,
                PatternPackBinding {
                    display_name: name.clone(),
                    arguments: args.to_vec(),
                },
            );
            return Ok((
                vec![derivation],
                SpecificityTuple {
                    max_depth: 1,
                    sum_depth: 1,
                    explicit_pack_match_count: 1,
                    ..SpecificityTuple::default()
                },
            ));
        }
        NormPattern::Binder { .. }
        | NormPattern::Skeleton {
            skeleton: NormSkeleton::Wildcard { .. },
            ..
        } => SpecificityTuple {
            max_depth: 1,
            sum_depth: 1,
            pack_discard_count: 1,
            ..SpecificityTuple::default()
        },
        NormPattern::Product { .. } => {
            return Err(unsupported(
                "bare Product Pack operands are non-canonical after Pattern normalization",
                Some(Provenance::from_norm_origin("pack Pattern", origin)),
            ));
        }
        _ => {
            return Err(unsupported(
                "current relational Pack query supports only whole-remainder binders and discards",
                Some(Provenance::from_norm_origin("pack Pattern", origin)),
            ));
        }
    };
    Ok((vec![PatternRelationDerivation::default()], specificity))
}

#[derive(Clone, Copy)]
enum RelationalTerm<'a> {
    Named(&'a str),
    Hole(HoleBinderId, &'a lang_syntax::NormOrigin),
    Wildcard,
}

fn solve_skeleton(
    skeleton: &NormSkeleton,
    arg: &OverloadArgShape,
    context: &PatternRelationContext<'_>,
) -> Result<Vec<PatternRelationDerivation>, PatternRelationFailure> {
    let mut terms = Vec::new();
    collect_skeleton_terms(skeleton, &mut terms);
    solve_terms(&terms, arg, context)
}

fn solve_terms(
    terms: &[RelationalTerm<'_>],
    arg: &OverloadArgShape,
    context: &PatternRelationContext<'_>,
) -> Result<Vec<PatternRelationDerivation>, PatternRelationFailure> {
    let has_wildcard = terms
        .iter()
        .any(|term| matches!(term, RelationalTerm::Wildcard));
    if !has_wildcard {
        return Err(unsupported(
            "current relational alternative query requires an explicit discard position",
            Some(arg.provenance.clone()),
        ));
    }
    let observation = extracted_type(arg)
        .ok_or_else(|| inapplicable("Pattern could not observe an argument type", arg))?;
    let mut solutions = Vec::new();
    for term in terms {
        match term {
            RelationalTerm::Named(name) => {
                if named_matches(name, observation, context) {
                    solutions.push(PatternRelationDerivation::default());
                }
            }
            RelationalTerm::Hole(target, origin) => {
                let provenance = Provenance::from_norm_origin("Pattern hole alternative", origin);
                let hole = context.qualify_hole(*target, &provenance)?;
                let mut derivation = PatternRelationDerivation::default();
                derivation.bind_hole(hole, observation);
                solutions.push(derivation);
            }
            RelationalTerm::Wildcard => {}
        }
    }
    if solutions.is_empty() {
        return Err(inapplicable(
            "argument does not satisfy any relational Pattern alternative",
            arg,
        ));
    }
    solutions.dedup();
    Ok(solutions)
}

fn solve_named(
    name: &str,
    arg: &OverloadArgShape,
    context: &PatternRelationContext<'_>,
) -> Result<Vec<PatternRelationDerivation>, PatternRelationFailure> {
    let observation = extracted_type(arg)
        .ok_or_else(|| inapplicable("named Pattern could not observe an argument type", arg))?;
    if named_matches(name, observation, context) {
        Ok(vec![PatternRelationDerivation::default()])
    } else {
        Err(inapplicable(
            "argument does not satisfy the resolved named Pattern",
            arg,
        ))
    }
}

fn named_matches(
    name: &str,
    actual: ExtractedTypeObservation,
    context: &PatternRelationContext<'_>,
) -> bool {
    let Some(expected) = context.resolve_named.and_then(|resolve| resolve(name)) else {
        return false;
    };
    actual.semantic_eq(ExtractedTypeObservation {
        pattern: expected.pattern,
        core: expected.core,
    })
}

fn extracted_type(arg: &OverloadArgShape) -> Option<ExtractedTypeObservation> {
    Some(ExtractedTypeObservation {
        pattern: arg.pattern_value?,
        core: arg.type_core_observation,
    })
}

fn relational_product(
    left: Vec<PatternRelationDerivation>,
    right: Vec<PatternRelationDerivation>,
) -> Vec<PatternRelationDerivation> {
    let mut product = Vec::new();
    for lhs in &left {
        for rhs in &right {
            if let Some(merged) = lhs.clone().merge(rhs.clone()) {
                if !product.contains(&merged) {
                    product.push(merged);
                }
            }
        }
    }
    product
}

fn collect_relational_terms<'a>(pattern: &'a NormPattern, out: &mut Vec<RelationalTerm<'a>>) {
    match pattern {
        NormPattern::Name { name, .. } => out.push(RelationalTerm::Named(name)),
        NormPattern::HoleRef { target, origin, .. } => {
            out.push(RelationalTerm::Hole(*target, origin))
        }
        NormPattern::Binder { name, .. } if name == "_" => out.push(RelationalTerm::Wildcard),
        NormPattern::Skeleton { skeleton, .. } => collect_skeleton_terms(skeleton, out),
        NormPattern::Sequence { elements, .. } => {
            for element in elements {
                collect_relational_terms(element, out);
            }
        }
        _ => {}
    }
}

fn collect_skeleton_terms<'a>(skeleton: &'a NormSkeleton, out: &mut Vec<RelationalTerm<'a>>) {
    match skeleton {
        NormSkeleton::Wildcard { .. } => out.push(RelationalTerm::Wildcard),
        NormSkeleton::Name { name, .. } => out.push(RelationalTerm::Named(name)),
        NormSkeleton::HoleRef { target, origin, .. } => {
            out.push(RelationalTerm::Hole(*target, origin))
        }
        NormSkeleton::Segment { elements, .. } => {
            for element in elements {
                collect_skeleton_terms(element, out);
            }
        }
        NormSkeleton::Product { elements, .. } => {
            for element in elements {
                if let NormSkeletonElem::Skeleton(skeleton) = element {
                    collect_skeleton_terms(skeleton, out);
                }
            }
        }
        NormSkeleton::Nav { .. } | NormSkeleton::Literal { .. } | NormSkeleton::Error(_) => {}
    }
}

fn first_frontend_pattern_root(
    head: &lang_syntax::NormClosureHead,
) -> Option<lang_syntax::PatternRootId> {
    head.deduce
        .first()
        .map(|hole| hole.id.pattern_root())
        .or_else(|| head.params.iter().find_map(first_root_in_element))
        .or_else(|| head.returns.as_ref().and_then(first_root_in_slot))
}

fn first_root_in_element(element: &NormPatternElem) -> Option<lang_syntax::PatternRootId> {
    match element {
        NormPatternElem::Pattern(pattern) => first_root_in_pattern(pattern),
        NormPatternElem::BindingSlot(slot) => first_root_in_slot(slot),
        NormPatternElem::Unit { .. } => None,
    }
}

fn first_root_in_slot(slot: &lang_syntax::NormBindingSlot) -> Option<lang_syntax::PatternRootId> {
    slot.deduce
        .first()
        .map(|hole| hole.id.pattern_root())
        .or_else(|| first_root_in_pattern(&slot.value_pattern))
        .or_else(|| {
            slot.annotation
                .as_ref()
                .and_then(|annotation| first_root_in_pattern(&annotation.pattern))
        })
}

fn first_root_in_pattern(pattern: &NormPattern) -> Option<lang_syntax::PatternRootId> {
    match pattern {
        NormPattern::HoleRef { target, .. } => Some(target.pattern_root()),
        NormPattern::Product { elements, .. } => elements.iter().find_map(first_root_in_element),
        NormPattern::Pack { inner, .. } => first_root_in_pattern(inner),
        NormPattern::Sequence { elements, .. } => elements.iter().find_map(first_root_in_pattern),
        NormPattern::Skeleton { skeleton, .. } => first_root_in_skeleton(skeleton),
        NormPattern::BindingSlot { slot, .. } => first_root_in_slot(slot),
        _ => None,
    }
}

fn first_root_in_skeleton(skeleton: &NormSkeleton) -> Option<lang_syntax::PatternRootId> {
    match skeleton {
        NormSkeleton::HoleRef { target, .. } => Some(target.pattern_root()),
        NormSkeleton::Segment { elements, .. } => elements.iter().find_map(first_root_in_skeleton),
        NormSkeleton::Product { elements, .. } => {
            elements.iter().find_map(|element| match element {
                NormSkeletonElem::Skeleton(skeleton) => first_root_in_skeleton(skeleton),
                NormSkeletonElem::Unit { .. } => None,
            })
        }
        _ => None,
    }
}

fn specificity_for_pattern(pattern: &NormPattern) -> SpecificityTuple {
    match pattern {
        NormPattern::Binder { name, .. } if name != "_" => SpecificityTuple {
            max_depth: 1,
            sum_depth: 1,
            non_discard_explicit_node_count: 1,
            ..SpecificityTuple::default()
        },
        NormPattern::HoleRef { .. } => SpecificityTuple {
            max_depth: 1,
            sum_depth: 1,
            non_discard_explicit_node_count: 1,
            ..SpecificityTuple::default()
        },
        NormPattern::Binder { .. }
        | NormPattern::Skeleton {
            skeleton: NormSkeleton::Wildcard { .. },
            ..
        } => SpecificityTuple {
            max_depth: 1,
            sum_depth: 1,
            explicit_discard_count: 1,
            ..SpecificityTuple::default()
        },
        NormPattern::Name { .. } | NormPattern::Skeleton { .. } | NormPattern::Sequence { .. } => {
            SpecificityTuple {
                max_depth: 1,
                sum_depth: 2,
                non_discard_explicit_node_count: 1,
                explicit_discard_count: 1,
                ..SpecificityTuple::default()
            }
        }
        _ => SpecificityTuple::default(),
    }
}

fn param_is_pack(element: &NormPatternElem) -> bool {
    matches!(
        element,
        NormPatternElem::BindingSlot(slot)
            if matches!(&slot.value_pattern, NormPattern::Pack { .. })
    )
}

fn inapplicable(message: &str, arg: &OverloadArgShape) -> PatternRelationFailure {
    PatternRelationFailure::Inapplicable(Diagnostic::hard_error(
        message,
        Some(arg.provenance.clone()),
    ))
}

fn unsupported(message: &str, provenance: Option<Provenance>) -> PatternRelationFailure {
    PatternRelationFailure::Unsupported(Diagnostic::hard_error(message, provenance))
}

fn param_provenance(element: &NormPatternElem) -> Option<Provenance> {
    Some(match element {
        NormPatternElem::Pattern(pattern) => {
            Provenance::from_norm_origin("parameter Pattern", pattern_origin(pattern))
        }
        NormPatternElem::BindingSlot(slot) => {
            Provenance::from_norm_origin("parameter binding slot", &slot.origin)
        }
        NormPatternElem::Unit { origin } => Provenance::from_norm_origin("parameter unit", origin),
    })
}

fn pattern_origin(pattern: &NormPattern) -> &lang_syntax::NormOrigin {
    match pattern {
        NormPattern::Binder { origin, .. }
        | NormPattern::OperatorBinder { origin, .. }
        | NormPattern::Product { origin, .. }
        | NormPattern::Pack { origin, .. }
        | NormPattern::Unit { origin }
        | NormPattern::HoleRef { origin, .. }
        | NormPattern::AnonymousHole { origin }
        | NormPattern::Name { origin, .. }
        | NormPattern::Literal { origin, .. }
        | NormPattern::Nav { origin, .. }
        | NormPattern::Sequence { origin, .. }
        | NormPattern::Skeleton { origin, .. }
        | NormPattern::BindingSlot { origin, .. }
        | NormPattern::Unsupported { origin, .. } => origin,
        NormPattern::Error(error) => &error.origin,
    }
}

#[allow(dead_code)]
fn _keep_navigation_type_explicit(_: &CanonicalFullNavigation, _: SemanticValueId) {}
