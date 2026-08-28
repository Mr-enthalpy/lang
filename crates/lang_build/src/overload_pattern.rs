use crate::{
    identity::{SemanticValueId, TypeValueId},
    model::{Provenance, SymbolId},
    policy_pair::PolicyResultEntry,
    product_shape::{ArgProductShape, NonValueArgKind, RawArgValueClass},
    semantic_world::PatternValueId,
};

/// Already-observed argument content supplied to the Pattern relation.
///
/// This is a transport/view, not a Pattern AST and not an applicability
/// authority. `pattern_relation` is the only production consumer that may
/// decide whether a formal Pattern relates to this content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverloadArgShape {
    /// Diagnostic compatibility spelling only.
    pub top_pattern_name: Option<String>,
    /// Carrier/graph coordinate only.
    pub type_symbol_id: Option<SymbolId>,
    /// Opaque Core lookup key only; never used for Pattern equality.
    pub value_type: Option<TypeValueId>,
    pub pattern_value: Option<PatternValueId>,
    /// Ordinary type-equality observation, `Addr(Norm(Core(tau)))`.
    pub type_core_observation: Option<crate::CanonicalValueAddr>,
    /// Whole immutable type snapshot for explicitly snapshot-sensitive later
    /// consumers. Base Pattern applicability does not use it.
    pub complete_type_observation: Option<crate::CanonicalValueAddr>,
    pub effective_view: Option<PolicyResultEntry<SemanticValueId, PatternValueId>>,
    pub semantic_value: Option<SemanticValueId>,
    pub is_value: bool,
    pub provenance: Provenance,
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

/// Candidate-shape preparation fact retained for the open full Pattern-space
/// representation. It does not perform `R_Gamma` and cannot make a candidate
/// applicable by itself.
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpecificityTuple {
    pub max_depth: usize,
    pub sum_depth: usize,
    pub non_discard_explicit_node_count: usize,
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
                RawArgValueClass::NonValue(NonValueArgKind::CoreTypeProjection) => {
                    raw_arg.known_type_symbol_id
                }
                _ => None,
            };
            let top_pattern_name = match raw_arg.value_class {
                RawArgValueClass::NonValue(NonValueArgKind::CoreTypeProjection) => raw_arg
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
                type_core_observation: raw_arg.known_type_observation,
                complete_type_observation: raw_arg.known_complete_type_observation,
                effective_view: raw_arg.known_type_member_view.clone(),
                semantic_value: raw_arg.known_semantic_value,
                is_value: matches!(raw_arg.value_class, RawArgValueClass::Value),
                provenance: raw_arg.provenance.clone(),
            }
        })
        .collect()
}
