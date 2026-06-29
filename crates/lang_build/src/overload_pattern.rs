use std::collections::BTreeMap;

use lang_syntax::{NormAnnotation, NormPattern, NormPatternElem, NormSkeleton, NormSkeletonElem};

use crate::{
    model::{Diagnostic, Provenance, SymbolId},
    product_shape::{ArgProductShape, NonValueArgKind, RawArgValueClass},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverloadArgShape {
    pub top_pattern_name: Option<String>,
    pub type_symbol_id: Option<SymbolId>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestrictedParamPattern {
    Binder {
        name: String,
        provenance: Provenance,
    },
    NamedDiscard {
        alternatives: Vec<String>,
        provenance: Provenance,
    },
    Unsupported {
        reason: String,
        provenance: Provenance,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternMatchOutcome {
    pub bindings: BTreeMap<String, OverloadArgShape>,
    pub specificity: SpecificityTuple,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpecificityTuple {
    pub max_depth: usize,
    pub sum_depth: usize,
    pub non_discard_explicit_node_count: usize,
}

impl SpecificityTuple {
    pub fn add(self, other: Self) -> Self {
        Self {
            max_depth: self.max_depth.max(other.max_depth),
            sum_depth: self.sum_depth + other.sum_depth,
            non_discard_explicit_node_count: self.non_discard_explicit_node_count
                + other.non_discard_explicit_node_count,
        }
    }
}

pub fn overload_args_from_classified_shape(
    shape: &ArgProductShape,
    symbol_name: impl Fn(SymbolId) -> Option<String>,
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
            OverloadArgShape {
                top_pattern_name: type_symbol_id.and_then(&symbol_name),
                type_symbol_id,
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
    if !is_type_annotation(slot.annotation.as_ref()) {
        return RestrictedParamPattern::Unsupported {
            reason: "restricted overload parameter must be annotated as `type`".to_string(),
            provenance: Provenance::from_norm_origin("parameter pattern", &slot.origin),
        };
    }

    match &slot.value_pattern {
        NormPattern::Binder { name, origin } if name != "_" => RestrictedParamPattern::Binder {
            name: name.clone(),
            provenance: Provenance::from_norm_origin("binder parameter pattern", origin),
        },
        NormPattern::Skeleton { skeleton, origin } => {
            let mut has_discard = false;
            let mut alternatives = Vec::new();
            collect_restricted_skeleton(skeleton, &mut has_discard, &mut alternatives);
            if has_discard && !alternatives.is_empty() {
                alternatives.sort();
                alternatives.dedup();
                RestrictedParamPattern::NamedDiscard {
                    alternatives,
                    provenance: Provenance::from_norm_origin(
                        "named discard parameter pattern",
                        origin,
                    ),
                }
            } else {
                RestrictedParamPattern::Unsupported {
                    reason: "unsupported restricted overload skeleton pattern".to_string(),
                    provenance: Provenance::from_norm_origin("parameter skeleton", origin),
                }
            }
        }
        other => RestrictedParamPattern::Unsupported {
            reason: "unsupported restricted overload parameter pattern".to_string(),
            provenance: Provenance::from_norm_origin("parameter pattern", pattern_origin(other)),
        },
    }
}

pub fn match_param_pattern(
    pattern: &RestrictedParamPattern,
    arg: &OverloadArgShape,
) -> Result<PatternMatchOutcome, Diagnostic> {
    match pattern {
        RestrictedParamPattern::Binder { name, .. } => {
            if arg.type_symbol_id.is_none() {
                return Err(Diagnostic::hard_error(
                    "parameter extraction-pattern applicability failed: binder expected a type-pattern argument",
                    Some(arg.provenance.clone()),
                ));
            }
            let mut bindings = BTreeMap::new();
            bindings.insert(name.clone(), arg.clone());
            Ok(PatternMatchOutcome {
                bindings,
                specificity: SpecificityTuple {
                    max_depth: 1,
                    sum_depth: 1,
                    non_discard_explicit_node_count: 1,
                },
            })
        }
        RestrictedParamPattern::NamedDiscard {
            alternatives,
            provenance: _,
        } => {
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
                // `_ name` explicitly visits the matched top node and an
                // explicit discard node. The selected alternative alone
                // contributes; extra alternatives add no rank.
                specificity: SpecificityTuple {
                    max_depth: 1,
                    sum_depth: 2,
                    non_discard_explicit_node_count: 1,
                },
            })
        }
        RestrictedParamPattern::Unsupported { reason, provenance } => Err(Diagnostic::hard_error(
            format!("unsupported parameter extraction pattern: {reason}"),
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
        | NormPattern::Unit { origin }
        | NormPattern::HoleRef { origin, .. }
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
