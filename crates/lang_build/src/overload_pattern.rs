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
    PackBinder {
        name: String,
        provenance: Provenance,
    },
    PackDiscard {
        provenance: Provenance,
    },
    StructuredPack {
        elements: Vec<RestrictedPackElement>,
        provenance: Provenance,
    },
    Unsupported {
        reason: String,
        provenance: Provenance,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestrictedPackElement {
    Binder {
        name: String,
        provenance: Provenance,
    },
    Discard {
        provenance: Provenance,
    },
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
    /// Each explicit node inside a structured pack contributes one pack-class
    /// node. The number of remainder arguments absorbed never contributes.
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
            NormPattern::Product { elements, .. } => {
                match decode_structured_pack_elements(elements) {
                    Ok(elements) => RestrictedParamPattern::StructuredPack {
                        elements,
                        provenance,
                    },
                    Err(reason) => RestrictedParamPattern::Unsupported { reason, provenance },
                }
            }
            _ => RestrictedParamPattern::Unsupported {
                reason:
                    "restricted pack pattern must bind, discard, or structurally match the remaining product"
                        .to_string(),
                provenance,
            },
        };
    }
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

fn decode_structured_pack_elements(
    elements: &[NormPatternElem],
) -> Result<Vec<RestrictedPackElement>, String> {
    elements
        .iter()
        .map(|element| {
            let pattern = match element {
                NormPatternElem::Pattern(pattern) => pattern,
                NormPatternElem::BindingSlot(slot)
                    if slot.policy.is_none()
                        && slot.deduce.is_empty()
                        && slot.annotation.is_none()
                        && slot.with_clause.is_none()
                        && slot.initializer.is_none() =>
                {
                    &slot.value_pattern
                }
                NormPatternElem::BindingSlot(_) => {
                    return Err(
                        "restricted structured pack elements do not yet support policy, annotation, deduce, with, or initializer clauses"
                            .to_string(),
                    );
                }
                NormPatternElem::Unit { .. } => {
                    return Err(
                        "restricted structured pack matching does not yet support unit elements"
                            .to_string(),
                    );
                }
            };
            match pattern {
                NormPattern::Binder { name, origin } if name != "_" => {
                    Ok(RestrictedPackElement::Binder {
                        name: name.clone(),
                        provenance: Provenance::from_norm_origin(
                            "structured pack binder",
                            origin,
                        ),
                    })
                }
                NormPattern::Binder { origin, .. } => Ok(RestrictedPackElement::Discard {
                    provenance: Provenance::from_norm_origin("structured pack discard", origin),
                }),
                NormPattern::Skeleton { skeleton, origin }
                    if matches!(skeleton, NormSkeleton::Wildcard { .. }) =>
                {
                    Ok(RestrictedPackElement::Discard {
                        provenance: Provenance::from_norm_origin(
                            "structured pack discard",
                            origin,
                        ),
                    })
                }
                other => Err(format!(
                    "restricted structured pack element is not yet supported: {:?}",
                    pattern_origin(other)
                )),
            }
        })
        .collect()
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
                pack_bindings: BTreeMap::new(),
                specificity: SpecificityTuple {
                    max_depth: 1,
                    sum_depth: 1,
                    non_discard_explicit_node_count: 1,
                    ..SpecificityTuple::default()
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
        RestrictedParamPattern::PackBinder { .. }
        | RestrictedParamPattern::PackDiscard { .. }
        | RestrictedParamPattern::StructuredPack { .. } => Err(Diagnostic::hard_error(
            "pack parameter matching requires the remaining argument slice",
            Some(arg.provenance.clone()),
        )),
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
        RestrictedParamPattern::StructuredPack {
            elements,
            provenance,
        } => {
            if args.len() != elements.len() {
                return Err(Diagnostic::hard_error(
                    format!(
                        "structured pack applicability failed: expected {} remainder elements, got {}",
                        elements.len(),
                        args.len()
                    ),
                    Some(provenance.clone()),
                ));
            }

            let mut bindings = BTreeMap::new();
            let mut specificity = SpecificityTuple::default();
            for (element, arg) in elements.iter().zip(args) {
                match element {
                    RestrictedPackElement::Binder { name, .. } => {
                        bindings.insert(name.clone(), arg.clone());
                        specificity.explicit_pack_match_count += 1;
                    }
                    RestrictedPackElement::Discard { .. } => {
                        specificity.pack_discard_count += 1;
                    }
                }
                specificity.max_depth = 1;
                specificity.sum_depth += 1;
            }
            Ok(PatternMatchOutcome {
                bindings,
                pack_bindings: BTreeMap::new(),
                specificity,
            })
        }
        RestrictedParamPattern::Binder { provenance, .. }
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
