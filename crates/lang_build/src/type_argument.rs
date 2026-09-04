//! Type-argument classification boundary.
//!
//! Classifies `UnknownExpression` arguments in an `ArgProductShape` by
//! resolving their corresponding product-atom names through a supplied
//! `TypeResolutionEnv`. Classification records the complete-type observation,
//! Core lookup handle, binding view, and carrier Place required by later
//! consumers. `NonValue(CoreTypeProjection)` is the argument-role tag; Pattern
//! applicability is supplied separately by `R_Gamma`.
//!
//! This module does **not** resolve call targets, does **not** perform type
//! checking, does **not** insert mechanical pass actions, and does **not**
//! classify value/non-type arguments.

use lang_syntax::NormExpr;

use crate::{
    identity::SemanticValueId,
    model::{Diagnostic, NamespaceNodeId, Provenance, SymbolId},
    policy_pair::PolicyResultEntry,
    product_shape::{ArgProductShape, ProductAtom, RawArgValueClass},
    semantic_name_index::ResolverContext,
    semantic_world::{ObjectPlaceId, PatternValueId, SemanticWorld},
    TypeValueId,
};

/// Classification report: carries the classified shape alongside unresolved
/// type-name entries for near-cause diagnostics.
#[derive(Clone, Debug)]
pub struct TypeArgumentClassificationReport {
    pub classified_shape: ArgProductShape,
    pub unresolved_names: Vec<String>,
}

/// Result of resolving one bare name as a type in some resolution
/// environment. `represented_type` is an opaque Core lookup handle; complete
/// type identity flows through `complete_type_observation`. The carrier Symbol
/// is graph navigation material and is absent for semantic-world resolutions.
///
/// `effective_view` is the resolved carrier's own binding-level pure-P member
/// view.  It travels with the resolution because a represented TypeValue is
/// shared by every carrier that binds it (`P_a let T: type = X;` and
/// `P_b let U: type = X;` have one TypeValue and two binding Policies), so no
/// later stage can recover the binding view from the TypeValue alone.
///
/// `carrier_place` is that carrier object's own Val2 place.  A pure P is a
/// real object, so two carriers of one Pattern can hold different Val2 and
/// the type argument's normal form must be observed from the carrier that was
/// actually named:
///
/// ```text
/// Norm_type(x) = ⟨Norm_P(P_x), Norm_Val2(Val2_x)⟩
/// ```
///
/// The place is the observation coordinate only; it never enters the normal
/// form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedTypeResolution {
    pub carrier_symbol: Option<SymbolId>,
    pub represented_type: TypeValueId,
    pub effective_view: Option<PolicyResultEntry<SemanticValueId, PatternValueId>>,
    pub carrier_place: Option<ObjectPlaceId>,
    pub complete_type_observation: Option<crate::CanonicalValueAddr>,
}

/// Outcome of checking a selected source-body local `let` initializer against
/// the environment's evaluation discipline.
#[derive(Clone, Debug)]
pub enum BodyLocalInitializerCheck {
    Accepted,
    Residual {
        reason: String,
        provenance: Provenance,
    },
    Rejected(Diagnostic),
}

/// Type-resolution environment boundary.
///
/// The canonical build/invocation spine resolves type names through the
/// semantic world; graph projection paths keep name-index reads
/// behind the same interface. Callers never read
/// `SemanticNameIndex` / `SymbolPayload::CompleteTypeProjection` directly.
pub trait TypeResolutionEnv {
    /// Resolve one bare name as a complete type value under open-static policy.
    fn resolve_type_name(
        &self,
        name: &str,
        context: &ResolverContext,
    ) -> Option<NamedTypeResolution>;

    /// Resolve one complete inner-to-outer navigation as a complete type value.
    ///
    /// A navigated type argument (`f::T`) must reach the same terminal Symbol
    /// as the same path in any other use context; only the projected facet
    /// differs.  Environments without recursive Val2 navigation resolve a
    /// single-component path and reject longer ones.
    fn resolve_type_path(
        &self,
        path: &[String],
        context: &ResolverContext,
    ) -> Option<NamedTypeResolution> {
        match path {
            [name] => self.resolve_type_name(name, context),
            _ => None,
        }
    }

    /// Resolve a struct field type path, producing the field-type carrier
    /// Symbol (non-identity installation material) and represented type.
    fn resolve_field_type_path(
        &self,
        path: &[String],
        context: &ResolverContext,
        provenance: &Provenance,
    ) -> Result<(SymbolId, TypeValueId), Diagnostic>;

    /// Check a selected source-body local `let` initializer.
    fn check_body_local_initializer(
        &self,
        declaration_namespace: Option<NamespaceNodeId>,
        initializer: &NormExpr,
        context: &ResolverContext,
        provenance: Provenance,
    ) -> BodyLocalInitializerCheck;
}

/// Canonical semantic-world environment. Resolution flows through recursive
/// ClusterSymbol lookup (`resolve_symbol_path` → pure-P → pattern type); no
/// graph Symbol payload is read on this path.
pub struct SemanticTypeEnv<'a> {
    world: &'a SemanticWorld,
}

impl<'a> SemanticTypeEnv<'a> {
    pub fn new(world: &'a SemanticWorld) -> Self {
        Self { world }
    }

