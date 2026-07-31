use std::collections::BTreeMap;

use lang_syntax::{NormAnnotation, NormPattern, NormPatternElem, NormSkeleton, NormSkeletonElem};

use crate::{
    identity::{SemanticValueId, TypeValueId},
    model::{Diagnostic, Provenance, SymbolId},
    policy_pair::PolicyResultEntry,
    product_shape::{ArgProductShape, NonValueArgKind, RawArgValueClass},
    semantic_world::PatternValueId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverloadArgShape {
    pub top_pattern_name: Option<String>,
    pub type_symbol_id: Option<SymbolId>,
    pub value_type: Option<TypeValueId>,
    /// Resolved semantic Pattern identity of the argument type/value.
    ///
    /// The source carrier name is not semantic identity. The connected
    /// ordinary invocation trunk fills this field from TypeValue ->
    /// PatternValue and uses it for named Pattern applicability.
    pub pattern_value: Option<PatternValueId>,
    /// The binding-level member view of the argument's own carrier, when the
    /// argument was classified from a named type carrier.
    ///
    /// A bound formal parameter that is later used as a `let f::t = param`
    /// right-hand side needs the *binding* view of what was passed in. Neither
    /// `value_type` nor `pattern_value` can supply it, because both are shared
    /// by every carrier of the same type, so the view travels with the shape.
    pub effective_view: Option<PolicyResultEntry<SemanticValueId, PatternValueId>>,
    pub semantic_value: Option<SemanticValueId>,
    pub is_value: bool,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestrictedParamPattern {
    Binder {
        name: String,
        provenance: Provenance,
    },
    ValueBinder {
        name: String,
        provenance: Provenance,
    },
    NamedDiscard {
        alternatives: Vec<String>,
        provenance: Provenance,
    },
    PackBinder {
        name: String,
        provenance: Provenance,
    },
    PackDiscard {
        provenance: Provenance,
    },
    Unsupported {
        reason: String,
        provenance: Provenance,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatternLayerOrder {
    Ordered,
    Unordered,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackOperandClass {
    WholeRemainderBinder,
    Structured { stable_top_mode: bool },
}

/// Semantic admissibility after P normalization and layer-order discovery.
/// This does not perform name or Pattern-head resolution; its caller supplies
/// the resulting stable-top-mode fact.
pub fn pack_operand_is_admissible(order: PatternLayerOrder, operand: PackOperandClass) -> bool {
    match (order, operand) {
        (_, PackOperandClass::WholeRemainderBinder) => true,
        (
            PatternLayerOrder::Ordered,
            PackOperandClass::Structured {
                stable_top_mode: true,
            },
        ) => true,
        (
            PatternLayerOrder::Ordered,
            PackOperandClass::Structured {
                stable_top_mode: false,
            },
        )
        | (PatternLayerOrder::Unordered, PackOperandClass::Structured { .. }) => false,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternMatchOutcome {
    pub bindings: BTreeMap<String, OverloadArgShape>,
    pub pack_bindings: BTreeMap<String, Vec<OverloadArgShape>>,
    pub specificity: SpecificityTuple,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpecificityTuple {
    pub max_depth: usize,
    pub sum_depth: usize,
    pub non_discard_explicit_node_count: usize,
    /// The outward Pack position contributes one pack-class node. Captured
    /// width and nested operand evidence never inflate this same-level count.
    pub explicit_pack_match_count: usize,
    pub explicit_discard_count: usize,
    pub pack_discard_count: usize,
}

impl SpecificityTuple {
    pub fn add(self, other: Self) -> Self {
        Self {
            max_depth: self.max_depth.max(other.max_depth),
            sum_depth: self.sum_depth + other.sum_depth,
            non_discard_explicit_node_count: self.non_discard_explicit_node_count
                + other.non_discard_explicit_node_count,
            explicit_pack_match_count: self.explicit_pack_match_count
                + other.explicit_pack_match_count,
            explicit_discard_count: self.explicit_discard_count + other.explicit_discard_count,
            pack_discard_count: self.pack_discard_count + other.pack_discard_count,
        }
    }
}

pub fn overload_args_from_classified_shape(
    shape: &ArgProductShape,
    symbol_name: impl Fn(SymbolId) -> Option<String>,
    pattern_for_type: impl Fn(TypeValueId) -> Option<PatternValueId>,
) -> Vec<OverloadArgShape> {
    shape
        .raw_args
        .iter()
        .map(|raw_arg| {
            let type_symbol_id = match raw_arg.value_class {
                RawArgValueClass::NonValue(NonValueArgKind::TypeObject) => {
                    raw_arg.known_type_symbol_id
                }
                _ => None,
            };
            let top_pattern_name = match raw_arg.value_class {
                RawArgValueClass::NonValue(NonValueArgKind::TypeObject) => raw_arg
                    .known_type_pattern_name
                    .clone()
                    .or_else(|| type_symbol_id.and_then(&symbol_name)),
                _ => None,
            };
            OverloadArgShape {
                top_pattern_name,
                type_symbol_id,
                value_type: raw_arg.known_first_order_type_value,
                pattern_value: raw_arg
                    .known_first_order_type_value
                    .and_then(&pattern_for_type),
                effective_view: raw_arg.known_type_member_view.clone(),
                semantic_value: raw_arg.known_semantic_value,
                is_value: matches!(raw_arg.value_class, RawArgValueClass::Value),
                provenance: raw_arg.provenance.clone(),
            }
        })
        .collect()
}

pub fn decode_param_pattern(element: &NormPatternElem) -> RestrictedParamPattern {
    let NormPatternElem::BindingSlot(slot) = element else {
        return RestrictedParamPattern::Unsupported {
            reason: "parameter element is not a binding slot".to_string(),
            provenance: Provenance::new("unsupported parameter element"),
        };
    };
    if let NormPattern::Pack { inner, origin } = &slot.value_pattern {
        let provenance = Provenance::from_norm_origin("pack parameter pattern", origin);
        return match inner.as_ref() {
            NormPattern::Binder { name, .. } if name != "_" => RestrictedParamPattern::PackBinder {
                name: name.clone(),
                provenance,
            },
            NormPattern::Binder { name, .. } if name == "_" => {
                RestrictedParamPattern::PackDiscard { provenance }
            }
            NormPattern::Skeleton { skeleton, .. }
                if matches!(skeleton, NormSkeleton::Wildcard { .. }) =>
            {
                RestrictedParamPattern::PackDiscard { provenance }
            }
            NormPattern::Product { .. } => RestrictedParamPattern::Unsupported {
                reason:
                    "bare Product Pack operands are non-canonical after P normalization"
                        .to_string(),
                provenance,
            },
            _ => RestrictedParamPattern::Unsupported {
                reason:
                    "restricted pack matching currently supports only whole-remainder binders and discards"
                        .to_string(),
                provenance,
            },
        };
    }
    match &slot.value_pattern {
        NormPattern::Binder { name, origin }
            if name != "_" && is_type_annotation(slot.annotation.as_ref()) =>
        {
            RestrictedParamPattern::Binder {
                name: name.clone(),
                provenance: Provenance::from_norm_origin("binder parameter pattern", origin),
            }
        }
        NormPattern::Binder { name, origin } if name != "_" && slot.annotation.is_some() => {
            RestrictedParamPattern::ValueBinder {
                name: name.clone(),
                provenance: Provenance::from_norm_origin("value binder parameter pattern", origin),
            }
        }
        NormPattern::Skeleton { skeleton, origin } => {
            let mut has_discard = false;
            let mut alternatives = Vec::new();
            collect_restricted_skeleton(skeleton, &mut has_discard, &mut alternatives);
            finish_restricted_named_discard(
                has_discard,
                alternatives,
                Provenance::from_norm_origin("parameter skeleton", origin),
            )
        }
        NormPattern::Sequence { elements, origin } => {
            let mut has_discard = false;
            let mut alternatives = Vec::new();
            for element in elements {
                collect_restricted_pattern(element, &mut has_discard, &mut alternatives);
            }
            finish_restricted_named_discard(
                has_discard,
                alternatives,
                Provenance::from_norm_origin("parameter sequence", origin),
            )
        }
        other => RestrictedParamPattern::Unsupported {
            reason: "unsupported restricted overload parameter pattern".to_string(),
            provenance: Provenance::from_norm_origin("parameter pattern", pattern_origin(other)),
        },
    }
}

fn finish_restricted_named_discard(
    has_discard: bool,
    mut alternatives: Vec<String>,
    provenance: Provenance,
) -> RestrictedParamPattern {
    if has_discard && !alternatives.is_empty() {
        alternatives.sort();
        alternatives.dedup();
        RestrictedParamPattern::NamedDiscard {
            alternatives,
            provenance,
        }
    } else {
        RestrictedParamPattern::Unsupported {
            reason: "unsupported restricted overload skeleton pattern".to_string(),
            provenance,
        }
    }
}

fn collect_restricted_pattern(
    pattern: &NormPattern,
    has_discard: &mut bool,
    alternatives: &mut Vec<String>,
) {
    match pattern {
        NormPattern::Skeleton { skeleton, .. } => {
            collect_restricted_skeleton(skeleton, has_discard, alternatives)
        }
        NormPattern::Name { name, .. } | NormPattern::HoleRef { name, .. } => {
            alternatives.push(name.clone())
        }
        NormPattern::Sequence { elements, .. } => {
            for element in elements {
                collect_restricted_pattern(element, has_discard, alternatives);
            }
        }
        NormPattern::Product { elements, .. } => {
            for element in elements {
                match element {
                    NormPatternElem::Pattern(pattern) => {
                        collect_restricted_pattern(pattern, has_discard, alternatives)
                    }
                    NormPatternElem::BindingSlot(slot) => {
                        collect_restricted_pattern(&slot.value_pattern, has_discard, alternatives)
                    }
                    NormPatternElem::Unit { .. } => {}
                }
            }
        }
        NormPattern::Binder { name, .. } if name == "_" => *has_discard = true,
        NormPattern::Binder { .. }
        | NormPattern::OperatorBinder { .. }
        | NormPattern::Pack { .. }
        | NormPattern::Unit { .. }
        | NormPattern::AnonymousHole { .. }
        | NormPattern::Literal { .. }
        | NormPattern::Nav { .. }
        | NormPattern::BindingSlot { .. }
        | NormPattern::Error(_)
        | NormPattern::Unsupported { .. } => {}
    }
}

pub fn match_param_pattern(
    pattern: &RestrictedParamPattern,
    arg: &OverloadArgShape,
    resolve_named_pattern: Option<&dyn Fn(&str) -> Option<TypeValueId>>,
) -> Result<PatternMatchOutcome, Diagnostic> {
    match pattern {
        RestrictedParamPattern::Binder { name, .. } => {
            if arg.type_symbol_id.is_none()
                && arg.top_pattern_name.is_none()
                && !(arg.is_value && arg.value_type.is_some())
            {
                return Err(Diagnostic::hard_error(
                    "parameter extraction-pattern applicability failed: binder expected a type-pattern argument",
                    Some(arg.provenance.clone()),
                ));
            }
            let mut bindings = BTreeMap::new();
            bindings.insert(name.clone(), arg.clone());
            Ok(PatternMatchOutcome {
                bindings,
                pack_bindings: BTreeMap::new(),
                specificity: SpecificityTuple {
                    max_depth: 1,
                    sum_depth: 1,
                    non_discard_explicit_node_count: 1,
                    ..SpecificityTuple::default()
                },
            })
        }
        RestrictedParamPattern::ValueBinder { name, .. } => {
            if !arg.is_value || arg.value_type.is_none() {
                return Err(Diagnostic::hard_error(
                    "parameter structural applicability failed: value binder expected a typed Val1 argument",
                    Some(arg.provenance.clone()),
                ));
            }
            let mut bindings = BTreeMap::new();
            bindings.insert(name.clone(), arg.clone());
            Ok(PatternMatchOutcome {
                bindings,
                pack_bindings: BTreeMap::new(),
                specificity: SpecificityTuple {
                    max_depth: 1,
                    sum_depth: 2,
                    non_discard_explicit_node_count: 2,
                    ..SpecificityTuple::default()
                },
            })
        }
        RestrictedParamPattern::NamedDiscard {
            alternatives,
            provenance: _,
        } => {
            if let (Some(actual_type), Some(resolve_named_pattern)) =
                (arg.value_type, resolve_named_pattern)
            {
                if !alternatives
                    .iter()
                    .filter_map(|name| resolve_named_pattern(name))
                    .any(|expected| expected == actual_type)
                {
                    return Err(Diagnostic::hard_error(
                        format!(
                            "parameter extraction-pattern applicability failed: argument TypeValue {:?} does not match any resolved alternative [{}]",
                            actual_type,
                            alternatives.join(", ")
                        ),
                        Some(arg.provenance.clone()),
                    ));
                }
                return Ok(PatternMatchOutcome {
                    bindings: BTreeMap::new(),
                    pack_bindings: BTreeMap::new(),
                    specificity: SpecificityTuple {
                        max_depth: 1,
                        sum_depth: 2,
                        non_discard_explicit_node_count: 1,
                        explicit_discard_count: 1,
                        ..SpecificityTuple::default()
                    },
                });
            }
            let Some(top_pattern_name) = &arg.top_pattern_name else {
                return Err(Diagnostic::hard_error(
                    "parameter extraction-pattern applicability failed: named pattern expected a top type-pattern name",
                    Some(arg.provenance.clone()),
                ));
            };
            if !alternatives.iter().any(|name| name == top_pattern_name) {
                return Err(Diagnostic::hard_error(
                    format!(
                        "parameter extraction-pattern applicability failed: expected one of [{}], got `{top_pattern_name}`",
                        alternatives.join(", ")
                    ),
                    Some(arg.provenance.clone()),
                ));
            }
            Ok(PatternMatchOutcome {
                bindings: BTreeMap::new(),
                pack_bindings: BTreeMap::new(),
                // `_ name` explicitly visits the matched top node and an
                // explicit discard node. The selected alternative alone
                // contributes; extra alternatives add no rank.
                specificity: SpecificityTuple {
                    max_depth: 1,
                    sum_depth: 2,
                    non_discard_explicit_node_count: 1,
                    explicit_discard_count: 1,
                    ..SpecificityTuple::default()
                },
            })
        }
        RestrictedParamPattern::PackBinder { .. } | RestrictedParamPattern::PackDiscard { .. } => {
            Err(Diagnostic::hard_error(
                "pack parameter matching requires the remaining argument slice",
                Some(arg.provenance.clone()),
            ))
        }
        RestrictedParamPattern::Unsupported { reason, provenance } => Err(Diagnostic::hard_error(
            format!("unsupported parameter extraction pattern: {reason}"),
            Some(provenance.clone()),
        )),
    }
}

pub fn match_pack_param_pattern(
    pattern: &RestrictedParamPattern,
    args: &[OverloadArgShape],
) -> Result<PatternMatchOutcome, Diagnostic> {
    match pattern {
        RestrictedParamPattern::PackBinder { name, .. } => {
            let mut pack_bindings = BTreeMap::new();
            pack_bindings.insert(name.clone(), args.to_vec());
            Ok(PatternMatchOutcome {
                bindings: BTreeMap::new(),
                pack_bindings,
                specificity: SpecificityTuple {
                    max_depth: 1,
                    sum_depth: 1,
                    explicit_pack_match_count: 1,
                    ..SpecificityTuple::default()
                },
            })
        }
        RestrictedParamPattern::PackDiscard { .. } => Ok(PatternMatchOutcome {
            bindings: BTreeMap::new(),
            pack_bindings: BTreeMap::new(),
            specificity: SpecificityTuple {
                max_depth: 1,
                sum_depth: 1,
                pack_discard_count: 1,
                ..SpecificityTuple::default()
            },
        }),
        RestrictedParamPattern::Binder { provenance, .. }
        | RestrictedParamPattern::ValueBinder { provenance, .. }
        | RestrictedParamPattern::NamedDiscard { provenance, .. }
        | RestrictedParamPattern::Unsupported { provenance, .. } => Err(Diagnostic::hard_error(
            "non-pack parameter cannot consume an argument slice",
            Some(provenance.clone()),
        )),
    }
}

fn is_type_annotation(annotation: Option<&NormAnnotation>) -> bool {
    matches!(
        annotation.map(|annotation| &annotation.pattern),
        Some(NormPattern::Name { name, .. }) if name == "type"
    )
}

fn collect_restricted_skeleton(
    skeleton: &NormSkeleton,
    has_discard: &mut bool,
    alternatives: &mut Vec<String>,
) {
    match skeleton {
        NormSkeleton::Wildcard { .. } => *has_discard = true,
        NormSkeleton::Name { name, .. } => alternatives.push(name.clone()),
        NormSkeleton::HoleRef { name, .. } => alternatives.push(name.clone()),
        NormSkeleton::Segment { elements, .. } => {
            for element in elements {
                collect_restricted_skeleton(element, has_discard, alternatives);
            }
        }
        NormSkeleton::Product { elements, .. } => {
            for element in elements {
                if let NormSkeletonElem::Skeleton(skeleton) = element {
                    collect_restricted_skeleton(skeleton, has_discard, alternatives);
                }
            }
        }
        NormSkeleton::Nav { .. } | NormSkeleton::Literal { .. } | NormSkeleton::Error(_) => {}
    }
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