    /// Resolve a path to its pure-P carrier facts: represented TypeValue plus
    /// the carrier's own binding-level member view and object place.
    ///
    /// The path walks the one shared recursive Symbol navigation, so `f::T`
    /// means `Val2(T)[f]` here exactly as it does in a call target; this
    /// context only projects the terminal Symbol's pure-P facet afterwards.
    fn resolve_path_carrier(
        &self,
        path: &[String],
        context: &ResolverContext,
    ) -> Option<NamedTypeResolution> {
        let navigation = self
            .world
            .navigate_semantic_path(
                path,
                context.current_namespace,
                &context.explicit_mount_roots,
                &context.default_mounts,
            )
            .ok()?;
        let cell = self.world.symbol(navigation.terminal_symbol)?;
        let pattern = cell.pure_p_pattern()?;
        Some(NamedTypeResolution {
            carrier_symbol: self
                .world
                .backing_declaration_for_symbol(navigation.terminal_symbol),
            represented_type: self.world.type_for_pattern(pattern)?,
            effective_view: cell.pure_p_view().cloned(),
            carrier_place: cell.pure_p_place(),
            complete_type_observation: cell.pure_p.and_then(|member| member.complete_type),
        })
    }
}

impl TypeResolutionEnv for SemanticTypeEnv<'_> {
    fn resolve_type_name(
        &self,
        name: &str,
        context: &ResolverContext,
    ) -> Option<NamedTypeResolution> {
        self.resolve_path_carrier(&[name.to_string()], context)
    }

    fn resolve_type_path(
        &self,
        path: &[String],
        context: &ResolverContext,
    ) -> Option<NamedTypeResolution> {
        self.resolve_path_carrier(path, context)
    }

    fn resolve_field_type_path(
        &self,
        path: &[String],
        context: &ResolverContext,
        provenance: &Provenance,
    ) -> Result<(SymbolId, TypeValueId), Diagnostic> {
        let type_path_str = path.join("::");
        let resolution = self.resolve_path_carrier(path, context).ok_or_else(|| {
            Diagnostic::hard_error(
                format!("unknown struct field type `{type_path_str}`"),
                Some(provenance.clone()),
            )
        })?;
        // Field-type graph projection used for namespace installation only;
        // it is non-identity material (see
        // `FieldSignatureMaterial::field_type_carrier_symbol`).
        Ok((
            resolution.carrier_symbol.ok_or_else(|| {
                Diagnostic::hard_error(
                    "resolved struct field type has no graph declaration projection",
                    Some(provenance.clone()),
                )
            })?,
            resolution.represented_type,
        ))
    }

    fn check_body_local_initializer(
        &self,
        _declaration_namespace: Option<NamespaceNodeId>,
        _initializer: &NormExpr,
        _context: &ResolverContext,
        _provenance: Provenance,
    ) -> BodyLocalInitializerCheck {
        // The semantic world has no best-effort graph evaluator. Local
        // bindings that selected source-body execution later references are rejected at
        // the reference site; unreferenced locals impose no residual check.
        BodyLocalInitializerCheck::Accepted
    }
}

/// Classifies argument content through a semantic type-resolution environment.
///
/// Successful hits record the source pattern name for binder substitution
/// alongside the complete type observation.
///
/// A navigated argument (`f::T`) is classified through the same shared
/// recursive Symbol navigation as a bare name: the path denotes one terminal
/// Symbol, and this context projects its pure-P facet. The resolved carrier's
/// own object place travels with the classified argument, because
/// `Norm_type(x) = ⟨Norm_P(P_x), Norm_Val2(Val2_x)⟩` must be observed from the
/// carrier that was actually named.
pub fn classify_type_arguments_env_with_report(
    shape: &ArgProductShape,
    env: &dyn TypeResolutionEnv,
    context: &ResolverContext,
) -> TypeArgumentClassificationReport {
    let mut args = shape.raw_args.clone();
    let mut unresolved = Vec::new();
    for raw_arg in &mut args {
        if !matches!(raw_arg.value_class, RawArgValueClass::UnknownExpression) {
            continue;
        }
        let atom = match shape.flattened.atoms.get(raw_arg.index) {
            Some(atom) => atom,
            None => continue,
        };
        let path = match atom {
            ProductAtom::Expression { expr, .. } => match type_argument_path(expr) {
                Some(path) => path,
                None => continue,
            },
            _ => continue,
        };
        let name = path.join("::");
        match env.resolve_type_path(&path, context) {
            Some(resolution) => {
                *raw_arg = raw_arg.clone().as_complete_type_projection_named(
                    name,
                    resolution.represented_type,
                    resolution.carrier_symbol,
                    resolution.effective_view,
                    resolution.carrier_place,
                    resolution.complete_type_observation,
                );
            }
            None => {
                unresolved.push(name);
            }
        }
    }
    TypeArgumentClassificationReport {
        classified_shape: ArgProductShape {
            raw_args: args,
            ..shape.clone()
        },
        unresolved_names: unresolved,
    }
}

/// The inner-to-outer path of one type-argument atom.
///
/// Only complete name navigations participate; any other component family is
/// left unclassified rather than partially interpreted.
fn type_argument_path(expr: &NormExpr) -> Option<Vec<String>> {
    match expr {
        NormExpr::Name { text, .. } => Some(vec![text.clone()]),
        NormExpr::Nav { components, .. } => components
            .iter()
            .map(|component| match component {
                lang_syntax::NormNavComponent::Name { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}
